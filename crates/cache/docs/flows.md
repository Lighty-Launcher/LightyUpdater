# Flows

## Boot

```mermaid
sequenceDiagram
    participant Run as run_server
    participant CM as CacheManager
    participant Orch as RescanOrchestrator

    Run->>CM: new(config, events, storage)
    CM->>CM: build CacheStore + FileCacheManager
    CM->>CM: ServerPathCache::rebuild
    CM->>Orch: new(deps)
    CM-->>Run: Arc<CacheManager>

    Run->>CM: initialize
    CM->>Orch: scan_all_servers
    CM->>CM: file_cache.load_all_servers
    CM-->>Run: Ok

    Run->>CM: start_auto_rescan
    CM->>Orch: spawn run_rescan_loop
```

`new` does no I/O beyond reading `config.read().await`. The first
disk touch is `initialize`.

## Force rescan

```mermaid
sequenceDiagram
    participant API
    participant CM as CacheManager
    participant Orch as RescanOrchestrator

    API->>CM: force_rescan("survival")
    CM->>Orch: force_rescan_server
    Orch-->>CM: Result
    CM-->>API: Result
```

Loud variant — errors propagate.

## Config hot-reload

```mermaid
sequenceDiagram
    participant Watcher
    participant CM as CacheManager
    participant SPC as ServerPathCache

    Watcher->>CM: pause_rescan
    Watcher->>CM: rebuild_server_cache_with_data
    CM->>SPC: rebuild
    loop per deleted server
        Watcher->>CM: remove_server
        CM->>SPC: remove_server
    end
    Watcher->>CM: resume_rescan
```

## Shutdown

```mermaid
sequenceDiagram
    participant Sig as ctrl-c
    participant Run as run_server
    participant CM as CacheManager
    participant Loop as rescan task

    Sig->>Run: SIGINT
    Run->>CM: shutdown
    CM->>CM: shutdown_tx.send (broadcast)
    Loop-->>Loop: select recv exits
    CM->>CM: await every spawned handle
    CM-->>Run: ()
```

Single broadcast fans out to every receiver.

## See also

- [overview.md](./overview.md)
- [../../rescan/docs/flows.md](../../rescan/docs/flows.md) — the interesting diff-driven pipeline
