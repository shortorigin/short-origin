---
adr_id: UI-ADR-DOM-FIRST-COMPOSITOR
title: DOM-First Compositor
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
  - compositor
  - dom
---

# ADR: DOM-First Compositor

## Context

The shell compositor needed a clear rendering posture that preserves accessibility and browser/PWA
parity while keeping transient pointer and frame behavior subordinate to Rust-owned runtime policy.

## Decision

The shell compositor is DOM-first and Rust-orchestrated. `desktop_runtime::origin_compositor` owns
transient drag, resize, pointer capture, and frame scheduling, while persistent window policy
remains in the reducer-owned window manager state.

## Constraints

- Browser and WASM preview remain first-class execution paths.
- Accessibility behavior must remain standards-aligned.
- Persistent shell policy remains reducer-owned rather than compositor-owned.

## Alternatives Considered

- `GPU-first canvas or scene-graph shell rendering`: rejected because it increases accessibility
  and browser-parity risk.
- `Pure DOM with unstructured local component state`: rejected because it fragments ownership and
  weakens determinism.

## Rationale

DOM-backed shell surfaces preserve browser standards and accessibility, while Rust-owned policy
keeps geometry, focus, and stacking deterministic.

## Implementation Implications

- Pointer sessions, RAF scheduling, and transient transforms stay isolated in compositor modules.
- Final geometry and shell policy changes commit back through reducer actions.
- Decorative effects may use richer rendering paths without displacing the primary shell model.

## Acceptance Checks

- Persistent window policy changes continue to flow through reducer-owned state.
- Browser and WASM preview remain viable without a separate renderer path.
- Accessibility semantics are preserved on primary shell surfaces.

## Traceability

- Governing source: UI-local adopted ADR under the repository-wide ADR contract
- Related roadmap phases: none
- Related controls: `docs/adr/README.md`, `ui/` shell architecture
