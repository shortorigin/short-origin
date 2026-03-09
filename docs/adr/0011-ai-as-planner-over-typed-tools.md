---
adr_id: ADR-0011
title: AI as Planner over Typed Tools
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - agents
  - platform
  - workflows
  - services
impacted_domains:
  - intelligence
  - tooling
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 4
  - Phase 8
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - ai
  - typed-tools
---

# ADR 0011: AI as Planner over Typed Tools

## Context

Models are useful for planning, classification, interpretation, and recommendation, but they do not
provide the deterministic execution, rollback, or access-control guarantees required for enterprise
mutation.

## Decision

AI components shall operate as constrained planners, classifiers, summarizers, and decision-support
modules over authenticated typed tools. They shall not directly mutate systems of record or own
final execution semantics.

## Constraints

- No direct model-to-database writes.
- No direct privileged external actions without tool mediation.
- Every AI-originated act must be traceable to tool contracts and workflow state.

## Alternatives Considered

- `Fully autonomous agents with direct credentials`: rejected because they bypass governance and
  rollback.
- `Prompt-scripted automation without typed contracts`: rejected because it weakens validation and
  auditability.
- `AI-only orchestration`: rejected because it makes execution correctness probabilistic.

## Rationale

Keeping AI at the planning boundary preserves flexibility while keeping enterprise mutation under
typed, policy-governed control.

## Implementation Implications

- Define typed tool contracts with schemas, auth scope, side-effect class, idempotency, timeout,
  and compensation behavior.
- Require workflow binding and policy checks on every side-effecting tool call.
- Separate planning, tool selection, evidence collection, and recommendation output.

## Acceptance Checks

- AI cannot directly mutate a system of record without going through a typed tool path.
- Every side-effecting tool request carries workflow linkage and policy context.
- Audit evidence can reconstruct the AI plan, the tool request, the structured result, and the
  final workflow decision.

## Traceability

- Blueprint ADR: `ADR-006`
- Related roadmap phases: `Phase 4`, `Phase 8`
- Related invariants: `3`, `5`, `7`
