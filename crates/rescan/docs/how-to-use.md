# Using lighty-rescan

`lighty-cache::CacheManager` normally builds the orchestrator for you.
Use these snippets when you want a thinner composition root.

## Build the orchestrator

```rust
use lighty_rescan::{
    CacheStore, RescanOrchestrator, RescanOrchestratorDeps, ServerPathCache,
};
use std::sync::Arc;

let (store, _version_cache) = CacheStore::new();
let server_path_cache = Arc::new(ServerPathCache::new());
server_path_cache.rebuild(&config.servers, base_path);

let orchestrator = Arc::new(RescanOrchestrator::new(RescanOrchestratorDeps {
    cache: Arc::new(store),
    file_cache_manager,
    last_updated,
    config: shared_config,
    events,
    storage: Some(backend),
    base_path: base_path.into(),
    server_path_cache,
}));
```

## Initial scan

```rust
orchestrator.scan_all_servers().await?;
```

Loud variant — errors come back as `RescanError`.

## Background loop

```rust
let orchestrator = Arc::clone(&orchestrator);
let mut shutdown_rx = shutdown_tx.subscribe();

tokio::spawn(async move {
    tokio::select! {
        _ = orchestrator.run_rescan_loop() => {}
        _ = shutdown_rx.recv() => {}
    }
});
```

Polling if `rescan_interval > 0`, file-watcher if `0`.

## Force one server

```rust
orchestrator.force_rescan_server("survival").await?;
```

## Pause / resume during config reload

```rust
orchestrator.pause();
// ... reload config ...
orchestrator.resume();
```

In flight scans run to completion, then the next tick is skipped while paused.

## Standalone `ChangeDetector`

```rust
use lighty_rescan::ChangeDetector;

let (changed, summary) = ChangeDetector::detect_changes(&old, &new);
```

## Errors

```rust
pub enum RescanError {
    Io(io::Error),
    Scan(lighty_scanner::ScanError),
    Storage(lighty_storage::StorageError),
    Join(JoinError),
    FileCache(lighty_file_cache::FileCacheError),
    ServerNotFound(String),
    InvalidConfig(String),
}
```

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
- [flows.md](./flows.md)
- [events.md](./events.md)
