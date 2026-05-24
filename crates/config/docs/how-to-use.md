# Using lighty-config

The struct definitions are public, but you almost never construct a
`Config` by hand: `lighty-adapters::load_config` does that for you
from the on-disk TOML file. The crate is here so other crates can
type their dependency on the shape of the data.

## 1. Read a value from a snapshot

```rust
let cfg = config_lock.read().await;
let interval = cfg.cache.rescan_interval;
let backend  = &cfg.storage.backend;
```

`config_lock: Arc<RwLock<Config>>` is the standard shape passed
through the runtime.

## 2. Iterate over enabled servers

```rust
let cfg = config_lock.read().await;
let enabled: Vec<_> = cfg.servers.iter().filter(|server| server.enabled).collect();
```

`Config::servers: Vec<Arc<ServerConfig>>` — clone the `Arc` if you
need to hold a reference longer than the read lock.

## 3. Construct a `Config` for tests

```rust
use lighty_config::{Config, ServerConfig, ServerSettings, /* ... */};

let cfg = Config {
    server: ServerSettings { host: "127.0.0.1".into(), port: 0, .. },
    // ... fill remaining sections ...
    servers: vec![],
};
```

Tedious but type-checked. For end-to-end tests prefer writing a
fixture `config.toml` and round-tripping through `load_config`.

## 4. Switch backend at boot

```toml
# config.toml
[storage]
backend = "s3"

[storage.s3]
enabled = true
endpoint_url = "https://r2.example.com"
bucket_name = "lighty-prod"
# ...
```

`StorageBackend::S3` triggers the S3 path in `lighty-runtime`; without
`--features s3` the boot fails with `InvalidConfiguration`.

## Errors at a glance

```rust
pub enum ConfigError {
    Validation(String),
    // ... see errors.md for the full list
}
```

Surfaced by `lighty-adapters::AdaptersError::Validation` in practice;
the schema crate itself rarely returns one.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`migration.md`](./migration.md), [`hot-reload.md`](./hot-reload.md)
- [`../../adapters/docs/overview.md`](../../adapters/docs/overview.md)
