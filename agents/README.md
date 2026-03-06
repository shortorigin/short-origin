# Agents

## Purpose
`agents/` defines role-based AI operational agents that support architecture, strategy, finance, legal, and compliance execution. Agent behavior must remain deterministic, auditable, and bound to repository contracts.

## Scope
In scope:
- Agent role definitions under `agents/*`.
- Prompt/runtime policies for each role.
- Cross-agent handoff conventions and validation expectations.

Out of scope:
- Owning source-of-truth business policies (belongs to `enterprise/`).
- Owning runtime APIs or schema contracts (belongs to `services/` and `schemas/`).

## Interfaces
- Input interfaces: structured tasks, schema references, and policy context.
- Output interfaces: actionable plans, change proposals, and traceable decision artifacts.
- Integration interface: role-specific invocation through `platform/runtime` and `platform/sdk`.

## Dependencies
- `enterprise/` for policy and ontology context.
- `schemas/` for event/contract compliance.
- `services/` and `workflows/` for execution touchpoints.
- `platform/` for invocation, tool access, and runtime boundaries.

## Development Workflow
1. Define each agent’s objective, authority, and constraints in its folder.
2. Encode deterministic behavior: input contract, reasoning boundaries, output schema.
3. Add evaluation scenarios for correctness, safety, and policy adherence.
4. Validate handoff behavior with other agents and human review checkpoints.
5. Start with a GitHub issue, link it from the PR, and use the procedures under `agents/procedures/` only as archived reference material where they still help explain historical agent-stage artifacts.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```
Add agent evaluation suites (fixture-based) as they are introduced.

## Integration Patterns
- Agents consume enterprise policy and ontology before proposing system changes.
- Agent outputs map to structured contracts or workflow requests, not free-form side effects.
- Multi-agent collaboration uses explicit handoff artifacts with assumptions and unresolved risks.

## Reuse Opportunities
- Reuse shared instruction primitives (risk grading, contract checks, policy gates).
- Standardize common prompt and tool wrappers in future shared agent libraries.

## Out of Scope
- Direct infrastructure mutation without workflow or policy approval.
- Creating new schema contracts outside `schemas/` governance.
- Acting as a replacement for CI quality gates.
