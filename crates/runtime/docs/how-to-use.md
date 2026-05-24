# Using lighty-runtime

The runtime is the only crate the two binaries (`lighty-app` and
`lighty` for the CLI) reach into. Most users embed it in their own
`#[tokio::main]`.

## 1. Default binary

```rust
// app/main.rs
#[tokio::main]
async fn main() -> Result<(), lighty_runtime::RuntimeError> {
    lighty_runtime::run_server_with_default_config(env!("CARGO_PKG_VERSION")).await
}
```

`run_server_with_default_config` reads `LIGHTY_CONFIG` or falls back
to `./config.toml`.

## 2. Pin the config path

```rust
#[tokio::main]
async fn main() -> Result<(), lighty_runtime::RuntimeError> {
    let config_path = std::path::PathBuf::from("/etc/lighty/server.toml")
        .to_string_lossy()
        .into_owned();
    lighty_runtime::run_server(config_path, env!("CARGO_PKG_VERSION")).await
}
```

`run_server` takes the path as a `String`. It blocks until either
`ctrl-c` is delivered or the server fails to bind.

## 3. With the `s3` feature

```toml
# Cargo.toml of your binary
[dependencies]
lighty-runtime = { workspace = true, features = ["s3"] }
```

```rust
// no code change required; the feature unlocks the S3 backend selection
// from config.toml's [storage] section.
```

Without `s3`, selecting `backend = "s3"` in `config.toml` yields
`RuntimeError::InvalidConfiguration` at boot.

## 4. Replace the bootstrap event sink

This is rarely needed, but if you want to swap `ConsoleEventSink` for
something else (e.g. a JSON logger), fork `run_server` in your own
crate and pass a different `EventSink` into the `CompositeEventSink`
constructor. See [overview.md](./overview.md) for the wiring rules.

## Errors at a glance

```rust
pub enum RuntimeError {
    IoError(std::io::Error),
    BindAddress { addr: String, source: std::io::Error },
    InvalidConfiguration(String),
    Adapters(lighty_adapters::AdaptersError),
    Cache(lighty_cache::CacheError),
    Config(lighty_config::ConfigError),
    Storage(lighty_storage::StorageError),
    Watcher(lighty_watcher::WatcherError),
}
```

`BindAddress` is the one your operator will see most often when the
port is taken. The error message in `runner.rs` already lists
remediation steps (stop the other app, change port, `lsof` / `netstat`).

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`flows.md`](./flows.md)
