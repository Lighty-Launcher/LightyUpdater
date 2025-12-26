use super::RescanOrchestrator;
use super::errors::CacheError;
use lighty_config::{Config, ServerConfig};
use lighty_events::{AppEvent, EventBus};
use lighty_scanner::ServerScanner;
use lighty_models::VersionBuilder;
use dashmap::DashMap;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::collections::{HashSet, HashMap};

type Result<T> = std::result::Result<T, CacheError>;

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
        storage: Option<Arc<dyn lighty_storage::StorageBackend>>,
        cloudflare: Option<Arc<super::cloudflare::CloudflareClient>>,
        base_path: PathBuf,
        server_path_cache: Arc<super::server_path_cache::ServerPathCache>,
    ) -> Self {
        Self {
            cache,
            last_updated,
            config,
            events,
            paused: Arc::new(AtomicBool::new(false)),
            storage,
            cloudflare,
            base_path,
            server_path_cache,
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
                // Relaxed ordering is sufficient for simple flag check
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

                for server_config in &servers {
                    if !server_config.enabled {
                        continue;
                    }
                    self.rescan_server(server_config, base_path.as_ref()).await;
                }
            }
        }
    }

    /// Runs file watcher loop for continuous monitoring (event-driven instead of polling)
    async fn run_file_watcher_loop(&self) {
        // Check if file watcher is enabled
        let (enabled, debounce_ms) = {
            let config = self.config.read().await;
            (
                config.hot_reload.files.enabled,
                config.hot_reload.files.debounce_ms,
            )
        };

        if !enabled {
            tracing::warn!("File watcher hot-reload is disabled, continuous scan will not monitor file changes");
            // Wait indefinitely since the feature is disabled
            std::future::pending::<()>().await;
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Setup file watcher
        let mut watcher = match notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
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
            let server_path = PathBuf::from(base_path.as_ref()).join(server.name.as_ref());
            if server_path.exists() {
                if let Err(e) = watcher.watch(&server_path, RecursiveMode::Recursive) {
                    tracing::warn!("Failed to watch server folder {}: {}", server.name, e);
                }
            }
        }

        // Debounce settings: wait after last event before rescanning
        let debounce_duration = Duration::from_millis(debounce_ms);
        let mut pending_servers: HashSet<String> = HashSet::new();
        let mut debounce_timer: Option<tokio::time::Instant> = None;

        loop {
            tokio::select! {
                // File system event received
                Some(event) = rx.recv() => {
                    // Check if paused (Relaxed ordering sufficient)
                    if self.paused.load(Ordering::Relaxed) {
                        continue;
                    }

                    // Determine which server(s) are affected using O(1) cache lookup
                    for path in event.paths {
                        if let Some(server_name) = self.server_path_cache.find_server(&path) {
                            pending_servers.insert(server_name);
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
                        let (servers, base_path) = {
                            let config = self.config.read().await;
                            (config.servers.clone(), config.server.base_path.clone())
                        };

                        // O(1) HashMap lookup instead of O(n) find
                        let servers_map: HashMap<_, _> = servers
                            .iter()
                            .map(|s| (s.name.as_ref(), s))
                            .collect();

                        for server_name in pending_servers.drain() {
                            if let Some(server_config) = servers_map.get(server_name.as_str()) {
                                if server_config.enabled {
                                    tracing::debug!("File change detected, rescanning server: {}", server_name);
                                    self.rescan_server(server_config, base_path.as_ref()).await;
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
        base_path: &str,
    ) {
        let (batch_config, buffer_size) = {
            let config = self.config.read().await;
            (config.cache.batch.clone(), config.cache.checksum_buffer_size)
        };

        if let Some(storage) = &self.storage {
            match ServerScanner::scan_server_silent(server_config, storage, base_path, &batch_config, buffer_size).await {
                Ok(builder) => {
                    self.update_cache_if_changed(server_config, builder).await;
                }
                Err(_) => {
                    // Silent error - server may be incomplete or removed
                }
            }
        }
    }

    /// Updates cache if changes are detected
    async fn update_cache_if_changed(
        &self,
        server_config: &ServerConfig,
        new_builder: VersionBuilder,
    ) {
        let old_builder = self.cache.get(&server_config.name);

        // Compute granular changes using FileDiff
        let diff = super::file_diff::FileDiff::compute(
            &server_config.name,
            old_builder.as_ref().map(|arc| arc.as_ref()),
            &new_builder,
        );

        let has_changes = !diff.added.is_empty()
            || !diff.modified.is_empty()
            || !diff.removed.is_empty();

        if has_changes {
            // Sync with cloud storage if configured
            if let Some(storage) = &self.storage {
                if storage.is_remote() {
                    if let Err(e) = self.sync_cloud_storage(&server_config.name, &diff).await {
                        tracing::error!(
                            "Failed to sync cloud storage for server {}: {}",
                            server_config.name,
                            e
                        );
                    }
                }
            }

            let is_new = old_builder.is_none();

            // Update URL map incrementally (more efficient than full rebuild)
            let mut new_builder_mut = new_builder;
            if is_new {
                // First scan: build full URL map
                new_builder_mut.build_url_map();
            } else {
                // Incremental update: apply only the changes
                diff.apply_to_url_map(&mut new_builder_mut);
            }

            self.cache.insert(server_config.name.to_string(), Arc::new(new_builder_mut));
            self.last_updated.insert(server_config.name.to_string(), get_current_timestamp());

            // Purge Cloudflare cache
            if let Some(cloudflare) = &self.cloudflare {
                if let Err(e) = cloudflare.purge_cache(&server_config.name).await {
                    tracing::warn!(
                        "Failed to purge Cloudflare cache for {}: {}",
                        server_config.name,
                        e
                    );
                }
            }

            if is_new {
                self.events.emit(AppEvent::CacheNew {
                    server: server_config.name.to_string(),
                });
            } else {
                let change_summary = format!(
                    "{} added, {} modified, {} removed",
                    diff.added.len(),
                    diff.modified.len(),
                    diff.removed.len()
                );
                self.events.emit(AppEvent::CacheUpdated {
                    server: server_config.name.to_string(),
                    changes: vec![change_summary],
                });
            }
        } else {
            self.events.emit(AppEvent::CacheUnchanged {
                server: server_config.name.to_string(),
            });
        }
    }

    /// Synchronizes files with cloud storage (upload added/modified, delete removed)
    async fn sync_cloud_storage(
        &self,
        server_name: &str,
        diff: &super::file_diff::FileDiff,
    ) -> Result<()> {
        let storage = self.storage.as_ref().unwrap();

        tracing::info!(
            "Syncing cloud storage for {}: {} added, {} modified, {} removed",
            server_name,
            diff.added.len(),
            diff.modified.len(),
            diff.removed.len()
        );

        // Upload added and modified files in parallel
        let upload_tasks: Vec<_> = diff
            .added
            .iter()
            .chain(diff.modified.iter())
            .map(|change| {
                let storage = Arc::clone(storage);
                let local_path = self.base_path.join(&change.local_path);
                let remote_key = change.remote_key.clone();

                tokio::spawn(async move {
                    tracing::debug!("Uploading: {}", remote_key);
                    storage.upload_file(&local_path, &remote_key).await
                })
            })
            .collect();

        for task in upload_tasks {
            task.await??;
        }

        // Delete removed files in parallel
        let delete_tasks: Vec<_> = diff
            .removed
            .iter()
            .map(|change| {
                let storage = Arc::clone(storage);
                let remote_key = change.remote_key.clone();

                tokio::spawn(async move {
                    tracing::debug!("Deleting: {}", remote_key);
                    storage.delete_file(&remote_key).await
                })
            })
            .collect();

        for task in delete_tasks {
            task.await??;
        }

        tracing::info!("Cloud storage sync complete for {}", server_name);
        Ok(())
    }

    /// Scans all enabled servers initially
    pub async fn scan_all_servers(&self) -> Result<()> {
        let storage = self.storage.as_ref()
            .ok_or_else(|| CacheError::CacheOperationFailed("Storage backend not initialized".to_string()))?;

        let (servers, base_path, batch_config, buffer_size) = {
            let config = self.config.read().await;
            (
                config.servers.clone(),
                config.server.base_path.clone(),
                config.cache.batch.clone(),
                config.cache.checksum_buffer_size,
            )
        };

        let scan_futures: Vec<_> = servers
            .iter()
            .filter(|server_config| server_config.enabled)
            .map(|server_config| {
                let config = server_config.clone();
                let storage = Arc::clone(storage);
                let base_path = base_path.clone();
                let batch_config = batch_config.clone();
                let buffer_size = buffer_size;
                async move {
                    let result = ServerScanner::scan_server(&config, &storage, base_path.as_ref(), &batch_config, buffer_size).await;
                    (config.name.clone(), result)
                }
            })
            .collect();

        let results = futures::future::join_all(scan_futures).await;

        // Update cache with results
        for (server_name, result) in results {
            match result {
                Ok(mut builder) => {
                    // Build URL map for initial scan
                    builder.build_url_map();
                    self.cache.insert(server_name.to_string(), Arc::new(builder));
                    self.last_updated.insert(server_name.to_string(), get_current_timestamp());
                    self.events.emit(AppEvent::CacheNew { server: server_name.to_string() });
                }
                Err(e) => {
                    // Server scan failed (probably empty folders), add empty version to cache anyway
                    tracing::warn!("Server {} initial scan failed (probably empty), adding empty version to cache: {}", server_name, e);

                    // Get server config to create empty builder
                    let (server_config, _) = {
                        let config = self.config.read().await;
                        let server_config = config.servers.iter()
                            .find(|s| s.name.as_ref() == server_name.as_ref())
                            .cloned();
                        (server_config, config.server.base_path.clone())
                    };

                    if let Some(config) = server_config {
                        let mut empty_builder = VersionBuilder {
                            main_class: lighty_models::MainClass {
                                main_class: config.main_class.clone(),
                            },
                            java_version: lighty_models::JavaVersion {
                                major_version: config.java_version,
                            },
                            arguments: lighty_models::Arguments {
                                game: config.game_args.clone(),
                                jvm: config.jvm_args.clone(),
                            },
                            libraries: Vec::new(),
                            mods: Vec::new(),
                            natives: None,
                            client: None,
                            assets: Vec::new(),
                            url_to_path_map: std::collections::HashMap::new(),
                        };
                        empty_builder.build_url_map();
                        self.cache.insert(server_name.to_string(), Arc::new(empty_builder));
                        self.last_updated.insert(server_name.to_string(), get_current_timestamp());
                        self.events.emit(AppEvent::CacheNew { server: server_name.to_string() });
                    } else {
                        self.events.emit(AppEvent::Error {
                            context: format!("Failed to scan server {}", server_name),
                            error: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Forces a rescan of a specific server
    pub async fn force_rescan_server(&self, server_name: &str) -> Result<()> {
        let storage = self.storage.as_ref()
            .ok_or_else(|| CacheError::CacheOperationFailed("Storage backend not initialized".to_string()))?;

        let (server_config, base_path, batch_config, buffer_size) = {
            let config = self.config.read().await;
            let server_config = config
                .servers
                .iter()
                .find(|s| s.name.as_ref() == server_name)
                .ok_or_else(|| CacheError::ServerNotFound(server_name.to_string()))?
                .clone();
            (
                server_config,
                config.server.base_path.clone(),
                config.cache.batch.clone(),
                config.cache.checksum_buffer_size,
            )
        };

        // Try to scan the server, but add it to cache even if scan fails (empty server)
        match ServerScanner::scan_server(&server_config, storage, base_path.as_ref(), &batch_config, buffer_size).await {
            Ok(mut builder) => {
                // Build URL map for forced rescan
                builder.build_url_map();
                self.cache.insert(server_name.to_string(), Arc::new(builder));
                self.last_updated.insert(server_name.to_string(), get_current_timestamp());
                tracing::info!("✓ Successfully rescanned server: {}", server_name);
            }
            Err(e) => {
                // Server scan failed (probably empty folders), add empty version to cache anyway
                tracing::warn!("Server {} scan failed (probably empty), adding empty version to cache: {}", server_name, e);
                let mut empty_builder = VersionBuilder {
                    main_class: lighty_models::MainClass {
                        main_class: server_config.main_class.clone(),
                    },
                    java_version: lighty_models::JavaVersion {
                        major_version: server_config.java_version,
                    },
                    arguments: lighty_models::Arguments {
                        game: server_config.game_args.clone(),
                        jvm: server_config.jvm_args.clone(),
                    },
                    libraries: Vec::new(),
                    mods: Vec::new(),
                    natives: None,
                    client: None,
                    assets: Vec::new(),
                    url_to_path_map: std::collections::HashMap::new(),
                };
                empty_builder.build_url_map();
                self.cache.insert(server_name.to_string(), Arc::new(empty_builder));
                self.last_updated.insert(server_name.to_string(), get_current_timestamp());
            }
        }

        Ok(())
    }
}
