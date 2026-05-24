# Using lighty-rescan

The orchestrator is normally constructed for you by `lighty-cache`.
These snippets show how to wire it directly when you want a thinner
composition root (tests, alternative facades).

## 1. Build the orchestrator

```rust
use lighty_rescan::{
    CacheStore, RescanOrchestrator, RescanOrchestratorDeps, ServerPathCache,
};
use lighty_file_cache::FileCacheManager;
use std::sync::Arc;

let (store, version_cache) = CacheStore::new();
let server_path_cache      = Arc::new(ServerPathCache::new());
server_path_cache.rebuild(&config.servers, config.server.base_path.as_ref());

let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
let file_cache_manager = Arc::new(FileCacheManager::new(2, shutdown_tx));

let orchestrator = Arc::new(RescanOrchestrator::new(RescanOrchestratorDeps {
    cache: Arc::new(store),
    file_cache_manager,
    last_updated: Arc::new(dashmap::DashMap::new()),
    config: shared_config,
    events,
    storage: Some(local_backend),
    base_path: base_path.into(),
    server_path_cache,
}));
```

`CacheStore::new` returns a `(Self, Arc<DashMap<String, Arc<VersionBuilder>>>)`
tuple; keep the second one if any other component needs read access to
the same map.

## 2. Trigger the initial scan

```rust
orchestrator.scan_all_servers().await?;
```

This is what `CacheManager::initialize` calls once, after emitting
`InitialScanStarted`. It populates the version cache and emits
`CacheNew` (or `Error` for unscan­nable servers).

## 3. Start the background loop

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

`run_rescan_loop` picks polling vs file-watcher mode from
`config.cache.rescan_interval` at first call. Pair it with a
`broadcast::Sender<()>` so shutdown is graceful.

## 4. Force-rescan a single server (CLI / admin endpoint)

```rust
orchestrator.force_rescan_server("survival").await?;
```

This calls the **loud** `ServerScanner::scan_server` (not the silent
one used by the loop), so errors surface as `RescanError::Scan` rather
than getting swallowed. If a previous cache entry exists it is kept
on failure; if not, an empty `VersionBuilder` is inserted as a
fallback.

## 5. Pause / resume during config reload

```rust
orchestrator.pause();
// reload config, rebuild server path cache, etc.
orchestrator.resume();
```

The pause/resume flag is checked at the top of each loop iteration —
in flight scans run to completion, then the next tick is skipped
while paused.

## 6. Subscribe to a rescan from outside

`lighty-rescan` doesn't expose a "subscriber" API; consumers attach to
the workspace `EventBus` and filter on the rescan-related variants.
See [events.md](./events.md) for the catalogue.

```rust
struct CaptureSink {
    seen: std::sync::Mutex<Vec<lighty_events::AppEvent>>,
}

impl lighty_events::EventSink for CaptureSink {
    fn handle(&self, event: &lighty_events::AppEvent) {
        if matches!(
            event,
            lighty_events::AppEvent::CacheNew { .. }
                | lighty_events::AppEvent::CacheUpdated { .. }
                | lighty_events::AppEvent::CdnPurgeRequested { .. }
        ) {
            self.seen.lock().unwrap().push(event.clone());
        }
    }
}
```

## 7. Use `ChangeDetector` standalone

`ChangeDetector` is also exposed standalone — handy when you need a
human-readable diff summary without running the full orchestrator.

```rust
use lighty_rescan::ChangeDetector;

let (changed, summary) = ChangeDetector::detect_changes(&old, &new);
if changed {
    println!("changes: {}", summary.join(", "));
}
```

## Errors at a glance

```rust
pub enum RescanError {
    Io(io::Error),
    Scan(lighty_scanner::ScanError),
    Storage(lighty_storage::StorageError),
    Join(tokio::task::JoinError),
    FileCache(lighty_file_cache::FileCacheError),
    ServerNotFound(String),
    InvalidConfig(String), // e.g. "cache.hash_concurrency must be greater than 0"
}
```

Surfaced on `CacheError` via `#[from]`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`flows.md`](./flows.md) — full sequence diagrams
- [`events.md`](./events.md) — emitted event variants
