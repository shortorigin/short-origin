# Execution Plan

## Summary
- Integrate the host-installed SurrealDB service into the governed storage layer by adding a durable connection configuration surface, preserving the existing in-memory helpers for tests, and documenting the runtime/infrastructure contract required for reliable service interaction.
- Risk tier: `medium` with multi-plane execution across `shared/`, `infrastructure/`, `docs/`, and supporting repository metadata because the change affects the primary system-of-record path but remains additive.
- Architectural references: `ARCHITECTURE.md`, `docs/architecture/layer-boundaries.md`, `docs/architecture/runtime-composition.md`, `docs/adr/0004-wasmcloud-first-ui-shell.md`, `docs/adr/0007-consistency-semantics-by-criticality.md`, `docs/adr/0010-durable-workflow-execution-plane.md`.

## Task Contract
- Task contract: `plans/123-surrealdb-host-integration/task-contract.json`
- GitHub issue: `#123`
- Branch: `feature/123-surrealdb-host-integration`

## Scope Boundaries
- Allowed touchpoints are limited to `shared/`, `platform/`, `services/`, `workflows/`, `infrastructure/`, `docs/`, `plans/`, `Cargo.toml`, `Cargo.lock`, and `README.md`.
- The implementation keeps all direct SurrealDB client usage inside `shared/surrealdb-access` and any governed higher-level connection helpers inside `shared/governed-storage`.
- Non-goals are adding UI database access, changing canonical schema ownership, or introducing a new managed-database topology.

## Implementation Slices
- Add typed durable connection configuration and validation in `shared/surrealdb-access`, including explicit namespace/database selection and a health check that can confirm the host-installed database is reachable.
- Expose the governed durable connection path from `shared/governed-storage` while preserving the test-only in-memory helper so existing service and workflow tests remain deterministic.
- Update or add tests that prove missing or invalid durable connection settings fail deterministically and that the in-memory helper remains available for local harnesses.
- Update repository and infrastructure-facing docs so operators know which environment variables, host bindings, and validation commands the runtime expects for the SurrealDB host service.

## Validation Plan
- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `cargo test --workspace --all-targets`.
- Run `cargo verify-repo`.
- Capture targeted test evidence for `shared/surrealdb-access` and `shared/governed-storage`.

## Rollout and Rollback
- Rollout is additive: merge after required checks pass, then use the durable connection helper in runtime-facing integration points while keeping in-memory helpers for tests only.
- Rollback is a single revert of the shared storage connection/configuration changes, related tests, and documentation so the repository returns to the previous in-memory-only baseline without mixed assumptions.

## Open Questions
- Determine whether the current workspace needs any additional runtime consumer wiring beyond the shared storage layer once the durable connection helper exists.
