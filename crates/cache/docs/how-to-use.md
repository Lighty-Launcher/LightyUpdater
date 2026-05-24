# Using lighty-cache

Hold one `Arc<CacheManager>`, share it everywhere.

## Construct + initialize

```rust
use lighty_cache::CacheManager;
use std::sync::Arc;

let cache_manager = Arc::new(
    CacheManager::new(config, events, Some(storage)).await,
);
cache_manager.initialize().await?;
```

`initialize` runs the initial scan and bulk-loads files into the LRU
(when `cache.auto_scan = true`).

## Start the background rescan

```rust
cache_manager.start_auto_rescan().await;
```

Picks polling or file-watcher mode from `cache.rescan_interval`.

## Read a file (HTTP)

```rust
if let Some(cached) = cache_manager.get_file("survival", "client/client.jar").await {
    serve_bytes(cached.data, cached.mime_type, cached.sha1);
}
```

## Read the manifest

```rust
let manifest = cache_manager.get("survival").await;
```

Returns `Option<Arc<VersionBuilder>>`.

## Force rescan (CLI / admin)

```rust
cache_manager.force_rescan("survival").await?;
```

Loud variant — errors propagate.

## Hot-reload housekeeping

```rust
cache_manager.pause_rescan();
cache_manager.rebuild_server_cache_with_data(&servers, &base_path);
// loop deleted servers and call remove_server
cache_manager.resume_rescan();
```

## Shutdown

```rust
cache_manager.shutdown().await;
```

## Errors

```rust
pub enum CacheError {
    IoError(io::Error),
    Rescan(lighty_rescan::RescanError),
    FileCache(lighty_file_cache::FileCacheError),
    Cdn(lighty_cdn::CdnError),
}
```

All sub-errors flow through `#[from]`.

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
- [flows.md](./flows.md)
