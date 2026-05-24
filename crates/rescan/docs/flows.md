# Flows

Sequence diagrams for every loop and the diff-driven side-effect
pipeline.

## Initial scan (boot)

Triggered once by `CacheManager::initialize` after the config has
loaded.

```mermaid
sequenceDiagram
    participant Init as CacheManager::initialize
    participant Orch as RescanOrchestrator
    participant Scanner as lighty-scanner
    participant Storage as StorageBackend
    participant Bus as EventBus

    Init->>Bus: emit InitialScanStarted
    Init->>Orch: scan_all_servers()
    loop per enabled server (buffer_unordered N)
        Orch->>Scanner: scan_server(server, storage, base_path, batch, buf)
        alt Ok(builder)
            Scanner-->>Orch: VersionBuilder
            Orch->>Orch: builder.build_url_map()
            Orch->>Orch: cache.insert(name, Arc::new(builder))
            Orch->>Orch: last_updated.insert(now)
            Orch->>Bus: emit CacheNew { server }
        else Err(scan_error)
            Orch->>Orch: insert empty VersionBuilder fallback
            Orch->>Bus: emit CacheNew { server }
        end
    end
    Orch-->>Init: Ok(())
```

Concurrency is `cache.hash_concurrency` servers at a time. Per-server
work happens inside `lighty-scanner`.

## Polling loop (`rescan_interval > 0`)

```mermaid
sequenceDiagram
    participant Loop as run_rescan_loop
    participant Cfg as Config
    participant Orch as RescanOrchestrator
    participant Scanner as ServerScanner::scan_server_silent

    Loop->>Cfg: read rescan_interval
    Loop->>Loop: tokio::time::interval(interval)
    loop forever
        Loop->>Loop: interval.tick()
        Loop->>Orch: if paused → continue
        Loop->>Cfg: snapshot servers + base_path
        loop per enabled server
            Loop->>Scanner: scan_server_silent(server, ...)
            alt Ok(builder)
                Scanner-->>Loop: VersionBuilder
                Loop->>Orch: update_cache_if_changed
            else Err
                Note over Loop: silently ignored<br/>(loud variant runs in force_rescan_server)
            end
        end
    end
```

The polling variant is "loud failure on init, quiet failure on
continuous". A missing file mid-update or a transient disk error
should not flood the logs.

## File-watcher loop (`rescan_interval == 0`)

```mermaid
sequenceDiagram
    participant Loop as run_file_watcher_loop
    participant Notify as notify::Watcher
    participant SPC as ServerPathCache
    participant Orch as RescanOrchestrator
    participant Bus as EventBus

    Loop->>Bus: emit ContinuousScanEnabled
    Loop->>Notify: recommended_watcher(tx)
    loop per enabled server
        Loop->>Notify: watch(server_path, RecursiveMode::Recursive)
    end
    loop forever
        Notify-->>Loop: Event (via mpsc)
        Loop->>Loop: if paused → continue
        Loop->>SPC: find_server(path) for each event.paths
        Loop->>Loop: sleep(debounce_ms)
        Loop->>Loop: try_recv drain (coalesce burst)
        loop per impacted server (HashSet)
            Loop->>Orch: rescan_server(server_config, base_path)
        end
    end
```

Notes:
- `ServerPathCache` is sorted longest-first, so the most specific path
  matches before any parent.
- The `try_recv` drain after the debounce sleep coalesces bursts (a
  modpack copy can emit thousands of events in a second; we want one
  rescan, not thousands).
- The `mpsc` channel is bounded; `Full` events get dropped with
  `tracing::trace!` — safe because we rescan the *whole* server, not
  per-file deltas.

## Diff-driven side effects

The heart of the crate. Called by both loops via `rescan_server`.

```mermaid
sequenceDiagram
    participant Orch as update_cache_if_changed
    participant Diff as lighty-file-diff
    participant FC as FileCacheManager
    participant Storage as StorageBackend
    participant Bus as EventBus

    Orch->>Diff: FileDiff::compute(server, old, new)
    Diff-->>Orch: added / modified / removed
    alt no changes
        Orch->>Bus: emit CacheUnchanged
    else has changes
        Orch->>FC: invalidate_file (removed)
        Orch->>FC: refresh_file_from_disk (added ∪ modified)
        alt storage.is_remote
            Orch->>Storage: sync_cloud_storage (upload + delete)
            Orch->>Bus: emit CdnPurgeRequested { server, urls }
        end

        alt was first scan
            Orch->>Orch: new_builder.build_url_map()
        else
            Orch->>Orch: diff.apply_to_url_map(&mut new_builder)
        end
        Orch->>Orch: cache.insert / last_updated.insert
        Orch->>Bus: emit CloudflarePurgeRequested { server }

        alt was first scan
            Orch->>Bus: emit CacheNew
        else
            Orch->>Bus: emit CacheUpdated { changes }
        end
    end
```

The order is intentional:
1. Invalidate RAM cache **first** so requests in flight don't read
   stale bytes during the upload.
2. Refresh RAM cache from disk **before** any upload — keeps "served
   from RAM" consistent with what's on the wire.
3. Upload to cloud storage **before** emitting `CdnPurgeRequested` —
   otherwise the launcher could be redirected to a CDN URL that's
   already purged but not yet uploaded.
4. Emit `CloudflarePurgeRequested` **after** inserting the new
   manifest into the in-memory cache, so the next GET on
   `/{server}.json` reads the new payload.

## Cloud sync detail (`sync_cloud_storage`)

```mermaid
flowchart TD
    Start[sync_cloud_storage diff] --> Upload[upload added ∪ modified]
    Upload --> P1[buffer_unordered N parallel uploads]
    P1 --> Wait1[await all upload Results]
    Wait1 --> Err1{any upload failed?}
    Err1 -->|yes| Bail[return Err]
    Err1 -->|no| Delete[delete removed]
    Delete --> P2[buffer_unordered N parallel deletes]
    P2 --> Wait2[await all delete Results]
    Wait2 --> Err2{any delete failed?}
    Err2 -->|yes| Bail
    Err2 -->|no| OK[Ok]
```

`N` is `cache.hash_concurrency`. Uploads happen before deletes so a
launcher that fetches the manifest mid-sync sees the new file before
the old one disappears.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`events.md`](./events.md) — exhaustive list of emitted events
