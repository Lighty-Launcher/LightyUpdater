use lighty_config::Config;
use lighty_events::EventBus;
use lighty_models::VersionBuilder;
use bytes::Bytes;
use dashmap::DashMap;
use moka::future::Cache;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

/// Trait for updating the version cache (allows decoupling from internal implementation)
pub trait CacheUpdater: Send + Sync {
    /// Insert or update a server version in the cache
    fn insert(&self, server_name: String, version: Arc<VersionBuilder>);

    /// Get a server version from the cache
    fn get(&self, server_name: &str) -> Option<Arc<VersionBuilder>>;

    /// Check if a server exists in the cache
    fn contains(&self, server_name: &str) -> bool;
}

/// Simple cache store wrapper (implements CacheUpdater for DashMap)
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

/// Main cache manager coordinating all caching operations
pub struct CacheManager {
    pub(super) cache: Arc<DashMap<String, Arc<VersionBuilder>>>,
    pub(super) file_cache_manager: Arc<FileCacheManager>,
    pub(super) last_updated: Arc<DashMap<String, String>>,
    pub(super) rescan_orchestrator: Arc<RescanOrchestrator>,
    pub config: Arc<RwLock<Config>>,
    pub(super) events: Arc<EventBus>,
    pub(super) shutdown_tx: broadcast::Sender<()>,
    pub(super) tasks: Arc<DashMap<usize, JoinHandle<()>>>,
    pub(super) task_counter: Arc<std::sync::atomic::AtomicUsize>,
}

/// Manages file caching using Moka LRU cache
pub struct FileCacheManager {
    pub(super) cache: Cache<Arc<str>, FileCache>,
    #[allow(dead_code)]
    pub(super) shutdown_tx: broadcast::Sender<()>,
    pub(super) tasks: Arc<DashMap<usize, JoinHandle<()>>>,
    #[allow(dead_code)]
    pub(super) task_counter: Arc<std::sync::atomic::AtomicUsize>,
}

/// Represents a cached file with its data and metadata
#[derive(Clone)]
pub struct FileCache {
    pub data: Bytes,
    pub sha1: String,
    pub size: u64,
    pub mime_type: String,
}

/// Detects changes between two VersionBuilder instances
pub struct ChangeDetector;

/// Orchestrates automatic and manual server rescanning
pub struct RescanOrchestrator {
    pub(super) cache: Arc<dyn CacheUpdater>,
    pub(super) last_updated: Arc<DashMap<String, String>>,
    pub(super) config: Arc<RwLock<Config>>,
    pub(super) events: Arc<EventBus>,
    pub(super) paused: Arc<AtomicBool>,
}
