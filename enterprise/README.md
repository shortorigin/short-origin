# Enterprise

## Purpose
`enterprise/` is the canonical source for business-domain language, policy intent, and organizational ontology used by services, workflows, and agents. It keeps finance, legal, compliance, operations, HR, sales, and engineering concepts consistent across the platform.

## Scope
In scope:
- Domain context under `enterprise/domains/*`.
- Policy frameworks under `enterprise/policies/*`.
- Shared ontology contracts in `enterprise/ontology/*.json`.

Out of scope for this module:
- Runtime service code.
- Infrastructure provisioning.
- UI implementation.

## Interfaces
- Ontology data contracts:
  - `enterprise/ontology/entities.json`
  - `enterprise/ontology/relationships.json`
  - `enterprise/ontology/domains.json`
- Policy-to-control interface: policies define governance and regulatory requirements consumed by `workflows/` and `services/`.
- Domain vocabulary interface: terms and definitions consumed by `schemas/` and agent instructions.

## Dependencies
- Depends on `schemas/` for concrete event and contract representation.
- Depends on `workflows/` for operationalization of policies.
- Consumed by `services/` for rule evaluation and domain validation.
- Consumed by `agents/` for planning, reasoning, and compliance checks.

## Development Workflow
1. Propose domain or policy changes in `enterprise/domains/*` or `enterprise/policies/*`.
2. Update ontology JSON files to keep entities, relationships, and domain mappings consistent.
3. Cross-check downstream impact in `schemas/`, `services/`, and `workflows/`.
4. Include migration notes in PR description when semantics change.
5. Start with a GitHub issue, link the issue in the PR, and capture ontology alignment plus downstream impacts in the issue discussion, PR notes, or ADRs when semantics change.

## Build/Test Commands
Run from repo root:
```bash
jq empty enterprise/ontology/entities.json enterprise/ontology/relationships.json enterprise/ontology/domains.json
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

## Integration Patterns
- Policies map to executable controls in `workflows/`.
- Ontology terms map to SurrealDB records and event payload fields in `schemas/`.
- Service APIs and messages must use enterprise-defined terminology, not ad hoc synonyms.

## Reuse Opportunities
- Centralize shared domain constants and validation rules in future reusable crates (for example, an optional `shared/domain-model` top-level area).
- Reuse policy control mappings across all services rather than duplicating domain logic per service.

## Out of Scope
- Service deployment configuration.
- Runtime orchestration logic.
- Frontend component design.
