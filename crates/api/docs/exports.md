# Exports

Public surface of `lighty-api`.

## Crate root

```rust
use lighty_api::{build, ApiError, AppState};
```

## `AppState`

```rust
pub struct AppState {
    pub cache:  Arc<lighty_cache::CacheManager>,
    pub config: Arc<tokio::sync::RwLock<lighty_config::Config>>,
}

impl AppState {
    pub fn new(
        cache:  Arc<lighty_cache::CacheManager>,
        config: Arc<tokio::sync::RwLock<lighty_config::Config>>,
    ) -> Self;
}
```

Cloneable via `Arc` semantics; pass it directly to `Router::with_state`.

## `build`

```rust
pub fn build(config: &lighty_config::Config, state: AppState) -> axum::Router;
```

Builds the full router with:

- Routes: `GET /`, `GET /{server}.json`, `GET /{server}/{*path}`.
- Middleware: `CompressionLayer` (configurable), `CorsLayer` (origins
  from config), `TimeoutLayer` (from `server.request_timeout_secs`),
  `RequestBodyLimitLayer` (from `server.max_body_size_mb`).
- State: `AppState`.

## `ApiError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]            Cache(#[from] lighty_cache::CacheError),
    #[error("Not found: {0}")]       NotFound(String),
    #[error("I/O error: {0}")]       Io(#[from] std::io::Error),
}

impl axum::response::IntoResponse for ApiError { /* maps to status codes */ }
```

`NotFound` → `404`, everything else → `500` with a JSON body.

## Cargo features

None.

## See also

- [`overview.md`](./overview.md)
- [`how-to-use.md`](./how-to-use.md)
- [`handlers.md`](./handlers.md)
- [`resolution.md`](./resolution.md)
- [`file-serving.md`](./file-serving.md)
