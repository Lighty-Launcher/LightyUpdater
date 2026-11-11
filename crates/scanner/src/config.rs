use super::errors::ScanError;
use lighty_models::ConfigFile;
use lighty_storage::StorageBackend;
use lighty_utils::{normalize_path, compute_sha1_with_size};
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;
use tokio::sync::Semaphore;
use futures::stream::{self, StreamExt};

type Result<T> = std::result::Result<T, ScanError>;

pub async fn scan_config(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, concurrency: usize, buffer_size: usize) -> Result<Vec<ConfigFile>> {
    let config_dir = path.join("config");

    if !config_dir.exists() {
        return Ok(vec![]);
    }

    if concurrency == 0 {
        return Err(ScanError::InvalidMetadata(
            "Config scan concurrency must be greater than 0".to_string(),
        ));
    }

    let file_paths = WalkDir::new(&config_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.path().to_path_buf());

    // Create semaphore to control concurrency
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let server = server.to_string();
    let storage = Arc::clone(storage);

    // Process all config files concurrently with semaphore control
    let results: Vec<Result<ConfigFile>> = stream::iter(file_paths)
        .map(|file_path| {
            let semaphore = Arc::clone(&semaphore);
            let config_dir = config_dir.clone();
            let server = server.clone();
            let storage = Arc::clone(&storage);

            async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.map_err(|_| {
                    ScanError::InvalidMetadata(
                        "Failed to acquire semaphore permit for config scan".to_string(),
                    )
                })?;

                let relative = file_path
                    .strip_prefix(&config_dir)
                    .map_err(|error| ScanError::InvalidMetadata(format!("Failed to strip prefix: {}", error)))?;

                let name = file_path
                    .file_name()
                    .ok_or_else(|| ScanError::InvalidMetadata("Failed to get filename".to_string()))?
                    .to_string_lossy()
                    .to_string();

                // Async hash computation
                let (sha1, size) = compute_sha1_with_size(&file_path, buffer_size).await?;

                let url_path = normalize_path(relative);
                let remote_key = format!("{}/config/{}", server, url_path);
                let url = storage.get_url(&remote_key);

                Ok(ConfigFile {
                    name,
                    url,
                    path: url_path,
                    sha1,
                    size,
                })
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    let mut config_files: Vec<ConfigFile> = results
        .into_iter()
        .filter_map(|result| {
            result
                .inspect_err(|error| tracing::warn!("Skipping config file: {}", error))
                .ok()
        })
        .collect();

    // buffer_unordered yields in completion order; a stable order keeps rescans from flagging false changes
    config_files.sort_unstable_by(|left, right| left.path.cmp(&right.path));

    Ok(config_files)
}
