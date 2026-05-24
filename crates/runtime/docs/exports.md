# Exports

Public surface of `lighty-runtime`.

## Crate root

```rust
use lighty_runtime::{run_server, run_server_with_default_config, Result, RuntimeError};
```

## Functions

```rust
pub fn default_config_path() -> String;
pub async fn run_server_with_default_config(app_version: &str) -> Result<()>;
pub async fn run_server(config_path: String, app_version: &str) -> Result<()>;
```

`run_server` is the canonical entry point. It:

1. Configures `tracing_subscriber::EnvFilter` (call once per process).
2. Loads + migrates config, initializes per-server folders.
3. Constructs the storage backend, optional CDN clients, and the
   composite event sink.
4. Builds `CacheManager`, runs the initial scan, starts the rescan
   loop.
5. Spawns the config watcher.
6. Binds Axum and `with_graceful_shutdown(ctrl_c)`.
7. On shutdown signal: aborts the watcher, drains the cache, emits
   `Shutdown`.

## `RuntimeError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to bind server on {addr}: {source}")]
    BindAddress { addr: String, source: std::io::Error },

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error(transparent)] Adapters(#[from] lighty_adapters::AdaptersError),
    #[error(transparent)] Cache(#[from] lighty_cache::CacheError),
    #[error(transparent)] Config(#[from] lighty_config::ConfigError),
    #[error(transparent)] Storage(#[from] lighty_storage::StorageError),
    #[error(transparent)] Watcher(#[from] lighty_watcher::WatcherError),
}
```

`Result<T>` is `std::result::Result<T, RuntimeError>` at the crate
level.

## Cargo features

| Feature | Effect |
|---|---|
| `s3` | Enables `lighty-storage/s3` so the `[storage] backend = "s3"` config path compiles in. |

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`flows.md`](./flows.md)
