# Flows

Lifecycle sequences for the facade itself. The interesting work
happens in the sub-crates; these diagrams show how `CacheManager`
glues them.

## Boot

```mermaid
sequenceDiagram
    participant Run as lighty-runtime::run_server
    participant CM as CacheManager
    participant Store as CacheStore (rescan)
    participant FC as FileCacheManager (file-cache)
    participant SPC as ServerPathCache (rescan)
    participant Orch as RescanOrchestrator (rescan)
    participant Bus as EventBus

    Run->>CM: new(config, events, Some(storage))
    CM->>Store: new() (returns store + shared DashMap)
    CM->>FC: new(max_cache_gb, shutdown_tx)
    CM->>SPC: new() then rebuild(servers, base_path)
    CM->>Orch: new(RescanOrchestratorDeps { ... })
    CM-->>Run: Arc<CacheManager>

    Run->>CM: initialize()
    CM->>Bus: emit InitialScanStarted
    CM->>Orch: scan_all_servers()
    CM->>FC: load_all_servers(servers, base_path, concurrency)
    CM-->>Run: Ok

    Run->>CM: start_auto_rescan()
    CM->>Orch: spawn(run_rescan_loop())
```

`new` does no I/O: it reads from `config.read()` (a tokio RwLock), but
none of the constructed pieces touch the disk until `initialize`. This
keeps `new` fast and predictable for tests.

## Force rescan from the CLI

```mermaid
sequenceDiagram
    participant CLI as lighty (CLI)
    participant HTTP as POST /admin/rescan/{server}
    participant API as lighty-api
    participant CM as CacheManager
    participant Orch as RescanOrchestrator

    CLI->>HTTP: rescan survival
    HTTP->>API: handler invokes state.cache.force_rescan
    API->>CM: force_rescan("survival")
    CM->>Orch: force_rescan_server("survival")
    Orch-->>CM: Ok / Err(RescanError)
    CM-->>API: Result<(), CacheError>
    API-->>HTTP: 200 / 5xx
```

Note that `force_rescan` propagates errors — unlike the continuous
loop, which swallows them. The CLI / admin endpoint wants to surface
failures.

## Config hot-reload

```mermaid
sequenceDiagram
    participant Watcher as lighty-watcher
    participant Cfg as config.toml on disk
    participant CM as CacheManager
    participant SPC as ServerPathCache

    Watcher-->>Watcher: notify event (debounced)
    Watcher->>Cfg: re-read into new Config
    Watcher->>CM: pause_rescan()
    Watcher->>CM: rebuild_server_cache_with_data(servers, base_path)
    CM->>SPC: rebuild(servers, base_path)
    Watcher->>Watcher: detect deleted servers
    loop per deleted server
        Watcher->>CM: remove_server(name)
        CM->>SPC: remove_server(name)
        CM->>CM: cache.remove(name) + file_cache.invalidate_server(name)
    end
    Watcher->>CM: resume_rescan()
```

`pause` -> `rebuild` -> `resume` is the canonical sequence. Doing it
in the other order risks an in-flight rescan reading the old server
path cache and missing the rename.

## Shutdown

```mermaid
sequenceDiagram
    participant Sig as ctrl-c
    participant Run as lighty-runtime
    participant CM as CacheManager
    participant Loop as rescan loop task
    participant FC as FileCacheManager

    Sig->>Run: SIGINT
    Run->>CM: shutdown()
    CM->>CM: shutdown_tx.send(()) (broadcast)
    Loop-->>Loop: select! recv triggers, loop exits
    CM->>CM: drain tasks DashMap, await each handle
    CM->>FC: shutdown() (drains its own tasks)
    CM-->>Run: ()
    Run->>Bus: emit Shutdown
```

All spawned tasks subscribe to the broadcast channel; the shutdown
signal is a single `send(())` that fans out to every receiver.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`../../rescan/docs/flows.md`](../../rescan/docs/flows.md) — the
  interesting diff-driven side-effect pipeline (delegated to that
  crate)
