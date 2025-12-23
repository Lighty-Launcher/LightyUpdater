use super::super::models::{JarScanner, JarFileInfo, FileInfo};
use lighty_utils::{compute_sha1_with_size_sync, normalize_path};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

impl JarScanner {
    pub fn new(base_dir: PathBuf, server: String, base_url: String, batch_size: usize) -> Self {
        Self {
            base_dir,
            server,
            base_url,
            batch_size,
        }
    }

    /// Scan directory for JAR files and process them in parallel with batching
    pub async fn scan<T, F>(self, mapper: F) -> Result<Vec<T>>
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

        // Process files in batches to improve responsiveness
        let batch_size = self.batch_size;
        let base_dir = self.base_dir;
        let server = self.server;
        let base_url = self.base_url;
        let mapper = std::sync::Arc::new(mapper);

        let mut all_results = Vec::new();

        for chunk in jar_paths.chunks(batch_size) {
            let chunk_owned: Vec<PathBuf> = chunk.to_vec();
            let base_dir = base_dir.clone();
            let server = server.clone();
            let base_url = base_url.clone();
            let mapper = mapper.clone();

            let results: Vec<Result<T>> = tokio::task::spawn_blocking(move || {
                chunk_owned
                    .par_iter()
                    .map(|jar_path| {
                        let relative = jar_path
                            .strip_prefix(&base_dir)
                            .context("Failed to strip prefix")?;

                        let (sha1, size) = compute_sha1_with_size_sync(jar_path)
                            .context("Failed to compute SHA1 and size")?;

                        let file_name = jar_path
                            .file_name()
                            .ok_or_else(|| anyhow::anyhow!("Failed to get filename"))?
                            .to_string_lossy()
                            .to_string();

                        let url_path = normalize_path(relative);
                        let url = format!("{}/{}/{}", base_url, server, url_path);

                        let info = JarFileInfo {
                            file_name,
                            relative_path: relative.to_path_buf(),
                            url,
                            url_path,
                            sha1,
                            size,
                        };

                        mapper(info)
                    })
                    .collect()
            })
            .await?;

            all_results.extend(results);

            // Yield to allow other tasks to run between batches
            tokio::task::yield_now().await;
        }

        // Filter out errors and collect successful results
        Ok(all_results.into_iter().filter_map(|r| r.ok()).collect())
    }
}

fn is_jar_file(path: &Path) -> bool {
    path.is_file() && path.extension().map_or(false, |ext| ext == "jar")
}

/// Scan files with a custom filter and processor
pub async fn scan_files_parallel<T, Filter, Mapper>(
    base_dir: PathBuf,
    server: String,
    base_url: String,
    filter: Filter,
    mapper: Mapper,
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

    // Process all files in parallel
    let base_dir_clone = base_dir.clone();

    let results: Vec<Result<T>> = tokio::task::spawn_blocking(move || {
        file_paths
            .par_iter()
            .map(|file_path| {
                let relative = file_path
                    .strip_prefix(&base_dir_clone)
                    .context("Failed to strip prefix")?;

                let (sha1, size) = compute_sha1_with_size_sync(file_path)
                    .context("Failed to compute SHA1 and size")?;

                let file_name = file_path
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get filename"))?
                    .to_string_lossy()
                    .to_string();

                let url_path = normalize_path(relative);
                let url = format!("{}/{}/{}", base_url, server, url_path);

                let info = FileInfo {
                    file_name,
                    relative_path: relative.to_path_buf(),
                    url,
                    url_path,
                    sha1,
                    size,
                };

                mapper(info)
            })
            .collect()
    })
    .await?;

    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}
