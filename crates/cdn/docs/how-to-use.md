# Using lighty-cdn

## Purge a server's JSON

```rust
use lighty_cdn::CloudflareClient;

let client = CloudflareClient::new(zone_id, api_token);
client.purge_cache("survival").await?;
```

3 retries with exponential backoff on 5xx / transport errors.

## Purge a batch of file URLs

```rust
use lighty_cdn::CdnClient;

let client = CdnClient::new("cloudflare", zone_id, api_token);
client.purge_files(vec![
    "https://cdn/survival/mods/iris.jar".into(),
    "https://cdn/survival/mods/sodium.jar".into(),
]).await?;
```

Empty URL slices return `Ok(())` without hitting the network.

## Mount the sink on the event bus

```rust
use lighty_cdn::CdnEventSink;
use lighty_events::{CompositeEventSink, EventBus, EventSink};
use lighty_adapters::ConsoleEventSink;
use std::sync::Arc;

let sinks: Vec<Arc<dyn EventSink>> = vec![
    Arc::new(ConsoleEventSink::new()),
    Arc::new(CdnEventSink::new(cdn, cloudflare)),
];
let events = EventBus::with_sink(true, Arc::new(CompositeEventSink::new(sinks)));
```

`CdnEventSink::new` captures `Handle::current()` — must be built
inside a tokio runtime. Both arcs `None` makes the sink a silent
no-op.

## Errors

```rust
pub enum CdnError {
    Http(String),        // from reqwest::Error
    Cloudflare(String),  // API returned success: false
}
```

## See also

- [overview.md](./overview.md)
- [exports.md](./exports.md)
- [flows.md](./flows.md)
