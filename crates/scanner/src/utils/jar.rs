use super::super::models::{JarScanner, JarFileInfo, FileInfo};
use super::super::errors::ScanError;
use lighty_storage::StorageBackend;
use lighty_utils::{compute_sha1_with_size, normalize_path};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use tokio::sync::Semaphore;
use futures::stream::{self, StreamExt};

type Result<T> = std::result::Result<T, ScanError>;

impl JarScanner {
    pub fn new(base_dir: PathBuf, server: String, storage: Arc<dyn StorageBackend>, batch_size: usize) -> Self {
        Self {
            base_dir,
            server,
            storage,
            batch_size,
        }
    }

    /// Scan directory for JAR files and process them with controlled concurrency
    pub async fn scan<T, F>(self, mapper: F, buffer_size: usize) -> Result<Vec<T>>
    where
        T: Send + 'static,
        F: Fn(JarFileInfo) -> Result<T> + Send + Sync + 'static,
    {
        if !self.base_dir.exists() {
            return Ok(vec![]);
        }

        // Collect all jar file paths first (sync operation)
        let jar_paths: Vec<PathBuf> = WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| is_jar_file(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect();

        // Create semaphore to control concurrency
        let semaphore = Arc::new(Semaphore::new(self.batch_size));
        let base_dir = self.base_dir;
        let server = self.server;
        let storage = self.storage;
        let mapper = std::sync::Arc::new(mapper);

        // Process files concurrently with semaphore control
        let results: Vec<Result<T>> = stream::iter(jar_paths)
            .map(|jar_path| {
                let sem = Arc::clone(&semaphore);
                let base_dir = base_dir.clone();
                let server = server.clone();
                let storage = Arc::clone(&storage);
                let mapper = Arc::clone(&mapper);
                let buffer_size = buffer_size;

                async move {
                    // Acquire semaphore permit
                    let _permit = sem.acquire().await.unwrap();

                    let relative = jar_path
                        .strip_prefix(&base_dir)
                        .map_err(|e| ScanError::InvalidMetadata(format!("Failed to strip prefix: {}", e)))?;

                    // Async hash computation
                    let (sha1, size) = compute_sha1_with_size(&jar_path, buffer_size).await?;

                    let file_name = jar_path
                        .file_name()
                        .ok_or_else(|| ScanError::InvalidMetadata("Failed to get filename".to_string()))?
                        .to_string_lossy()
                        .to_string();

                    let url_path = normalize_path(relative);
                    let remote_key = format!("{}/{}", server, url_path);
                    let url = storage.get_url(&remote_key);

                    let info = JarFileInfo {
                        file_name,
                        relative_path: relative.to_path_buf(),
                        url,
                        url_path,
                        sha1,
                        size,
                    };

                    mapper(info)
                }
            })
            .buffer_unordered(self.batch_size)
            .collect()
            .await;

        // Filter out errors and collect successful results
        Ok(results.into_iter().filter_map(|r| r.ok()).collect())
    }
}

fn is_jar_file(path: &Path) -> bool {
    path.is_file() && path.extension().map_or(false, |ext| ext == "jar")
}

/// Scan files with a custom filter and processor (async with concurrency control)
pub async fn scan_files_parallel<T, Filter, Mapper>(
    base_dir: PathBuf,
    server: String,
    storage: Arc<dyn StorageBackend>,
    filter: Filter,
    mapper: Mapper,
    concurrency: usize,
    buffer_size: usize,
) -> Result<Vec<T>>
where
    T: Send + 'static,
    Filter: Fn(&Path) -> bool + Send + Sync + 'static,
    Mapper: Fn(FileInfo) -> Result<T> + Send + Sync + 'static,
{
    if !base_dir.exists() {
        return Ok(vec![]);
    }

    // Collect all matching file paths
    let file_paths: Vec<PathBuf> = WalkDir::new(&base_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| filter(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    // Create semaphore to control concurrency
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mapper = Arc::new(mapper);

    // Process all files concurrently with semaphore control
    let results: Vec<Result<T>> = stream::iter(file_paths)
        .map(|file_path| {
            let sem = Arc::clone(&semaphore);
            let base_dir = base_dir.clone();
            let server = server.clone();
            let storage = Arc::clone(&storage);
            let mapper = Arc::clone(&mapper);
            let buffer_size = buffer_size;

            async move {
                // Acquire semaphore permit
                let _permit = sem.acquire().await.unwrap();

                let relative = file_path
                    .strip_prefix(&base_dir)
                    .map_err(|e| ScanError::InvalidMetadata(format!("Failed to strip prefix: {}", e)))?;

                // Async hash computation
                let (sha1, size) = compute_sha1_with_size(&file_path, buffer_size).await?;

                let file_name = file_path
                    .file_name()
                    .ok_or_else(|| ScanError::InvalidMetadata("Failed to get filename".to_string()))?
                    .to_string_lossy()
                    .to_string();

                let url_path = normalize_path(relative);
                let remote_key = format!("{}/{}", server, url_path);
                let url = storage.get_url(&remote_key);

                let info = FileInfo {
                    file_name,
                    relative_path: relative.to_path_buf(),
                    url,
                    url_path,
                    sha1,
                    size,
                };

                mapper(info)
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}
