# Workflows

## Purpose
`workflows/` defines cross-domain operational flows (payroll, procurement, contract lifecycle, incident response) that orchestrate services using contract-driven, policy-aware execution.

## Scope
In scope:
- Workflow definitions under `workflows/*`.
- State transitions, compensating actions, and escalation rules.
- Adjacent wasmCloud component adapters for deployable workflow workloads.
- Policy gating integration with `enterprise/policies`.
- Cross-service capital-markets orchestration such as `quant_strategy_promotion`, which coordinates data ingest, research, simulation, execution evidence, compliance, and governance handoff.

Out of scope:
- Low-level service business logic.
- Source-of-truth schema ownership.
- Infrastructure provisioning.

## Interfaces
- Input interfaces: workflow triggers from services, operators, or agents.
- Control interfaces: contract-bound commands dispatched to services.
- Output interfaces: event emissions and audit artifacts for downstream consumers.

## Dependencies
- `services/` for executable capabilities.
- `schemas/` for trigger, command, and event contracts.
- `enterprise/` for policy and compliance constraints.
- `platform/runtime` for orchestration execution environment.
- `platform/wasmcloud` for deployable workflow component bindings.

## Development Workflow
1. Define workflow states, transitions, and failure handling.
2. Bind each transition to explicit service contracts.
3. Add policy checkpoints and audit event outputs.
4. Create test scenarios for nominal, failure, and recovery paths.
5. Start with a GitHub issue, link it from the PR, and capture workflow failure, compensation, policy-gate, and audit-output evidence in the issue or PR notes.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --features integration
```

## Integration Patterns
- Workflows orchestrate via contracts/events, never through private service internals.
- Every workflow step must emit auditable state changes.
- Incident and compliance-sensitive flows require explicit policy checks before side effects.
- Promotion workflows hand off final approval to canonical downstream workflows such as `strategy_review` and `compliance_attestation` instead of embedding local approval logic.
- UI surfaces consume workflow visibility through typed SDK snapshots and event streams, not direct workflow internals.

## Reuse Opportunities
- Reuse shared orchestration primitives for retries, timeouts, and compensations.
- Standardize approval and escalation subflows across domains.

## Out of Scope
- Defining new domain ontology terms.
- Direct database schema ownership.
- Long-lived UI state management.
