use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    StorageError(#[from] lighty_storage::StorageError),

    #[error("Utils error: {0}")]
    UtilsError(#[from] lighty_utils::UtilsError),

    #[error("Join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Server folder does not exist: {0}")]
    ServerFolderNotFound(String),

    #[error("Invalid JAR file: {0}")]
    InvalidJar(String),

    #[error("Failed to scan directory: {0}")]
    ScanDirectoryError(String),

    #[error("Failed to compute hash: {0}")]
    HashError(String),

    #[error("Invalid file metadata: {0}")]
    InvalidMetadata(String),
}
