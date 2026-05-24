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

    #[error(transparent)]
    FileCache(#[from] lighty_file_cache::FileCacheError),

    #[error(transparent)]
    Cdn(#[from] lighty_cdn::CdnError),
}
