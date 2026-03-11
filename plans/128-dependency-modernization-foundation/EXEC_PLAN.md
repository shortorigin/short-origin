# Execution Plan

## Summary
- Phase 1 establishes the modernization foundation by adding issue-linked execution artifacts, removing the floating-toolchain hardening blocker, centralizing governed external dependencies in the root workspace manifest, and refreshing `xtask` validation dependencies without changing validation intent.

## Task Contract
- Task contract: `plans/128-dependency-modernization-foundation/task-contract.json`
- GitHub issue: `#128`
- Branch: `infra/128-dependency-modernization-foundation`

## Scope Boundaries
- Allowed touchpoints are `.github/`, `plans/`, the root `Cargo.toml` and `Cargo.lock`, and `xtask/`.
- Non-goals are the SurrealDB, Wasmtime, Leptos, and Edition 2024 migrations reserved for later phases.

## Implementation Slices
- Add `plans/127-rust-dependency-modernization/` and `plans/128-dependency-modernization-foundation/` bundles so the umbrella program and current phase are both locally discoverable.
- Update the workflow hardening path so the pinned Rust toolchain model is recognized and validated.
- Add `workspace.package.rust-version` and expand `[workspace.dependencies]` to cover governed external crates used across crates or critical runtimes, then switch member manifests to workspace inheritance.
- Replace `serde_yaml` with `serde_yaml_ng`, upgrade `jsonschema`, and resolve any resulting `xtask` API adjustments.

## Validation Plan
- Run `cargo fmt --all --check`.
- Run `cargo test -p xtask --all-targets`.
- Run `cargo xtask architecture audit-boundaries`.
- Run `cargo xtask github audit-process`.
- Run `cargo xtask verify profile ui`.
- Run `cargo xtask ui-hardening`.

## Rollout and Rollback
- Roll out as a single phase-1 PR tied to issue `#128`, with the umbrella issue `#127` providing program-level context.
- Roll back all phase-1 changes together so the repo returns to the pre-foundation manifest, workflow, and validator state.

## Open Questions
- None.
