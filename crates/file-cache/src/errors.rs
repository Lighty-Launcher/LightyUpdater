use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileCacheError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
