---
adr_id: ADR-0015
title: GitOps and Policy-as-Code Control Artifacts
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - docs
  - .github
  - infrastructure
  - workflows
impacted_domains:
  - delivery-governance
  - policy
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 0
  - Phase 6
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - gitops
  - policy-as-code
---

# ADR 0015: GitOps and Policy-as-Code Control Artifacts

## Context

Infrastructure, workflow definitions, policy rules, and evaluation thresholds all shape runtime
behavior. If they are not versioned and reviewed like source code, the platform loses
reproducibility and rollback control.

## Decision

Authoritative platform state shall be expressed as declarative, version-controlled artifacts managed
through Git review and automated synchronization. Workflow definitions, policy rules, and
evaluation thresholds are treated as GitOps-controlled artifacts alongside infrastructure.

## Constraints

- Manual production-only changes are non-authoritative and must be reconciled back into Git.
- Workflow definitions, policy rules, and evaluation thresholds carry compatibility metadata.
- Rollback procedures must exist for all authoritative control artifacts.

## Alternatives Considered

- `Console-managed operations`: rejected because they break reproducibility and auditability.
- `Separate configuration stores outside version control`: rejected because they drift from the
  reviewed source of truth.
- `Code-reviewed software but manually managed governance thresholds`: rejected because governance
  changes alter runtime behavior too.

## Rationale

GitOps preserves reproducibility, reviewability, and rollback across infrastructure and governance
surfaces, not just application binaries.

## Implementation Implications

- Maintain repository directories for infrastructure, workflows, policies, and evaluation configs.
- Define promotion flow across environments.
- Enforce integrity controls and rollback paths for control artifacts.

## Acceptance Checks

- Workflow definitions, policies, and evaluation thresholds are versioned and reviewable in Git.
- Manual environment changes are treated as drift until reconciled back into the repository.
- Production rollback can restore a previously known-good control artifact set.

## Traceability

- Blueprint ADR: `ADR-010`
- Related roadmap phases: `Phase 0`, `Phase 6`
- Related invariants: `6`, `7`
