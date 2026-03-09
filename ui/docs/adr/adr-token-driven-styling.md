---
adr_id: UI-ADR-TOKEN-DRIVEN-STYLING
title: Token-Driven Styling
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - ui
  - platform
impacted_domains:
  - design-system
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - ui
  - tokens
  - styling
---

# ADR: Token-Driven Styling

## Context

The shell styling model had drifted across multiple CSS files and theme variants. The UI needed one
reviewable source of truth for visual values shared across Rust and CSS.

## Decision

All shell and shared UI styling flows from canonical design tokens stored in
`system_ui/tokens/tokens.toml`. Generated outputs provide Rust token accessors, CSS variables, and
token-backed Tailwind theme extensions.

## Constraints

- Tokens remain the canonical reviewable source.
- Generated files are derived artifacts rather than authoring surfaces.
- Shared primitives and components must read from token-defined spacing, typography, surfaces,
  motion, and chrome metrics.

## Alternatives Considered

- `Hand-maintained CSS variables only`: rejected because Rust-owned metrics would drift.
- `Tailwind-only authored styling`: rejected because it creates a competing source of truth.

## Rationale

Token-driven styling makes the shell visual system deterministic, reviewable, and shareable across
Rust and CSS.

## Implementation Implications

- Hard-coded shell values should be replaced by token references.
- New visual changes start in token source or component semantics, not ad hoc CSS.
- Generated files remain artifacts and should be regenerated from token source.

## Acceptance Checks

- Shared shell styling values continue to originate from canonical tokens.
- Generated CSS and Rust token outputs remain derived artifacts.
- New visual changes are introduced through token or semantic component updates rather than ad hoc
  styling forks.

## Traceability

- Governing source: UI-local adopted ADR under the repository-wide ADR contract
- Related roadmap phases: none
- Related controls: `docs/adr/README.md`, token generation pipeline
