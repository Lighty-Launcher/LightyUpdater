use crate::errors::FileCacheError;
use crate::file_cache::FileCache;
use dashmap::DashMap;
use futures::stream::{self, StreamExt};
use lighty_config::ServerConfig;
use lighty_filesystem::FileSystem;
use moka::future::Cache;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use walkdir::WalkDir;

type Result<T> = std::result::Result<T, FileCacheError>;

pub struct FileCacheManager {
    cache: Cache<Arc<str>, FileCache>,
    #[allow(dead_code)]
    shutdown_tx: broadcast::Sender<()>,
    tasks: Arc<DashMap<usize, JoinHandle<()>>>,
    #[allow(dead_code)]
    task_counter: Arc<AtomicUsize>,
}

impl FileCacheManager {
    /// Creates a new FileCacheManager with specified capacity in GB (0 = unlimited)
    pub fn new(max_capacity_gb: u64, shutdown_tx: broadcast::Sender<()>) -> Self {
        let cache = if max_capacity_gb == 0 {
            Cache::builder()
                .weigher(|_key: &Arc<str>, value: &FileCache| -> u32 {
                    file_cache_weight_bytes(value)
                })
                .build()
        } else {
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

    pub async fn get_file(&self, server: &str, path: &str) -> Option<FileCache> {
        let key = cache_key(server, path);
        self.cache.get(&key).await
    }

    async fn add_file(&self, server: &str, path: &str, file: FileCache) -> Result<()> {
        let key = cache_key(server, path);
        self.cache.insert(key, file).await;
        Ok(())
    }

    pub async fn invalidate_file(&self, server: &str, path: &str) {
        let key = cache_key(server, path);
        self.cache.invalidate(&key).await;
    }

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

    pub fn get_stats(&self) -> (u64, u64) {
        let entry_count = self.cache.entry_count();
        let weighted_size_kb = self.cache.weighted_size() / 1024;
        (entry_count, weighted_size_kb)
    }

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
            return Err(FileCacheError::InvalidConfig(
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
            tracing::warn!(
                "Failed to load {} server(s): {:?}",
                failures.len(),
                failures.iter().map(|(n, _)| n).collect::<Vec<_>>()
            );
        }

        Ok(())
    }

    async fn load_server_files(
        &self,
        server_config: &ServerConfig,
        base_path: &str,
    ) -> Result<()> {
        let server_path = FileSystem::build_server_path(base_path, &server_config.name);

        let files: Vec<_> = WalkDir::new(&server_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                let path = e.path();
                path.extension().is_some_and(|ext| ext == "jar" || ext == "json")
                    || path.starts_with(server_path.join("assets"))
            })
            .map(|e| e.path().to_path_buf())
            .collect();

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

                    let file_cache = FileCache::from_file_sync(file_path).ok()?;

                    Some((relative_path, file_cache))
                })
                .collect::<Vec<_>>()
        })
        .await?;

        for (path, file) in results {
            self.add_file(&server_name, &path, file).await?;
        }

        Ok(())
    }

    pub async fn shutdown(&self) {
        tracing::info!("FileCacheManager: Shutting down gracefully...");

        let task_ids: Vec<usize> = self.tasks.iter().map(|entry| *entry.key()).collect();

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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn sample(size: usize) -> FileCache {
        let data = vec![0u8; size];
        FileCache {
            data: Bytes::from(data),
            sha1: "sha".to_string(),
            size: size as u64,
            mime_type: "application/octet-stream".to_string(),
        }
    }

    async fn new_manager(max_gb: u64) -> FileCacheManager {
        let (tx, _rx) = broadcast::channel(1);
        FileCacheManager::new(max_gb, tx)
    }

    #[tokio::test]
    async fn insert_then_get_returns_same_entry() {
        let mgr = new_manager(0).await;
        mgr.add_file("srv", "client.jar", sample(128)).await.unwrap();

        let got = mgr.get_file("srv", "client.jar").await.unwrap();
        assert_eq!(got.size, 128);
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let mgr = new_manager(0).await;
        assert!(mgr.get_file("srv", "missing").await.is_none());
    }

    #[tokio::test]
    async fn invalidate_removes_entry() {
        let mgr = new_manager(0).await;
        mgr.add_file("srv", "client.jar", sample(64)).await.unwrap();
        mgr.invalidate_file("srv", "client.jar").await;
        assert!(mgr.get_file("srv", "client.jar").await.is_none());
    }

    #[tokio::test]
    async fn invalidate_server_drops_only_matching_prefix() {
        let mgr = new_manager(0).await;
        mgr.add_file("a", "x.jar", sample(64)).await.unwrap();
        mgr.add_file("b", "y.jar", sample(64)).await.unwrap();

        mgr.invalidate_server("a").await;
        mgr.cache.run_pending_tasks().await;

        assert!(mgr.get_file("a", "x.jar").await.is_none());
        assert!(mgr.get_file("b", "y.jar").await.is_some());
    }

    #[tokio::test]
    async fn refresh_file_from_disk_invalidates_when_missing() {
        let mgr = new_manager(0).await;
        mgr.add_file("srv", "ghost.jar", sample(8)).await.unwrap();

        let nonexistent = std::path::Path::new("/definitely/does/not/exist.jar");
        mgr.refresh_file_from_disk("srv", "ghost.jar", nonexistent)
            .await
            .unwrap();

        assert!(mgr.get_file("srv", "ghost.jar").await.is_none());
    }

    #[tokio::test]
    async fn refresh_file_from_disk_loads_existing_file() {
        let mgr = new_manager(0).await;
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("client.jar");
        std::fs::write(&file, b"hello world").unwrap();

        mgr.refresh_file_from_disk("srv", "client.jar", &file)
            .await
            .unwrap();

        let got = mgr.get_file("srv", "client.jar").await.unwrap();
        assert_eq!(got.size, 11);
        assert!(!got.sha1.is_empty());
    }
}
