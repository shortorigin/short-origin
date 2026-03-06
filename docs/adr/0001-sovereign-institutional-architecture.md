# ADR 0001: Sovereign Institutional Architecture

## Status
Accepted

## Decision
The repository operates as an institutional platform with seven authoritative top-level modules and a root Rust workspace. `enterprise/` owns ontology and policy semantics. `schemas/` owns public contracts. `workflows/` owns cross-domain mutation paths. `services/` implement typed service boundaries only against `schemas/`. `agents/` emit typed workflow requests and do not mutate infrastructure or production data directly.

## Consequences
- Every material mutation requires policy evaluation, approval verification, and evidence capture.
- Public interface evolution is additive-first and versioned.
- Pulumi remains the infrastructure implementation language while runtime, orchestration, SDK, and shared tooling are Rust-first.
- Nomad plus wasmCloud plus Wasmtime form the required runtime baseline.
