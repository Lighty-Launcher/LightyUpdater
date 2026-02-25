# Architecture Contract v1

Status: Active  
Effective date: 2026-02-25  
Scope: `Cargo.toml` workspace at repository root

## Intent

This contract defines architecture boundaries for all internal crates and binaries.
It exists to prevent new coupling, reduce duplication, and make future refactors predictable.

## Layer Model

- `L0 Domain Core`: `lighty-models`
- `L0 Shared Contracts`: `lighty-events`, `lighty-utils`
- `L1 Infrastructure Adapters`: `lighty-filesystem`, `lighty-storage`, `lighty-config`, `lighty-config-io`, `lighty-events-console`
- `L2 Application Services`: `lighty-scanner`, `lighty-cache`, `lighty-watcher`
- `L3 Interface Adapters`: `lighty-api`
- `L4 Composition Runtime`: `lighty-runtime`
- `L5 Entrypoints`: `lighty-app`, `lighty-cli`
- `L6 Facade`: `lighty-updater`

## Mandatory Rules

1. Dependencies must flow downward only across layers.
2. No crate below `L4` can depend on `lighty-app` or `lighty-cli`.
3. `lighty-runtime` is the single composition root for server startup, lifecycle, and wiring.
4. Entrypoints (`lighty-app`, `lighty-cli`) should stay thin and delegate runtime orchestration to `lighty-runtime`.
5. `lighty-updater` facade can re-export APIs but must not own runtime behavior.
6. Domain data (`lighty-models`) must not depend on infrastructure or application crates.
7. New internal crate dependencies require a matrix update in `docs/architecture/DEPENDENCY_MATRIX.md`.
8. Any boundary exception requires an explicit waiver entry in this file.
9. Any architecture-affecting change must include a completed `docs/architecture/PR_CHECKLIST.md`.
10. Architectural behavior docs and code must stay aligned for routes, flows, and ownership.

## Current Waivers

- `WAIVER-001`: `lighty-cache` still contains orchestration concerns across update/sync/scan modules.
- `WAIVER-002`: `lighty-scanner` depends directly on `lighty-config` and `lighty-storage`.

## Done Criteria For Architecture Changes

- Dependency direction still respects this contract.
- No new duplicate startup/orchestration logic appears across binaries.
- Matrix and backlog docs are updated in the same PR.
- `cargo check` passes for changed crates.
