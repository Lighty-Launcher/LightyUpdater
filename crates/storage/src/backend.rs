use crate::StorageError;
use std::path::Path;

/// Storage backend trait for file storage abstraction
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload file to storage, returns public URL
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<String, StorageError>;

    /// Delete file from storage
    async fn delete_file(&self, remote_key: &str) -> Result<(), StorageError>;

    /// Get public URL for a file (without uploading)
    fn get_url(&self, remote_key: &str) -> String;

    /// Check if backend is local or remote
    fn is_remote(&self) -> bool;
}
