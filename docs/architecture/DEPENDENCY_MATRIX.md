# Dependency Matrix

Snapshot date: 2026-05-24
Source: `Cargo.toml` files in workspace

## Observed Internal Dependencies

- `lighty-models` -> (none)
- `lighty-events` -> (none)
- `lighty-utils` -> (none)
- `lighty-filesystem` -> (none)
- `lighty-config` -> (none)
- `lighty-storage` -> `lighty-config`
- `lighty-adapters` -> `lighty-config`, `lighty-events`
- `lighty-scanner` -> `lighty-models`, `lighty-config`, `lighty-utils`, `lighty-storage`, `lighty-filesystem`
- `lighty-file-diff` -> `lighty-models`
- `lighty-file-cache` -> `lighty-config`, `lighty-filesystem`
- `lighty-cdn` -> `lighty-events`
- `lighty-rescan` -> `lighty-models`, `lighty-config`, `lighty-events`, `lighty-scanner`, `lighty-storage`, `lighty-file-diff`, `lighty-file-cache`
- `lighty-cache` -> `lighty-models`, `lighty-config`, `lighty-events`, `lighty-storage`, `lighty-file-diff`, `lighty-file-cache`, `lighty-cdn`, `lighty-rescan`
- `lighty-watcher` -> `lighty-config`, `lighty-cache`, `lighty-adapters`, `lighty-filesystem`
- `lighty-api` -> `lighty-cache`, `lighty-models`, `lighty-config`, `lighty-filesystem`
- `lighty-runtime` -> `lighty-api`, `lighty-adapters`, `lighty-cache`, `lighty-cdn`, `lighty-config`, `lighty-events`, `lighty-storage`, `lighty-watcher`, `lighty-filesystem`
- `lighty-app` -> `lighty-runtime`
- `lighty-cli` -> `lighty-runtime`
- `lighty-updater` -> `lighty-models`, `lighty-events`, `lighty-utils`, `lighty-filesystem`, `lighty-config`, `lighty-scanner`, `lighty-cache`, `lighty-watcher`, `lighty-api`

## Notable changes (refactor/cache-modularization)

- `lighty-cache` is now a thin facade (~270 LOC) that composes the four extracted crates and re-exports their public API.
- Rescan no longer calls CDN/Cloudflare directly. Instead, `lighty-rescan` emits `AppEvent::CdnPurgeRequested` / `CloudflarePurgeRequested`, and `lighty-cdn::CdnEventSink` (mounted in `lighty-runtime`) executes the HTTP work on tokio.
- `lighty-events` now supports fan-out through `CompositeEventSink`.

## Allowed Dependencies (v1)

- Leaf domain (`models`, `events`, `utils`, `filesystem`, `config`): no internal deps.
- Pure logic crates (`file-diff`): only `lighty-models`.
- Infra crates (`storage`, `file-cache`): may depend on `lighty-config` plus their own technology deps.
- HTTP clients (`cdn`): may depend on `lighty-events` (for event sink integration) only.
- Scanning (`scanner`, `rescan`): may depend on domain + config + events + storage + file-diff + file-cache.
- Facade (`cache`): may depend on every service crate below it, but exposes them via re-exports only.
- Adapters (`adapters`): may depend on `config`, `events` (for sink implementations).
- Watcher: may depend on `cache`, `config`, `adapters`, `filesystem`.
- API: may depend on `cache`, `models`, `config`, `filesystem`.
- Runtime: composition root — may depend on every service/adapter crate.
- Binaries (`app`, `cli`): only `lighty-runtime`.

## Forbidden Patterns

- Entrypoint crates used as dependencies by service crates.
- New duplicate bootstrap/orchestration logic in `app` and `cli`.
- New cross-layer shortcut dependencies that bypass `lighty-runtime`.
- New direct calls from `lighty-rescan` to `lighty-cdn`: cross-crate side effects must continue to flow through `lighty-events`.
