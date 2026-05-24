# Exports

Public surface of `lighty-config`.

## Crate root

```rust
use lighty_config::{
    Config,
    ServerSettings,
    CacheSettings,
    BatchConfig,
    StorageSettings,
    StorageBackend,        // enum { Local, S3 }
    CdnSettings,
    CloudflareSettings,
    HotReloadSettings,
    ServerConfig,
    ConfigError,
};
```

## `Config`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server:     ServerSettings,
    pub cache:      CacheSettings,
    pub storage:    StorageSettings,
    pub cdn:        CdnSettings,
    pub cloudflare: CloudflareSettings,
    pub hot_reload: HotReloadSettings,
    pub servers:    Vec<Arc<ServerConfig>>,
}
```

## `ServerConfig`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub name:           Arc<str>,
    pub enabled:        bool,
    pub loader:         String,
    pub loader_version: Arc<str>,
    pub minecraft_version: Arc<str>,
    pub main_class:     String,
    pub java_version:   u8,
    pub game_args:      Vec<String>,
    pub jvm_args:       Vec<String>,
    pub enable_client:    bool,
    pub enable_libraries: bool,
    pub enable_mods:      bool,
    pub enable_natives:   bool,
    pub enable_assets:    bool,
}
```

`name` and `loader_version` are `Arc<str>` so the cache and rescan
can hold cheap clones.

## Other types

The remaining sub-structs (`ServerSettings`, `CacheSettings`,
`BatchConfig`, `StorageSettings`, `CdnSettings`, `CloudflareSettings`,
`HotReloadSettings`) mirror their `[section]` in `config.toml` one
to one. See `architecture.md` for the per-field intent.

`StorageBackend`:

```rust
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend { Local, S3 }
```

## `ConfigError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Validation error: {0}")] Validation(String),
    // see errors.md
}
```

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`migration.md`](./migration.md)
- [`hot-reload.md`](./hot-reload.md)
