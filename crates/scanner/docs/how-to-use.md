# Using lighty-scanner

The scanner is sync-friendly from the caller's side (just `.await`).
You rarely call it directly — `lighty-rescan` does that for you —
but the snippets below are exactly what the orchestrator uses
internally.

## 1. Scan one server (loud variant)

```rust
use lighty_scanner::ServerScanner;

let builder = ServerScanner::scan_server(
    &server_config,           // &ServerConfig
    &storage_backend,         // &Arc<dyn StorageBackend>
    base_path.as_ref(),       // base_path: &str
    &batch_config,            // &BatchConfig from lighty_config
    checksum_buffer_size,     // usize
).await?;
```

Failures (missing folder, unreadable jar, hash mismatch) come back
through `ScanError`.

## 2. Scan one server (silent variant)

```rust
match ServerScanner::scan_server_silent(
    &server_config,
    &storage_backend,
    base_path.as_ref(),
    &batch_config,
    checksum_buffer_size,
).await {
    Ok(builder) => use_it(builder),
    Err(_) => { /* logged via tracing::warn inside */ }
}
```

Same parameters; failures are still returned as `Err` so the caller
can decide what to do, but the scanner stays quiet rather than
emitting events.

## 3. Validate a server path before doing real work

```rust
ServerScanner::validate_server_path(base_path.as_ref(), &server_config)?;
```

Useful in CLI commands that want to fail fast (e.g. `lighty status
--validate`) without paying for the full scan.

## 4. Hand the result to `lighty-rescan`

```rust
let builder = ServerScanner::scan_server_silent(...).await?;
orchestrator.update_cache_if_changed(&server_config, builder).await;
```

`update_cache_if_changed` does the diff, the cache refresh, the cloud
sync and the event emissions.

## Errors at a glance

```rust
pub enum ScanError {
    Io(io::Error),
    InvalidServerPath(String),
    Hash(/* sha1 / read */),
    JarRead(/* zip error */),
    // ... see exports.md
}
```

`ScanError` is surfaced on `RescanError::Scan` and through the chain
all the way up to `CacheError::Rescan(RescanError::Scan(...))`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`architecture.md`](./architecture.md)
- [`flow.md`](./flow.md)
- [`jar-scanner.md`](./jar-scanner.md)
