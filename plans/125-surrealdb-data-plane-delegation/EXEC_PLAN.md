# Execution Plan

## Summary
- Delegate the `data_knowledge` pilot domain's storage, searchable retrieval, graph lineage, read-model materialization, and typed change propagation to SurrealDB while preserving service/workflow authority for policy-gated mutation and approvals.
- Risk tier: `high` because the change spans `schemas/`, `shared/`, `services/`, `workflows/`, `platform/`, `docs/`, and `plans/`.
- Related issues: `#123` remains the durable runtime connectivity prerequisite, `#113` remains the workflow transactionality prerequisite, and the repository-boundary cleanup intent from `#106` is folded into the touched knowledge-path refactor.

## Task Contract
- Task contract: `plans/125-surrealdb-data-plane-delegation/task-contract.json`
- GitHub issue: `#125`
- Branch: `feature/125-surrealdb-data-plane-delegation`

## Scope Boundaries
- Allowed touchpoints are limited to `schemas/`, `shared/`, `services/`, `workflows/`, `platform/`, `docs/`, `plans/`, `testing/`, `Cargo.toml`, `Cargo.lock`, and `README.md`.
- The umbrella delivers reusable shared substrate plus a full `data_knowledge` pilot only; non-knowledge domains stay on follow-up issues once the pattern is proven.
- Database-side logic is limited to schema enforcement, graph relations, projections, and change propagation. Approval semantics, external fetches, policy checks, and workflow sequencing stay in services/workflows.

## Implementation Slices
- Slice 1: uplift `surrealdb` to a single workspace dependency and align the shared access layer plus touched consumers to the newer SDK surface.
- Slice 2: replace the record-name-only schema surface with versioned DDL artifacts and embed them in the schema registry.
- Slice 3: extend `schemas/crates/surrealdb-model` and `shared/governed-storage` with typed search, graph, projection, and change-notification models and ports.
- Slice 4: implement durable connection/configuration, DDL bootstrapping, transactions, graph traversal, search, read-model queries, and typed notifications in `shared/surrealdb-access`.
- Slice 5: migrate `services/knowledge-service` and `workflows/knowledge_publication` from Memvid runtime retrieval to SurrealDB-backed chunks, relations, projections, and atomic persistence.
- Slice 6: remove runtime-facing dependence on `platform/runtime/memory-provider` from the touched knowledge flow, retain only the governed shared access path, and refresh docs/tests.

## Validation Plan
- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `cargo test --workspace --all-targets`.
- Run `cargo verify-repo`.
- Add targeted tests for DDL bootstrapping, retrieval behavior, projection correctness, typed notifications, and rollback on transactional failure.

## Rollout and Rollback
- Rollout is repository-only: merge after required checks pass, then use the shared SurrealDB substrate as the sole runtime retrieval/storage path for `data_knowledge`.
- Rollback is a full revert of the version uplift, DDL artifacts, shared access changes, knowledge-path migration, and docs/tests so runtime behavior returns to the prior baseline without hybrid drift.

## Open Questions
- None at implementation start. If host/runtime compatibility fails under the SurrealDB uplift, stop after the upgrade slice and re-scope rather than silently restoring the old runtime retrieval path.
