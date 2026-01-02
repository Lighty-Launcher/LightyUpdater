use super::models::AppState;
use lighty_cache::CacheManager;
use lighty_config::Config;
use axum::extract::FromRef;
use std::sync::Arc;
use tokio::sync::RwLock;

impl AppState {
    pub fn new(cache: Arc<CacheManager>, config: Arc<RwLock<Config>>) -> Self {
        Self {
            cache,
            config,
        }
    }
}

impl FromRef<AppState> for Arc<CacheManager> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.cache)
    }
}
