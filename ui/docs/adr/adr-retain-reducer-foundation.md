---
adr_id: UI-ADR-RETAIN-REDUCER-FOUNDATION
title: Retain and Extend the Reducer Foundation
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - ui
  - platform
impacted_domains:
  - ui-shell
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - ui
  - reducer
  - runtime
---

# ADR: Retain and Extend the Reducer Foundation

## Context

The existing desktop runtime already had a reducer-driven state, manifest integration, and runtime
policy model. Replacing it would have discarded working determinism instead of formalizing it.

## Decision

The existing `desktop_runtime` reducer, model, and window-manager direction is retained and
formalized as the `origin_wm` subsystem rather than replaced with a new state architecture.

## Constraints

- Desktop state and transition policy remain deterministic.
- View-local or compositor-local state must not own persistent window policy.
- Built-in applications integrate through typed runtime contracts and manifests.

## Alternatives Considered

- `Rewrite the shell runtime from zero`: rejected because it discards validated runtime behavior.
- `Push policy ownership into local UI state`: rejected because it weakens correctness and
  auditability.

## Rationale

Keeping the reducer foundation preserves the strongest existing correctness boundary while allowing
the shell to evolve incrementally.

## Implementation Implications

- `origin_wm` becomes the explicit home of desktop state and transition rules.
- Compositor code remains transient and subordinate to reducer-owned state.
- Built-in apps continue to integrate through typed runtime contracts and manifest registration.

## Acceptance Checks

- Persistent shell policy remains reducer-owned rather than view-local.
- Compositor sessions do not become the system of record for focus, modality, or geometry.
- Built-in apps continue to register through typed contracts and manifests.

## Traceability

- Governing source: UI-local adopted ADR under the repository-wide ADR contract
- Related roadmap phases: none
- Related controls: `docs/adr/README.md`, `desktop_runtime` reducer model
