# lighty-rescan

Rescan orchestration: timed scan loop, file-watcher mode, change
detection and cloud sync.

## Overview

**Version**: 26.5.0
**Part of**: [LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)

Composes `lighty-scanner` (to rebuild `VersionBuilder` snapshots),
`lighty-file-diff` (to detect what changed) and `lighty-file-cache`
(to refresh in-memory entries). Cross-crate side effects — CDN purges,
Cloudflare metadata flush — are emitted as `AppEvent`s, not direct
calls. The `lighty-cdn::CdnEventSink` in the runtime picks them up.

## Quick start

```rust
use lighty_rescan::{CacheStore, RescanOrchestrator, RescanOrchestratorDeps, ServerPathCache};

let (store, cache) = CacheStore::new();
let server_path_cache = Arc::new(ServerPathCache::new());
server_path_cache.rebuild(&servers, base_path.as_ref());

let orchestrator = Arc::new(RescanOrchestrator::new(RescanOrchestratorDeps {
    cache: Arc::new(store),
    file_cache_manager,
    last_updated,
    config,
    events,
    storage,
    base_path,
    server_path_cache,
}));

// Initial sweep
orchestrator.scan_all_servers().await?;

// Background loop (timer-based when rescan_interval > 0, file-watcher when 0)
orchestrator.clone().run_rescan_loop().await;
```

## What it provides

- `RescanOrchestrator` — the entry point, with `scan_all_servers`,
  `force_rescan_server`, `run_rescan_loop`, `pause` and `resume`.
- `ChangeDetector::detect_changes(old, new)` — human-readable change
  summaries (used for the `CacheUpdated.changes` field).
- `ServerPathCache` — O(k) sorted lookup of which server owns a given
  filesystem path (used by the file watcher).
- `CacheUpdater` trait + `CacheStore` — minimal adapter that lets the
  orchestrator stay decoupled from `lighty-cache`.
- `RescanError` — surfaced on `CacheError` via `#[from]`.

## Events emitted

| Event                        | When                                            |
|------------------------------|-------------------------------------------------|
| `CacheNew`                   | First-time scan succeeded for a server          |
| `CacheUpdated`               | Diff detected (added / modified / removed)      |
| `CacheUnchanged`             | Rescan produced no diff                         |
| `CdnPurgeRequested`          | Remote storage diff → CDN purge needed          |
| `CloudflarePurgeRequested`   | Any update → server JSON metadata changed       |
| `AutoScanEnabled`            | Timer-based loop started                        |
| `ContinuousScanEnabled`      | File-watcher loop started                       |

## Tests

Four `ChangeDetector` unit tests cover empty diff, client-added,
library count change, and size-only library change. Run them with:

```
cargo test -p lighty-rescan
```

## Related crates

- [`lighty-file-diff`](../file-diff) — used internally to compute the
  diff between two `VersionBuilder` snapshots.
- [`lighty-file-cache`](../file-cache) — invalidated and refreshed
  whenever the diff lists added/modified/removed files.
- [`lighty-scanner`](../scanner) — produces the new `VersionBuilder`
  snapshot.
- [`lighty-cdn`](../cdn) — consumes `CdnPurgeRequested` /
  `CloudflarePurgeRequested` events emitted here.

## Licence

MIT
