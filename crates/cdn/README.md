# lighty-cdn

Cloudflare / CDN HTTP clients and the event sink that drives them.

## Overview

**Version**: 26.5.0
**Part of**: [LightyUpdater](https://github.com/Lighty-Launcher/LightyUpdater)

Two HTTP clients plus a synchronous-friendly `EventSink`:

- `CloudflareClient` — purges a server's metadata JSON via the
  Cloudflare API, with exponential-backoff retries.
- `CdnClient` — fans out per-URL purges (Cloudflare today, CloudFront
  stub planned).
- `CdnEventSink` — listens for `AppEvent::CdnPurgeRequested` and
  `CloudflarePurgeRequested` on the event bus and `tokio::spawn`s the
  actual network call. This is how `lighty-rescan` triggers purges
  without taking a hard dependency on `reqwest`.

## Quick start

Direct use (e.g. from a one-shot CLI):

```rust
use lighty_cdn::CloudflareClient;

let client = CloudflareClient::new(zone_id, api_token);
client.purge_cache("survival").await?;
```

Sink mount (composition root):

```rust
use lighty_cdn::CdnEventSink;
use lighty_events::{CompositeEventSink, EventBus, EventSink};

let sinks: Vec<Arc<dyn EventSink>> = vec![
    Arc::new(ConsoleEventSink::new()),
    Arc::new(CdnEventSink::new(cdn, cloudflare)),
];
let events = EventBus::with_sink(true, Arc::new(CompositeEventSink::new(sinks)));
```

## Errors

`CdnError` covers HTTP transport errors (`#[from] reqwest::Error`) and
API-level failures (`Cloudflare(reason)`). All upper layers wrap it
with `#[from]` so it bubbles up transparently.

## Tests

`wiremock`-driven integration tests cover: 200/success, transient 5xx
retry-then-success, `success: false` failure, empty-URL short-circuit,
CloudFront stub path. Run them with:

```
cargo test -p lighty-cdn
```

## Related crates

- [`lighty-events`](../events) — provides `EventSink`, `AppEvent` and
  the variants this crate listens for.
- [`lighty-rescan`](../rescan) — emits the purge events this sink
  consumes.

## Licence

MIT
