# Schemas

## Purpose
`schemas/` is the canonical contract layer for data models, events, service agreements, and wasmCloud interface definitions, with SurrealDB as the primary persistence model target. It ensures interface stability across services, workflows, agents, and UI shells.

## Scope
In scope:
- SurrealDB-focused schema definitions under `schemas/surrealdb`.
- Event contracts under `schemas/events`.
- Service and integration contracts under `schemas/contracts`.
- Repository execution contracts such as task-contract validation artifacts under `schemas/contracts`.
- Plugin/module manifest contracts for governed platform applications.
- WIT packages for wasmCloud component boundaries under `schemas/wit`.
- Capital-markets contract families for market data, execution lifecycle, research, portfolio, risk, promotion, and compliance payloads.

Out of scope:
- Service implementation details.
- Domain policy authoring.
- Infrastructure provisioning.

## Interfaces
- Database interface: SurrealDB record shapes, relation constraints, and query assumptions.
- Event interface: typed, versioned event payload definitions.
- Contract interface: request/response and capability contracts shared by services, SDKs, and UI shells.
- Plugin manifest interface: governed module metadata shared by the shell runtime, build tooling, and
  contributor documentation.
- Component interface: WIT worlds used to generate wasmCloud-compatible bindings.

## Dependencies
- `enterprise/ontology` for canonical naming and domain alignment.
- `services/` for producer/consumer implementation.
- `workflows/` for orchestration contract consumption.
- `platform/sdk` for typed interface distribution.

## Development Workflow
1. Introduce or modify schema contracts in one subdomain (`surrealdb`, `events`, `contracts`, or `wit`).
2. Version every breaking change and provide compatibility notes.
3. Update dependent service and workflow fixtures.
4. Validate forward/backward compatibility where required.
5. Start with a GitHub issue, link it from the PR, and record compatibility notes plus fixture updates in the issue discussion, PR description, or ADRs.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```
Add schema compatibility checks and fixture validation tests as contracts mature.

## Integration Patterns
- Events, contracts, and generated WIT bindings are the only approved cross-component integration boundary.
- SurrealDB schema evolution must be coordinated with service rollout sequencing.
- Contract versions are consumed through SDK abstractions, not ad hoc parsing.
- Finance pipeline fixtures in `testing/fixtures/finance/` must track schema-compatible payloads for workflow regression coverage.

## Reuse Opportunities
- Centralize serialization/deserialization, validation, and version negotiation code.
- Share contract fixture packs across services, agents, and workflow tests.

## Out of Scope
- Business policy decisions.
- Runtime host configuration.
- UI-only presentation models not used in contracts.
