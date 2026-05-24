# Exports

Public surface of `lighty-rescan`.

## Crate root

```rust
use lighty_rescan::{
    CacheStore,
    CacheUpdater,
    ChangeDetector,
    RescanError,
    RescanOrchestrator,
    RescanOrchestratorDeps,
    ServerPathCache,
};
```

## Type details

### `RescanOrchestrator`

```rust
pub struct RescanOrchestrator { /* private fields */ }

impl RescanOrchestrator {
    pub fn new(deps: RescanOrchestratorDeps) -> Self;

    pub fn pause(&self);
    pub fn resume(&self);

    pub async fn scan_all_servers(&self) -> Result<(), RescanError>;
    pub async fn force_rescan_server(&self, server_name: &str) -> Result<(), RescanError>;

    pub async fn run_rescan_loop(self: Arc<Self>);
}
```

`run_rescan_loop` takes `self: Arc<Self>` so the caller can keep its
own `Arc` and shutdown via `tokio::select!`.

### `RescanOrchestratorDeps`

```rust
pub struct RescanOrchestratorDeps {
    pub cache:              Arc<dyn CacheUpdater>,
    pub file_cache_manager: Arc<lighty_file_cache::FileCacheManager>,
    pub last_updated:       Arc<dashmap::DashMap<String, String>>,
    pub config:             Arc<tokio::sync::RwLock<lighty_config::Config>>,
    pub events:             Arc<lighty_events::EventBus>,
    pub storage:            Option<Arc<dyn lighty_storage::StorageBackend>>,
    pub base_path:          std::path::PathBuf,
    pub server_path_cache:  Arc<ServerPathCache>,
}
```

`storage = None` is supported; the orchestrator simply skips
`sync_cloud_storage` and CDN purge events when there's no backend.

### `CacheUpdater` trait + `CacheStore`

```rust
pub trait CacheUpdater: Send + Sync {
    fn insert(&self, server_name: String, version: Arc<VersionBuilder>);
    fn get(&self, server_name: &str) -> Option<Arc<VersionBuilder>>;
    fn contains(&self, server_name: &str) -> bool;
}

pub struct CacheStore { /* Arc<DashMap<String, Arc<VersionBuilder>>> */ }

impl CacheStore {
    pub fn new() -> (Self, Arc<DashMap<String, Arc<VersionBuilder>>>);
}

impl CacheUpdater for CacheStore { ... }
```

`CacheManager` from `lighty-cache` also impls `CacheUpdater`, so the
facade can be its own updater.

### `ChangeDetector`

```rust
pub struct ChangeDetector;

impl ChangeDetector {
    pub fn detect_changes(old: &VersionBuilder, new: &VersionBuilder) -> (bool, Vec<String>);
}
```

Pure function: no I/O, no allocation when nothing changed. Returns
`(false, vec![])` for identical snapshots.

### `ServerPathCache`

```rust
pub struct ServerPathCache { /* parking_lot::RwLock<Vec<(PathBuf, String)>> */ }

impl ServerPathCache {
    pub fn new() -> Self;
    pub fn rebuild(&self, servers: &[Arc<ServerConfig>], base_path: &str);
    pub fn find_server(&self, path: &Path) -> Option<String>;
    pub fn update_server(&self, server_name: String, server_path: PathBuf);
    pub fn remove_server(&self, server_name: &str);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Internally stores entries sorted by path length (descending), so
`find_server` matches the most specific path first.

### `RescanError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum RescanError {
    #[error("I/O error: {0}")]              Io(#[from] std::io::Error),
    #[error("Scanner error: {0}")]           Scan(#[from] lighty_scanner::ScanError),
    #[error("Storage error: {0}")]           Storage(#[from] lighty_storage::StorageError),
    #[error("Join error: {0}")]              Join(#[from] tokio::task::JoinError),
    #[error(transparent)]                    FileCache(#[from] lighty_file_cache::FileCacheError),
    #[error("Server not found: {0}")]        ServerNotFound(String),
    #[error("Invalid configuration: {0}")]   InvalidConfig(String),
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`flows.md`](./flows.md)
- [`events.md`](./events.md)
