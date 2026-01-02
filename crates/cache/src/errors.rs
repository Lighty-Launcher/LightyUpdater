use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Scanner error: {0}")]
    ScanError(#[from] lighty_scanner::ScanError),

    #[error("Storage error: {0}")]
    StorageError(#[from] lighty_storage::StorageError),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Cache operation failed: {0}")]
    CacheOperationFailed(String),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Cloudflare API error: {0}")]
    CloudflareError(String),
}

// Convert reqwest errors to CacheError
impl From<reqwest::Error> for CacheError {
    fn from(err: reqwest::Error) -> Self {
        CacheError::HttpError(err.to_string())
    }
}
