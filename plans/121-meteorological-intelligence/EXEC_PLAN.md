# Execution Plan

## Summary
- Add the `meteorological_intelligence` domain, typed weather contracts/events, fixture-backed weather service and workflow, SDK query surfaces, and a governed weather UI app while preserving existing architectural boundaries.
- Risk tier: `high` because the change spans `enterprise/`, `schemas/`, `services/`, `workflows/`, `platform/`, `ui/`, `docs/`, `testing/`, and `plans/`.
- Current follow-up context: the branch has already been rebased onto current `main`; required GitHub workflows were found disabled manually and must be re-enabled for PR checks to surface again.

## Task Contract
- Task contract: `plans/121-meteorological-intelligence/task-contract.json`
- GitHub issue: `#121`
- Branch: `feature/121-meteorological-intelligence`

## Scope Boundaries
- Allowed touchpoints are limited to `enterprise/`, `schemas/`, `services/`, `workflows/`, `platform/`, `ui/`, `testing/`, `docs/`, `plans/`, `Cargo.toml`, and `Cargo.lock`.
- UI integration must remain contract-driven through `platform/sdk` and shared shell contracts; no direct `ui/ -> services/` or direct `ui/ -> SurrealDB` coupling is permitted.
- The issue remains additive: no live external connectors in default validation, no finance-service mutation changes, and no unrelated platform-shell redesign.

## Implementation Slices
- Slice 1: add the `meteorological_intelligence` enterprise domain metadata, supporting ADR, and architecture notes.
- Slice 2: add weather contracts, events, SurrealDB record catalogs, and schema/codegen coverage under `schemas/`.
- Slice 3: implement `services/meteorological-service` with deterministic fixture-backed ingest, weather products, and map-scene delivery surfaces.
- Slice 4: implement `workflows/weather_ingestion` as the approved ingest/backfill mutation path.
- Slice 5: extend `platform/sdk` with typed weather queries and snapshot support, then integrate read-only weather consumers in `knowledge-service` and `quant-research-service`.
- Slice 6: register the governed weather app under `ui/` and keep host/runtime integration aligned with the workspace Leptos/Tauri baseline.
- Slice 7: add execution artifacts, run the full local validation suite on the rebased head, and restore GitHub check visibility by re-enabling the required workflows.

## Validation Plan
- Run `cargo xtask github validate-execution-artifacts --issue-id 121 --branch feature/121-meteorological-intelligence --risk-class high`.
- Run `cargo fmt --all --check`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `cargo test --workspace --all-targets`.
- Run `cargo verify-repo`.
- Run `cargo xtask verify profile ui`.
- Run `cargo xtask ui-hardening`.
- Re-enable the disabled `CI`, `Governance`, and `Security` GitHub workflows and confirm checks appear on PR `#122`.

## Rollout and Rollback
- Rollout is repository-only: merge PR `#122` after required checks are restored, current validation passes, and review is requested.
- Rollback is a full revert of the weather domain, contracts, service/workflow, SDK/UI, fixtures, docs, and execution artifacts so the repository returns cleanly to the prior baseline.

## Open Questions
- None after restoring the required workflow visibility. If GitHub workflow re-enable fails due repository permissions, record that blocker on the PR and stop before assuming checks can be satisfied another way.
