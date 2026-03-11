# Execution Plan

## Summary
- Phase 4 uplifts the repository from Edition 2021 to Edition 2024 and modernizes eligible internal async interfaces where native `async fn` or RPITIT improves maintainability without breaking required trait-object boundaries.

## Task Contract
- Task contract: `plans/131-edition-2024-rust-modernization/task-contract.json`
- GitHub issue: `#131`
- Branch: `refactor/131-edition-2024-rust-modernization`

## Scope Boundaries
- Allowed touchpoints are the root manifest and lockfile plus the internal crates that need edition or async-interface updates: `shared/`, `platform/`, `services/`, `workflows/`, `ui/`, `xtask/`, and `plans/`.
- Non-goals are contract redesign, unrelated feature work, and removing boxed async host boundaries that still require dyn dispatch.

## Implementation Slices
- Add the Phase 4 execution artifacts so the edition uplift is locally discoverable and bounded.
- Switch the workspace to Edition 2024, remove local `edition = "2021"` overrides, and fix resulting compiler and macro fallout.
- Replace eligible internal `BoxFuture`-based trait methods with native async methods or RPITIT while preserving boxed object-safe host abstractions.
- Re-run the full repository and UI validation surfaces so the stacked modernization branch ends in a fully green state.

## Validation Plan
- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `cargo verify-repo`.
- Run `cargo xtask verify profile ui`.
- Run `cargo xtask ui-hardening`.

## Rollout and Rollback
- Roll out as a dedicated Phase 4 PR tied to issue `#131`, stacked on the completed Phase 3 branch until the dependency-modernization sequence lands.
- Roll back the edition and async-interface changes together to avoid mixed-edition or mixed-async-style assumptions.

## Open Questions
- None.
