# Architecture

## Summary

Origin is organized around explicit module ownership, versioned contracts, and workflow-gated mutations.

- `enterprise/` owns ontology and policy meaning.
- `schemas/` owns canonical contracts, events, and schema semantics.
- `services/` own domain-facing runtime behavior only through published contracts.
- `workflows/` orchestrate cross-service behavior without bypassing service boundaries.
- `platform/` owns reusable runtime and SDK integration.
- `ui/` owns Leptos/Tauri presentation and host-facing adapters.
- `shared/` owns reusable cross-cutting Rust support crates.
- `infrastructure/` owns deployment and environment automation without business logic.

## Core Invariants

- Rust is the default implementation language for backend, SDK, orchestration, and tooling.
- Public interfaces originate from `schemas/` and enterprise ontology, not service-local types.
- Services and workflows communicate through versioned contracts and events.
- UI code does not connect directly to SurrealDB.
- Agents do not mutate infrastructure or production data outside approved workflows.
- Changes favor additive compatibility over breaking revisions.

## Delivery Model

Engineering delivery is GitHub-native and issue-driven. See
[`DEVELOPMENT_MODEL.md`](DEVELOPMENT_MODEL.md) for the operating model,
[`docs/process/github-workflow-migration.md`](docs/process/github-workflow-migration.md) for the
current automation flow, and [`AGENTS.md`](AGENTS.md) for repository-specific architecture and
agent constraints.
