# Adoption Playbook

## Purpose

This playbook operationalizes the architecture contract in day-to-day delivery.

## Rollout Steps

1. Merge and announce `docs/architecture/*` as the source of truth.
2. Use `PR_CHECKLIST.md` for every architecture-affecting PR.
3. Review dependency changes before merge.
4. Keep waivers explicit and time-bound.

## Team Cadence

- Every PR: complete architecture checklist.
- Weekly (30 min): review matrix/backlog delta and new waivers.
- Monthly (45 min): architecture metrics review and priority refresh.

## Metrics

- `M1 Runtime Duplication`: count duplicated startup/orchestration modules across binaries.
- `M2 Layer Violations`: count of internal dependencies outside matrix.
- `M3 Orchestration Hotspots`: count of modules above 300 lines in orchestrator domains.
- `M4 Waiver Count`: open waivers and average age.
- `M5 Documentation Drift`: mismatches between docs and active routes/flows.

## Guardrails

- No new internal dependency without matrix update.
- No new waiver without expiration target.
- No architecture-affecting merge without checklist completion.
- No route/flow documentation changes left stale after behavior changes.

## Escalation Path

1. Mark issue in PR with violated rule ID.
2. Decide fix now or waiver with due date.
3. Record decision in backlog and next review agenda.
