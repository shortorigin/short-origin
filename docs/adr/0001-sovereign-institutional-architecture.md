---
adr_id: ADR-0001
title: Sovereign Institutional Architecture
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - schemas
  - services
  - workflows
  - agents
impacted_domains:
  - platform-governance
  - repository-topology
source_report: null
roadmap_phases: []
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - topology
  - governance
  - contracts
---

# ADR 0001: Sovereign Institutional Architecture

## Context

The repository needed one authoritative platform topology so domain semantics, contracts,
orchestration, and governed automation would not drift into overlapping ownership models.

## Decision

The repository operates as an institutional platform with seven authoritative top-level modules and
a root Rust workspace. `enterprise/` owns ontology and policy semantics. `schemas/` owns public
contracts. `workflows/` owns cross-domain mutation paths. `services/` implement typed service
boundaries only against `schemas/`. `agents/` emit typed workflow requests and do not mutate
infrastructure or production data directly.

## Constraints

- Canonical contracts must not originate from service-local code.
- Workflow-gated mutation remains the only approved cross-domain mutation path.
- Agent behavior must remain subordinate to contracts, policy, and evidence capture.

## Alternatives Considered

- `Flat workspace without plane ownership`: rejected because it weakens semantic and runtime
  boundaries.
- `Service-owned contracts`: rejected because it creates incompatible public interfaces.
- `Agent-led mutation without workflow ownership`: rejected because it bypasses policy and audit.

## Rationale

The platform needs institutional boundaries before it can scale features safely. This topology keeps
semantic ownership, public interfaces, and governed execution aligned with the repository model.

## Implementation Implications

- Top-level planes remain authoritative for contributor workflow and boundary audits.
- Public interface evolution is additive-first and versioned through `schemas/`.
- Workflow, policy, and evidence requirements apply across all deployable workloads.

## Acceptance Checks

- Material cross-domain mutations continue to route through `workflows/` rather than direct
  service-to-service bypasses.
- New public contracts are added under `schemas/`, not inside `services/`.
- Agent procedures remain unable to mutate infrastructure or production data without an approved
  workflow path.

## Traceability

- Governing source: repository baseline ADR retained under the first-principles architecture pack
- Related roadmap phases: none
- Related controls: `ARCHITECTURE.md`, `AGENTS.md`, `enterprise/README.md`
