use super::models::ConfigWatcher;
use lighty_cache::CacheManager;
use lighty_config::Config;
use lighty_filesystem::FileSystem;
use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

impl ConfigWatcher {
    pub fn new(config: Arc<RwLock<Config>>, config_path: String, cache_manager: Arc<CacheManager>) -> Self {
        Self {
            config,
            config_path,
            cache_manager,
        }
    }

    #[allow(dead_code)]
    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn start_watching(self: Arc<Self>) -> Result<tokio::task::JoinHandle<()>> {
        let config_path = self.config_path.clone();
        let config = Arc::clone(&self.config);
        let cache_manager = Arc::clone(&self.cache_manager);

        let handle = tokio::spawn(async move {
            if let Err(e) = Self::watch_config_file(&config_path, config, cache_manager).await {
                tracing::error!("Config watcher error: {}", e);
            }
        });

        Ok(handle)
    }

    async fn watch_config_file(
        config_path: &str,
        config: Arc<RwLock<Config>>,
        cache_manager: Arc<CacheManager>,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher: RecommendedWatcher = notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                    ) {
                        let _ = tx.blocking_send(());
                    }
                }
            },
        )?;

        watcher.watch(Path::new(config_path), RecursiveMode::NonRecursive)?;

        while rx.recv().await.is_some() {
            // Get debounce time from current config
            let debounce_ms = {
                let config_read = config.read().await;
                config_read.cache.config_watch_debounce_ms
            };

            tokio::time::sleep(tokio::time::Duration::from_millis(debounce_ms)).await;

            // Check if config file still exists
            if !std::path::Path::new(config_path).exists() {
                tracing::warn!("Config file deleted, ignoring event");
                continue;
            }

            match Config::from_file_with_events(config_path, None).await {
                Ok(new_config) => {
                    // CRITICAL: Pause rescan to prevent race condition during config reload
                    cache_manager.pause_rescan();

                    let old_servers = {
                        let config_read = config.read().await;
                        config_read
                            .servers
                            .iter()
                            .map(|s| s.name.clone())
                            .collect::<HashSet<_>>()
                    };

                    let new_servers = new_config
                        .servers
                        .iter()
                        .map(|s| s.name.clone())
                        .collect::<HashSet<_>>();

                    let added_servers: Vec<_> =
                        new_servers.difference(&old_servers).cloned().collect();

                    // Update config with exclusive write lock
                    let mut config_write = config.write().await;
                    *config_write = new_config;

                    // Resume rescan BEFORE dropping lock to avoid race condition
                    cache_manager.resume_rescan();

                    tracing::info!("✓ Configuration reloaded successfully from {}", config_path);

                    if !added_servers.is_empty() {
                        for server_name in &added_servers {
                            if let Some(server_config) =
                                config_write.servers.iter().find(|s| &s.name == server_name)
                            {
                                // Skip if server is disabled
                                if !server_config.enabled {
                                    continue;
                                }

                                tracing::info!("🆕 New server detected: {}", server_name);

                                let base_path = config_write.server.base_path.clone();
                                let folder = server_config.name.clone();

                                // Drop lock before I/O operations
                                drop(config_write);

                                if let Err(e) = FileSystem::ensure_server_structure(&base_path, &folder).await {
                                    tracing::error!(
                                        "Failed to create folders for {}: {}",
                                        server_name,
                                        e
                                    );
                                }

                                if let Err(e) = cache_manager.force_rescan(server_name).await {
                                    tracing::error!("Failed to scan new server {}: {}", server_name, e);
                                }

                                // Re-acquire lock for next iteration
                                config_write = config.write().await;
                            }
                        }
                    }

                    drop(config_write);
                }
                Err(e) => {
                    tracing::error!("Failed to reload config: {}", e);
                }
            }
        }

        Ok(())
    }
}
