# lighty-file-cache

Moka-backed LRU cache for file payloads served by LightyUpdater.

## Overview

**Version**: 26.5.0
**Part of**: [LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)

Holds a weighted `Cache<Arc<str>, FileCache>` so the API layer can
serve `client.jar`, libraries, mods, natives and assets straight from
RAM with zero-copy `Arc<Bytes>`. Capacity is configured in GB
(0 = unlimited); entries beyond the limit are evicted under Moka's
weighted policy.

## Quick start

```rust
use lighty_file_cache::{FileCache, FileCacheManager};
use tokio::sync::broadcast;

let (shutdown_tx, _) = broadcast::channel(1);
let manager = FileCacheManager::new(2 /* GB */, shutdown_tx);

manager
    .refresh_file_from_disk("survival", "client/client.jar", &disk_path)
    .await?;

if let Some(cached) = manager.get_file("survival", "client/client.jar").await {
    serve_bytes(&cached.data, &cached.mime_type);
}
```

## What it provides

- `FileCache { data, sha1, size, mime_type }` — fully populated from
  `FileCache::from_file_sync` (synchronous read + SHA1 + MIME).
- `FileCacheManager` — `get_file`, `refresh_file_from_disk`,
  `invalidate_file`, `invalidate_server` (prefix-based) and
  `load_all_servers` (parallel bulk load via rayon).

## Tests

Six tokio tests cover insert/get, missing entries, single-entry
invalidation, server-prefix invalidation, refresh-from-missing-disk,
and refresh-from-present-disk:

```
cargo test -p lighty-file-cache
```

## Related crates

- [`lighty-rescan`](../rescan) — calls `invalidate_file` /
  `refresh_file_from_disk` from `update_cache_if_changed`.
- [`lighty-api`](../api) — `CacheManager::get_file` returns the
  `FileCache` this crate stores.

## Licence

MIT
