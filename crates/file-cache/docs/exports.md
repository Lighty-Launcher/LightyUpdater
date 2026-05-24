# Exports

Public surface of `lighty-file-cache`.

## Crate root

```rust
use lighty_file_cache::{FileCache, FileCacheError, FileCacheManager};
```

## Type details

### `FileCache`

```rust
#[derive(Clone)]
pub struct FileCache {
    pub data:      bytes::Bytes,
    pub sha1:      String,        // hex, 40 chars
    pub size:      u64,           // bytes
    pub mime_type: String,        // from mime_guess
}

impl FileCache {
    pub fn from_file_sync(path: &Path) -> Result<Self, FileCacheError>;
    pub fn memory_usage(&self) -> u64; // == data.len()
}
```

`data` is `bytes::Bytes` so cloning is `Arc::clone` under the hood.
HTTP handlers can return `Body::from(cached.data.clone())` without
copying.

### `FileCacheManager`

```rust
pub struct FileCacheManager { /* moka Cache + dashmap of tasks + broadcast tx */ }

impl FileCacheManager {
    pub fn new(max_capacity_gb: u64, shutdown_tx: broadcast::Sender<()>) -> Self;

    pub async fn get_file(&self, server: &str, path: &str) -> Option<FileCache>;
    pub async fn invalidate_file(&self, server: &str, path: &str);
    pub async fn invalidate_server(&self, server: &str);

    pub async fn refresh_file_from_disk(
        &self,
        server: &str,
        path: &str,
        full_path: &Path,
    ) -> Result<(), FileCacheError>;

    pub async fn load_all_servers(
        &self,
        servers: &[Arc<ServerConfig>],
        base_path: &str,
        server_parallelism: usize,
    ) -> Result<(), FileCacheError>;

    pub fn get_stats(&self) -> (u64 /* entries */, u64 /* weighted size KiB */);
    pub async fn shutdown(&self);
}
```

The store backs a Moka `Cache<Arc<str>, FileCache>` with a weigher
that returns `value.data.len().min(u32::MAX)` per entry. Keys are
`"{server}/{path}"`.

### `FileCacheError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum FileCacheError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

Surfaced on `CacheError` via `#[from]`.

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`flows.md`](./flows.md)
