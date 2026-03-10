---
adr_id: ADR-0018
title: Adopt Quantitative Decision Architecture for Governed Platform Automation
status: Adopted
date_adopted: 2026-03-10
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - schemas
  - shared
  - services
  - workflows
  - agents
  - docs
impacted_domains:
  - strategy_governance
  - operations
  - audit_assurance
source_report: docs/architecture/quantitative-decision-architecture.md
roadmap_phases:
  - Phase 4
  - Phase 5
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - decisioning
  - governance
  - probabilistic
  - game-theory
  - explainability
---

# ADR 0018: Adopt Quantitative Decision Architecture for Governed Platform Automation

## Context

Origin already enforces workflow-gated mutation, typed contracts, explicit policy ownership, and
auditable repository control artifacts. Strategic and operational decisions still need a common
evaluation substrate that can score alternatives, represent uncertainty, explain tradeoffs, and
preserve workflow and policy authority across repository planes.

## Decision

Adopt a repository-integrated quantitative decision architecture as a governed advisory substrate.
The architecture shall:

- use versioned contracts in `schemas/` for decision context, options, constraints, recommendations,
  audit records, and policy-gate results;
- use deterministic shared Rust primitives for quantitative scoring, uncertainty handling, and
  recommendation assembly;
- use `governance-service` as the pure evaluation host and `workflows/decision_evaluation` as the
  read-only orchestration path;
- emit ranked recommendations, explicit policy-gate results, and reconstructable audit artifacts for
  every evaluation path.

## Constraints

- Game theory is used for bounded strategic interaction modeling, not as a replacement for workflow
  authority.
- Probabilistic algorithms are used for uncertainty-aware scoring, selection, and risk handling.
- Deep learning is Phase 1 interface-only: learned adapters may contribute in the future, but they
  are not directly empowered to mutate system state in this phase.
- Workflow and policy gates remain authoritative for executable actions.
- Phase 1 must remain deterministic, seedable, explainable, and additive to the existing
  architecture.
- Decision logic must stay out of `ui/` and `infrastructure/`.

## Alternatives Considered

- `Service-local heuristics`: rejected because duplicating evaluation logic across services would
  fragment governance, auditability, and contract ownership.
- `Direct learned controller`: rejected because opaque model-driven mutation would violate current
  workflow and policy constraints.
- `UI-owned decisioning`: rejected because presentation must not own domain logic or governance.
- `Infrastructure-owned policy evaluation`: rejected because business-domain semantics belong in
  enterprise, contracts, shared crates, services, and workflows.

## Rationale

This decision keeps Origin aligned to its existing contract-first, workflow-gated architecture while
making strategic and operational evaluation machine-executable. Bounded game analysis and seeded
probabilistic baselines provide practical utility immediately, while learned-model interfaces remain
replaceable and governed rather than privileged.

## Implementation Implications

- Add decision contracts and schema-registry coverage in `schemas/`.
- Add enterprise vocabulary for decision governance and ontology entities for recommendation and
  audit records.
- Add a shared decision core for score types, traits, probabilistic algorithms, game analysis,
  policy gating, and recommendation composition.
- Extend governance-service and add a read-only decision-evaluation workflow.
- Preserve auditability with structured recommendation and audit artifacts, deterministic seeds, and
  explicit rejection rationale.
- Defer durable storage hookups, remote inference, and autonomous mutation paths to later phases.

## Acceptance Checks

- Decision evaluation returns structured recommendation, policy-gate, and audit outputs.
- Seeded stochastic algorithms produce stable results under repeated test execution.
- Learned adapters default to non-operative advisory behavior when not configured.
- Workflow and policy gates remain the only path to executable actions.
- No decisioning logic is embedded in `ui/` or `infrastructure/`.

## Traceability

- Governing source: `docs/architecture/quantitative-decision-architecture.md`
- Related roadmap phases: `Phase 4`, `Phase 5`
- Related issues or PRs: `#119`
