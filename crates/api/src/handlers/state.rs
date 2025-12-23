use super::models::AppState;
use lighty_cache::CacheManager;
use axum::extract::FromRef;
use std::sync::Arc;

impl AppState {
    pub fn new(cache: Arc<CacheManager>, base_url: String, base_path: String, streaming_threshold_mb: u64) -> Self {
        Self {
            cache,
            base_url: Arc::new(base_url),
            base_path: Arc::new(base_path),
            streaming_threshold_bytes: streaming_threshold_mb * 1024 * 1024,
        }
    }
}

impl FromRef<AppState> for Arc<CacheManager> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.cache)
    }
}
