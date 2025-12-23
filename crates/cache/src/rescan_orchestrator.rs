use super::{ChangeDetector, RescanOrchestrator};
use lighty_config::{Config, ServerConfig};
use lighty_events::{AppEvent, EventBus};
use lighty_scanner::ServerScanner;
use lighty_models::VersionBuilder;
use anyhow::Result;
use dashmap::DashMap;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::collections::HashSet;

fn get_current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let datetime = chrono::DateTime::from_timestamp(now as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    datetime.to_rfc3339()
}

impl RescanOrchestrator {
    pub fn new(
        cache: Arc<dyn super::models::CacheUpdater>,
        last_updated: Arc<DashMap<String, String>>,
        config: Arc<RwLock<Config>>,
        events: Arc<EventBus>,
    ) -> Self {
        Self {
            cache,
            last_updated,
            config,
            events,
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Pauses the rescan loop
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        tracing::debug!("Rescan paused");
    }

    /// Resumes the rescan loop
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        tracing::debug!("Rescan resumed");
    }

    /// Runs the continuous rescan loop
    pub async fn run_rescan_loop(&self) {
        let rescan_interval = {
            let config_read = self.config.read().await;
            config_read.cache.rescan_interval
        };

        if rescan_interval == 0 {
            self.events.emit(AppEvent::ContinuousScanEnabled);
            self.run_file_watcher_loop().await;
        } else {
            self.events.emit(AppEvent::AutoScanEnabled { interval: rescan_interval });
            let mut interval = interval(Duration::from_secs(rescan_interval));
            interval.tick().await;

            loop {
                interval.tick().await;

                // Check if rescan is paused (e.g., during config reload)
                if self.paused.load(Ordering::SeqCst) {
                    continue;
                }

                let (servers, base_url, base_path) = {
                    let config_read = self.config.read().await;
                    (
                        config_read.servers.clone(),
                        config_read.server.base_url.clone(),
                        config_read.server.base_path.clone(),
                    )
                };

                for server_config in &servers {
                    if !server_config.enabled {
                        continue;
                    }
                    self.rescan_server(server_config, &base_url, &base_path).await;
                }
            }
        }
    }

    /// Runs file watcher loop for continuous monitoring (event-driven instead of polling)
    async fn run_file_watcher_loop(&self) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Setup file watcher
        let mut watcher = match notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Only trigger on actual file modifications, not metadata changes
                if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)) {
                    let _ = tx.send(event);
                }
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Failed to create file watcher: {}", e);
                return;
            }
        };

        // Watch all server directories
        let (servers, base_path) = {
            let config_read = self.config.read().await;
            (config_read.servers.clone(), config_read.server.base_path.clone())
        };

        for server in &servers {
            if !server.enabled {
                continue;
            }
            let server_path = PathBuf::from(&base_path).join(&server.name);
            if server_path.exists() {
                if let Err(e) = watcher.watch(&server_path, RecursiveMode::Recursive) {
                    tracing::warn!("Failed to watch server folder {}: {}", server.name, e);
                }
            }
        }

        // Debounce settings: wait after last event before rescanning
        let debounce_ms = {
            let config = self.config.read().await;
            config.cache.file_watcher_debounce_ms
        };
        let debounce_duration = Duration::from_millis(debounce_ms);
        let mut pending_servers: HashSet<String> = HashSet::new();
        let mut debounce_timer: Option<tokio::time::Instant> = None;

        loop {
            tokio::select! {
                // File system event received
                Some(event) = rx.recv() => {
                    // Check if paused
                    if self.paused.load(Ordering::SeqCst) {
                        continue;
                    }

                    // Determine which server(s) are affected
                    let (servers, base_path) = {
                        let config = self.config.read().await;
                        (config.servers.clone(), config.server.base_path.clone())
                    };

                    for path in event.paths {
                        for server in &servers {
                            let server_path = PathBuf::from(&base_path).join(&server.name);
                            if path.starts_with(&server_path) {
                                pending_servers.insert(server.name.clone());
                                break;
                            }
                        }
                    }

                    // Reset debounce timer
                    debounce_timer = Some(tokio::time::Instant::now() + debounce_duration);
                }

                // Debounce timer expired
                _ = async {
                    match debounce_timer {
                        Some(deadline) => tokio::time::sleep_until(deadline).await,
                        None => std::future::pending().await,
                    }
                }, if debounce_timer.is_some() => {
                    // Rescan all pending servers
                    if !pending_servers.is_empty() {
                        let (servers, base_url, base_path) = {
                            let config = self.config.read().await;
                            (config.servers.clone(), config.server.base_url.clone(), config.server.base_path.clone())
                        };

                        for server_name in pending_servers.drain() {
                            if let Some(server_config) = servers.iter().find(|s| s.name == server_name) {
                                if server_config.enabled {
                                    tracing::debug!("File change detected, rescanning server: {}", server_name);
                                    self.rescan_server(server_config, &base_url, &base_path).await;
                                }
                            }
                        }
                    }

                    debounce_timer = None;
                }
            }
        }
    }

    /// Rescans a single server and updates cache if changed
    async fn rescan_server(
        &self,
        server_config: &ServerConfig,
        base_url: &str,
        base_path: &str,
    ) {
        let batch_config = {
            let config = self.config.read().await;
            config.cache.batch.clone()
        };

        match ServerScanner::scan_server_silent(server_config, base_url, base_path, &batch_config).await {
            Ok(builder) => {
                self.update_cache_if_changed(server_config, builder).await;
            }
            Err(_) => {
                // Silent error - server may be incomplete or removed
            }
        }
    }

    /// Updates cache if changes are detected
    async fn update_cache_if_changed(
        &self,
        server_config: &ServerConfig,
        new_builder: VersionBuilder,
    ) {
        let (changed, change_details) = {
            let old_builder = self.cache.get(&server_config.name);

            match old_builder.as_ref() {
                Some(old) => ChangeDetector::detect_changes(old, &new_builder),
                None => (true, vec![]),
            }
        };

        if changed {
            let is_new = !self.cache.contains(&server_config.name);
            self.cache.insert(server_config.name.clone(), Arc::new(new_builder));
            self.last_updated.insert(server_config.name.clone(), get_current_timestamp());

            if is_new {
                self.events.emit(AppEvent::CacheNew { server: server_config.name.clone() });
            } else {
                self.events.emit(AppEvent::CacheUpdated {
                    server: server_config.name.clone(),
                    changes: change_details,
                });
            }
        } else {
            self.events.emit(AppEvent::CacheUnchanged { server: server_config.name.clone() });
        }
    }

    /// Scans all enabled servers initially
    pub async fn scan_all_servers(&self) -> Result<()> {
        let (servers, base_url, base_path, batch_config) = {
            let config = self.config.read().await;
            (
                config.servers.clone(),
                config.server.base_url.clone(),
                config.server.base_path.clone(),
                config.cache.batch.clone(),
            )
        };

        let scan_futures: Vec<_> = servers
            .iter()
            .filter(|server_config| server_config.enabled)
            .map(|server_config| {
                let config = server_config.clone();
                let base_url = base_url.clone();
                let base_path = base_path.clone();
                let batch_config = batch_config.clone();
                async move {
                    let result = ServerScanner::scan_server(&config, &base_url, &base_path, &batch_config).await;
                    (config.name.clone(), result)
                }
            })
            .collect();

        let results = futures::future::join_all(scan_futures).await;

        // Update cache with results
        for (server_name, result) in results {
            match result {
                Ok(builder) => {
                    self.cache.insert(server_name.clone(), Arc::new(builder));
                    self.last_updated.insert(server_name.clone(), get_current_timestamp());
                    self.events.emit(AppEvent::CacheNew { server: server_name });
                }
                Err(e) => {
                    self.events.emit(AppEvent::Error {
                        context: format!("Failed to scan server {}", server_name),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Forces a rescan of a specific server
    pub async fn force_rescan_server(&self, server_name: &str) -> Result<()> {
        let (server_config, base_url, base_path, batch_config) = {
            let config = self.config.read().await;
            let server_config = config
                .servers
                .iter()
                .find(|s| s.name == server_name)
                .ok_or_else(|| anyhow::anyhow!("Server not found: {}", server_name))?
                .clone();
            (
                server_config,
                config.server.base_url.clone(),
                config.server.base_path.clone(),
                config.cache.batch.clone(),
            )
        };

        let builder = ServerScanner::scan_server(&server_config, &base_url, &base_path, &batch_config).await?;

        self.cache.insert(server_name.to_string(), Arc::new(builder));
        self.last_updated.insert(server_name.to_string(), get_current_timestamp());

        Ok(())
    }
}
