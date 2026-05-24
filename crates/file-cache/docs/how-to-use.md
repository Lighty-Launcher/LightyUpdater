# Using lighty-file-cache

One example per public method. All methods on `FileCacheManager` are
`async` (Moka's async API).

## 1. Build the manager

```rust
use lighty_file_cache::FileCacheManager;
use tokio::sync::broadcast;

let (shutdown_tx, _shutdown_rx) = broadcast::channel(1);
let manager = FileCacheManager::new(/* gb */ 2, shutdown_tx);
```

`gb = 0` means unlimited (no eviction by capacity). The
`shutdown_tx` is reserved for future graceful-shutdown hooks; pass any
broadcast sender you already have.

## 2. Get a file (the hot path)

```rust
if let Some(cached) = manager.get_file("survival", "client/client.jar").await {
    println!("{} bytes, sha1 {}", cached.size, cached.sha1);
    serve_bytes(cached.data, cached.mime_type);
}
```

`FileCache::data: Bytes` is `Arc`-shared, so cloning it for the HTTP
response body is free.

## 3. Refresh a single entry after a diff

```rust
let full_path = base_path.join("survival/mods/iris.jar");
manager
    .refresh_file_from_disk("survival", "mods/iris.jar", &full_path)
    .await?;
```

If `full_path` no longer exists, this just invalidates the cache entry
— matching the "removed" branch of the rescan diff without a separate
code path on the caller side.

## 4. Invalidate one or all entries for a server

```rust
manager.invalidate_file("survival", "mods/iris.jar").await;
manager.invalidate_server("survival").await; // every key prefixed by "survival/"
```

`invalidate_server` walks the cache iterator, collects matching keys
and invalidates them. Useful when the operator removes a server from
`config.toml`.

## 5. Bulk-load every configured server at boot

```rust
manager
    .load_all_servers(
        &config.servers,
        config.server.base_path.as_ref(),
        config.cache.hash_concurrency,
    )
    .await?;
```

- Filter is hard-coded: `.jar`, `.json` files and anything under
  `<server>/assets/`.
- Per-server work runs in `spawn_blocking` + rayon, so the tokio
  worker stays free.
- Returns `Ok(())` even if individual servers fail; failures are
  logged at `warn!` and the total `success / failure` count goes to
  `info!`.

## 6. Read cache stats (for the `/health` endpoint, future use)

```rust
let (entry_count, weighted_size_kb) = manager.get_stats();
```

`weighted_size_kb` is the sum of every entry's stored byte count,
divided by 1024.

## Errors at a glance

```rust
pub enum FileCacheError {
    Io(io::Error),       // from std::fs::read inside from_file_sync
    Join(JoinError),     // from tokio::task::spawn_blocking
    InvalidConfig(String), // e.g. "cache.hash_concurrency must be greater than 0"
}
```

Surfaced on `CacheError` via `#[from]`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`flows.md`](./flows.md)
