# ADR: Token-Driven Styling

## Status
Accepted

## Decision
All shell and shared UI styling flows from canonical design tokens stored in `system_ui/tokens/tokens.toml`. Generated outputs provide Rust token accessors, CSS variables, and token-backed Tailwind theme extensions.

## Rationale
- The prior shell styling model drifted across multiple CSS files and skin variants.
- Tokens make visual values reviewable, deterministic, and shareable across Rust and CSS.
- Shared primitives and components need one source of truth for spacing, typography, surfaces, motion, and chrome metrics.

## Alternatives Rejected
- Hand-maintained CSS variables only: rejected because Rust-owned shell metrics would drift.
- Tailwind-only authored styling: rejected because it would create a second competing source of truth.

## Consequences
- Hard-coded shell values should be replaced by token references.
- New visual changes start in token source or component semantics, not ad hoc CSS.
- Generated files are artifacts; token source remains the reviewable authority.
