# Using lighty-api

## 1. Embed the router

```rust
use lighty_api::{build, AppState};
use std::sync::Arc;

let state  = AppState::new(Arc::clone(&cache_manager), Arc::clone(&config));
let config = config_lock.read().await;
let router = build(&config, state);
```

`build(&config, state)` returns the fully assembled `Router` —
routes, middleware (compression, CORS, timeout, body limit), and
state are all wired. The config snapshot drives the middleware
parameters (timeout duration, max body, allowed origins).

## 2. Serve it with Axum

```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;

axum::serve(listener, router.into_make_service())
    .tcp_nodelay(true)
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.ok();
    })
    .await?;
```

This is exactly what `lighty-runtime::run_server` does.

## 3. Hit the endpoints

```bash
# List enabled servers
curl http://localhost:8080/

# Read a server's manifest
curl http://localhost:8080/survival.json

# Download a file
curl -O http://localhost:8080/survival/mods/iris-1.8.0.jar
```

The file endpoint reads from RAM if `FileCacheManager` has a hit;
otherwise it falls back to a streaming read from disk.

## 4. Inspect cache stats from your own admin endpoint

```rust
let (entries, weighted_kib) = state.cache.get_cache_stats();
```

`AppState` exposes the full `CacheManager` API; build whatever
admin / metrics routes you need on top of it.

## Errors at a glance

```rust
pub enum ApiError {
    Cache(lighty_cache::CacheError),
    NotFound(String),
    Io(io::Error),
    // mapped to status: 4xx for NotFound, 5xx for the rest
}
```

The conversion to `IntoResponse` lives in `errors.rs`.

## See also

- [`overview.md`](./overview.md)
- [`exports.md`](./exports.md)
- [`flow.md`](./flow.md)
- [`resolution.md`](./resolution.md) — URL-to-path resolution
