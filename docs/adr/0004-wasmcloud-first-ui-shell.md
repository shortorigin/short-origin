# ADR 0004: WasmCloud-First Platform and UI Shell

## Status
Accepted

## Context
The repository already centered Rust, SurrealDB, and wasmCloud-compatible services, but the deployment model, strategy sandbox runtime, and desktop shell architecture were not consistently enforced across code, tooling, and documentation. The OS shell also lived outside the workspace in `../os`, which prevented the repository from treating the user interface as a governed first-class module.

## Decision
We standardize on the following:

1. Backend workloads deploy through wasmCloud on Wasmtime. Services and workflows remain pure Rust crates, and each deployable workload exposes an adjacent wasmCloud component adapter plus versioned lattice metadata.
2. `ui/` is a first-class top-level module that owns all Leptos/Tauri code, browser preview entrypoints, and desktop host composition. The browser/PWA runtime is the baseline surface; Tauri extends that surface as a capability-enhancing desktop host.
3. SurrealDB remains the system of record, but data access is centralized in shared Rust repository/query abstractions. UI code never talks to SurrealDB directly.
4. `platform/sdk` is the typed boundary for UI commands, queries, dashboard snapshots, and event streams. Host-only concerns continue through Tauri/web adapters.
5. Strategy execution sandboxes use Wasmtime by default. In-memory execution is retained only as a test/feature-gated compatibility path.

## Consequences
- Infrastructure and Nomad definitions deploy lattice hosts and support services instead of native service binaries.
- Schema governance now includes WIT definitions for wasmCloud bindings.
- Contributor workflows must treat `ui/` as a governed module with explicit evidence for host-boundary compliance, dual-target compatibility, and typed SDK usage.
- Imported OS shell crates can be trimmed to the product release surface without breaking older deep links because compatibility placeholders remain inside the shared runtime.
