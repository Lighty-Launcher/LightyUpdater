# Flows

## Bootstrap → Ready

```mermaid
sequenceDiagram
    participant Bin
    participant Run as run_server
    participant Cfg
    participant Stg as StorageBackend
    participant Bus as EventBus
    participant Cache as CacheManager
    participant Axum

    Bin->>Run: run_server(path, version)
    Run->>Run: init tracing
    Run->>Bus: bootstrap bus (Console only)
    Run->>Cfg: load + migrate
    Cfg-->>Run: Config
    Run->>Run: initialize_folders
    Run->>Stg: initialize_storage
    Run->>Run: initialize_cdn + cloudflare
    Run->>Bus: build real bus (Console + Cdn)
    Run->>Cache: new + initialize + start_auto_rescan
    Run->>Run: spawn ConfigWatcher
    Run->>Axum: bind
    Run->>Bus: emit Ready
    Run->>Axum: serve with graceful_shutdown
```

## Shutdown

```mermaid
sequenceDiagram
    participant Sig as ctrl-c
    participant Axum
    participant Run
    participant Watch as ConfigWatcher
    participant Cache as CacheManager
    participant Bus

    Sig->>Axum: shutdown
    Axum-->>Run: serve resolves
    Run->>Watch: abort + await
    Run->>Cache: shutdown
    Cache-->>Run: drained
    Run->>Bus: emit Shutdown
```

## Storage backend decision

```mermaid
flowchart TD
    Cfg[storage.backend] --> Sw{value}
    Sw -->|local| LB[LocalBackend]
    Sw -->|s3| Feat{compiled with s3?}
    Feat -->|no| Err[InvalidConfiguration]
    Feat -->|yes| Enabled{storage.s3.enabled}
    Enabled -->|no| Err
    Enabled -->|yes| S3[S3Backend]
```

`s3` must be threaded all the way from the binary down to
`lighty-storage`.

## See also

- [overview.md](./overview.md)
- [../../cache/docs/flows.md](../../cache/docs/flows.md)
