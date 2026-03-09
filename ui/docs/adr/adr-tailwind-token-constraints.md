---
adr_id: UI-ADR-TAILWIND-TOKEN-CONSTRAINTS
title: Tailwind Token Constraints
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - ui
impacted_domains:
  - design-system
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - ui
  - tailwind
  - design-tokens
---

# ADR: Tailwind Token Constraints

## Context

The workspace needed a practical utility compiler for shell CSS, but unrestricted utility styling
would create a second visual system that bypasses shared tokens and semantic component boundaries.

## Decision

Tailwind is permitted only as a token-driven utility compiler and layering aid. It must derive from
the canonical token source and must not become a parallel visual system.

## Constraints

- Tailwind configuration derives from canonical tokens.
- Shared primitives and components remain the primary styling surface.
- Arbitrary values and ad hoc visual forks are treated as architecture violations.

## Alternatives Considered

- `No Tailwind at all`: rejected because token-backed utility compilation remains useful.
- `Arbitrary Tailwind usage across app and shell markup`: rejected because it recreates fragmented
  styling behavior.

## Rationale

Constraining Tailwind preserves utility where it is helpful without allowing it to displace the
token and component model.

## Implementation Implications

- Tailwind config is generated from tokens.
- Shell and shared components prefer semantic components and authored layers over utility-heavy
  markup.
- Visual drift through arbitrary utility usage is treated as a boundary failure.

## Acceptance Checks

- Tailwind configuration remains derived from the canonical token source.
- Token-backed semantic components remain the preferred styling path for shared UI.
- New shell styling does not introduce a second untethered visual system.

## Traceability

- Governing source: UI-local adopted ADR under the repository-wide ADR contract
- Related roadmap phases: none
- Related controls: `docs/adr/README.md`, token-driven shell styling baseline
