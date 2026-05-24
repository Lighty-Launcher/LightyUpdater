# Exports

Public surface of `lighty-cdn`.

## Crate root

```rust
use lighty_cdn::{CdnClient, CdnEventSink, CdnError, CdnProvider, CloudflareClient};
```

## Type details

### `CdnClient`

```rust
pub struct CdnClient { /* provider + zone_id + api_token + reqwest::Client */ }

impl CdnClient {
    pub fn new(provider: &str, zone_id: String, api_token: String) -> Self;

    pub async fn purge_files(&self, file_urls: Vec<String>) -> Result<(), CdnError>;
}
```

`provider` is matched case-insensitively against `"cloudfront"`;
anything else falls back to Cloudflare. Empty URL slices return
`Ok(())` without contacting the network.

### `CloudflareClient`

```rust
pub struct CloudflareClient { /* zone_id + api_token + reqwest::Client + base URL */ }

impl CloudflareClient {
    pub fn new(zone_id: String, api_token: String) -> Self;

    pub async fn purge_cache(&self, server_name: &str) -> Result<(), CdnError>;
}
```

`purge_cache(name)` purges the single file `/{name}.json` — that's
the manifest endpoint the launcher fetches before any download.

### `CdnEventSink`

```rust
pub struct CdnEventSink { /* opt Arcs + tokio::runtime::Handle */ }

impl CdnEventSink {
    pub fn new(
        cdn: Option<Arc<CdnClient>>,
        cloudflare: Option<Arc<CloudflareClient>>,
    ) -> Self;
}

impl lighty_events::EventSink for CdnEventSink {
    fn handle(&self, event: &AppEvent);
}
```

Captures `tokio::runtime::Handle::current()` at construction. Listens
for `AppEvent::CdnPurgeRequested { server, urls }` and
`AppEvent::CloudflarePurgeRequested { server }`, and `tokio::spawn`s
the matching client call. With both arcs set to `None` the sink is a
no-op.

### `CdnProvider`

```rust
#[derive(Debug, Clone)]
pub enum CdnProvider { Cloudflare, CloudFront }
```

Resolved internally by `CdnClient::new`; not constructed by callers.

### `CdnError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum CdnError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Cloudflare API error: {0}")]
    Cloudflare(String),
}

impl From<reqwest::Error> for CdnError { /* maps to CdnError::Http */ }
```

Surfaced on `CacheError` via `#[from]`, so callers that already match
on `CacheError` keep working.

## Cargo features

None. All deps are non-optional.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`flows.md`](./flows.md)
- [`../../events/docs/exports.md`](../../events/docs/exports.md) — the
  `AppEvent` variants this crate consumes
