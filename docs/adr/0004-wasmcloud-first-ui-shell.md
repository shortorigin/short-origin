---
adr_id: ADR-0004
title: WasmCloud-First Platform and UI Shell
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - ui
  - platform
  - services
  - workflows
impacted_domains:
  - platform-shell
  - wasmcloud-runtime
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - wasmcloud
  - ui-shell
  - runtime
---

# ADR 0004: WasmCloud-First Platform and UI Shell

## Context

The repository centered Rust, SurrealDB, and wasmCloud-compatible services, but deployment model,
shell runtime ownership, and UI host composition were not yet enforced consistently across code and
documentation.

## Decision

Backend workloads deploy through wasmCloud on Wasmtime. Services and workflows remain pure Rust
crates, and each deployable workload exposes an adjacent wasmCloud component adapter plus versioned
lattice metadata. `ui/` is a first-class top-level module that owns all Leptos/Tauri code, browser
preview entrypoints, and desktop host composition. The browser/PWA runtime is the baseline surface;
Tauri extends that surface as a capability-enhancing desktop host. SurrealDB remains the system of
record, but data access is centralized in shared Rust repository and query abstractions. UI code
never talks to SurrealDB directly. `platform/sdk` remains the typed boundary for UI commands,
queries, dashboard snapshots, and event streams.

## Constraints

- WasmCloud and Wasmtime remain the default backend deployment model.
- `ui/` owns host-facing presentation concerns without forking the product contract model.
- Shared data access remains centralized and typed.

## Alternatives Considered

- `Native per-service binaries`: rejected because they diverge from the platform runtime model.
- `UI-owned database access`: rejected because it bypasses governed SDK contracts.
- `Separate desktop product model`: rejected because it forks the platform surface.

## Rationale

The platform needs one deployable model and one typed shell boundary so UI, runtime, and service
evolution stay compatible instead of drifting into host-specific exceptions.

## Implementation Implications

- Infrastructure deploys lattice hosts and support services instead of native service binaries.
- Schema governance includes the wasmCloud/WIT contract surfaces.
- UI and SDK changes must preserve shared contract and host-boundary compatibility.

## Acceptance Checks

- Deployable backend workloads continue to expose wasmCloud-compatible adapters.
- UI code does not add direct SurrealDB connections or alternate contract models.
- Browser/PWA remains the baseline runtime surface, with Tauri extending rather than forking it.

## Traceability

- Governing source: repository baseline ADR retained under the first-principles architecture pack
- Related roadmap phases: none
- Related controls: `docs/architecture/runtime-composition.md`, `docs/architecture/layer-boundaries.md`
