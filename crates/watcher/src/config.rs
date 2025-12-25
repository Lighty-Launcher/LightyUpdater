use super::models::ConfigWatcher;
use super::errors::WatcherError;
use lighty_cache::CacheManager;
use lighty_config::{Config, ServerConfig};
use lighty_filesystem::FileSystem;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, WatcherError>;

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

    /// Checks if any significant server config field changed
    fn server_config_changed(old: &ServerConfig, new: &ServerConfig) -> bool {
        old.enabled != new.enabled
            || old.loader != new.loader
            || old.loader_version != new.loader_version
            || old.minecraft_version != new.minecraft_version
            || old.main_class != new.main_class
            || old.java_version != new.java_version
            || old.enable_client != new.enable_client
            || old.enable_libraries != new.enable_libraries
            || old.enable_mods != new.enable_mods
            || old.enable_natives != new.enable_natives
            || old.enable_assets != new.enable_assets
            || old.game_args != new.game_args
            || old.jvm_args != new.jvm_args
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
        // Get channel size from config
        let channel_size = {
            let config_read = config.read().await;
            config_read.cache.config_reload_channel_size
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(channel_size);

        let mut watcher: RecommendedWatcher = notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
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

                    let (old_servers, old_configs) = {
                        let config_read = config.read().await;
                        let names = config_read
                            .servers
                            .iter()
                            .map(|s| s.name.clone())
                            .collect::<HashSet<_>>();
                        let configs = config_read.servers.clone();
                        (names, configs)
                    };

                    let new_servers = new_config
                        .servers
                        .iter()
                        .map(|s| s.name.clone())
                        .collect::<HashSet<_>>();

                    let added_servers: Vec<_> =
                        new_servers.difference(&old_servers).cloned().collect();

                    // Detect modified servers (existing servers with config changes)
                    let mut modified_servers = Vec::new();
                    for new_server in &new_config.servers {
                        if let Some(old_server) = old_configs.iter().find(|s| s.name == new_server.name) {
                            // Check if any config field changed
                            if Self::server_config_changed(old_server, new_server) {
                                modified_servers.push(new_server.name.clone());
                            }
                        }
                    }

                    // Update config with exclusive write lock
                    let mut config_write = config.write().await;
                    *config_write = new_config;

                    // Rebuild server path cache after config update
                    cache_manager.rebuild_server_cache().await;

                    // Resume rescan BEFORE dropping lock to avoid race condition
                    cache_manager.resume_rescan();

                    tracing::info!("✓ Configuration reloaded successfully from {}", config_path);

                    // Rescan modified servers
                    if !modified_servers.is_empty() {
                        for server_name in &modified_servers {
                            tracing::info!("🔄 Server config changed, rescanning: {}", server_name);
                            drop(config_write);
                            if let Err(e) = cache_manager.force_rescan(server_name).await {
                                tracing::error!("Failed to rescan modified server {}: {}", server_name, e);
                            }
                            config_write = config.write().await;
                        }
                    }

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
