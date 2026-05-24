use crate::errors::CacheError;
use crate::models::CacheManager;
use dashmap::DashMap;
use lighty_config::{Config, ServerConfig};
use lighty_events::{AppEvent, EventBus};
use lighty_file_cache::{FileCache, FileCacheManager};
use lighty_models::VersionBuilder;
use lighty_rescan::{
    CacheStore, CacheUpdater, RescanOrchestrator, RescanOrchestratorDeps, ServerPathCache,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

type Result<T> = std::result::Result<T, CacheError>;

impl CacheManager {
    pub async fn new(
        config: Arc<RwLock<Config>>,
        events: Arc<EventBus>,
        storage: Option<Arc<dyn lighty_storage::StorageBackend>>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        let (cache_store, cache) = CacheStore::new();
        let last_updated = Arc::new(DashMap::new());

        let (max_cache_gb, base_path, servers) = {
            let config_read = config.read().await;
            (
                config_read.cache.max_memory_cache_gb,
                std::path::PathBuf::from(config_read.server.base_path.as_ref()),
                config_read.servers.clone(),
            )
        };

        let file_cache_manager = Arc::new(FileCacheManager::new(max_cache_gb, shutdown_tx.clone()));

        let server_path_cache = Arc::new(ServerPathCache::new());
        server_path_cache.rebuild(&servers, &base_path.to_string_lossy());

        let rescan_orchestrator = Arc::new(RescanOrchestrator::new(RescanOrchestratorDeps {
            cache: Arc::new(cache_store),
            file_cache_manager: Arc::clone(&file_cache_manager),
            last_updated: Arc::clone(&last_updated),
            config: Arc::clone(&config),
            events: Arc::clone(&events),
            storage,
            base_path,
            server_path_cache: Arc::clone(&server_path_cache),
        }));

        Self {
            cache,
            file_cache_manager,
            last_updated,
            rescan_orchestrator,
            server_path_cache,
            config,
            events,
            shutdown_tx,
            tasks: Arc::new(DashMap::new()),
            task_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn pause_rescan(&self) {
        self.rescan_orchestrator.pause();
    }

    pub fn resume_rescan(&self) {
        self.rescan_orchestrator.resume();
    }

    pub async fn rebuild_server_cache(&self) {
        let (servers, base_path) = {
            let config = self.config.read().await;
            (config.servers.clone(), config.server.base_path.clone())
        };
        self.server_path_cache.rebuild(&servers, base_path.as_ref());
        tracing::debug!("Server path cache rebuilt after config reload");
    }

    pub fn rebuild_server_cache_with_data(&self, servers: &[Arc<ServerConfig>], base_path: &str) {
        self.server_path_cache.rebuild(servers, base_path);
        tracing::debug!("Server path cache rebuilt after config reload (with provided data)");
    }

    pub async fn shutdown(&self) {
        tracing::info!("CacheManager: Initiating graceful shutdown...");
        let _ = self.shutdown_tx.send(());

        let handles: Vec<_> = self.tasks.iter().map(|entry| *entry.key()).collect();

        for task_id in handles {
            if let Some((_, handle)) = self.tasks.remove(&task_id) {
                let _ = handle.await;
            }
        }

        self.file_cache_manager.shutdown().await;

        tracing::info!("CacheManager: All tasks shut down gracefully");
    }

    pub async fn get_file(&self, server: &str, path: &str) -> Option<FileCache> {
        self.file_cache_manager.get_file(server, path).await
    }

    pub fn get_cache_stats(&self) -> (u64, u64) {
        self.file_cache_manager.get_stats()
    }

    pub async fn initialize(&self) -> Result<()> {
        let config = self.config.read().await;
        if !config.cache.enabled {
            return Ok(());
        }

        if config.cache.auto_scan {
            let servers = config.servers.clone();
            let base_path = config.server.base_path.clone();
            let server_parallelism = config.cache.hash_concurrency;
            drop(config);

            self.events.emit(AppEvent::InitialScanStarted);
            self.rescan_orchestrator.scan_all_servers().await?;
            self.file_cache_manager
                .load_all_servers(&servers, base_path.as_ref(), server_parallelism)
                .await?;
        }

        Ok(())
    }

    pub async fn start_auto_rescan(&self) {
        let enabled = {
            let config = self.config.read().await;
            config.cache.enabled
        };

        if !enabled {
            return;
        }

        let orchestrator = Arc::clone(&self.rescan_orchestrator);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = orchestrator.run_rescan_loop() => {
                    tracing::info!("Rescan loop ended");
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Rescan loop shutting down gracefully");
                }
            }
        });

        let task_id = self.task_counter.fetch_add(1, Ordering::SeqCst);
        self.tasks.insert(task_id, handle);
    }

    pub async fn get(&self, server_name: &str) -> Option<Arc<VersionBuilder>> {
        self.cache.get(server_name).map(|entry| Arc::clone(entry.value()))
    }

    pub async fn force_rescan(&self, server_name: &str) -> Result<()> {
        self.rescan_orchestrator.force_rescan_server(server_name).await?;
        Ok(())
    }

    pub async fn get_all_servers(&self) -> Vec<String> {
        let config = self.config.read().await;
        config
            .servers
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.name.to_string())
            .collect()
    }

    pub async fn get_server_config(&self, name: &str) -> Option<Arc<ServerConfig>> {
        let config = self.config.read().await;
        config.servers.iter().find(|s| s.name.as_ref() == name).map(Arc::clone)
    }

    pub async fn get_version(&self, name: &str) -> Option<Arc<VersionBuilder>> {
        self.cache.get(name).map(|entry| Arc::clone(entry.value()))
    }

    pub fn get_last_update(&self, name: &str) -> Option<String> {
        self.last_updated.get(name).map(|entry| entry.value().clone())
    }

    pub async fn remove_server(&self, server_name: &str) {
        if self.cache.remove(server_name).is_some() {
            tracing::debug!("Removed server {} from version builder cache", server_name);
        }
        self.file_cache_manager.invalidate_server(server_name).await;
        tracing::debug!("Removed server {} from file cache", server_name);

        if self.last_updated.remove(server_name).is_some() {
            tracing::debug!("Removed server {} from last updated cache", server_name);
        }

        self.server_path_cache.remove_server(server_name);
        tracing::debug!("Removed server {} from server path cache", server_name);
    }
}

impl CacheUpdater for CacheManager {
    fn insert(&self, server_name: String, version: Arc<VersionBuilder>) {
        self.cache.insert(server_name, version);
    }

    fn get(&self, server_name: &str) -> Option<Arc<VersionBuilder>> {
        self.cache.get(server_name).map(|entry| Arc::clone(entry.value()))
    }

    fn contains(&self, server_name: &str) -> bool {
        self.cache.contains_key(server_name)
    }
}
