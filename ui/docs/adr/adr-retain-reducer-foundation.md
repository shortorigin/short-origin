# ADR: Retain and Extend the Reducer Foundation

## Status
Accepted

## Decision
The existing `desktop_runtime` reducer/model/window-manager direction is retained and formalized as the `origin_wm` subsystem rather than replaced with a new state architecture.

## Rationale
- The repository already has structurally sound reducer-driven state, app manifests, and runtime integration boundaries.
- Reusing those foundations reduces migration risk and preserves existing runtime strengths.
- A single reducer-owned policy layer is the clearest way to keep focus, stacking, modality, and geometry deterministic.

## Alternatives Rejected
- Rewrite the shell runtime from zero: rejected because it would discard validated runtime and manifest behavior.
- Push policy ownership into view-local state or compositor sessions: rejected because it weakens correctness and auditability.

## Consequences
- `origin_wm` becomes the explicit home of desktop state and transition rules.
- Compositor code remains transient and subordinate to reducer-owned state.
- Built-in apps continue to integrate through typed runtime contracts and manifest-driven registration.
