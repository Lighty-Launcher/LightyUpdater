# Events

Every `AppEvent` variant emitted by `lighty-rescan`, with the call
site and the conditions for emission.

## Lifecycle

| Event | Emitted when |
|---|---|
| `InitialScanStarted` | (Emitted by `lighty-cache::CacheManager::initialize` before calling `scan_all_servers`.) |
| `ContinuousScanEnabled` | `run_rescan_loop` selects the file-watcher branch (`rescan_interval == 0`). |
| `AutoScanEnabled { interval }` | `run_rescan_loop` selects the timer branch and is about to start ticking. |

## Per-server lifecycle

| Event | Emitted when |
|---|---|
| `CacheNew { server }` | First time a server's `VersionBuilder` is inserted into the cache (either from a successful initial scan, an empty-fallback after a failed initial scan, or the first successful continuous rescan). |
| `CacheUpdated { server, changes }` | A continuous rescan produced a non-empty `FileDiff`. `changes` is a single-element vector with the form `"N added, M modified, P removed"`. |
| `CacheUnchanged { server }` | A continuous rescan produced an empty `FileDiff`. |

## Side effects

| Event | Emitted when |
|---|---|
| `CdnPurgeRequested { server, urls }` | Diff has non-empty changes AND `storage.is_remote()`. `urls` is the union of `added ∪ modified ∪ removed` filtered to non-empty URLs. |
| `CloudflarePurgeRequested { server }` | A non-empty diff was applied, after the new `VersionBuilder` is inserted. The launcher fetches `/{server}.json`, so this clears the metadata edge cache. |

`lighty-cdn::CdnEventSink` listens for both of these and `tokio::spawn`s
the HTTP work. They never trigger from `force_rescan_server` if the
diff is empty.

## Errors

| Event | Emitted when |
|---|---|
| `Error { context, error }` | Initial scan failed AND the server config can no longer be resolved (so an empty fallback can't be inserted). |

## Subscription pattern

```rust
use lighty_events::{AppEvent, EventSink};

struct RescanLogger;

impl EventSink for RescanLogger {
    fn handle(&self, event: &AppEvent) {
        match event {
            AppEvent::CacheNew { server } => println!("[new] {server}"),
            AppEvent::CacheUpdated { server, changes } => {
                println!("[upd] {server}: {}", changes.join(", "));
            }
            AppEvent::CacheUnchanged { server } => println!("[ok ] {server}"),
            _ => {}
        }
    }
}
```

Mount through `CompositeEventSink` alongside `ConsoleEventSink` and
`CdnEventSink` — see [`../../runtime/docs/overview.md`](../../runtime/docs/overview.md)
for the composition root.

## See also

- [`overview.md`](./overview.md)
- [`flows.md`](./flows.md)
- [`../../events/docs/exports.md`](../../events/docs/exports.md) — the
  full `AppEvent` enum
- [`../../cdn/docs/flows.md`](../../cdn/docs/flows.md) — what happens
  on the other side of `CdnPurgeRequested`
