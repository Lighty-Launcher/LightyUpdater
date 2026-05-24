# Exports

Public surface of `lighty-watcher`.

## Crate root

```rust
use lighty_watcher::{ConfigWatcher, WatcherError};
```

## `ConfigWatcher`

```rust
pub struct ConfigWatcher { /* config + path + cache_manager */ }

impl ConfigWatcher {
    pub fn new(
        config:        Arc<tokio::sync::RwLock<Config>>,
        config_path:   String,
        cache_manager: Arc<lighty_cache::CacheManager>,
    ) -> Self;

    pub async fn start_watching(self: Arc<Self>) -> Result<JoinHandle<()>, WatcherError>;
}
```

`start_watching` consumes the `Arc<Self>` so the spawned task can
keep its own reference. Abort the returned handle to stop the
watcher.

## `WatcherError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("I/O error: {0}")]      Io(#[from] std::io::Error),
    #[error("Notify error: {0}")]   Notify(#[from] notify::Error),
    #[error(transparent)]           Adapters(#[from] lighty_adapters::AdaptersError),
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`hot-reload.md`](./hot-reload.md)
