use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] lighty_adapters::ConfigIoError),

    #[error("Cache error: {0}")]
    Cache(#[from] lighty_cache::CacheError),

    #[error("Storage error: {0}")]
    Storage(#[from] lighty_storage::StorageError),

    #[error("Watcher error: {0}")]
    Watcher(#[from] lighty_watcher::WatcherError),

    #[error("Filesystem error: {0}")]
    FileSystem(#[from] lighty_filesystem::FileSystemError),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Failed to bind server on {addr}: {source}")]
    BindAddress {
        addr: String,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
