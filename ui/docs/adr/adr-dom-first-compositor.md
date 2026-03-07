# ADR: DOM-First Compositor

## Status
Accepted

## Decision
The shell compositor is DOM-first and Rust-orchestrated. `desktop_runtime::origin_compositor` owns transient drag, resize, pointer capture, and frame scheduling, while persistent window policy remains in the reducer-owned window manager state.

## Rationale
- Browser/WASM preview must remain a first-class execution path.
- The desktop shell needs standards-aligned HTML, CSS, focus, and ARIA behavior.
- DOM-backed window surfaces are easier to keep accessible and cross-browser consistent than a GPU-first scene graph.
- Rust remains authoritative for lifecycle, geometry, focus, and stacking policy.

## Alternatives Rejected
- GPU-first canvas or scene-graph shell rendering: rejected because it increases accessibility and browser parity risk.
- Pure DOM with unstructured local component state: rejected because it fragments reducer ownership and weakens determinism.

## Consequences
- Pointer sessions, RAF scheduling, and transient transforms are isolated in compositor modules.
- Final geometry and shell policy changes are committed back through reducer actions.
- Wallpaper and decorative effects may use richer rendering paths, but the primary shell remains DOM-first.
