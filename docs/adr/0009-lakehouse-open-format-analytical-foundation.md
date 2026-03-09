---
adr_id: ADR-0009
title: Lakehouse-Style Analytical and ML Foundation Using Open Formats
status: Adopted
date_adopted: 2026-03-09
owners:
  - core-maintainers
architectural_planes:
  - shared
  - services
  - platform
  - docs
impacted_domains:
  - analytics
  - machine-learning
source_report: docs/architecture/first-principles-systems-architecture-report.md
roadmap_phases:
  - Phase 2
  - Phase 9
supersedes: []
superseded_by: []
review_cadence: annual
tags:
  - first-principles
  - lakehouse
  - open-formats
---

# ADR 0009: Lakehouse-Style Analytical and ML Foundation Using Open Formats

## Context

The platform needs one governed historical and analytical substrate for BI, analytics, ML, and AI
workloads instead of a separate storage stack per consumer type.

## Decision

Analytical, feature, and historical AI workloads shall converge on a lakehouse-style foundation
using open table and file formats with multiple compatible compute engines over shared governed
datasets.

## Constraints

- Open formats are mandatory for durable historical facts.
- The analytical plane must not become a shadow operational system of record.
- Data products landing in the analytical plane retain lineage to canonical events.

## Alternatives Considered

- `Separate warehouse, feature store, and AI corpus`: rejected because it fragments governance and
  lineage.
- `Vendor-locked analytical stack`: rejected because it reduces long-term optionality.
- `Direct analytics reads from operational stores`: rejected because it couples reporting and ML to
  runtime systems.

## Rationale

A shared analytical substrate prevents storage and compute fragmentation while preserving future
engine choice and lineage clarity.

## Implementation Implications

- Define event-to-lake ingestion contracts.
- Establish governed refinement zones for historical facts.
- Publish lineage from event envelope to analytical tables and retrieval-ready corpora.

## Acceptance Checks

- Historical analytical datasets land on open table or file formats rather than opaque proprietary
  stores.
- Analytical artifacts can trace lineage back to canonical events or governed data products.
- Operational stores are not treated as the primary analytical substrate.

## Traceability

- Blueprint ADR: `ADR-004`
- Related roadmap phases: `Phase 2`, `Phase 9`
- Related invariants: `4`, `7`, `8`
