use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "s3")]
    #[error("S3 SDK error: {0}")]
    S3SdkError(String),

    #[error("Upload failed for '{0}': {1}")]
    UploadError(String, String),

    #[error("Delete failed for '{0}': {1}")]
    DeleteError(String, String),

    #[error("Invalid storage configuration: {0}")]
    ConfigError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}
