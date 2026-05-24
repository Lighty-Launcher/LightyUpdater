# Using lighty-cdn

Two ways in: drive the clients directly from your own async code, or
mount the sink on the event bus and let rescan trigger it.

## 1. Direct call — purge a single server's JSON

```rust
use lighty_cdn::CloudflareClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CloudflareClient::new(
        std::env::var("CLOUDFLARE_ZONE_ID")?,
        std::env::var("CLOUDFLARE_API_TOKEN")?,
    );

    client.purge_cache("survival").await?;
    Ok(())
}
```

`purge_cache("survival")` POSTs `{"files": ["/survival.json"]}` to
`https://api.cloudflare.com/client/v4/zones/{zone}/purge_cache`. It
retries on 5xx / transport errors and gives up after three attempts.

## 2. Direct call — purge a batch of file URLs

```rust
use lighty_cdn::CdnClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = CdnClient::new(
        "cloudflare",
        std::env::var("CDN_ZONE_ID")?,
        std::env::var("CDN_API_TOKEN")?,
    );

    let urls = vec![
        "https://cdn.example.com/survival/mods/iris.jar".into(),
        "https://cdn.example.com/survival/mods/sodium.jar".into(),
    ];
    client.purge_files(urls).await?;
    Ok(())
}
```

`CdnClient::new` resolves the provider from a string
(`"cloudflare"` today; `"cloudfront"` is a stub). Empty URL slices
short-circuit without contacting the API.

## 3. Mount the sink on a fan-out bus

```rust
use lighty_cdn::CdnEventSink;
use lighty_events::{CompositeEventSink, EventBus, EventSink};
use lighty_adapters::ConsoleEventSink;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cdn        = Some(Arc::new(/* build CdnClient */));
    let cloudflare = Some(Arc::new(/* build CloudflareClient */));

    let sinks: Vec<Arc<dyn EventSink>> = vec![
        Arc::new(ConsoleEventSink::new()),
        Arc::new(CdnEventSink::new(cdn, cloudflare)),
    ];

    let _events = EventBus::with_sink(
        /* verbose */ true,
        Arc::new(CompositeEventSink::new(sinks)),
    );

    // Now: any AppEvent::CdnPurgeRequested or CloudflarePurgeRequested
    // emitted on `_events` will end up on the wire.
    Ok(())
}
```

`CdnEventSink::new` captures `Handle::current()` so it must be
constructed from within a running tokio runtime. With both
`Arc<Option<_>>`s set to `None` it silently drops every purge event —
the safe default when CDN integration is disabled in config.

## 4. Plug a mock with wiremock in tests

```rust
use lighty_cdn::CloudflareClient;
use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn purges_via_mock() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/client/v4/zones/zone-1/purge_cache"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({ "success": true })
        ))
        .mount(&server)
        .await;

    // `with_api_base` is pub(crate); use only inside the cdn crate.
    // For external tests, override env vars and run against the real API.
}
```

The crate's own tests use `CloudflareClient::with_api_base` to swap
the base URL — that helper is `pub(crate)`, so reach for env-based
overrides if you're testing from another crate.

## Errors at a glance

```rust
pub enum CdnError {
    Http(String),                 // wraps reqwest::Error via #[from]
    Cloudflare(String),           // API returned success: false (or stub failure)
}
```

`Http` keeps the original `reqwest` message; `Cloudflare` carries the
human-readable reason for an API-level rejection.

## See also

- [`overview.md`](./overview.md) — what the crate does and how it wires
  into the bus
- [`exports.md`](./exports.md) — full public surface
- [`flows.md`](./flows.md) — sequence diagram of a real purge from
  rescan → sink → API
