# Layer Boundaries

This document is the authoritative dependency-direction model for Origin. The goal is to keep the
platform modular, keep blast radius narrow, and make boundary violations detectable by repository
validation.

## Responsibility Allowlist

| Plane | Primary concern | Allowed direct dependencies |
| --- | --- | --- |
| `enterprise/` | Ontology and policy meaning | `enterprise/`, `schemas/`, `shared/` |
| `schemas/` | Canonical contracts and schemas | `schemas/`, `enterprise/`, `shared/` |
| `shared/` | Foundational reusable primitives | `shared/`, `schemas/`, `enterprise/` |
| `platform/` | Runtime, SDK, and integration abstractions | `platform/`, `schemas/`, `shared/` |
| `services/` | Contract-driven backend behavior | `services/`, `platform/`, `schemas/`, `shared/`, `enterprise/` |
| `workflows/` | Cross-service orchestration | `workflows/`, `services/`, `platform/`, `schemas/`, `shared/`, `enterprise/` |
| `ui/` | Leptos/WebAssembly shell and host adapters | `ui/`, `platform/`, `schemas/`, `shared/` |
| `agents/` | Governed agent runtime integrations | `agents/`, `platform/`, `schemas/`, `shared/`, `enterprise/`, `workflows/` |
| `testing/` | Test harnesses and regression support | any plane required by the test target |
| `xtask/` | Repository-owned validation and tooling | `xtask/`, `platform/`, `schemas/`, `shared/` |

Repository validation classifies dedicated test harness crates as `testing/` even when they are
nested under another top-level directory. Current examples include smoke-test style crates that
exist only to validate cross-plane behavior.

## Required Dependency Rules

- `ui/` may depend on stable platform interfaces, schemas, shared primitives, and internal UI
  crates only.
- `services/` may depend on `schemas/`, `shared/`, `enterprise/`, and `platform/` runtime
  interfaces.
- `platform/` may expose integration and runtime abstractions consumed by `ui/`, `services/`, and
  `workflows/`, but it must remain implementation-neutral with respect to business policy.
- `shared/` must not depend on `ui/` or infrastructure-specific implementation code.
- `infrastructure/` consumes build outputs, manifests, and environment configuration only.
- `schemas/` defines contracts, events, WIT packages, and schema artifacts; it must remain
  implementation-neutral.

## Prohibited Couplings

- `ui/ -> services/`, `ui/ -> workflows/`, `ui/ -> infrastructure/`
- `services/ -> ui/`, `services/ -> infrastructure/`
- `shared/ -> ui/`, `shared/ -> infrastructure/`, `shared/ -> services/`, `shared/ -> workflows/`
- `schemas/ -> services/`, `schemas/ -> workflows/`, `schemas/ -> ui/`, `schemas/ -> infrastructure/`
- direct SurrealDB client usage outside the governed shared access layer
- plugin modules importing core shell internals directly instead of registering through manifests
  and typed contracts

## Invalid Examples

- A `ui/` crate importing a service crate to call business logic directly.
- A `shared/` crate depending on `desktop_tauri` or Pulumi helper code.
- A `schemas/` crate importing runtime or service implementation types.
- A plugin module patching shell routes by editing core registry code without a governed manifest
  entry.

## Enforcement

Repository-owned validation runs through:

```bash
cargo xtask architecture audit-boundaries
```

The audit validates direct and transitive workspace dependencies, checks source-level workspace
crate imports, enforces targeted source-scan rules, verifies foundational dependency governance in
workspace manifests, and is part of `Governance / governance-gate`.

Remaining gaps are still explicit:
- non-Rust asset and generated-code dependency analysis is not yet fully enforced
- macro-generated imports and repository-external build tooling remain review-time concerns unless
  they surface through the existing manifest or source scans
