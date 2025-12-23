use lighty_cache::CacheManager;
use std::sync::Arc;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub(super) cache: Arc<CacheManager>,
    pub(super) base_url: Arc<String>,
    pub(super) base_path: Arc<String>,
    pub(super) streaming_threshold_bytes: u64,
}

/// API error types
#[derive(Debug)]
pub enum ApiError {
    ServerNotFound {
        server: String,
        available: Vec<String>,
    },
    NotFound,
    InternalError(String),
    InvalidPath(String),
}
