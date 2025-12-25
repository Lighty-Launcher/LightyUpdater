use super::errors::CacheError;
use bytes::Bytes;
use std::path::Path;
use super::models::FileCache;

type Result<T> = std::result::Result<T, CacheError>;

impl FileCache {
    pub fn from_file_sync(path: &Path) -> Result<Self> {
        // Read file into memory synchronously
        let data = std::fs::read(path)?;
        let size = data.len() as u64;

        // Calculate SHA1
        let sha1 = {
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            hasher.update(&data);
            format!("{:x}", hasher.finalize())
        };

        // Get MIME type
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
