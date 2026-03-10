# Execution Plan

## Summary
- Implement Phase 1 of a governed quantitative decision substrate that adds deterministic decision contracts, probabilistic and game-theoretic baseline evaluation, workflow-authoritative policy gating, and auditable recommendation artifacts without creating autonomous mutation paths.
- Risk tier: `high` because the change spans `enterprise/`, `schemas/`, `shared/`, `services/`, `workflows/`, `agents/`, `testing/`, and `docs/`.
- Architectural references: `ARCHITECTURE.md`, `docs/architecture/layer-boundaries.md`, `docs/adr/0010-durable-workflow-execution-plane.md`, `docs/adr/0013-zero-trust-ai-governance-control-plane.md`, `docs/adr/0015-gitops-and-policy-as-code-control-artifacts.md`, `docs/adr/0017-replaceable-compute-and-governed-state.md`.
- Completed work:
  - Added decision contracts, enterprise governance vocabulary, shared `decision-core`, governance-service evaluation, a read-only `decision_evaluation` workflow, strategist agent alignment, fixture-backed tests, and supporting architecture/ADR documents.
  - Preserved workflow and policy authority by keeping recommendation output advisory, rejected, or executable only through explicit gate results.
- Repository-convention assumptions confirmed during implementation:
  - `plans/` already existed locally as the active execution-artifact surface, so the work followed `plans/<issue-id>-<slug>/`.
  - `cargo verify-repo` already existed locally as the canonical non-UI validation surface, so no new alias or `xtask` profile had to be introduced in this change set.
- Working-note inventory from the repository scan:
  - Policy semantics: `enterprise/policies/*.toml`, `enterprise/crates/policy-registry`, `enterprise/domains/*/service_boundaries.toml`, `services/policy-service`, `platform/runtime/enforcement`.
  - Scoring and evaluation: `shared/trading-core`, `services/quant-research-service`, `workflows/quant_strategy_promotion`, `services/governance-service`.
  - Workflow gating: `workflows/engine`, `services/policy-service`, `services/approval-service`, `enterprise/domains/strategy_governance/service_boundaries.toml`.
  - Telemetry and validation: `shared/telemetry`, `xtask verify`, `xtask github audit-process`, `docs/process/platform-regression-guardrails.md`.
  - Agent procedures: `agents/procedures/strategist.md`, `agents/strategist/allowed_actions.toml`, `agents/strategist/allowed_inputs.toml`, `agents/runtime`.
  - Execution artifacts: `plans/README.md`, `plans/templates/*`, `docs/process/execution-artifacts.md`.

## Task Contract
- Task contract: `plans/119-quantitative-decision-architecture/task-contract.json`
- GitHub issue: `#119`
- Branch: `feature/119-quantitative-decision-architecture`

## Scope Boundaries
- Allowed touchpoints are limited to `enterprise/`, `schemas/`, `shared/`, `services/`, `workflows/`, `agents/`, `testing/`, `docs/`, `plans/`, `Cargo.toml`, `Cargo.lock`, and `README.md`.
- Impacted modules are `enterprise/crates/policy-registry`, `schemas/crates/contracts`, `schemas/crates/codegen`, `schemas/crates/surrealdb-model`, `shared/decision-core`, `services/governance-service`, `workflows/decision_evaluation`, `agents/strategist`, and `agents/runtime`.
- Non-goals are adding UI decision surfaces, introducing remote or online learned inference, or allowing decision outputs to mutate systems outside workflow and policy gates.

## Implementation Slices
- Completed slices:
  - Added ten JSON decision contracts under `schemas/contracts/v1/` plus Rust mirrors, schema-registry embedding, and serde round-trip coverage.
  - Added `enterprise/policies/decision_governance.toml`, ontology entity updates, and strategy-governance service-boundary ownership for decision recommendation and audit artifacts.
  - Added `shared/decision-core` with documented score types, composable traits, seeded Monte Carlo and Thompson-sampling baselines, bounded normal-form game analysis, deterministic learned-adapter stubs, and policy-gated recommendation assembly.
  - Extended `services/governance-service` with pure decision evaluation and added `workflows/decision_evaluation` as the canonical read-only orchestration path.
  - Updated strategist inputs/procedures, added SurrealDB representability, and added fixture-backed integration tests plus new architecture and ADR docs.
- Files changed:
  - `Cargo.toml`
  - `Cargo.lock`
  - `README.md`
  - `agents/procedures/strategist.md`
  - `agents/runtime/tests/governance.rs`
  - `agents/strategist/allowed_actions.toml`
  - `agents/strategist/allowed_inputs.toml`
  - `docs/README.md`
  - `docs/adr/0018-adopt-quantitative-decision-architecture-for-governed-platform-automation.md`
  - `docs/adr/README.md`
  - `docs/architecture/quantitative-decision-architecture.md`
  - `enterprise/crates/policy-registry/src/lib.rs`
  - `enterprise/crates/policy-registry/tests/policies.rs`
  - `enterprise/domains/strategy_governance/service_boundaries.toml`
  - `enterprise/ontology/entities.json`
  - `enterprise/policies/decision_governance.toml`
  - `plans/119-quantitative-decision-architecture/EXEC_PLAN.md`
  - `plans/119-quantitative-decision-architecture/task-contract.json`
  - `schemas/contracts/v1/decision-audit-record-v1.json`
  - `schemas/contracts/v1/decision-constraint-v1.json`
  - `schemas/contracts/v1/decision-context-v1.json`
  - `schemas/contracts/v1/decision-option-v1.json`
  - `schemas/contracts/v1/decision-recommendation-v1.json`
  - `schemas/contracts/v1/outcome-distribution-v1.json`
  - `schemas/contracts/v1/outcome-estimate-v1.json`
  - `schemas/contracts/v1/policy-gate-result-v1.json`
  - `schemas/contracts/v1/risk-assessment-v1.json`
  - `schemas/contracts/v1/utility-breakdown-v1.json`
  - `schemas/crates/codegen/src/lib.rs`
  - `schemas/crates/codegen/tests/schema_registry.rs`
  - `schemas/crates/contracts/src/decision.rs`
  - `schemas/crates/contracts/src/lib.rs`
  - `schemas/crates/contracts/tests/decision_contracts.rs`
  - `schemas/crates/surrealdb-model/src/lib.rs`
  - `schemas/surrealdb/v1/record-types-v1.json`
  - `services/governance-service/Cargo.toml`
  - `services/governance-service/src/lib.rs`
  - `shared/decision-core/Cargo.toml`
  - `shared/decision-core/src/engine.rs`
  - `shared/decision-core/src/game.rs`
  - `shared/decision-core/src/learned.rs`
  - `shared/decision-core/src/lib.rs`
  - `shared/decision-core/src/policy.rs`
  - `shared/decision-core/src/probabilistic.rs`
  - `shared/decision-core/src/scores.rs`
  - `testing/fixtures/decisioning/run-2026-03-10/advisory_recommendation.json`
  - `testing/fixtures/decisioning/run-2026-03-10/policy_rejection_recommendation.json`
  - `testing/fixtures/decisioning/run-2026-03-10/ranked_audit.json`
  - `testing/fixtures/decisioning/run-2026-03-10/rollout_recommendation.json`
  - `workflows/decision_evaluation/Cargo.toml`
  - `workflows/decision_evaluation/src/lib.rs`
  - `workflows/decision_evaluation/tests/decision_evaluation.rs`

## Validation Plan
- Commands executed:
  - `cargo fmt --all`
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets`
  - `cargo verify-repo`
  - `cargo xtask verify profile ui`
  - `cargo xtask ui-hardening`
- Validation results:
  - `cargo fmt --all --check`: passed.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed after tightening `decision-core` to satisfy workspace pedantic lint rules.
  - `cargo test --workspace --all-targets`: passed.
  - `cargo verify-repo`: passed, including `xtask architecture audit-boundaries`, `xtask plugin validate-manifests`, and `xtask github audit-process`.
  - `cargo xtask verify profile ui`: passed.
  - `cargo xtask ui-hardening`: command completed and produced a remediation report, but the report status remained `INCOMPLETE` because repository CI still uses a floating Rust toolchain instead of pinning `dtolnay/rust-toolchain@1.91.1`. This was reported as an existing repository hardening defect and was not introduced by this decisioning change set.

## Rollout and Rollback
- Rollout is repository-only: merge after required checks pass, then use the read-only workflow and governance-service entry point as the authoritative evaluation path until future phases add broader runtime hookups.
- Rollback is a full revert of the decision contracts, shared crate, governance-service/workflow integration, agent procedure changes, fixtures, and docs so no partial advisory pathway remains.
- Known limitations:
  - Decision recommendation and audit artifacts are stored only in service-local memory for Phase 1; durable persistence is intentionally deferred.
  - Learned-model integration remains interface-only and returns `NotConfigured` by default.
  - The new workflow path is read-only and does not add mutation hooks beyond existing governed workflow boundaries.

## Open Questions
- No blocking open questions remain for Phase 1 delivery.
- Follow-up items:
  - Add durable governed storage and retrieval for decision recommendation and audit records when the runtime persistence hookup is available.
  - Add governed local learned-model adapters, model metadata, and evaluation policy once the repository defines the next-phase inference controls.
  - Resolve the existing `cargo xtask ui-hardening` remediation finding by pinning the repository’s Rust-installing CI workflows to `dtolnay/rust-toolchain@1.91.1`.
