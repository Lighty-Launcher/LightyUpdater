use crate::server_path_cache::ServerPathCache;
use dashmap::DashMap;
use lighty_config::Config;
use lighty_events::EventBus;
use lighty_file_cache::FileCacheManager;
use lighty_models::VersionBuilder;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait CacheUpdater: Send + Sync {
    fn insert(&self, server_name: String, version: Arc<VersionBuilder>);
    fn get(&self, server_name: &str) -> Option<Arc<VersionBuilder>>;
    fn contains(&self, server_name: &str) -> bool;
}

pub struct CacheStore {
    cache: Arc<DashMap<String, Arc<VersionBuilder>>>,
}

impl CacheStore {
    pub fn new() -> (Self, Arc<DashMap<String, Arc<VersionBuilder>>>) {
        let cache = Arc::new(DashMap::new());
        let store = Self {
            cache: Arc::clone(&cache),
        };
        (store, cache)
    }
}

impl CacheUpdater for CacheStore {
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

pub struct ChangeDetector;

pub struct RescanOrchestrator {
    pub(crate) cache: Arc<dyn CacheUpdater>,
    pub(crate) file_cache_manager: Arc<FileCacheManager>,
    pub(crate) last_updated: Arc<DashMap<String, String>>,
    pub(crate) config: Arc<RwLock<Config>>,
    pub(crate) events: Arc<EventBus>,
    pub(crate) paused: Arc<AtomicBool>,
    pub(crate) storage: Option<Arc<dyn lighty_storage::StorageBackend>>,
    pub(crate) base_path: std::path::PathBuf,
    pub(crate) server_path_cache: Arc<ServerPathCache>,
}

pub struct RescanOrchestratorDeps {
    pub cache: Arc<dyn CacheUpdater>,
    pub file_cache_manager: Arc<FileCacheManager>,
    pub last_updated: Arc<DashMap<String, String>>,
    pub config: Arc<RwLock<Config>>,
    pub events: Arc<EventBus>,
    pub storage: Option<Arc<dyn lighty_storage::StorageBackend>>,
    pub base_path: std::path::PathBuf,
    pub server_path_cache: Arc<ServerPathCache>,
}
