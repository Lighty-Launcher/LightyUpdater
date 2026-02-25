# Architecture Governance

This folder is the architecture control plane for the workspace.
It defines rules, current boundaries, and the migration plan to clean the architecture without changing runtime behavior.

## Documents

- `ARCHITECTURE_CONTRACT_V1.md`: mandatory boundaries and dependency rules.
- `DEPENDENCY_MATRIX.md`: observed and allowed crate dependency graph.
- `REFACTOR_BACKLOG.md`: prioritized refactor backlog with acceptance criteria.
- `ADOPTION_PLAYBOOK.md`: rollout process, cadence, and metrics.
- `PR_CHECKLIST.md`: architecture gate for every pull request.

## Quick Start

1. Read `ARCHITECTURE_CONTRACT_V1.md`.
2. Validate your change against `DEPENDENCY_MATRIX.md`.
3. Copy `PR_CHECKLIST.md` into your PR description and complete it.

