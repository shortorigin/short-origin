---
adr_id: ADR-0003
title: Capital Markets Integration
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - schemas
  - shared
  - platform
  - services
  - workflows
impacted_domains:
  - capital-markets
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - capital-markets
  - integration
  - workspace
---

# ADR 0003: Capital Markets Integration

## Context

The capital-markets capability needed to be absorbed into the main repository without preserving a
parallel workspace, duplicate control-plane logic, or finance-local contract conventions.

## Decision

The standalone finance source tree is decomposed into the primary super-repository instead of being
imported as a sibling workspace. Capital-markets ontology and service ownership live in
`enterprise/`, versioned finance contracts and events live in `schemas/`, deterministic reusable
trading logic lives in `shared/`, the Wasmtime-safe strategy runtime lives in
`platform/runtime/strategy-sandbox`, stateful capital-markets capabilities live in dedicated
services, and strategy promotion orchestration lives in `workflows/quant_strategy_promotion`.

## Constraints

- Capital-markets public contracts must align with the repository-wide schema model.
- Finance workloads must reuse the canonical event, evidence, and workflow layers.
- Strategy execution must remain Wasmtime-safe and reviewable.

## Alternatives Considered

- `Sibling finance workspace`: rejected because it duplicates governance and weakens shared
  validation.
- `Single finance service`: rejected because it collapses distinct bounded contexts.

## Rationale

Decomposition into the primary repository preserves capital-markets specialization while forcing it
through the same contract, workflow, and evidence boundaries as the rest of the platform.

## Implementation Implications

- Finance aggregates remain separate bounded contexts rather than collapsing into one service.
- Deterministic fixtures stay in the shared testing tree.
- New capital-markets work must pass the root workspace validation gates.

## Acceptance Checks

- Capital-markets contracts and events continue to originate from `schemas/`.
- Strategy promotion remains workflow-gated rather than driven by ad hoc service orchestration.
- Finance fixtures and regression tests remain in the root workspace validation path.

## Traceability

- Governing source: repository baseline ADR retained under the first-principles architecture pack
- Related roadmap phases: none
- Related controls: `testing/fixtures/finance/`, `platform/runtime/strategy-sandbox`
