---
adr_id: ADR-0014
title: Observability, SLOs, and Error Budgets as Platform Primitives
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - shared
  - platform
  - services
  - workflows
impacted_domains:
  - observability
  - reliability
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 1
  - Phase 7
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - observability
  - slo
---

# ADR 0014: Observability, SLOs, and Error Budgets as Platform Primitives

## Context

Automation failures are difficult to localize without end-to-end traces, metrics, logs, and
explicit reliability targets. AI-assisted outcomes also need budgeted governance signals rather than
best-effort monitoring.

## Decision

Observability, reliability targets, and error-budget governance are mandatory platform primitives
for services, workflows, and AI-assisted outcomes.

## Constraints

- Every workflow step, tool call, and significant data movement must emit telemetry.
- All critical journeys require SLO definitions.
- Error-budget burn must trigger governance actions such as throttling autonomy or increasing
  review.

## Alternatives Considered

- `Best-effort logging`: rejected because it does not support end-to-end diagnosis.
- `API-only monitoring`: rejected because workflow and tool outcomes remain invisible.
- `Post hoc incident analysis only`: rejected because it cannot constrain autonomy in time.

## Rationale

Without end-to-end telemetry and explicit budgets, the platform cannot localize failures or
operate autonomy safely at scale.

## Implementation Implications

- Define telemetry schema and trace propagation standards.
- Create service and workflow SLO catalogs.
- Automate governance actions on budget burn thresholds.

## Acceptance Checks

- Every critical workflow emits traceable telemetry across workflow, tool, and data boundaries.
- Critical journeys have published SLOs and measurable error budgets.
- Budget burn can trigger reduced autonomy or higher review requirements.

## Traceability

- Blueprint ADR: `ADR-009`
- Related roadmap phases: `Phase 1`, `Phase 7`
- Related invariants: `2`, `6`, `8`
