# Flows

## End-to-end purge

```mermaid
sequenceDiagram
    participant Rescan
    participant Bus as EventBus
    participant Sink as CdnEventSink
    participant CF as Cloudflare API

    Rescan->>Rescan: FileDiff::compute
    Rescan->>Bus: emit CdnPurgeRequested
    Bus->>Sink: handle
    Sink->>CF: POST purge_cache
    CF-->>Sink: 200 success=true

    Rescan->>Bus: emit CloudflarePurgeRequested
    Bus->>Sink: handle
    Sink->>CF: POST purge_cache (JSON only)
    CF-->>Sink: 200 success=true
```

The bus is sync: `emit` returns as soon as `handle` returns. The sink
only `spawn`s, so rescan never waits for the network.

## Retry loop

```mermaid
flowchart TD
    Start[purge] --> Post[POST /zones/zone/purge_cache]
    Post --> Resp{response}
    Resp -->|2xx success=true| Ok
    Resp -->|2xx success=false| Err[CdnError::Cloudflare]
    Resp -->|5xx or transport| Retry{attempts < 3}
    Retry -->|yes| Sleep[sleep 100ms x 2^attempt]
    Sleep --> Post
    Retry -->|no| Err
```

## Disabled in config

```mermaid
flowchart LR
    Cfg[cdn.enabled = false] --> Init[initialize_cdn]
    Init --> None[Option::None]
    None --> Sink[CdnEventSink]
    Bus[CdnPurgeRequested] --> Sink
    Sink --> Drop[early return in handle]
```

## See also

- [overview.md](./overview.md)
- [../../rescan/docs/flows.md](../../rescan/docs/flows.md)
