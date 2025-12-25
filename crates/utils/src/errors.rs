use thiserror::Error;

#[derive(Error, Debug)]
pub enum UtilsError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to compute hash for file: {0}")]
    HashError(String),

    #[error("Invalid path: {0}")]
    PathError(String),

    #[error("Path conversion error: {0}")]
    PathConversionError(String),
}
