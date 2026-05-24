# Overview

`lighty-api` is the HTTP surface of LightyUpdater. It exposes three
routes implemented over Axum:

| Method | Path | Returns |
|---|---|---|
| `GET` | `/` | JSON array of every enabled server (with `loader_version`). |
| `GET` | `/{server}.json` | The `VersionBuilder` of the named server, serialized. |
| `GET` | `/{server}/{...rest}` | A file from the server's working directory (mod, library, asset, native, client). |

Everything else is plumbing: `AppState` (the per-request handle to
the cache), URL resolution (cache lookup + on-disk fallback), and
error mapping to HTTP status codes.

## What's inside

| Module | Purpose |
|---|---|
| `routes` | Builds the Axum `Router<AppState>` with the three routes plus tower-http middleware (CORS, compression, timeout, request limit). |
| `handlers::servers` | `list_servers` and `get_server_metadata`. |
| `handlers::files` | `serve_file`, plus the `resolver` submodule that translates URL paths to on-disk paths. |
| `handlers::files::cache` | RAM-first read path: hit `FileCacheManager::get_file` and bail out before touching disk. |
| `handlers::state` | `AppState { cache, config }`. |
| `errors` | `ApiError` with `#[from]` from `CacheError` plus the HTTP status mapping. |

## Routing

```mermaid
flowchart LR
    Cli[HTTP client] -->|GET /| LS[list_servers]
    Cli -->|GET /{name}.json| GM[get_server_metadata]
    Cli -->|GET /{name}/path/to/file| SF[serve_file]

    LS --> CM[CacheManager::get_all_servers]
    GM --> Cache[CacheManager::get]
    SF --> Res[resolver: cache key lookup]
    Res --> RAM{FileCacheManager hit?}
    RAM -->|yes| Resp[Body::from(Bytes)]
    RAM -->|no| Disk[FileSystem::read_bytes]
    Disk --> Resp
```

## Response semantics

- `Content-Type` from `mime_guess` (cached on the `FileCache` entry).
- `ETag` from the SHA1 (also cached on `FileCache`).
- 404 when the server is unknown or the file is missing.
- 5xx when the cache layer surfaces a real I/O error.

## Cargo features

None.

## See also

- [`how-to-use.md`](./how-to-use.md) — how to embed the router in
  your own binary
- [`exports.md`](./exports.md) — full public surface
- [`architecture.md`](./architecture.md), [`flow.md`](./flow.md),
  [`handlers.md`](./handlers.md), [`resolution.md`](./resolution.md),
  [`file-serving.md`](./file-serving.md) — deeper dives that pre-date
  the cache refactor (still accurate for the API layer)
- [`../../cache/docs/overview.md`](../../cache/docs/overview.md) — the
  facade this crate reads from
