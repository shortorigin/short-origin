# Origin OS UI Shell

## Purpose
`ui/` is the Origin OS UI shell runtime and application composition layer. It owns shell presentation, Leptos-based runtime composition, built-in app mounting, and host-facing adapters required to run the interface in either the Tauri desktop runtime or the browser/WASM preview. The module is Rust-first: UI behavior, state transitions, and host integration are defined in Rust and exposed through typed contracts.

## Scope
In scope: shell presentation primitives and components, desktop runtime state and reducers, built-in shell app composition, Tauri host integration, browser/WASM preview entrypoints, and typed host adapter wiring for UI-facing platform services.

Out of scope: canonical business-domain models, schema ownership, service or workflow policy logic, direct SurrealDB access, and private calls into platform or service internals.

## Architecture
`ui/` is organized as a layered shell subsystem:

- `system_ui` provides design tokens, primitives, and reusable shell components.
- `desktop_runtime` owns shared shell state, reducer logic, compositor orchestration, window manager behavior, persistence hooks, and Leptos runtime composition.
- `site` is the browser/WASM entrypoint used for preview and parity validation.
- `desktop_tauri` is the desktop host and distribution wrapper for the Tauri runtime.
- `platform_host` defines typed host-service contracts used by the runtime and app crates.
- `platform_host_web` provides browser and desktop-webview adapter wiring behind the `platform_host` contracts.
- `desktop_app_contract`, `system_shell_contract`, `system_shell`, and the built-in app crates define the managed shell app boundary and shell command surface.

This layer owns UI composition only. It adapts typed contracts into presentation and runtime behavior; it does not define canonical enterprise semantics or bypass platform boundaries.

## Interfaces
`ui/` integrates with the rest of the workspace through typed Rust contracts:

- `platform/sdk/sdk-rs` supplies reusable platform SDK interfaces consumed by runtime code.
- `platform_host` supplies UI-facing host traits and shared models for storage, cache, notifications, filesystem, wallpaper, external URLs, and terminal process integration.
- `desktop_app_contract` defines the app lifecycle, capabilities, mount context, and managed app registration surface.
- `system_shell_contract` defines shell command and shell-facing contract types.
- `schemas/` and `shared/` crates provide versioned contract types and reusable support crates where the UI needs stable cross-workspace models.

Transport-specific behavior stays behind adapter crates. Browser and desktop transports are implementation details of `platform_host_web` and `desktop_tauri`, not public integration surfaces for app or shell code.

## Dependencies
Primary dependencies for this module are Rust workspace crates plus the Leptos and Tauri runtimes:

- Rust workspace crates under `platform/`, `schemas/`, and `shared/` provide typed contracts and reusable support code.
- Leptos provides the WASM UI runtime and component model.
- Tauri provides the authoritative desktop runtime and packaging layer.
- Browser/WASM support exists for preview, standards validation, and parity checks against the desktop shell.

`ui/` must depend on typed SDK and contract crates rather than redefining models locally. UI code must not couple directly to SurrealDB or service-private APIs.

## Runtime Model
The shell supports two runtime targets with a shared Rust composition core:

- Desktop: `desktop_tauri` hosts the Leptos shell through Tauri and uses the typed host contracts to expose desktop capabilities.
- Browser preview: `site` mounts the same shell runtime for `wasm32` builds and uses browser-compatible host adapters for preview behavior.

`desktop_runtime` is the common execution core for both targets. It keeps reducer-owned state, effect generation, compositor behavior, and shell composition in one Rust runtime while host capabilities vary by adapter selection.

## Observability and Errors
`ui/` is moving toward typed host/runtime errors plus structured tracing-based diagnostics, but the current development rule is visible-first diagnostics.

- `shared/error-model` provides shared error classification metadata such as category and visibility.
- `shared/telemetry` provides stable runtime-target and environment-profile types used by UI logs.
- `platform_host` owns the canonical UI host error contract through `HostError` and `HostResult<T>`.
- `desktop_runtime` is the target home for shared runtime logging metadata helpers and future structured `tracing` events.
- `site` currently installs the panic hook used for browser debugging, but does not yet install a tracing subscriber in the checked-in entrypoint.
- `desktop_tauri` does not yet install the native JSON tracing subscriber described by the target architecture in the checked-in entrypoint.

Current development policy:
- Development builds must preserve visible warnings, runtime diagnostics, panic hooks, reducer errors, host failures, and persistence failures in active browser and desktop workflows.
- Until browser and native tracing bootstrap is implemented and verified, warning paths in browser/runtime code should use the currently observable diagnostics channel instead of tracing-only emission.
- Migration from `leptos::logging` to `tracing` must be gated on end-to-end subscriber initialization and verified parity for developer-visible output.

Required log fields:
- `timestamp`
- `level`
- `target`
- `event`
- `operation`
- `component`
- `runtime_target`
- `environment`

Optional fields should be additive and stable, for example `window_id`, `app_id`, `host_strategy`, `error_category`, and `error_code`.

Development builds may emit richer diagnostics and should retain comprehensive warning and validation signals needed for debugging. Production defaults should stay concise, favor `warn` and `error` in wasm/browser flows, and avoid leaking internal details to end users.

Prohibited patterns:
- `Result<_, String>` on public UI host boundaries when a typed `HostError` is appropriate
- ad hoc free-form runtime diagnostics when structured `tracing` events can be emitted without reducing current developer-visible diagnostics
- logging secrets, raw credentials, uncontrolled payload dumps, or sensitive absolute paths
- non-test `unwrap`/`expect` on recoverable runtime and host paths

## Development Workflow
Use the browser/WASM preview for fast shell iteration and parity checks. Use the Tauri path to validate desktop-only integrations and packaged behavior. Keep all new UI integration behind typed contracts and place presentation-specific models only in `ui/`.

When extending the shell:

- add or refine shared UI substrate in `system_ui` before duplicating shell controls;
- keep state transitions and runtime policy in `desktop_runtime`;
- add host capability contracts in `platform_host` and implement adapters in `platform_host_web` or `desktop_tauri`;
- integrate platform and schema changes through typed crates, not ad hoc JSON payloads or direct persistence calls.

## Build Commands
Run from the repository root:

```bash
cargo ui-dev
cargo ui-build
cargo verify-ui
cargo check -p desktop_runtime
cargo check -p site
cargo check -p desktop_tauri
```

`cargo ui-dev` is the preferred browser/WASM preview workflow. `cargo ui-build` drives the corresponding build pipeline. `cargo verify-ui` now exercises the preview toolchain with a real `site_app` wasm build, Trunk packaging, and a localhost smoke probe so wasm-only browser regressions are caught before merge. Use the crate-level `cargo check` commands for focused iteration in the shared runtime, browser entrypoint, and desktop host.

## Integration Patterns
All UI-to-platform integration must flow through typed contracts.

- UI crates consume SDK, schema, and shared contract crates; they do not define canonical business models.
- Shell and app code depend on `desktop_app_contract`, `system_shell_contract`, and `platform_host` traits instead of transport-specific APIs.
- Desktop-specific behavior is implemented in `desktop_tauri`; browser-compatible behavior is implemented in `platform_host_web`.
- Cross-workspace changes should remain additive and compatibility-preserving, with UI adapters updated to consume the new contract versions rather than introducing local forks.
