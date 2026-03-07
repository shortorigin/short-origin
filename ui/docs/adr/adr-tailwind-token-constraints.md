# ADR: Tailwind Token Constraints

## Status
Accepted

## Decision
Tailwind is permitted only as a token-driven utility compiler and layering aid. It must derive from the canonical token source and must not become a parallel visual system.

## Rationale
- The workspace needs a practical way to compile utility layers and shared CSS output.
- Unconstrained utility styling would bypass tokens, primitives, and component boundaries.
- The shell requires semantic composition, not utility sprawl.

## Alternatives Rejected
- No Tailwind at all: rejected because token-backed utility output is useful for controlled layering.
- Arbitrary Tailwind usage across app and shell markup: rejected because it recreates fragmented styling behavior.

## Consequences
- Tailwind config is generated from tokens.
- Shell and shared components should prefer semantic components and authored layers over utility-heavy markup.
- Arbitrary values and ad hoc visual forks should be treated as architecture violations.
