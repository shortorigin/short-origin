# Architecture

## Summary

Origin is a cloud-native, WebAssembly-first enterprise application platform. The repository is not
organized as a single end-user application; it is organized as a modular platform substrate that
hosts governed plugin applications, typed platform APIs, wasmCloud service workloads, and
environment-specific delivery automation.

- `ui/` is the Leptos/WebAssembly shell and interaction surface.
- `platform/` is the reusable runtime, SDK, and execution boundary between the shell and backend.
- `services/` and `workflows/` are the wasmCloud-based distributed execution fabric.
- `schemas/` is the canonical contract boundary for data, events, and interface definitions.
- `infrastructure/` is the deployment plane that wires AWS-hosted runtime capacity to Cloudflare
  ingress and edge mediation.

The baseline runtime is the browser-delivered PWA. Tauri is a progressive enhancement host runtime
for the same platform surface: it extends local capabilities such as notifications, filesystem, and
desktop integration without creating a separate product fork.

## Platform Role

Platform responsibilities:

- publish stable contracts, SDKs, and runtime interfaces;
- host the shared UI shell and governed plugin application model;
- route operator interactions from the shell into typed platform APIs;
- execute backend behavior through wasmCloud services and workflows;
- promote immutable, digest-pinned release artifacts across environments.

Application or plugin responsibilities:

- declare identity, UI entrypoints, routes, requested capabilities, and runtime targets;
- consume platform contracts instead of private module internals;
- use published service and workflow contracts rather than bypassing the platform boundary;
- stay within declared permission and host capability limits.

## Responsibility Map

Each top-level module has one primary concern:

- `enterprise/`: domain language, ontology, and policy intent.
- `schemas/`: canonical contracts, events, WIT packages, and schema semantics.
- `shared/`: foundational reusable primitives such as error models, telemetry, identity, and
  governed data-access helpers.
- `platform/`: runtime, SDK, wasmCloud integration, and delivery abstractions.
- `services/`: domain-facing backend behavior exposed through platform contracts.
- `workflows/`: cross-service orchestration and policy-gated mutation paths.
- `ui/`: Leptos/WebAssembly shell, presentation adapters, and progressive host integration.
- `infrastructure/`: environment automation, manifests, and provider configuration.
- `agents/`: governed agent roles and runtime integration surfaces.
- `testing/`: fixtures and cross-plane regression support.
- `plans/`: repo-linked execution artifacts for long, multi-step, or high-risk delivery work.
- `xtask/`: repository-owned validation and workflow tooling.

## Runtime and Deployment Model

- User interactions enter through the `ui/` shell.
- The shell calls typed platform interfaces from `platform/` and `schemas/`.
- Platform/runtime boundaries route requests into wasmCloud services and workflows.
- Services and workflows execute on AWS-hosted wasmCloud/Wasmtime infrastructure.
- Cloudflare provides public DNS, ingress, routing, and edge/network mediation in front of AWS
  origins.
- `dev` auto-promotes from green `main`, `stage` is the release-candidate environment, and
  `production` is the final promotion target. Production manifests are currently rendered under the
  `prod/` artifact directory.

## Core Invariants

- Rust is the default implementation language for backend, SDK, orchestration, and tooling.
- Public interfaces originate from `schemas/` and enterprise ontology, not service-local types.
- Services and workflows communicate through versioned contracts and events.
- Shared foundational dependency versions are centralized in the root workspace manifest and
  validated by repository-owned tooling.
- IO-bearing platform, service, workflow, and storage ports are async-ready; in-memory adapters are
  limited to tests, local harnesses, or explicitly named simulation modes.
- Core execution identifiers at runtime boundaries use typed IDs from shared contracts and identity
  primitives rather than raw strings.
- Semantic contract parity tests must parse source-of-truth artifacts instead of relying on
  substring checks.
- Simulation, heuristic, and synthetic data adapters do not participate in default production
  service paths unless they are explicitly injected and documented.
- The shell runtime keeps one foreground execution slot per session and retains only a bounded,
  sequenced in-memory event log.
- UI code does not connect directly to SurrealDB or backend private internals.
- Agents do not mutate infrastructure or production data outside approved workflows.
- Changes favor additive compatibility over breaking revisions.

## Non-Goals

- Origin is not documented or delivered as a single monolithic application.
- Plugin modules must not bypass `schemas/`, platform SDKs, or runtime interfaces to call private
  shell, service, workflow, or infrastructure internals.
- The Tauri host is not a separate product line; it is a capability-extending runtime for the same
  shell and platform contracts.
- `shared/` is not a general-purpose dumping ground for cross-plane logic.

## Authoritative References

- [First-Principles Systems Architecture Baseline](docs/architecture/first-principles-systems-architecture-report.md)
- [First-Principles Implementation Roadmap](docs/architecture/first-principles-implementation-roadmap.md)
- [Docs Index](docs/README.md)
- [ADR Catalog](docs/adr/README.md)
- [Layer Boundaries](docs/architecture/layer-boundaries.md)
- [Plugin Application Model](docs/architecture/plugin-application-model.md)
- [Runtime Composition and Delivery](docs/architecture/runtime-composition.md)
- [DEVELOPMENT_MODEL.md](DEVELOPMENT_MODEL.md)
- [docs/process/platform-regression-guardrails.md](docs/process/platform-regression-guardrails.md)

## Delivery Model

Engineering delivery is GitHub-native and issue-driven. See
[`DEVELOPMENT_MODEL.md`](DEVELOPMENT_MODEL.md) for the operating model,
[`docs/process/github-workflow-migration.md`](docs/process/github-workflow-migration.md) for the
current automation flow, [`docs/process/execution-artifacts.md`](docs/process/execution-artifacts.md)
for repo-linked execution artifacts, and [`AGENTS.md`](AGENTS.md) for repository-specific
architecture and agent constraints. Repository-local validation includes both process audit and
architecture boundary audit through `xtask`.
