# Using lighty-cache

`CacheManager` is the one thing you hold onto. Build it once, wrap it
in an `Arc`, share it across the HTTP layer, the watcher and the CLI.

## 1. Construct + initialize at boot

```rust
use lighty_cache::CacheManager;
use std::sync::Arc;

let cache_manager = Arc::new(
    CacheManager::new(config_arc, events_arc, Some(storage_arc)).await,
);
cache_manager.initialize().await?;
```

`new` is async because the constructor reads from
`Arc<RwLock<Config>>`. `initialize` performs the first scan (when
`cache.auto_scan = true`) and bulk-loads every server's files into the
LRU.

## 2. Start the background rescan loop

```rust
cache_manager.start_auto_rescan().await;
```

This spawns a tokio task that runs either the polling or the
file-watcher loop, depending on `cache.rescan_interval`. The task is
tracked internally and shut down by `CacheManager::shutdown`.

## 3. Serve a file from the HTTP layer

```rust
if let Some(cached) = cache_manager.get_file("survival", "client/client.jar").await {
    // cached.data is bytes::Bytes — cheap to clone for the response body
    return Response::builder()
        .header("content-type", cached.mime_type)
        .header("etag", format!("\"{}\"", cached.sha1))
        .body(Body::from(cached.data));
}
```

`get_file` is a thin wrapper over `FileCacheManager::get_file`. A
miss returns `None`; the API layer falls back to a streamed read from
disk.

## 4. Read the manifest JSON

```rust
if let Some(version) = cache_manager.get("survival").await {
    let json = serde_json::to_string(&*version)?;
    return Response::builder()
        .header("content-type", "application/json")
        .body(Body::from(json));
}
```

`get` returns `Option<Arc<VersionBuilder>>`. The `Arc` clone is free;
the JSON serialization re-runs each request (small, dominated by
network).

## 5. Force a rescan from the admin endpoint / CLI

```rust
cache_manager.force_rescan("survival").await?;
```

Uses the loud variant of the scanner so the caller learns about
failures via `Result`. Existing cache entries survive a failed
rescan; empty servers get an empty fallback.

## 6. Hot-reload housekeeping

When the watcher reloads `config.toml`:

```rust
cache_manager.pause_rescan();
// ... rewrite config in-place, then ...
cache_manager.rebuild_server_cache_with_data(&servers, &base_path);
cache_manager.resume_rescan();
```

`pause` flips an `AtomicBool` checked at every loop iteration; in
flight scans run to completion. `rebuild_server_cache_with_data`
takes the new server list straight out of the reloader to avoid a
re-read deadlock.

If a server was deleted from config, also call:

```rust
cache_manager.remove_server("creative").await;
```

That clears the version cache, the file cache (by prefix) and the
server path cache.

## 7. Graceful shutdown

```rust
cache_manager.shutdown().await;
```

Broadcasts a stop signal to every spawned task, awaits each handle in
turn, then asks the file cache to drain its own background work.
Returns when everything is quiet.

## Errors at a glance

```rust
pub enum CacheError {
    IoError(io::Error),
    Rescan(lighty_rescan::RescanError),
    FileCache(lighty_file_cache::FileCacheError),
    Cdn(lighty_cdn::CdnError),
}
```

`#[from]` is set on all sub-error variants — propagating from a
sub-crate is a `?` away.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`flows.md`](./flows.md) — lifecycle sequences
- [`../../rescan/docs/how-to-use.md`](../../rescan/docs/how-to-use.md)
  for orchestrator-level wiring
