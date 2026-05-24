# Exports

Public surface of `lighty-cache`.

## Crate root

```rust
use lighty_cache::{
    // Facade
    CacheManager,
    CacheError,

    // Re-exported from lighty-file-cache
    FileCache,
    FileCacheManager,

    // Re-exported from lighty-file-diff
    FileChange,
    FileDiff,
    FileType,

    // Re-exported from lighty-cdn
    CdnClient,
    CdnError,
    CloudflareClient,

    // Re-exported from lighty-rescan
    ChangeDetector,
    RescanOrchestrator,
    ServerPathCache,
};
```

Consumers of `lighty-cache` from before the refactor keep exactly the
same imports.

## `CacheManager`

```rust
pub struct CacheManager { /* private fields */ }

impl CacheManager {
    pub async fn new(
        config:  Arc<tokio::sync::RwLock<Config>>,
        events:  Arc<lighty_events::EventBus>,
        storage: Option<Arc<dyn lighty_storage::StorageBackend>>,
    ) -> Self;

    // Lifecycle
    pub async fn initialize(&self) -> Result<(), CacheError>;
    pub async fn start_auto_rescan(&self);
    pub async fn shutdown(&self);

    // Reads (HTTP-side)
    pub async fn get(&self, server_name: &str)               -> Option<Arc<VersionBuilder>>;
    pub async fn get_version(&self, name: &str)              -> Option<Arc<VersionBuilder>>;
    pub async fn get_file(&self, server: &str, path: &str)   -> Option<FileCache>;
    pub fn       get_cache_stats(&self)                       -> (u64, u64);
    pub fn       get_last_update(&self, name: &str)           -> Option<String>;
    pub async fn get_all_servers(&self)                       -> Vec<String>;
    pub async fn get_server_config(&self, name: &str)         -> Option<Arc<ServerConfig>>;

    // Admin / lifecycle hooks
    pub async fn force_rescan(&self, server_name: &str)       -> Result<(), CacheError>;
    pub async fn remove_server(&self, server_name: &str);
    pub fn       pause_rescan(&self);
    pub fn       resume_rescan(&self);
    pub async fn rebuild_server_cache(&self);
    pub fn       rebuild_server_cache_with_data(
                     &self,
                     servers: &[Arc<ServerConfig>],
                     base_path: &str,
                 );
}

impl lighty_rescan::CacheUpdater for CacheManager { /* uses internal DashMap */ }
```

## `CacheError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("I/O error: {0}")]   IoError(#[from] std::io::Error),
    #[error(transparent)]        Rescan(#[from] lighty_rescan::RescanError),
    #[error(transparent)]        FileCache(#[from] lighty_file_cache::FileCacheError),
    #[error(transparent)]        Cdn(#[from] lighty_cdn::CdnError),
}
```

Every sub-crate's error surfaces transparently via `#[from]`, so a
`?` from `lighty-rescan` inside a `CacheManager` method returns a
`CacheError::Rescan(...)` with no manual wrapping.

## Re-exports — where they actually live

| Re-export | Defined in |
|---|---|
| `FileCache`, `FileCacheManager` | [`lighty-file-cache`](../../file-cache/docs/exports.md) |
| `FileDiff`, `FileChange`, `FileType` | [`lighty-file-diff`](../../file-diff/docs/exports.md) |
| `CdnClient`, `CloudflareClient`, `CdnError` | [`lighty-cdn`](../../cdn/docs/exports.md) |
| `ChangeDetector`, `RescanOrchestrator`, `ServerPathCache` | [`lighty-rescan`](../../rescan/docs/exports.md) |

If you're writing a new crate against this codebase, prefer importing
from the source crate directly — it documents intent and survives
future facade reshuffles.

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`flows.md`](./flows.md)
