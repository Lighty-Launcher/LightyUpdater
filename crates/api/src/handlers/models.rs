use lighty_cache::CacheManager;
use lighty_config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub(super) cache: Arc<CacheManager>,
    pub(super) config: Arc<RwLock<Config>>,
}
