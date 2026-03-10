# Quantitative Decision Architecture

## Purpose

Phase 1 establishes a governed quantitative decision substrate for Origin. It is not a full
autonomous control plane. The phase delivers machine-executable evaluation, bounded strategic
analysis, uncertainty-aware scoring, policy-authoritative recommendation status, and audit-ready
artifacts while leaving all mutations workflow-gated.

## Decision Taxonomy

Phase 1 supports five primary decision classes:

- `strategic_prioritization`: compare competing initiatives, sequencing choices, and institutional
  tradeoffs.
- `release_risk_assessment`: evaluate rollout alternatives, release guardrails, and promotion
  posture.
- `resource_allocation`: score competing staffing, budget, capacity, or compute allocations.
- `anomaly_escalation_routing`: route incidents, anomaly triage, and escalation decisions under
  uncertainty.
- `policy_constrained_action_selection`: select among feasible actions where governance thresholds
  and reversibility remain explicit.

Each decision must preserve a stable identifier, provenance metadata, evaluation seed, constraints,
ranked options, policy status, and rollback or reversibility context.

## Decision Lifecycle

The Phase 1 engine and workflow path use this ordered lifecycle:

1. `context_ingestion`: load `DecisionContextV1` and preserve provenance, actor, decision class,
   seed, and policy references.
2. `option_normalization`: sort and normalize bounded option data for deterministic processing.
3. `constraint_checking`: evaluate hard and soft constraints before policy authority is considered.
4. `probabilistic_scoring`: compute weighted expected value, seeded Monte Carlo summaries,
   Thompson-sampling scores, confidence aggregation, and risk aggregation.
5. `game_analysis`: analyze finite normal-form games when supplied, including dominated-strategy
   elimination, best responses, equilibrium detection, and scenario summary output.
6. `utility_aggregation`: combine expected value, probabilistic sampling, confidence, risk, and
   bounded strategic bonuses into final utility scores.
7. `recommendation_selection`: rank options and choose the provisional best option.
8. `policy_gating`: determine whether the ranked result is executable, advisory, rejected, or
   non-executable.
9. `audit_record_creation`: capture the full context, evaluated options, gate result, engine trace,
   assumptions, and learned-adapter status.
10. `workflow_consumption`: hand the advisory or executable result to a workflow path that remains
    authoritative for any later side effect.

## Repository Plane Responsibilities

| Plane | Phase 1 responsibility |
| --- | --- |
| `enterprise/` | Define decision governance vocabulary, risk tiers, approval requirements, reversibility classes, and ontology names for recommendation and audit artifacts. |
| `schemas/` | Own versioned decision contracts, schema registry entries, and SurrealDB record representability for auditable artifacts. |
| `shared/` | Provide `decision-core`: score types, traits, probabilistic baselines, game analysis, policy gating, learned-model extension points, and decision composition. |
| `services/` | Host pure evaluation in `governance-service` and return structured recommendation packages without mutating external systems. |
| `workflows/` | Own the read-only `decision_evaluation` workflow boundary and preserve workflow-authoritative control over any later execution path. |
| `agents/` | Allow governed read-only requests for decision evaluation through approved procedures only. |
| `platform/` | Continue to own runtime and SDK abstractions; it does not author domain decision policy in Phase 1. |
| `ui/` | Presentation only. It may display decision artifacts later but does not own decision logic. |
| `testing/` | Hold deterministic fixtures and end-to-end recommendation coverage. |
| `docs/` and `plans/` | Record architectural intent, ADR traceability, and bounded execution artifacts. |

## Trust And Governance Boundaries

Phase 1 aligns decisioning to the repository’s workflow-gated governance model and the external
trust framework requested for design intent:

- `Valid and reliable`: evaluation uses explicit contracts, deterministic seeds, and bounded
  algorithms rather than hidden state.
- `Safe`: recommendations remain advisory or non-executable when constraints or policy gates block
  execution.
- `Secure and resilient`: decision logic stays in Rust-first shared and service layers, not in UI
  or infrastructure side channels.
- `Accountable and transparent`: recommendation, gate, and audit artifacts preserve provenance,
  assumptions, scores, selected action, and rejection rationale.
- `Explainable and interpretable`: rationale text, engine trace, ranked options, and strategic
  summaries are first-class outputs.

Lifecycle control functions map to the repository as follows:

- `Govern`: enterprise policy vocabulary, ADRs, execution artifacts, and service-boundary
  declarations.
- `Map`: decision contexts, provenance, constraints, options, and domain-owned contracts.
- `Measure`: probabilistic scoring, risk assessments, confidence aggregation, and audit evidence.
- `Manage`: workflow gating, policy verdicts, approval requirements, and rollback metadata.

## Probabilistic, Strategic, And Learned Components

Phase 1 explicitly separates the roles of different quantitative methods:

- Game theory models strategic interaction through bounded finite normal-form games. It informs the
  recommendation but is never the sole decision authority.
- Probabilistic algorithms handle uncertainty-aware scoring and selection through weighted
  expected-value scoring, Monte Carlo sampling, Thompson sampling, confidence aggregation, and
  risk-penalized utility combination.
- Deep-learning support is interface-only in Phase 1. Learned adapters are deterministic stubs that
  return `NotConfigured` until future phases supply governed implementations.

If learned support is requested while no adapter is configured, the baseline engine still returns a
recommendation package, but the status is downgraded to advisory and the audit record captures the
unconfigured learned-adapter state.

## Supported Decision Classes

### Strategic Prioritization

Use for choosing between initiatives, execution tracks, or sequencing candidates. A bounded
normal-form game can capture strategic interaction between teams or counterparties, while the
probabilistic layer preserves delivery and risk uncertainty.

### Rollout Or Release-Risk Assessment

Use for canary, staged, or immediate rollout options. The Phase 1 engine scores blast radius,
rollback posture, confidence, and downside risk, then requires explicit policy gating before any
promotion path becomes executable.

### Resource Allocation

Use for budget, staffing, capacity, or compute contention. Utility breakdowns and constraints make
tradeoffs explicit without embedding policy semantics in runtime or infrastructure code.

### Anomaly Or Escalation Routing

Use for triage decisions where the best path depends on uncertain severity, reversibility, and
operator capacity. Recommendations remain auditable and can be routed into later incident
workflows.

### Policy-Constrained Action Selection

Use for choices that may look attractive on utility but fail governance thresholds. Phase 1
preserves the ranked option list and explicit rejection rationale so reviewers can reconstruct why a
high-utility option was blocked.

## Phase Model

### Phase 1: Governed Substrate

- Versioned contracts, shared score types, deterministic baselines, game analysis, policy gates,
  audit records, service integration, workflow integration, and read-only agent alignment.

### Phase 2: Governed Learned Inference

- Replace `NotConfigured` learned adapters with deterministic, versioned local inference adapters
  behind the same traits and audit surfaces.
- Add model metadata, evaluation criteria, and rollout controls without changing workflow
  authority.

### Phase 3: Broader Runtime Integration

- Add durable storage, richer runtime routing, and more decision classes where repository-owned
  runtime capabilities exist.
- Keep recommendations reconstructable and keep workflow/policy gates authoritative.

## Phase 1 Boundaries

Phase 1 does not add remote inference endpoints, model weights, online training, autonomous state
mutation, or UI-owned business logic. It creates the governed substrate needed for future expansion
while preserving Origin’s existing architectural boundaries.
