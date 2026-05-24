# Flows

## Initial scan

```mermaid
sequenceDiagram
    participant Init as CacheManager::initialize
    participant Orch as RescanOrchestrator
    participant Scanner
    participant Bus as EventBus

    Init->>Bus: emit InitialScanStarted
    Init->>Orch: scan_all_servers
    loop per enabled server
        Orch->>Scanner: scan_server (loud)
        alt Ok
            Scanner-->>Orch: VersionBuilder
            Orch->>Orch: build_url_map + cache.insert
            Orch->>Bus: emit CacheNew
        else Err
            Orch->>Orch: insert empty fallback
            Orch->>Bus: emit CacheNew
        end
    end
```

## Polling loop

```mermaid
sequenceDiagram
    participant Loop as run_rescan_loop
    participant Cfg as Config
    participant Orch
    participant Scanner

    Loop->>Cfg: read rescan_interval
    loop tick
        Loop->>Orch: paused?
        Loop->>Cfg: snapshot servers
        loop per enabled server
            Loop->>Scanner: scan_server_silent
            alt Ok
                Loop->>Orch: update_cache_if_changed
            else Err
                Note over Loop: swallowed
            end
        end
    end
```

## File watcher loop

```mermaid
sequenceDiagram
    participant Loop as run_file_watcher_loop
    participant Notify
    participant SPC as ServerPathCache
    participant Orch

    Loop->>Notify: watch every server path
    loop forever
        Notify-->>Loop: event
        Loop->>SPC: find_server
        Loop->>Loop: sleep debounce
        Loop->>Loop: drain queued events
        loop per impacted server
            Loop->>Orch: rescan_server
        end
    end
```

`ServerPathCache` is sorted longest-first so the most specific path
matches first. The drain after the debounce coalesces bursts.

## Diff-driven side effects

```mermaid
sequenceDiagram
    participant Orch as update_cache_if_changed
    participant Diff
    participant FC as FileCacheManager
    participant Storage
    participant Bus

    Orch->>Diff: FileDiff::compute
    Diff-->>Orch: added / modified / removed
    alt no changes
        Orch->>Bus: emit CacheUnchanged
    else has changes
        Orch->>FC: invalidate removed
        Orch->>FC: refresh added or modified
        alt storage.is_remote
            Orch->>Storage: upload + delete (parallel)
            Orch->>Bus: emit CdnPurgeRequested
        end
        Orch->>Orch: insert new VersionBuilder
        Orch->>Bus: emit CloudflarePurgeRequested
        alt first scan
            Orch->>Bus: emit CacheNew
        else
            Orch->>Bus: emit CacheUpdated
        end
    end
```

Order matters: invalidate RAM cache first, then refresh, then upload,
then emit purge events. Otherwise an in-flight request could be
redirected to a CDN URL that's purged but not yet uploaded.

## Cloud sync

```mermaid
flowchart TD
    Start[sync_cloud_storage] --> Up[upload added + modified]
    Up --> Wait1[await results]
    Wait1 --> Err1{any failed}
    Err1 -->|yes| Bail[return Err]
    Err1 -->|no| Del[delete removed]
    Del --> Wait2[await results]
    Wait2 --> Err2{any failed}
    Err2 -->|yes| Bail
    Err2 -->|no| Ok
```

Uploads happen before deletes so the new file lands before the old
one disappears.

## See also

- [overview.md](./overview.md)
- [events.md](./events.md)
