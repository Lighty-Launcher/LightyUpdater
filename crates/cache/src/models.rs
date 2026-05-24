use dashmap::DashMap;
use lighty_config::Config;
use lighty_events::EventBus;
use lighty_file_cache::FileCacheManager;
use lighty_models::VersionBuilder;
use lighty_rescan::{RescanOrchestrator, ServerPathCache};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

pub struct CacheManager {
    pub(crate) cache: Arc<DashMap<String, Arc<VersionBuilder>>>,
    pub(crate) file_cache_manager: Arc<FileCacheManager>,
    pub(crate) last_updated: Arc<DashMap<String, String>>,
    pub(crate) rescan_orchestrator: Arc<RescanOrchestrator>,
    pub(crate) server_path_cache: Arc<ServerPathCache>,
    pub config: Arc<RwLock<Config>>,
    pub(crate) events: Arc<EventBus>,
    pub(crate) shutdown_tx: broadcast::Sender<()>,
    pub(crate) tasks: Arc<DashMap<usize, JoinHandle<()>>>,
    pub(crate) task_counter: Arc<std::sync::atomic::AtomicUsize>,
}
