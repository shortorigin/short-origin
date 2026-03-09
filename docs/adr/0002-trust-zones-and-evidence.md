---
adr_id: ADR-0002
title: Trust Zones and Evidence
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - infrastructure
  - platform
  - services
  - shared
impacted_domains:
  - security
  - evidence
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - trust-zones
  - evidence
  - audit
---

# ADR 0002: Trust Zones and Evidence

## Context

The platform needed an explicit trust-zone model so evidence capture, privileged actions, and
runtime placement could be reviewed against one consistent security boundary.

## Decision

The platform is partitioned into five trust zones: public edge, institutional control plane,
runtime plane, data plane, and management plane. Every material decision and side effect produces
an evidence manifest and a versioned decision or side-effect record.

## Constraints

- Trust-zone boundaries must remain explicit in deployment and runtime bindings.
- Audit reconstruction must use canonical evidence artifacts rather than ad hoc logs.
- Privileged actions must remain attributable to actors, policy decisions, and evidence.

## Alternatives Considered

- `Flat network trust model`: rejected because it weakens policy isolation and audit clarity.
- `Per-service evidence conventions`: rejected because it produces incomplete or incompatible audit
  trails.

## Rationale

Zone separation and evidence manifests make policy review, incident analysis, and privileged-action
reconstruction deterministic instead of interpretive.

## Implementation Implications

- Cloudflare remains the public ingress and edge policy layer.
- Runtime workloads run with explicit bindings into the runtime plane.
- Evidence storage and audit reconstruction depend on canonical manifests and event records.

## Acceptance Checks

- Material decisions and side effects continue to produce evidence-linked records.
- SurrealDB and evidence storage remain isolated from public-edge access paths.
- Audit reconstruction can be performed from envelope-linked events, approvals, risks, and evidence
  manifests alone.

## Traceability

- Governing source: repository baseline ADR retained under the first-principles architecture pack
- Related roadmap phases: none
- Related controls: `docs/process/platform-regression-guardrails.md`, `docs/process/github-governance-rollout.md`
