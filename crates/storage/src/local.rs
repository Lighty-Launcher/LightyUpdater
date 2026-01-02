use crate::backend::StorageBackend;
use crate::StorageError;
use std::path::{Path, PathBuf};

/// Local filesystem storage backend
pub struct LocalBackend {
    base_url: String,
}

impl LocalBackend {
    pub fn new(base_url: String, _base_path: PathBuf) -> Self {
        Self { base_url }
    }
}

#[async_trait::async_trait]
impl StorageBackend for LocalBackend {
    async fn upload_file(&self, _local_path: &Path, remote_key: &str) -> Result<String, StorageError> {
        // No-op for local storage (files already in place)
        Ok(self.get_url(remote_key))
    }

    async fn delete_file(&self, _remote_key: &str) -> Result<(), StorageError> {
        // No-op for local storage (files managed by scanner)
        Ok(())
    }

    fn get_url(&self, remote_key: &str) -> String {
        format!("{}/{}", self.base_url, remote_key)
    }

    fn is_remote(&self) -> bool {
        false
    }
}
