---
adr_id: ADR-0013
title: Zero-Trust AI Governance Control Plane
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - infrastructure
  - platform
  - services
  - workflows
  - shared
impacted_domains:
  - security
  - governance
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 0
  - Phase 6
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - zero-trust
  - governance
---

# ADR 0013: Zero-Trust AI Governance Control Plane

## Context

AI increases action surface area and introduces distinct risks such as prompt injection, insecure
output handling, and excessive agency. Those concerns require runtime governance, not just static
policy documents.

## Decision

Security and AI governance shall be implemented as a control plane enforcing identity and workload
identity, policy decision and policy enforcement, secrets and key management, audit and provenance
logging, risk-tiered approval policies, and AI-specific safeguards for prompt injection, insecure
output handling, excessive agency, and sandbox escape risk.

## Constraints

- Every privileged tool use requires continuous authorization.
- Least privilege applies equally to users, services, and agents.
- Governance must be measurable and continuous rather than one-time review.

## Alternatives Considered

- `Per-service local authorization only`: rejected because it fragments policy enforcement.
- `Deployment-time-only security review`: rejected because it cannot govern runtime decisions.
- `Prompt-only AI governance`: rejected because it is not enforceable or auditable.

## Rationale

Policy-first control is the only reliable way to keep AI-assisted action within identity, approval,
and audit boundaries.

## Implementation Implications

- Stand up a policy decision point with distributed enforcement points.
- Tag tools and workflows with risk class, scopes, and approval rules.
- Log every policy decision with workflow, actor, evidence, and outcome.

## Acceptance Checks

- No privileged tool call bypasses a policy decision.
- Policy decisions and privileged actions are logged to an immutable audit stream.
- AI-originated tool use is constrained by least-privilege scopes and risk-tier approval rules.

## Traceability

- Blueprint ADR: `ADR-008`
- Related roadmap phases: `Phase 0`, `Phase 6`
- Related invariants: `3`, `5`, `7`
