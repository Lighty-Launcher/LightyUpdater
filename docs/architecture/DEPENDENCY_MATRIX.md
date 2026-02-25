# Dependency Matrix

Snapshot date: 2026-02-25  
Source: `Cargo.toml` files in workspace

## Observed Internal Dependencies

- `lighty-models` -> (none)
- `lighty-events` -> (none)
- `lighty-events-console` -> `lighty-events`
- `lighty-utils` -> (none)
- `lighty-filesystem` -> `lighty-utils`
- `lighty-config` -> (none)
- `lighty-config-io` -> `lighty-config`, `lighty-events`
- `lighty-storage` -> `lighty-config`
- `lighty-scanner` -> `lighty-models`, `lighty-config`, `lighty-utils`, `lighty-storage`
- `lighty-cache` -> `lighty-models`, `lighty-config`, `lighty-events`, `lighty-scanner`, `lighty-filesystem`, `lighty-storage`
- `lighty-watcher` -> `lighty-config`, `lighty-config-io`, `lighty-cache`, `lighty-filesystem`
- `lighty-api` -> `lighty-cache`, `lighty-models`, `lighty-config`, `lighty-filesystem`
- `lighty-runtime` -> `lighty-api`, `lighty-cache`, `lighty-config`, `lighty-config-io`, `lighty-events`, `lighty-events-console`, `lighty-storage`, `lighty-watcher`, `lighty-filesystem`
- `lighty-app` -> `lighty-runtime`
- `lighty-cli` -> `lighty-runtime`
- `lighty-updater` -> `lighty-models`, `lighty-events`, `lighty-utils`, `lighty-filesystem`, `lighty-config`, `lighty-scanner`, `lighty-cache`, `lighty-watcher`, `lighty-api`

## Allowed Dependencies (v1)

- `lighty-models`: no internal deps.
- `lighty-events`: no internal deps.
- `lighty-events-console`: only `lighty-events`.
- `lighty-utils`: no internal deps.
- `lighty-filesystem`: only `lighty-utils`.
- `lighty-config`: no internal deps (pure model crate).
- `lighty-config-io`: `lighty-config`, `lighty-events`.
- `lighty-storage`: `lighty-config` (waiver pending config port split).
- `lighty-scanner`: `lighty-models`, `lighty-utils`, `lighty-config`, `lighty-storage` (waiver pending scan input port).
- `lighty-cache`: `lighty-models`, `lighty-events`, `lighty-scanner`, `lighty-storage`, `lighty-config`, `lighty-filesystem`.
- `lighty-watcher`: `lighty-cache`, `lighty-config`, `lighty-config-io`, `lighty-filesystem`.
- `lighty-api`: `lighty-cache`, `lighty-models`, `lighty-config`, `lighty-filesystem`.
- `lighty-runtime`: may depend on service/adapters crates only.
- `lighty-app`: only `lighty-runtime`.
- `lighty-cli`: only `lighty-runtime` for serve mode.
- `lighty-updater`: re-export facade only.

## Forbidden Patterns

- Entrypoint crates used as dependencies by service crates.
- New duplicate bootstrap/orchestration logic in `app` and `cli`.
- New cross-layer shortcut dependencies that bypass `lighty-runtime`.
