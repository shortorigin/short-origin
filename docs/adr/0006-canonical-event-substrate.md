---
adr_id: ADR-0006
title: Canonical Event Substrate as the Primary Source of Truth
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - schemas
  - shared
  - services
  - workflows
  - platform
impacted_domains:
  - all-domains
  - eventing
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 1
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - events
  - truth
---

# ADR 0006: Canonical Event Substrate as the Primary Source of Truth

## Context

The platform needs one canonical representation of business-significant state transitions so
operational state, analytics, and grounded intelligence all derive from the same causal record.

## Decision

All business-significant state transitions shall be represented in a canonical event model. State
projections, analytical facts, and AI retrieval context derive from this event substrate rather
than bypassing it.

## Constraints

- Events are immutable and append-only.
- Events carry causality, actor, source, timestamp, and scope metadata.
- Event design must support replay, audit, and process discovery.

## Alternatives Considered

- `CRUD-first operational systems with later CDC`: rejected because event truth becomes secondary
  and ambiguous.
- `Separate event feeds for analytics and AI`: rejected because it fragments shared reality.
- `Application-local logs only`: rejected because they do not produce a canonical enterprise record.

## Rationale

One canonical event substrate reduces ambiguity across workflows, humans, and models while enabling
replay-based debugging, simulation, and audit.

## Implementation Implications

- Define a canonical event envelope with identity, causality, actor, source, and policy context.
- Require event emission on every workflow transition and typed-tool side effect.
- Build projections as derived views rather than authoritative records.

## Acceptance Checks

- Workflow transitions emit canonical events with correlation and causation metadata.
- Projections and analytical facts remain derivations of the event substrate rather than hidden
  system-of-record tables.
- Retrieval and audit flows can trace a grounded fact back to the canonical event history.

## Traceability

- Blueprint ADR: `ADR-001`
- Related roadmap phases: `Phase 1`, `Phase 9`
- Related invariants: `2`, `4`, `8`
