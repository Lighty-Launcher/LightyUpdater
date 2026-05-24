# Using lighty-watcher

The watcher is constructed and started by `lighty-runtime`. Most
users never touch it directly; the snippet below shows the canonical
wiring.

## 1. Construct + start

```rust
use lighty_watcher::ConfigWatcher;
use std::sync::Arc;

let watcher = Arc::new(ConfigWatcher::new(
    Arc::clone(&config),
    config_path.clone(),
    Arc::clone(&cache_manager),
));

let handle = watcher.clone().start_watching().await?;
```

`start_watching` returns the `JoinHandle` of the spawned task. Store
it so shutdown can `.abort()` it.

## 2. Stop on shutdown

```rust
handle.abort();
let _ = handle.await;
```

`lighty-runtime` does this between the Axum `serve` future
resolving and `CacheManager::shutdown()`.

## Errors at a glance

```rust
pub enum WatcherError {
    Io(io::Error),
    Notify(notify::Error),
    Adapters(lighty_adapters::AdaptersError),
}
```

Surfaced on `RuntimeError::Watcher` via `#[from]`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`hot-reload.md`](./hot-reload.md)
- [`change-detection.md`](./change-detection.md)
