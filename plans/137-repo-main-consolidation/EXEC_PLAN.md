# Execution Plan

## Summary
- Consolidate the repository onto a clean, validated main lineage by merging the real remaining work branches, reconciling stale divergence where needed, and pruning already-landed or derivative refs only after the integrated result is green.

## Task Contract
- Task contract: `plans/137-repo-main-consolidation/task-contract.json`
- GitHub issue: `#137`
- Branch: `infra/137-repo-main-consolidation`

## Scope Boundaries
- Allowed touchpoints are the issue-linked `plans/` bundle, any repository paths touched by the merged branches and required stabilization fixes, `DEVELOPMENT_MODEL.md` if temporary placeholders must be documented, and the git refs/tags required for archival pruning.
- Non-goals are bypassing branch protection, promoting stale stash-only notes into `main`, or preserving stale branch tips where documented reconciliation is safer.

## Implementation Slices
- Record the consolidation issue and execution artifact bundle on the integration branch.
- Summarize the stale stash-only notes in the issue, then drop the stash entries before branch cleanup work begins.
- Merge `refactor/129-core-runtime-storage-upgrades` and `refactor/131-edition-2024-rust-modernization`, preserving the newer edition-2024 and UI/runtime API shapes while restoring any still-needed dependency or storage behavior from `129`.
- Merge `feature/121-meteorological-intelligence`, preserving the weather surfaces without regressing the modernization choices from the dependency and edition branches.
- Reconcile `feature/125-surrealdb-data-plane-delegation` by porting only still-missing additive knowledge/SurrealDB behavior into one curated follow-up commit, or mark it superseded if nothing additive remains.
- Merge the validated `refactor/136-rust-build-hygiene-tracing-baseline` branch.
- Stabilize the integrated branch until repo-wide and UI-specific validation is green, including the existing `shared/governed-storage` `clippy::manual_async_fn` failures.
- Fast-forward local `main` to the validated result, push the update, and prune/archive the merged, superseded, and derivative branches.

## Validation Plan
- Run `cargo check --workspace` plus focused tests after each merge.
- Run `cargo xtask verify profile ui` after merging `refactor/131`.
- Run the package-level weather tests after merging `feature/121`.
- Run `cargo test -p xtask`, `cargo test -p telemetry`, `cargo check -p desktop_tauri`, `cargo xtask verify profile ui`, and `cargo xtask ui-hardening` after merging `refactor/136`.
- Run `cargo fmt --all --check`, `cargo verify-repo`, `cargo xtask verify profile ui`, and `cargo xtask ui-hardening` before promoting `main`.

## Rollout and Rollback
- Roll forward through explicit merge commits and one optional reconciliation commit so each integration step remains identifiable and reversible.
- Keep archive tags for every pruned non-directly-merged branch and retain the integration branch until `main` is confirmed current and the cleanup is complete.

## Open Questions
- None. The consolidation rules for `feature/125`, the stale stashes, validation thresholds, and branch provenance have all been fixed in advance.
