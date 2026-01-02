use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Config error: {0}")]
    ConfigError(#[from] lighty_config::ConfigError),

    #[error("Cache error: {0}")]
    CacheError(#[from] lighty_cache::CacheError),

    #[error("Notify error: {0}")]
    NotifyError(#[from] notify::Error),

    #[error("File system error: {0}")]
    FileSystemError(String),

    #[error("Watch operation failed: {0}")]
    WatchFailed(String),
}
