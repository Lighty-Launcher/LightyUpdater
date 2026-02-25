# Refactor Backlog

Last updated: 2026-02-25

## P0 - Completed

### AR-001: Extract shared server runtime

- Status: Done
- Goal: remove duplicated server bootstrap between `app` and `cli`.
- Result: introduced `lighty-runtime`, made `lighty-app` and `lighty-cli` depend on it.
- Acceptance: both binaries compile and use shared startup wiring.

## P1 - High Priority

### AR-002: Split config model from config I/O

- Status: Done
- Goal: isolate pure config types/defaults from loading/migration side effects.
- Why: reduce coupling from `lighty-config` to `filesystem` and `events`.
- Result:
- `lighty-config` now contains pure config models/defaults.
- `lighty-config-io` now owns config loading and migration behavior.
- Runtime and watcher now use `lighty-config-io` for file loading.

### AR-003: Separate event model from presentation output

- Status: Done
- Goal: keep `lighty-events` as model/contracts, move console formatting to runtime adapters.
- Why: prevent domain/event crate from owning UI concerns.
- Result:
- `lighty-events` now exposes `AppEvent`, `EventBus`, and `EventSink` abstraction.
- Console formatting moved to `lighty-events-console`.
- Runtime now wires the console sink explicitly.

## P2 - Medium Priority

### AR-004: Decompose cache orchestration

- Status: In Progress
- Goal: split `rescan_orchestrator` into smaller services.
- Why: single module currently owns scheduling, file watching, diffing, sync, and purge flow.
- Current progress:
- split orchestration logic into dedicated modules: `rescan_orchestrator`, `rescan_update`, `rescan_sync`, `rescan_scan`.
- reduced single-file hotspot while preserving behavior.

### AR-005: Introduce scanner input ports

- Status: Planned
- Goal: scanner should depend on a narrow scan input model instead of full config/storage knowledge.
- Why: reduce implicit contracts and test surface.
- Acceptance:
- scanner entrypoints accept minimal scan options struct.
- runtime/cache build adapters from full config.

## P3 - Governance

### AR-006: Add architecture checks in CI

- Status: Planned
- Goal: prevent boundary regressions.
- Acceptance:
- automated check of internal dependency graph against matrix.
- PR template includes architecture checklist.
- any exception requires waiver update.
