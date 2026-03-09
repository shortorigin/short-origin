---
adr_id: ADR-0007
title: Consistency Semantics by Operation Criticality
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - services
  - workflows
  - shared
impacted_domains:
  - all-domains
  - runtime-consistency
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 1
  - Phase 3
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - consistency
  - criticality
---

# ADR 0007: Consistency Semantics by Operation Criticality

## Context

Enterprise automation spans money movement, identity, analytics, recommendations, and long-running
workflow state. Those operations do not require the same consistency guarantees and should not
inherit them accidentally.

## Decision

Origin applies operation-class consistency policy:

- `Class A`: strong consistency required.
- `Class B`: bounded staleness acceptable.
- `Class C`: eventual consistency acceptable.

## Constraints

- Critical operations must not inherit weak guarantees from noncritical infrastructure.
- Analytical and retrieval paths must not block critical operational writes.
- Retry and replay mechanisms must preserve correctness under partial failure.

## Alternatives Considered

- `Strong consistency everywhere`: rejected because it is too expensive and constraining.
- `Eventual consistency everywhere`: rejected because it is unsafe for critical operations.
- `Team-local consistency choices`: rejected because it produces silent mismatches.

## Rationale

Classifying consistency by operation criticality localizes the tradeoffs and prevents weakly
consistent paths from being reused for high-risk work.

## Implementation Implications

- Publish an operation criticality matrix.
- Tag workflows and tools with a declared consistency class.
- Block release approval when a workflow or tool lacks declared consistency semantics.

## Acceptance Checks

- Pilot workflows and privileged tools declare `Class A`, `Class B`, or `Class C`.
- Release or merge validation fails when a governed workflow lacks a consistency class.
- Strong-consistency operations remain isolated from eventual-consistency analytical backplanes.

## Traceability

- Blueprint ADR: `ADR-002`
- Related roadmap phases: `Phase 1`, `Phase 3`
- Related invariants: `1`, `2`, `4`
