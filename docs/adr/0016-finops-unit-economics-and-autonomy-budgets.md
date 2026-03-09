---
adr_id: ADR-0016
title: FinOps Unit Economics and Autonomy Budgets
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
  - cost-governance
  - autonomy
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 5
  - Phase 7
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - finops
  - autonomy-budgets
---

# ADR 0016: FinOps Unit Economics and Autonomy Budgets

## Context

Inference, retrieval, and observability costs grow with platform usage. Without unit economics and
budget-aware controls, automation cost can outpace business value and undermine governance.

## Decision

Every automation and AI-assisted capability must declare unit-economics metrics and operate within
domain-specific autonomy budgets for cost, latency, and review burden.

## Constraints

- Cost governance must tie to business value, not aggregate spend alone.
- Budget overrun must reduce autonomy or switch to safer fallback modes.
- Domain and task criticality determine spend and latency ceilings.

## Alternatives Considered

- `Aggregate monthly spend tracking`: rejected because it does not govern individual capabilities.
- `Feature adoption as the sole success metric`: rejected because it ignores cost and review load.
- `Cost review detached from runtime controls`: rejected because it cannot throttle autonomy when
  needed.

## Rationale

Unit economics turn cost into an operating control variable instead of a retrospective finance
report.

## Implementation Implications

- Tag workflow and tool calls with cost attribution metadata.
- Build per-workflow cost dashboards.
- Add budget-based routing, caching, and escalation rules.

## Acceptance Checks

- Workflow and tool executions emit cost-attribution metadata.
- Unit economics are visible per workflow family and capability.
- Budget overrun can reduce autonomy, increase review, or switch to deterministic fallback paths.

## Traceability

- Blueprint ADR: `ADR-011`
- Related roadmap phases: `Phase 5`, `Phase 7`
- Related invariants: `6`, `8`
