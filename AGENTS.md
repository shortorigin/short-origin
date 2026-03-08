# AGENTS

## Architecture Principles
- Use Rust as the default implementation language for backend, orchestration, SDK, and tooling components.
- Treat contracts (`schemas/`) and ontology (`enterprise/ontology`) as source-of-truth interfaces.
- Design for modularity: components communicate through versioned contracts and events, not private internals.
- Optimize for deterministic behavior, explicit dependencies, and auditable changes.
- Prefer additive evolution and compatibility-preserving changes before breaking revisions.

## Repository Organization
- Current top-level modules are authoritative:
  - `enterprise/`, `services/`, `infrastructure/`, `agents/`, `schemas/`, `workflows/`, `platform/`, `ui/`, `shared/`, `testing/`, `docs/`.
- Keep ownership local:
  - Domain and policy semantics in `enterprise/`.
  - Runtime service implementation in `services/`.
  - Contract and schema definitions in `schemas/`.
  - Orchestration logic in `workflows/`.
  - Runtime/SDK integration in `platform/`.
  - Leptos/Tauri shells, UI adaptation models, and desktop/web host composition in `ui/`.
  - Shared data access, validation, telemetry, and reusable Rust support crates in `shared/`.
- Future top-level directories are allowed and recommended for reuse when justified:
  - `contracts/` (generated bindings).

## Component Boundaries
- `services/` MUST NOT define canonical schema contracts; consume from `schemas/` only.
- `services/` and `workflows/` SHOULD expose adjacent wasmCloud component adapters for each deployable workload instead of native deployment binaries.
- `workflows/` MUST NOT bypass service contracts to call private internals.
- `agents/` MUST NOT mutate infrastructure or production data directly outside approved workflows.
- `infrastructure/` MUST NOT embed business-domain logic.
- `platform/` MUST expose reusable runtime/SDK interfaces and avoid domain-specific policy branching.
- `ui/` MUST be the only owner of Leptos/Tauri-specific models and host-facing presentation adapters.
- `ui/` MUST NOT connect directly to SurrealDB; all governed data flows through typed SDK/contracts.

## Shared Libraries and Reuse Strategy
- Before adding new code, search for existing reusable modules; duplication requires explicit rationale in PR notes.
- Shared logic belongs in common crates (existing or future `shared/`); avoid copy-paste across services.
- Contract types, validation helpers, telemetry primitives, and error models should be centralized and versioned.
- Generated or derived bindings must originate from contract definitions, not manual divergence.

## Coding Conventions (Rust-first)
- Follow stable Rust idioms and keep `cargo fmt` formatting unchanged.
- Enforce `clippy` with warnings denied for workspace code.
- Use explicit types at module boundaries and avoid hidden implicit conversions.
- Model recoverable failures with `Result` and domain-specific error enums.
- Keep unsafe code disallowed unless documented with justification and tests.

## Branding and Provenance Hygiene
- Repository artifacts MUST NOT include Codex, OpenAI, ChatGPT, or other assistant/vendor branding in source code, generated assets, UI copy, comments, docs, tests, fixtures, commit messages, PR text, or issue text unless the material is an intentional third-party reference, legal attribution, or external integration note.
- When AI tools assist with implementation, contributors must rewrite outputs so they reflect repository terminology and product language rather than tool branding.
- Placeholder text, scaffolding comments, and generated boilerplate MUST be normalized before merge.
- Reviewers should treat leaked assistant/vendor branding as a documentation and quality defect that blocks merge until removed or justified.

## Build, Lint, and Test Standards
- Required pre-merge quality gates from repository root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```
- Changes affecting integration boundaries MUST include integration tests.
- Contract or schema changes MUST include compatibility tests or fixture updates.
- CI failures block merge; no bypass without documented incident approval.

## GitHub Workflow Protocol
- Every material code, docs, schema, workflow, or infrastructure change MUST begin with a same-repository GitHub issue before implementation starts.
- The issue is the system of record and MUST include:
  - a concise summary of the proposed change, defect, or enhancement,
  - background and scope,
  - acceptance criteria,
  - implementation notes, constraints, or linked context when needed.
- Work MUST proceed on a dedicated short-lived branch created from the latest `main`.
- Branch names MUST follow `<type>/<issue-id>-<short-kebab-summary>`.
- Approved branch type prefixes are:
  - `feature/`
  - `fix/`
  - `docs/`
  - `refactor/`
  - `infra/`
  - `research/`
- Each branch MUST map to one primary issue and MUST reference the GitHub issue identifier in the branch name.
- Pull requests MUST target `main`, reference the originating issue, and include a closing directive in the PR body such as `Closes #123`.
- Pull requests MUST summarize the change, testing performed, and any rollout or migration impact.
- Direct commits to `main` are prohibited. All merges flow through reviewed pull requests after required checks pass.
- Squash merge is the default merge strategy unless repository governance explicitly requires another merge mode.
- The final merge action MUST preserve the issue-closing directive so GitHub automatically closes the linked issue when the PR lands.

## wasmCloud + Wasmtime Integration Model
- Services are designed for wasmCloud deployment with Wasmtime-compatible modules.
- Nomad and surrounding infrastructure deploy lattice hosts and support infrastructure, not native per-service binaries.
- Runtime capabilities and provider bindings must be explicit, versioned, and documented.
- Avoid platform-specific assumptions that break deterministic Wasmtime execution.
- Service startup, health, and lifecycle contracts should be uniform across all service modules.

## SurrealDB Data and Schema Standards
- SurrealDB is the primary data layer; schema semantics are defined in `schemas/surrealdb`.
- Record types, relationships, and query assumptions must map to enterprise ontology terminology.
- Data-access behavior in services should use shared abstractions, not scattered ad hoc query strings.
- Schema changes require migration notes, compatibility impact, and rollback guidance.

## Leptos/Tauri UI Integration Standards
- UI layers live under `ui/`, use Leptos/Tauri, and consume typed SDK or contract interfaces; no direct database coupling.
- UI-specific models should adapt from shared contracts instead of redefining domain structures.
- Client interactions must preserve event/contract version expectations and error semantics.
- Desktop/web shell concerns remain separate from business orchestration logic.
- Browser/WASM preview remains supported for parity checks, but Tauri desktop is the authoritative runtime and distribution target.

## Service-to-Service and Event Integration Patterns
- Prefer asynchronous, event-driven integration for cross-service coordination.
- Use versioned event envelopes and typed payload contracts from `schemas/events`.
- Synchronous calls are allowed only for bounded request/response use cases with explicit timeouts and retries.
- Cross-component integrations must emit traceable telemetry and audit-relevant context.

## Versioning, Compatibility, and Migration Rules
- Version all public contracts and events; increment versions on breaking changes.
- Favor backward-compatible additions before removing or renaming fields.
- Breaking changes require:
  - a migration path,
  - dual-read/dual-write or compatibility adapter strategy where needed,
  - staged rollout guidance across services/workflows/platform.
- Deprecation windows must be documented before removal.

## Agent Collaboration Protocol (AI-only)
- Agents must produce deterministic outputs with explicit assumptions, constraints, and unresolved risks.
- Every cross-agent handoff must include:
  - objective,
  - inputs used,
  - decisions made,
  - pending actions,
  - verification status.
- Agents may propose changes outside their domain but may not execute boundary-crossing mutations without policy/workflow authorization.
- When requirements conflict, agents prioritize contract correctness, policy compliance, and test pass criteria in that order.
- Agents MUST NOT introduce Codex, OpenAI, ChatGPT, or similar branding into repository artifacts unless explicitly required for a documented third-party reference or legal attribution.
- Agents performing repository delivery work MUST follow the GitHub workflow protocol above:
  - create or refine the issue first when the task is intended for GitHub tracking,
  - use an issue-derived branch name,
  - open a PR with `Closes #<issue-id>` in the body,
  - never bypass review or protected-branch policy.
