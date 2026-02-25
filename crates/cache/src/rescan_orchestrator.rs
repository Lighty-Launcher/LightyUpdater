use super::RescanOrchestrator;
use super::models::RescanOrchestratorDeps;
use lighty_config::ServerConfig;
use lighty_events::AppEvent;
use lighty_scanner::ServerScanner;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::{interval, Duration};

impl RescanOrchestrator {
    pub fn new(deps: RescanOrchestratorDeps) -> Self {
        Self {
            cache: deps.cache,
            file_cache_manager: deps.file_cache_manager,
            last_updated: deps.last_updated,
            config: deps.config,
            events: deps.events,
            paused: Arc::new(AtomicBool::new(false)),
            storage: deps.storage,
            cdn: deps.cdn,
            cloudflare: deps.cloudflare,
            base_path: deps.base_path,
            server_path_cache: deps.server_path_cache,
        }
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        tracing::debug!("Rescan paused");
    }

    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        tracing::debug!("Rescan resumed");
    }

    pub(super) fn current_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let datetime =
            chrono::DateTime::from_timestamp(now as i64, 0).unwrap_or_else(chrono::Utc::now);
        datetime.to_rfc3339()
    }

    pub async fn run_rescan_loop(self: Arc<Self>) {
        let rescan_interval = {
            let config_read = self.config.read().await;
            config_read.cache.rescan_interval
        };

        if rescan_interval == 0 {
            self.events.emit(AppEvent::ContinuousScanEnabled);
            self.run_file_watcher_loop().await;
        } else {
            self.events
                .emit(AppEvent::AutoScanEnabled { interval: rescan_interval });
            let mut interval = interval(Duration::from_secs(rescan_interval));
            interval.tick().await;

            loop {
                interval.tick().await;

                if self.paused.load(Ordering::Relaxed) {
                    continue;
                }

                let (servers, base_path) = {
                    let config_read = self.config.read().await;
                    (
                        config_read.servers.clone(),
                        config_read.server.base_path.clone(),
                    )
                };

                for server_config in servers {
                    if !server_config.enabled {
                        continue;
                    }
                    self.rescan_server(server_config, base_path.clone()).await;
                }
            }
        }
    }

    async fn run_file_watcher_loop(&self) {
        let (enabled, debounce_ms, event_channel_size) = {
            let config = self.config.read().await;
            (
                config.hot_reload.files.enabled,
                config.hot_reload.files.debounce_ms,
                config.cache.config_reload_channel_size,
            )
        };
        if event_channel_size == 0 {
            tracing::error!(
                "Invalid configuration: cache.config_reload_channel_size must be greater than 0"
            );
            return;
        }

        if !enabled {
            tracing::warn!(
                "File watcher hot-reload is disabled, continuous scan will not monitor file changes"
            );
            std::future::pending::<()>().await;
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(event_channel_size);

        let mut watcher = match notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        match tx.try_send(event) {
                            Ok(()) => {}
                            Err(TrySendError::Full(_)) => {
                                // Coalescing is safe because we debounce and then rescan by server.
                                tracing::trace!("File watcher queue is full, dropping event");
                            }
                            Err(TrySendError::Closed(_)) => {}
                        }
                    }
                }
            },
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!("Failed to create file watcher: {}", e);
                return;
            }
        };

        let (servers, base_path) = {
            let config_read = self.config.read().await;
            (config_read.servers.clone(), config_read.server.base_path.clone())
        };

        for server in &servers {
            if !server.enabled {
                continue;
            }
            let server_path = PathBuf::from(base_path.as_ref()).join(server.name.as_ref());
            if server_path.exists() {
                if let Err(e) = watcher.watch(&server_path, RecursiveMode::Recursive) {
                    tracing::warn!("Failed to watch server folder {}: {}", server.name, e);
                }
            }
        }

        let debounce_duration = Duration::from_millis(debounce_ms);

        while let Some(event) = rx.recv().await {
            if self.paused.load(Ordering::Relaxed) {
                continue;
            }

            let mut pending_servers: HashSet<String> = HashSet::new();
            for path in event.paths {
                if let Some(server_name) = self.server_path_cache.find_server(&path) {
                    pending_servers.insert(server_name);
                }
            }

            // Debounce burst events and coalesce impacted servers.
            tokio::time::sleep(debounce_duration).await;
            while let Ok(event) = rx.try_recv() {
                for path in event.paths {
                    if let Some(server_name) = self.server_path_cache.find_server(&path) {
                        pending_servers.insert(server_name);
                    }
                }
            }

            if pending_servers.is_empty() {
                continue;
            }

            let (servers, base_path) = {
                let config = self.config.read().await;
                (config.servers.clone(), config.server.base_path.clone())
            };

            let servers_map: HashMap<String, Arc<ServerConfig>> = servers
                .into_iter()
                .map(|server| (server.name.to_string(), server))
                .collect();

            for server_name in pending_servers {
                if let Some(server_config) = servers_map.get(server_name.as_str()).cloned() {
                    if server_config.enabled {
                        tracing::debug!("File change detected, rescanning server: {}", server_name);
                        self.rescan_server(server_config, base_path.clone()).await;
                    }
                }
            }
        }
    }

    async fn rescan_server(
        &self,
        server_config: Arc<ServerConfig>,
        base_path: std::sync::Arc<str>,
    ) {
        let (batch_config, buffer_size) = {
            let config = self.config.read().await;
            (config.cache.batch.clone(), config.cache.checksum_buffer_size)
        };

        if let Some(storage) = &self.storage {
            match ServerScanner::scan_server_silent(
                server_config.as_ref(),
                storage,
                base_path.as_ref(),
                &batch_config,
                buffer_size,
            )
            .await
            {
                Ok(builder) => {
                    self.update_cache_if_changed(server_config.as_ref(), builder).await;
                }
                Err(_) => {
                    // Silent error - server may be incomplete or removed
                }
            }
        }
    }
}
