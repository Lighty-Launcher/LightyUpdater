use crate::errors::FileCacheError;
use bytes::Bytes;
use std::path::Path;

type Result<T> = std::result::Result<T, FileCacheError>;

#[derive(Clone)]
pub struct FileCache {
    pub data: Bytes,
    pub sha1: String,
    pub size: u64,
    pub mime_type: String,
}

impl FileCache {
    pub fn from_file_sync(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let size = data.len() as u64;

        let sha1 = {
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        };

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        Ok(Self {
            data: Bytes::from(data),
            sha1,
            size,
            mime_type,
        })
    }

    pub fn memory_usage(&self) -> u64 {
        self.data.len() as u64
    }
}
