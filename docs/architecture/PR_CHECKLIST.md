# Architecture PR Checklist

Copy this checklist into the PR description for any architecture-affecting change.

- [ ] The change respects `docs/architecture/ARCHITECTURE_CONTRACT_V1.md`.
- [ ] No new internal dependency was added outside `docs/architecture/DEPENDENCY_MATRIX.md`.
- [ ] If a new dependency was required, matrix and rationale were updated in this PR.
- [ ] No duplicate startup/orchestration logic was introduced across binaries.
- [ ] Existing waivers were not silently expanded.
- [ ] Behavior docs were updated when routes/flows/boundaries changed.
- [ ] `cargo check` passes for all changed crates.
