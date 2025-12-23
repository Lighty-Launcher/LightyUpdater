use lighty_cache::CacheManager;
use lighty_config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConfigWatcher {
    pub(super) config: Arc<RwLock<Config>>,
    pub(super) config_path: String,
    pub(super) cache_manager: Arc<CacheManager>,
}
