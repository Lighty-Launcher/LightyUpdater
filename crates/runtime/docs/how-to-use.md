# Using lighty-runtime

## Default binary

```rust
// app/main.rs
#[tokio::main]
async fn main() -> Result<(), lighty_runtime::RuntimeError> {
    lighty_runtime::run_server_with_default_config(env!("CARGO_PKG_VERSION")).await
}
```

Reads `LIGHTY_CONFIG` or `./config.toml`.

## Pin the config path

```rust
#[tokio::main]
async fn main() -> Result<(), lighty_runtime::RuntimeError> {
    lighty_runtime::run_server(
        "/etc/lighty/server.toml".into(),
        env!("CARGO_PKG_VERSION"),
    ).await
}
```

## With the s3 feature

```toml
lighty-runtime = { workspace = true, features = ["s3"] }
```

Without it, selecting `backend = "s3"` in config returns
`InvalidConfiguration` at boot.

## Errors

```rust
pub enum RuntimeError {
    IoError(io::Error),
    BindAddress { addr: String, source: io::Error },
    InvalidConfiguration(String),
    Adapters(lighty_adapters::AdaptersError),
    Cache(lighty_cache::CacheError),
    Config(lighty_config::ConfigError),
    Storage(lighty_storage::StorageError),
    Watcher(lighty_watcher::WatcherError),
}
```

`BindAddress` is the one your operator will see most — the runner
logs remediation steps before returning it.

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
- [flows.md](./flows.md)
