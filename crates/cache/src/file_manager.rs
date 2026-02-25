use super::models::{FileCacheManager, FileCache};
use super::errors::CacheError;
use lighty_config::ServerConfig;
use lighty_filesystem::FileSystem;
use dashmap::DashMap;
use futures::stream::{self, StreamExt};
use moka::future::Cache;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::broadcast;
use walkdir::WalkDir;

type Result<T> = std::result::Result<T, CacheError>;

impl FileCacheManager {
    /// Creates a new FileCacheManager with specified capacity in GB (0 = unlimited)
    pub fn new(max_capacity_gb: u64, shutdown_tx: broadcast::Sender<()>) -> Self {
        let cache = if max_capacity_gb == 0 {
            // Unlimited capacity
            Cache::builder()
                .weigher(|_key: &Arc<str>, value: &FileCache| -> u32 {
                    file_cache_weight_bytes(value)
                })
                .build()
        } else {
            // Limited capacity
            let max_capacity_bytes = max_capacity_gb.saturating_mul(1024 * 1024 * 1024);
            Cache::builder()
                .max_capacity(max_capacity_bytes)
                .weigher(|_key: &Arc<str>, value: &FileCache| -> u32 {
                    file_cache_weight_bytes(value)
                })
                .build()
        };

        Self {
            cache,
            shutdown_tx,
            tasks: Arc::new(DashMap::new()),
            task_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Retrieves a file from cache
    pub async fn get_file(&self, server: &str, path: &str) -> Option<FileCache> {
        let key = cache_key(server, path);
        self.cache.get(&key).await
    }

    /// Adds a file to the cache
    async fn add_file(&self, server: &str, path: &str, file: FileCache) -> Result<()> {
        let key = cache_key(server, path);
        self.cache.insert(key, file).await;
        Ok(())
    }

    /// Invalidates one cached file entry.
    pub async fn invalidate_file(&self, server: &str, path: &str) {
        let key = cache_key(server, path);
        self.cache.invalidate(&key).await;
    }

    /// Refreshes one cache entry from disk; if the file is missing, only invalidates.
    pub async fn refresh_file_from_disk(
        &self,
        server: &str,
        path: &str,
        full_path: &Path,
    ) -> Result<()> {
        if !full_path.exists() {
            self.invalidate_file(server, path).await;
            return Ok(());
        }

        let path_buf = full_path.to_path_buf();
        let file_cache = tokio::task::spawn_blocking(move || FileCache::from_file_sync(&path_buf))
            .await??;

        self.add_file(server, path, file_cache).await
    }

    /// Invalidates all cached files for one server.
    pub async fn invalidate_server(&self, server: &str) {
        let prefix = format!("{}/", server);
        let keys: Vec<Arc<str>> = self
            .cache
            .iter()
            .filter_map(|(key, _)| {
                let cache_key = key.as_ref();
                if cache_key.starts_with(prefix.as_str()) {
                    Some(Arc::clone(cache_key))
                } else {
                    None
                }
            })
            .collect();

        for key in keys {
            self.cache.invalidate(&key).await;
        }
    }

    /// Gets cache statistics (entry count and weighted size in KB)
    pub fn get_stats(&self) -> (u64, u64) {
        let entry_count = self.cache.entry_count();
        let weighted_size_kb = self.cache.weighted_size() / 1024;
        (entry_count, weighted_size_kb)
    }

    /// Loads all files from all servers into cache (partial success: continues even if some fail)
    pub async fn load_all_servers(
        &self,
        servers: &[Arc<ServerConfig>],
        base_path: &str,
        server_parallelism: usize,
    ) -> Result<()> {
        let enabled_servers: Vec<_> = servers
            .iter()
            .filter(|s| s.enabled)
            .cloned()
            .collect();

        if server_parallelism == 0 {
            return Err(CacheError::CacheOperationFailed(
                "cache.hash_concurrency must be greater than 0".to_string(),
            ));
        }
        let concurrency = server_parallelism;
        let base_path = base_path.to_string();

        let results: Vec<_> = stream::iter(enabled_servers.into_iter())
            .map(|server_config| {
                let base_path = base_path.clone();
                async move {
                    let server_name = server_config.name.clone();
                    let result = self
                        .load_server_files(server_config.as_ref(), base_path.as_ref())
                        .await;
                    (server_name, result)
                }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        // Collect successes and failures (partial success)
        let mut success_count = 0;
        let mut failures = Vec::new();

        for (server_name, result) in results {
            match result {
                Ok(_) => {
                    success_count += 1;
                    tracing::debug!("Successfully loaded files for server: {}", server_name);
                }
                Err(e) => {
                    tracing::warn!("Failed to load files for server '{}': {}", server_name, e);
                    failures.push((server_name, e.to_string()));
                }
            }
        }

        if success_count > 0 {
            tracing::info!(
                "Loaded {} of {} servers into cache",
                success_count,
                success_count + failures.len()
            );
        }

        if !failures.is_empty() {
            tracing::warn!("Failed to load {} server(s): {:?}", failures.len(), failures.iter().map(|(n, _)| n).collect::<Vec<_>>());
        }

        // Return Ok even with partial failures (at least some servers loaded)
        Ok(())
    }

    /// Loads all files from a single server into cache
    async fn load_server_files(
        &self,
        server_config: &ServerConfig,
        base_path: &str,
    ) -> Result<()> {
        let server_path = FileSystem::build_server_path(base_path, &server_config.name);

        // Collect all files to cache
        let files: Vec<_> = WalkDir::new(&server_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                // Only cache .jar, .json, and asset files
                let path = e.path();
                path.extension().is_some_and(|ext| ext == "jar" || ext == "json")
                    || path.starts_with(server_path.join("assets"))
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        // Load files in parallel using thread pool
        let server_name = server_config.name.clone();
        let base_path_clone = server_path.clone();

        let results: Vec<_> = tokio::task::spawn_blocking(move || {
            use rayon::prelude::*;
            files
                .par_iter()
                .filter_map(|file_path| {
                    let relative_path = file_path
                        .strip_prefix(&base_path_clone)
                        .ok()?
                        .to_string_lossy()
                        .replace('\\', "/");

                    // Load file synchronously in thread pool
                    let file_cache = FileCache::from_file_sync(file_path).ok()?;

                    Some((relative_path, file_cache))
                })
                .collect::<Vec<_>>()
        })
        .await?;

        // Add to cache
        for (path, file) in results {
            self.add_file(&server_name, &path, file).await?;
        }

        Ok(())
    }

    /// Waits for all background tasks to complete
    pub async fn shutdown(&self) {
        tracing::info!("FileCacheManager: Shutting down gracefully...");

        // Collect task IDs (small allocation, necessary with DashMap)
        let task_ids: Vec<usize> = self.tasks.iter().map(|entry| *entry.key()).collect();

        // Remove and await each handle
        for task_id in task_ids {
            if let Some((_, handle)) = self.tasks.remove(&task_id) {
                let _ = handle.await;
            }
        }

        tracing::info!("FileCacheManager: All tasks shut down gracefully");
    }
}

fn cache_key(server: &str, path: &str) -> Arc<str> {
    format!("{}/{}", server, path).into()
}

fn file_cache_weight_bytes(value: &FileCache) -> u32 {
    value.memory_usage().min(u32::MAX as u64) as u32
}
