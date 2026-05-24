# Exports

Public surface of `lighty-scanner`.

## Crate root

```rust
use lighty_scanner::{ScanError, ServerScanner, scan_files_parallel};
```

`ServerScanner` is the only public scanner type — the per-component
scanners (`ClientScanner`, `LibraryScanner`, `ModScanner`,
`NativeScanner`, `AssetScanner`) and the `JarScanner` are internal.

## `ServerScanner`

```rust
pub struct ServerScanner;

impl ServerScanner {
    pub async fn scan_server(
        server_config: &ServerConfig,
        storage:       &Arc<dyn StorageBackend>,
        base_path:     &str,
        batch:         &BatchConfig,
        buffer_size:   usize,
    ) -> Result<VersionBuilder, ScanError>;

    pub async fn scan_server_silent(
        server_config: &ServerConfig,
        storage:       &Arc<dyn StorageBackend>,
        base_path:     &str,
        batch:         &BatchConfig,
        buffer_size:   usize,
    ) -> Result<VersionBuilder, ScanError>;

    pub fn validate_server_path(
        base_path:     &str,
        server_config: &ServerConfig,
    ) -> Result<(), ScanError>;
}
```

The two scan variants differ only in whether they emit
`tracing::info!` along the way; both return the same `Result`.

## `scan_files_parallel`

```rust
pub fn scan_files_parallel<F, R>(...) -> Vec<R>
where F: Fn(...) -> R + Send + Sync, R: Send;
```

Generic helper exported because `lighty-rescan` and future
admin-side tools might want to hash arbitrary directories. Most
callers don't need it.

## `ScanError`

See `architecture.md` and `errors.md` in this folder for the full
variant list; surfaced through `RescanError::Scan` via `#[from]`.

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`architecture.md`](./architecture.md)
- [`flow.md`](./flow.md)
- [`jar-scanner.md`](./jar-scanner.md)
