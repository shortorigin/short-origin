# UI

## Purpose
`ui/` owns the Origin desktop interface built with Leptos for browser/WASM preview and Tauri distribution. It is the only workspace area allowed to define shell presentation models, windowing behavior, compositor glue, and host-facing UI adapters.

## Architecture
The UI subsystem is organized under one required hierarchy:

1. Design Tokens
2. Primitives
3. Components
4. Application Shell

This hierarchy is implemented module-first inside the existing crates:

- `ui/crates/system_ui`
  - `origin_tokens`: canonical machine-readable token source plus generated Rust accessors
  - `origin_primitives`: foundational layout, surface, typography, and window-region primitives
  - `origin_components`: validated shared controls and shell components composed only from primitives
- `ui/crates/desktop_runtime`
  - `origin_wm`: reducer-driven desktop state, focus policy, geometry, stacking, and workspace scaffolding
  - `origin_compositor`: DOM-first transient orchestration for pointer capture, drag, resize, and frame scheduling
  - `origin_shell`: desktop shell composition, taskbar, launcher, window host, focus routing, and menus
- `ui/crates/site`
  - thin browser composition root only

## Ownership Rules
- Tokens are the only source of truth for visual values.
- Primitives define reusable semantics and layout contracts, not product policy.
- Components compose primitives into reusable interactive structures.
- Shell code consumes components; it does not handcraft low-level controls once a component exists.
- Apps render window contents only. They do not render titlebars, taskbars, launcher surfaces, or shell chrome.
- Window management policy is reducer-owned in Rust.
- Browser APIs are glue only; `web_sys` and `wasm_bindgen` must remain subordinate to Rust state.

## Styling Policy
- One primary contemporary shell baseline ships by default.
- Accessibility variants are allowed for high contrast and reduced motion.
- Legacy nostalgia skins are not part of the primary boot path.
- Raw CSS files are not the design-system source of truth.
- Tailwind is allowed only as a token-driven utility compiler and layering aid.
- Tailwind configuration must derive from the canonical token source and must not become a competing styling system.

Generated styling outputs:
- `ui/crates/system_ui/tokens/tokens.toml`: canonical token source
- `ui/crates/system_ui/src/origin_tokens/generated.rs`: generated Rust token accessors
- `ui/crates/site/src/generated/tokens.css`: generated CSS variables
- `ui/crates/site/tailwind.config.js`: generated token-backed Tailwind theme

## Runtime and Host Boundary
- `desktop_runtime` remains the authoritative owner of desktop state, reducer transitions, and app/runtime integration.
- `system_shell` and `system_shell_contract` remain the shell command and contract boundary.
- `site` mounts runtime providers and the shell root; it does not own shell architecture.
- Tauri remains the authoritative desktop runtime and distribution path.
- Browser/WASM preview remains first-class for parity and standards validation.

## Cross-Browser Contract
The shell must rely on open web standards and behave consistently across Blink, Gecko, and WebKit. Core interactions must use:

- semantic HTML and ARIA
- Pointer Events
- pointer capture
- ResizeObserver where measurement is required
- `requestAnimationFrame` for transient compositor scheduling

Do not introduce browser-specific hacks as architecture.

## Contribution Rules
- Keep Leptos/Tauri code under `ui/`; do not distribute UI logic into non-UI top-level modules.
- Adapt from typed contracts and SDK models rather than redefining domain types locally.
- Do not connect UI code directly to SurrealDB.
- Do not add app-level styling that bypasses tokens or primitives.
- Do not reintroduce legacy `.app-*` style contracts into the shell baseline.
- Public component exports must be justified by runtime or app usage.

## Build and Verification
Run from the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

UI-focused iteration commonly uses:

```bash
cargo check -p system_ui
cargo check -p desktop_runtime
cargo check -p site
```

## Scope
In scope:
- shared app/runtime contracts for managed shell applications
- desktop shell runtime, built-in apps, and browser preview entrypoints
- token generation, shell composition, window manager behavior, and compositor orchestration
- Tauri host bindings for preferences, cache, notifications, filesystem access, and external URL opening

Out of scope:
- canonical business contracts and events
- direct SurrealDB access
- service or workflow policy ownership
