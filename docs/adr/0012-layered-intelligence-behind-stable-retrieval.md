---
adr_id: ADR-0012
title: Layered Intelligence behind a Stable Retrieval Interface
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - platform
  - services
  - shared
  - agents
impacted_domains:
  - retrieval
  - intelligence
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 5
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - retrieval
  - intelligence
---

# ADR 0012: Layered Intelligence behind a Stable Retrieval Interface

## Context

Retrieval, reasoning, and action evolve at different speeds. The platform needs a stable consumer
boundary so retrieval infrastructure can change without forcing consumer rewrites or weakening
policy filtering.

## Decision

The intelligence stack is partitioned into a retrieval layer for authorized evidence access, a
reasoning layer for planning and interpretation, and an action layer for typed-tool invocation via
workflows. Vector stores and embedding indexes are implementation details hidden behind a stable
retrieval service contract.

## Constraints

- Retrieval results must include provenance.
- Retrieval access must be policy-filtered.
- Index technology must remain replaceable without consumer contract changes.

## Alternatives Considered

- `Direct coupling to a specific vector database`: rejected because it hard-codes a fast-changing
  implementation choice.
- `Parametric-only reasoning`: rejected because it weakens factuality and updateability.
- `Domain-local retrieval stacks with no common interface`: rejected because it fragments policy and
  evaluation.

## Rationale

The stable retrieval boundary preserves optionality while grounding intelligence in current,
authorized enterprise knowledge and state.

## Implementation Implications

- Publish a retrieval API contract with intent, authorization context, provenance, freshness, and
  ranking diagnostics.
- Separate operational-state retrieval from document retrieval even if they share a facade.
- Version prompts, retrieval policies, and evaluation suites together.

## Acceptance Checks

- Retrieval consumers depend on the retrieval API contract rather than index internals.
- Every retrieval response carries provenance and authorization-aware metadata.
- Retrieval configuration changes can ship without changing downstream service or workflow
  contracts.

## Traceability

- Blueprint ADR: `ADR-007`
- Related roadmap phases: `Phase 5`, `Phase 9`
- Related invariants: `4`, `5`, `8`
