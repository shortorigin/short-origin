---
adr_id: ADR-0005
title: Domain-Bounded Platform Topology with Federated Governance
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - services
  - workflows
  - docs
impacted_domains:
  - all-domains
  - platform-governance
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 0
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - domains
  - governance
---

# ADR 0005: Domain-Bounded Platform Topology with Federated Governance

## Context

Origin is treated as a complex adaptive enterprise platform. That requires architectural and
organizational boundaries that keep domain semantics local while preserving governed interoperability
across the platform.

## Decision

Origin will organize around domain execution cells that own domain workflows, semantics, and data
products; shared platform product lines that provide eventing, workflow runtime, retrieval,
governance, observability, and cost controls; and federated computational governance that sets
global standards while preserving domain ownership.

## Constraints

- Domain semantics remain domain-owned.
- Global interoperability must not depend on one centralized data or AI team.
- Platform services must reduce cross-team coordination cost rather than increase it.

## Alternatives Considered

- `Centralized enterprise data and AI ownership`: rejected because semantic change does not scale
  through one team.
- `Pure microservice decentralization`: rejected because it produces contract and policy drift.
- `Technical-stack-only layering`: rejected because it ignores business volatility and ownership.

## Rationale

Federated governance preserves local meaning while keeping contracts, policy, and platform tooling
coherent across the repository.

## Implementation Implications

- Define the domain map and platform product-line charters.
- Assign owners for vocabularies, event families, data products, and workflow families.
- Require ADRs for decisions that affect more than one domain or product line.

## Acceptance Checks

- Every domain vocabulary, event family, data product, and workflow family has a named owner.
- Cross-domain architectural changes reference one or more ADRs before implementation begins.
- Platform services do not redefine domain semantics outside the owning domain.

## Traceability

- Blueprint ADR: `ADR-000`
- Related roadmap phases: `Phase 0`, `Phase 9`
- Related invariants: `4`, `7`, `8`
