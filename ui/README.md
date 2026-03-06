# UI

## Purpose
`ui/` owns the Short Origin OS shell built with Leptos and distributed through Tauri. It is the only top-level module allowed to define UI-specific models, host composition, and desktop/web presentation behavior.

## Scope
In scope:
- Shared app/runtime contracts for managed shell applications.
- The Leptos desktop shell runtime and built-in product apps.
- Browser/WASM preview entrypoints used for parity checks.
- Tauri host bindings for app state, preferences, cache, notifications, filesystem access, and external URL opening.

Out of scope:
- Canonical business contracts and events.
- Direct SurrealDB access.
- Service or workflow policy ownership.

## Interfaces
- Inbound: typed SDK/contracts from `platform/sdk` and `schemas/`.
- Outbound: host capability calls through `platform_host` and Tauri/web adapters.
- Runtime: shell app manifests and capability-scoped services injected by `desktop_runtime`.

## Dependencies
- `platform/sdk` for typed commands, queries, dashboard snapshots, and event streams.
- `platform/wasmcloud` for lattice metadata surfaced in operator-facing UI.
- `schemas/` for canonical payloads and event versions.
- `shared/` crates for reusable non-UI support only.

## Development Workflow
1. Keep product shell code under `ui/`; do not add Leptos/Tauri code elsewhere in the workspace.
2. Adapt shared contracts into presentation models instead of redefining domain data structures locally.
3. Preserve dual-target compatibility for browser/WASM preview and Tauri desktop where host capabilities allow.
4. Route governed data access through typed SDK calls and connected cache state, never direct database clients.
5. Start with a GitHub issue, link it from the PR, and capture host-boundary compliance, dual-target compatibility, and typed SDK usage evidence in the issue or PR notes.

## Build/Test Commands
Run from repo root:
```bash
cargo doctor --domain ui
cargo dx
cargo ui-dev
cargo ui-build
cargo tauri-build
cargo ui-e2e --all-scenes
cargo ui-e2e --scene shell-default
cargo ui-verify
cargo verify-ui
```

These aliases stay intentionally close to the underlying `xtask` structure. `cargo ui-verify` runs the tracked compile checks, the browser preview build, and the Playwright browser scenes that exercise the runtime's deterministic `e2e-scene` and `os:e2e-ready` hooks, while `cargo x ui ...` remains available for the full command surface.
`cargo dx` complements those flows with a standalone Ratatui operator console for setup guidance, work tracking, task execution, workflow monitoring, and guarded release workflow dispatch.

## Integration Patterns
- Tauri is the authoritative runtime; browser/WASM remains for preview and parity validation.
- UI apps receive only capability-scoped services and typed platform snapshots from the runtime.
- Local storage is connected cache and preferences only; SurrealDB remains service-owned system of record.

## Out of Scope
- Direct service deployment logic.
- Private workflow orchestration internals.
- Unversioned host or data contracts.
