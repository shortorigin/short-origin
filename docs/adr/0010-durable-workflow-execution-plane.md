---
adr_id: ADR-0010
title: Durable Workflow Execution Plane
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - workflows
  - services
  - platform
  - shared
impacted_domains:
  - workflow-runtime
  - approvals
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 3
  - Phase 8
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - workflows
  - durability
---

# ADR 0010: Durable Workflow Execution Plane

## Context

Enterprise automation is long-running, failure-prone, and approval-sensitive. It cannot rely on
ad hoc retries or service-local orchestration if the platform is expected to recover predictably.

## Decision

All nontrivial automation shall execute through a durable workflow engine. Workflow state machines,
retries, approvals, and compensations are owned by the execution plane rather than embedded ad hoc
in services or agents.

## Constraints

- Activities must be idempotent.
- Workflow transitions must be replay-safe.
- Human approval gates are mandatory for predefined risk classes.

## Alternatives Considered

- `Stateless request and response orchestration`: rejected because it loses durable execution
  semantics.
- `Agent-managed action chains`: rejected because they do not provide reliable retries,
  compensations, or audit trails.
- `Service-local cron and retry logic`: rejected because it fragments operational control.

## Rationale

Durable workflows reconcile distributed-system failure with enterprise requirements for
repeatability, recoverability, and reviewable human intervention.

## Implementation Implications

- Select and govern one workflow runtime.
- Publish workflow metadata: ID, version, risk class, consistency class, rollback strategy, and
  approval policy.
- Require compensation design for side-effecting workflows.

## Acceptance Checks

- Nontrivial automations execute within a durable workflow instance rather than ad hoc service
  orchestration.
- Workflow histories can be replayed without duplicating non-idempotent side effects.
- Defined risk tiers can route through explicit approval gates before final side effects complete.

## Traceability

- Blueprint ADR: `ADR-005`
- Related roadmap phases: `Phase 3`, `Phase 8`
- Related invariants: `1`, `2`, `3`
