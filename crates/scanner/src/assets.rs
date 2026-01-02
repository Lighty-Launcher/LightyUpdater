use super::errors::ScanError;
use lighty_models::Asset;
use lighty_storage::StorageBackend;
use lighty_utils::{normalize_path, compute_sha1_with_size};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use tokio::sync::Semaphore;
use futures::stream::{self, StreamExt};

type Result<T> = std::result::Result<T, ScanError>;

pub async fn scan_assets(path: &Path, server: &str, storage: &Arc<dyn StorageBackend>, concurrency: usize, buffer_size: usize) -> Result<Vec<Asset>> {
    let assets_dir = path.join("assets");

    if !assets_dir.exists() {
        return Ok(vec![]);
    }

    // Collect all file paths in assets directory
    let file_paths: Vec<PathBuf> = WalkDir::new(&assets_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    // Create semaphore to control concurrency
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let server = server.to_string();
    let storage = Arc::clone(storage);

    // Process all assets concurrently with semaphore control
    let results: Vec<Result<Asset>> = stream::iter(file_paths)
        .map(|file_path| {
            let sem = Arc::clone(&semaphore);
            let assets_dir = assets_dir.clone();
            let server = server.clone();
            let storage = Arc::clone(&storage);
            let buffer_size = buffer_size;

            async move {
                // Acquire semaphore permit
                let _permit = sem.acquire().await.unwrap();

                let relative = file_path
                    .strip_prefix(&assets_dir)
                    .map_err(|e| ScanError::InvalidMetadata(format!("Failed to strip prefix: {}", e)))?;

                // Async hash computation
                let (hash, size) = compute_sha1_with_size(&file_path, buffer_size).await?;

                let url_path = normalize_path(relative);
                let remote_key = format!("{}/{}", server, url_path);
                let url = storage.get_url(&remote_key);

                Ok(Asset {
                    hash,
                    size,
                    url: Some(url),
                    path: Some(url_path),
                })
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    // Filter out errors and collect successful results
    let assets: Vec<Asset> = results.into_iter().filter_map(|r| r.ok()).collect();

    Ok(assets)
}
