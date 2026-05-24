use thiserror::Error;

#[derive(Error, Debug)]
pub enum RescanError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Scanner error: {0}")]
    Scan(#[from] lighty_scanner::ScanError),

    #[error("Storage error: {0}")]
    Storage(#[from] lighty_storage::StorageError),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    FileCache(#[from] lighty_file_cache::FileCacheError),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
