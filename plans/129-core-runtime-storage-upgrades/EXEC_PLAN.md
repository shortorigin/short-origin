# Execution Plan

## Summary
- Phase 2 upgrades the non-UI runtime, storage, and tooling dependencies to current stable releases, with the largest risk concentrated in the SurrealDB and Wasmtime major-version migrations.

## Task Contract
- Task contract: `plans/129-core-runtime-storage-upgrades/task-contract.json`
- GitHub issue: `#129`
- Branch: `refactor/129-core-runtime-storage-upgrades`

## Scope Boundaries
- Allowed touchpoints are the root manifest and lockfile, `shared/`, `platform/`, `services/`, `workflows/`, `xtask/`, `plans/`, and supporting docs only if migration notes become necessary.
- This phase does not upgrade the UI framework stack or change the workspace edition.

## Implementation Slices
- Re-verify the current latest stable non-UI dependency targets and identify the bounded call sites that require code changes.
- Upgrade SurrealDB under `shared/surrealdb-access` and `services/knowledge-service`, preserving the shared-access boundary and existing schema contracts.
- Upgrade Wasmtime under `platform/runtime/strategy-sandbox` and dependent workflow coverage, preserving deterministic sandbox behavior and error classification.
- Refresh supporting non-UI direct dependencies that need to move with those major upgrades, then validate the full repository.

## Validation Plan
- Run `cargo test -p surrealdb-access --all-targets`.
- Run `cargo test -p knowledge-service --all-targets`.
- Run `cargo test -p strategy-sandbox --all-targets`.
- Run `cargo test -p wasmcloud-bindings --all-targets`.
- Run `cargo test --workspace --all-targets`.
- Run `cargo verify-repo`.

## Rollout and Rollback
- Roll out as a phase-2 PR tied to issue `#129`; because it depends on the phase-1 manifest foundation, it may remain stacked until `#128` merges.
- Roll back the entire phase-2 slice together to avoid mixed runtime expectations across SurrealDB, Wasmtime, and their dependent crates.

## Open Questions
- None.
