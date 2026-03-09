---
adr_id: ADR-0017
title: Replaceable Compute and Governed Hard State
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - platform
  - services
  - shared
  - infrastructure
impacted_domains:
  - runtime
  - storage
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 1
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - compute
  - state
---

# ADR 0017: Replaceable Compute and Governed Hard State

## Context

Long-term platform resilience depends on treating runtime components as replaceable and provider
substitutable while keeping correctness-critical state in governed stores with explicit semantics.

## Decision

Runtime components shall be disposable compute units. Persistent correctness-critical state shall
live only in governed event, operational, analytical, and policy stores.

## Constraints

- Runtime instances must be restartable without correctness loss.
- Stateful caches are non-authoritative unless explicitly classified and governed.
- Model and provider selection must remain substitutable behind stable interfaces.

## Alternatives Considered

- `Stateful service instances`: rejected because they make recovery and scaling brittle.
- `Provider-coupled model integrations`: rejected because they reduce optionality.
- `Local caches as implicit system of record`: rejected because they hide correctness-critical
  state.

## Rationale

Replaceable compute improves resilience and optionality; governed stores preserve correctness,
lineage, and auditability.

## Implementation Implications

- Define an authoritative store taxonomy.
- Forbid correctness-critical state in runtime memory unless explicitly ratified.
- Encapsulate model providers behind stable internal service contracts.

## Acceptance Checks

- Runtime instances can restart or scale horizontally without losing correctness-critical state.
- Authoritative state lives in governed event, operational, analytical, or policy stores rather
  than local process memory.
- Model-provider changes can occur behind stable internal interfaces without consumer contract
  changes.

## Traceability

- Blueprint ADR: `ADR-012`
- Related roadmap phases: `Phase 1`, `Phase 9`
- Related invariants: `7`, `8`
