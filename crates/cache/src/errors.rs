use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Rescan(#[from] lighty_rescan::RescanError),

    #[error(transparent)]
    FileCache(#[from] lighty_file_cache::FileCacheError),

    #[error(transparent)]
    Cdn(#[from] lighty_cdn::CdnError),
}
