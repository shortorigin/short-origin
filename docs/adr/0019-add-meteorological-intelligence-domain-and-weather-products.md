---
adr_id: ADR-0019
title: Add Meteorological Intelligence as a Domain-Owned Weather Product Cell
status: Adopted
date_adopted: 2026-03-10
owners:
  - core-maintainers
architectural_planes:
  - enterprise
  - schemas
  - services
  - workflows
  - platform
  - ui
impacted_domains:
  - meteorological-intelligence
  - data-products
source_report: null
roadmap_phases:
  - Phase 2
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - weather
  - data-products
  - ui
  - analytics
---

# ADR 0019: Add Meteorological Intelligence as a Domain-Owned Weather Product Cell

## Context

Origin needs a governed weather capability that can support multiple consumers without coupling
UI, analytics, and future decision-support modules to private ingestion internals or ad hoc data
copies.

## Decision

Origin will add a new enterprise domain, `meteorological_intelligence`, owned by the Chief Data
Officer. The domain will publish canonical weather products through `meteorological-service`,
ingest data only through `weather_ingestion`, and expose typed contracts for UI and analytical
consumers.

## Constraints

- The weather capability must fit existing `enterprise/`, `schemas/`, `services/`, `workflows/`,
  `platform/`, and `ui/` planes.
- UI consumers must depend only on platform SDK surfaces and contracts.
- Heavy gridded or tabular weather artifacts must remain in open formats; governed metadata and
  event history remain in canonical records and events.
- Optional non-NOAA connectors must remain feature-gated.

## Alternatives Considered

- `Extend capital_markets_data`: rejected because weather semantics would be owned by a finance
  domain even though the consumers are broader.
- `Place weather inside data_knowledge`: rejected because document/retrieval semantics are a poor
  fit for operational forecast and alert products.
- `Build a separate crate tree`: rejected because it duplicates repository architecture and weakens
  governance.

## Rationale

Making weather a first-class domain keeps ownership, contracts, and downstream integrations
explicit while still allowing multiple consumers to reuse one normalized source of truth.

## Implementation Implications

- Add weather contracts, events, metadata record types, and a service boundary aligned to the new
  domain.
- Publish one canonical weather product ladder: immutable source assets, normalized weather
  products, derived view products, and derived feature products.
- Use platform SDK surfaces for UI reads and contract-driven read-only integrations for knowledge
  and quantitative research services.

## Acceptance Checks

- The repository defines `meteorological_intelligence` in enterprise domain and ontology artifacts.
- Weather contracts and events are versioned and embedded in the schema registry.
- `meteorological-service` and `weather_ingestion` compile and expose boundary metadata consistent
  with enterprise catalogs.
- The standalone weather app is registered by manifest and consumes only platform/contract
  surfaces.

## Traceability

- Related ADRs: `ADR-0005`, `ADR-0006`, `ADR-0008`, `ADR-0009`, `ADR-0012`, `ADR-0017`
- Related invariants: `2`, `4`, `7`, `8`
