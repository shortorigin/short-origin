# Execution Plan

## Summary
- Execute the repository-wide Rust modernization as a phased program: phase 1 establishes deterministic workflow and dependency governance, phases 2 and 3 upgrade the non-UI and UI stacks separately, and phase 4 completes the Edition 2024 uplift with targeted Rust-language cleanup.

## Task Contract
- Task contract: `plans/127-rust-dependency-modernization/task-contract.json`
- GitHub issue: `#127`
- Branch: `infra/127-rust-dependency-modernization-program`

## Scope Boundaries
- Allowed touchpoints are limited to the workspace manifest and lockfile, `.github/`, `xtask/`, `plans/`, and the Rust workspace crates under `shared/`, `platform/`, `services/`, `workflows/`, `ui/`, and `schemas/` when additive compatibility work is required.
- Non-goals are architectural dependency-direction changes, direct SurrealDB access outside `shared/surrealdb-access`, schema ownership drift out of `schemas/`, and browser/desktop UI contract forks.

## Implementation Slices
- Create the umbrella issue plus bounded phase issues and attach matching `plans/<issue-id>-<slug>/` bundles.
- Phase 1: pin workflow toolchain handling, centralize governed dependencies, and modernize `xtask` validation dependencies without changing validation intent.
- Phase 2: upgrade non-UI runtime, storage, and tooling dependencies while keeping contracts stable.
- Phase 3: upgrade the UI framework and WASM/Tauri stack as a coordinated unit.
- Phase 4: switch to Edition 2024 and modernize eligible first-party async interfaces while preserving required object-safe host APIs.

## Validation Plan
- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `cargo verify-repo`.
- Run `cargo xtask verify profile ui`.
- Run `cargo xtask ui-hardening`.

## Rollout and Rollback
- Roll out as one phase issue and PR at a time, keeping each branch mapped to exactly one primary phase issue.
- Roll back per phase so the repository never remains between incompatible framework, storage, or edition states.

## Open Questions
- None.
