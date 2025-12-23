use lighty_models::Asset;
use lighty_utils::{normalize_path, compute_sha1_with_size_sync};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn scan_assets(path: &Path, server: &str, base_url: &str) -> Result<Vec<Asset>> {
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

    // Process all assets in parallel using rayon
    let assets_dir_clone = assets_dir.clone();
    let server = server.to_string();
    let base_url = base_url.to_string();

    let results: Vec<Result<Asset>> = tokio::task::spawn_blocking(move || {
        file_paths
            .par_iter()
            .map(|file_path| {
                let relative = file_path
                    .strip_prefix(&assets_dir_clone)
                    .context("Failed to strip prefix")?;

                let (hash, size) = compute_sha1_with_size_sync(file_path)
                    .context("Failed to compute SHA1 and size")?;

                let url_path = normalize_path(relative);

                Ok(Asset {
                    hash,
                    size,
                    url: Some(format!("{}/{}/{}", base_url, server, url_path)),
                    path: Some(url_path),
                })
            })
            .collect()
    })
    .await?;

    // Filter out errors and collect successful results
    let assets: Vec<Asset> = results.into_iter().filter_map(|r| r.ok()).collect();

    Ok(assets)
}
