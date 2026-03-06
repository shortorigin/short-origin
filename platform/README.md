# Platform

## Purpose
`platform/` provides runtime and developer enablement layers for wasmCloud services, Wasmtime execution, SDK integration, and typed UI-facing platform access used across the repository.

## Scope
In scope:
- wasmCloud host/capability integration under `platform/wasmcloud`.
- Runtime orchestration and bootstrapping under `platform/runtime`.
- Shared SDKs and typed client interfaces under `platform/sdk`.
- Generated/interface bindings and lattice metadata for wasmCloud components.
- The strategy sandbox runtime and typed capital-markets workflow/client helpers used by the promotion pipeline.

Out of scope:
- Domain business policy ownership.
- Service-specific business workflows.
- Infrastructure provider provisioning details.

## Interfaces
- Runtime interface: standardized service lifecycle and capability bindings.
- SDK interface: typed client APIs for services, workflows, and UI shells.
- Host interface: wasmCloud/Wasmtime compatibility contracts.

## Dependencies
- `services/` for executable workloads.
- `schemas/` for interface contracts and type-safe payloads.
- `infrastructure/` for environment and deployment configuration.
- Optional shared crates for observability, auth, and error semantics.

## Development Workflow
1. Define runtime and SDK contracts before introducing new integration points.
2. Keep Wasmtime compatibility explicit for all runtime-facing dependencies.
3. Validate SDK ergonomics, UI transport boundaries, and backwards compatibility with contract versions.
4. Publish integration notes for services and workflows when interfaces change.
5. Start with a GitHub issue, link it from the PR, and capture runtime compatibility notes in the issue, PR description, or ADRs when interfaces change.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --features integration
```

## Integration Patterns
- Platform runtime is the approved gateway between deployed services, operational workflows, and operator-facing UI shells.
- SDKs encapsulate transport/runtime details so consumers depend on contracts, not infrastructure.
- wasmCloud capabilities are composed through explicit bindings and versioned interfaces.
- Strategy execution sandboxes use Wasmtime as the default runtime; in-memory execution exists only for tests and feature-gated compatibility paths.

## Reuse Opportunities
- Share runtime adapters and middleware as reusable crates.
- Provide common SDK scaffolds for new services and workflow clients.
- Keep finance-domain execution policy out of `platform/`; only deterministic runtime and client abstractions belong here.

## Out of Scope
- Business-domain policy authorship.
- Service-specific data ownership decisions.
- Environment-specific IaC templates.
