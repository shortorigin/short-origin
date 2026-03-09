# First-Principles Systems Architecture Baseline

This document ratifies the first-principles systems architecture for Origin as a governing
baseline. It replaces the prior report-only posture with a control-artifact posture: the narrative
here explains the platform, while the adopted decisions live in the root ADR corpus and the
implementation sequence lives in the first-principles roadmap.

## Status

The baseline is adopted as of 2026-03-09. The authoritative decision set is:

- [ADR-0005](../adr/0005-domain-bounded-platform-topology.md) through
  [ADR-0017](../adr/0017-replaceable-compute-and-governed-state.md)
- [ADR catalog and authoring rules](../adr/README.md)
- [First-Principles Implementation Roadmap](first-principles-implementation-roadmap.md)

## Purpose

Origin is an AI-driven enterprise platform whose unit of value is an end-to-end capability:

1. sense enterprise reality from governed facts and events,
2. reason under policy, cost, and risk constraints,
3. execute reliable actions across heterogeneous systems through typed tools and workflows.

The platform is treated as a complex adaptive system. That forces a specific architectural posture:
bounded domains, durable workflows, governed data products, explicit contracts, and AI operating
over a deterministic substrate rather than becoming the substrate itself.

## Global Invariants

The adopted baseline enforces these invariants:

1. No production automation exists outside a workflow instance.
2. No workflow executes without emitting canonical events.
3. No privileged action occurs without identity, policy evaluation, and audit.
4. No cross-domain data exchange occurs without a versioned contract.
5. No agent acts directly on systems of record except through typed tools.
6. No autonomy tier increases without evaluation evidence, SLO compliance, and budget compliance.
7. No platform change is authoritative unless versioned through Git-reviewed control artifacts.
8. Hard state lives in governed stores; compute is replaceable, horizontally scalable, and
   observable.

## Architectural Interpretation

The adopted baseline preserves four architectural priorities:

- Event truth first: canonical events define shared reality for operational state, analytics, and
  grounded intelligence.
- Durable execution second: automation must survive retries, approvals, outages, and compensations
  without relying on ad hoc service-local orchestration.
- Governed data third: domains publish explicit contracts, freshness, and lineage instead of
  leaking semantics through shared storage.
- Layered intelligence fourth: retrieval, reasoning, and action stay separated so model behavior
  can evolve without weakening policy, audit, or rollback controls.

This ordering is intentional. Reversing it would create a model-centric platform with weak control
surfaces, which the first-principles analysis explicitly rejects.

## Ratified ADR Pack

The governing decision corpus derived from the original report is:

- [ADR-0005](../adr/0005-domain-bounded-platform-topology.md): domain-bounded topology and
  federated governance
- [ADR-0006](../adr/0006-canonical-event-substrate.md): canonical event substrate
- [ADR-0007](../adr/0007-consistency-semantics-by-criticality.md): consistency by operation class
- [ADR-0008](../adr/0008-domain-owned-data-products-with-contracts.md): domain-owned data
  products
- [ADR-0009](../adr/0009-lakehouse-open-format-analytical-foundation.md): open-format analytical
  foundation
- [ADR-0010](../adr/0010-durable-workflow-execution-plane.md): durable workflow execution plane
- [ADR-0011](../adr/0011-ai-as-planner-over-typed-tools.md): AI as planner over typed tools
- [ADR-0012](../adr/0012-layered-intelligence-behind-stable-retrieval.md): layered intelligence
  behind a stable retrieval boundary
- [ADR-0013](../adr/0013-zero-trust-ai-governance-control-plane.md): zero-trust governance control
  plane
- [ADR-0014](../adr/0014-observability-slos-and-error-budgets.md): observability, SLOs, and error
  budgets as platform primitives
- [ADR-0015](../adr/0015-gitops-and-policy-as-code-control-artifacts.md): GitOps and policy-as-code
  control artifacts
- [ADR-0016](../adr/0016-finops-unit-economics-and-autonomy-budgets.md): FinOps and autonomy
  budgets
- [ADR-0017](../adr/0017-replaceable-compute-and-governed-state.md): replaceable compute and
  governed hard state

## Governing Build Order

The implementation roadmap is authoritative for sequencing:

1. Phase 0 ratifies governance and ownership.
2. Phase 1 establishes truth and trace foundations.
3. Phase 2 makes contracts and data products enforceable.
4. Phase 3 installs durable workflow execution.
5. Phase 4 constrains AI to typed, policy-governed tools.
6. Phase 5 adds retrieval and evaluation behind stable interfaces.
7. Phase 6 completes the policy, identity, and audit control plane.
8. Phase 7 applies SRE and FinOps runtime governance.
9. Phase 8 proves the model in one governed pilot domain.
10. Phase 9 scales the platform without architectural drift.

## GitOps Control Surface

The baseline is authoritative only when it is represented in version-controlled artifacts. Every
implementation task, policy change, contract change, and workflow definition that claims alignment
with this baseline must carry:

- one or more ADR references,
- impacted domains,
- affected consistency class,
- affected risk tier,
- rollback path,
- validation artifacts.

Those fields are the operational bridge between architecture narrative and repository delivery.

## Source Narrative

This baseline was rewritten from the first-principles systems architecture report that had been
preserved outside the working tree during prior branch cleanup. The report remains the narrative
source; the ADR pack, roadmap, and GitOps controls are the authoritative implementation surface.
