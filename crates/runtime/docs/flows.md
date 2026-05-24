# Flows

## Bootstrap → Ready

The order of operations matters: each step depends on the previous
one. The diagram below is the canonical sequence.

```mermaid
sequenceDiagram
    participant Bin as lighty-app
    participant Run as run_server
    participant Cfg as config.toml
    participant Bus1 as Bootstrap EventBus
    participant Stg as StorageBackend
    participant Bus2 as Real EventBus
    participant Cache as CacheManager
    participant Watch as ConfigWatcher
    participant Axum

    Bin->>Run: run_server(path, version)
    Run->>Run: initialize tracing
    Run->>Bus1: new + ConsoleEventSink
    Run->>Bus1: emit Starting { version }

    Run->>Cfg: load + migrate
    Cfg-->>Run: Config
    Run->>Run: emit ConfigLoaded
    Run->>Run: initialize_folders per server

    Run->>Stg: initialize_storage (Local or S3)
    Run->>Run: initialize_cdn (Option<Arc<CdnClient>>)
    Run->>Run: initialize_cloudflare (Option<Arc<CloudflareClient>>)

    Run->>Bus2: new + Composite(Console, CdnEventSink)

    Run->>Cache: new(config, events, Some(storage))
    Run->>Cache: initialize() (initial scan + bulk load)
    Run->>Cache: start_auto_rescan()

    Run->>Watch: new(config, path, cache) then start_watching
    Run->>Axum: TcpListener::bind addr
    Axum-->>Run: listener
    Run->>Bus2: emit Ready { addr, base_url }
    Run->>Axum: serve(app, listener).with_graceful_shutdown(ctrl_c)
```

## Shutdown

```mermaid
sequenceDiagram
    participant Sig as ctrl-c
    participant Axum
    participant Run as run_server
    participant Watch as ConfigWatcher
    participant Cache as CacheManager
    participant Bus as EventBus

    Sig->>Axum: shutdown signal
    Axum-->>Run: serve future resolves
    Run->>Watch: abort watcher task
    Run->>Watch: await task handle
    Run->>Cache: shutdown()
    Cache-->>Run: drained
    Run->>Bus: emit Shutdown
    Run-->>Bin: Ok(())
```

## Storage feature decision tree

```mermaid
flowchart TD
    Cfg[config.storage.backend] --> Local{value?}
    Local -->|"local"| LB[LocalBackend::new]
    Local -->|"s3"| Feat{compiled with --features s3?}
    Feat -->|yes| Enabled{config.storage.s3.enabled?}
    Feat -->|no| Err1[RuntimeError::InvalidConfiguration<br/>rebuild with --features s3]
    Enabled -->|yes| S3[S3Backend::new]
    Enabled -->|no| Err2[RuntimeError::InvalidConfiguration<br/>S3 backend selected but not enabled]
    LB --> Arc[Arc&lt;dyn StorageBackend&gt;]
    S3 --> Arc
```

This is why the `s3` feature must be threaded all the way up to the
binary: the backend selection happens at the runtime layer, but the
type only exists in `lighty-storage` when its `s3` feature is on.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`exports.md`](./exports.md)
- [`../../cache/docs/flows.md`](../../cache/docs/flows.md) — what
  `CacheManager::initialize` and `start_auto_rescan` actually do
- [`../../watcher/docs/`](../../watcher/) — config hot-reload flow
