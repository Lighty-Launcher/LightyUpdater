# Using lighty-file-cache

## Build the manager

```rust
use lighty_file_cache::FileCacheManager;
use tokio::sync::broadcast;

let (shutdown_tx, _) = broadcast::channel(1);
let manager = FileCacheManager::new(/* gb */ 2, shutdown_tx);
```

`gb = 0` = unlimited.

## Read

```rust
if let Some(cached) = manager.get_file("survival", "client/client.jar").await {
    serve_bytes(cached.data, cached.mime_type);
}
```

`cached.data` is `Bytes`, cloning is `Arc::clone`.

## Refresh after a diff

```rust
let full = base_path.join("survival/mods/iris.jar");
manager.refresh_file_from_disk("survival", "mods/iris.jar", &full).await?;
```

If `full` doesn't exist, the entry is invalidated instead.

## Invalidate

```rust
manager.invalidate_file("survival", "mods/iris.jar").await;
manager.invalidate_server("survival").await;
```

## Bulk-load at boot

```rust
manager
    .load_all_servers(&config.servers, base_path.as_ref(), config.cache.hash_concurrency)
    .await?;
```

Filter is hard-coded: `.jar`, `.json`, `assets/*`.

## Stats

```rust
let (entries, weighted_kb) = manager.get_stats();
```

## Errors

```rust
pub enum FileCacheError {
    Io(io::Error),
    Join(JoinError),
    InvalidConfig(String),
}
```

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
- [flows.md](./flows.md)
