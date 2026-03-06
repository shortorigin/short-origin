# Services

## Purpose
`services/` contains domain-facing backend services implemented in Rust and designed for wasmCloud deployment on Wasmtime. Services enforce business rules, expose stable interfaces, and coordinate with SurrealDB-backed data contracts through shared repository abstractions.

## Scope
In scope:
- Service modules under `services/*-service`.
- Service-level APIs, handlers, and business orchestration.
- Adjacent wasmCloud component adapters for each deployable service workload.
- Event production/consumption aligned with `schemas/events`.
- Capital-markets services such as `market-data-service`, `quant-research-service`, `execution-service`, `portfolio-service`, and `trading-risk-service`.

Out of scope:
- Shared schema ownership (belongs to `schemas/`).
- Workflow ownership (belongs to `workflows/`).
- Infrastructure provisioning (belongs to `infrastructure/`).

## Interfaces
- Inbound: command/query requests from platform runtime and workflow drivers.
- Outbound: domain events and contract-bound payloads defined in `schemas/`.
- Data: SurrealDB access via `shared/surrealdb-access`.
- SDK: typed interfaces exposed via `platform/sdk`.

## Dependencies
- `schemas/` for event and contract definitions.
- `platform/runtime` and `platform/wasmcloud` for execution integration.
- `enterprise/` for policy and ontology alignment.
- Optional shared crates for auth, logging, telemetry, and error models.

## Development Workflow
1. Implement service behavior in Rust with explicit module boundaries.
2. Bind all external I/O to typed contract definitions from `schemas/`.
3. Ensure wasmCloud component compatibility and Wasmtime-safe dependencies.
4. Add unit tests for domain logic and integration tests for interface behavior.
5. Start with a GitHub issue, link it from the PR, and capture contract references plus owned aggregate boundary impacts in the issue discussion or PR notes.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --features integration
```

## Integration Patterns
- Services publish and consume versioned events only from `schemas/events`.
- Service-to-service interactions prefer async event-driven contracts over direct tight coupling.
- SurrealDB interactions go through shared query abstractions to prevent query divergence.
- UI consumers reach services through typed SDK/contracts, never direct service-owned data stores.

## Reuse Opportunities
- Extract common auth, validation, error, and telemetry layers into shared crates.
- Reuse SDK adapters from `platform/sdk` instead of custom transport glue per service.
- Keep deterministic trading math, simulation, and error models in `shared/trading-*` crates instead of copying them across finance-facing services.

## Out of Scope
- Writing policy source documents.
- Defining canonical schema ownership outside `schemas/`.
- UI rendering logic and desktop shell code.
