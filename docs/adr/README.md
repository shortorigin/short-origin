# ADR Catalog

The repository uses architecture decision records as governing control artifacts. The enterprise
ADR corpus lives in `docs/adr/`; module-local ADRs may exist in module-owned documentation trees
such as `ui/docs/adr/`, but they must follow the same front-matter and section rules.

## Numbering Policy

- `docs/adr/` is the enterprise-authoritative ADR corpus.
- Root ADR files use the numeric pattern `NNNN-title.md`.
- Numeric IDs are contiguous, additive, and never reused.
- `ui/docs/adr/` remains module-local. Those files use namespaced `adr_id` values instead of the
  enterprise numeric sequence.

## Status Vocabulary

Use one of these status values in ADR front matter:

- `Adopted`: the decision is active and authoritative.
- `Superseded`: the decision has been replaced by a later ADR.
- `Deprecated`: the decision remains historical but should not guide new work.

All active ADRs in the current corpus use `Adopted`.

## Required Front Matter

Every ADR must begin with YAML front matter using these keys:

- `adr_id`
- `title`
- `status`
- `date_adopted`
- `owners`
- `architectural_planes`
- `impacted_domains`
- `source_report`
- `roadmap_phases`
- `supersedes`
- `superseded_by`
- `review_cadence`
- `tags`

`source_report` may be `null` when the ADR did not originate from a governing report.

## Required Body Sections

Every ADR must include these sections in order:

1. `Context`
2. `Decision`
3. `Constraints`
4. `Alternatives Considered`
5. `Rationale`
6. `Implementation Implications`
7. `Acceptance Checks`
8. `Traceability`

Use the acceptance-check section for concrete merge and runtime verification statements rather than
generic aspirations.

## First-Principles Baseline Mapping

The first-principles architecture baseline is represented by these adopted ADRs:

| Blueprint ADR | Repository ADR | File |
| --- | --- | --- |
| `ADR-000` | `ADR-0005` | `docs/adr/0005-domain-bounded-platform-topology.md` |
| `ADR-001` | `ADR-0006` | `docs/adr/0006-canonical-event-substrate.md` |
| `ADR-002` | `ADR-0007` | `docs/adr/0007-consistency-semantics-by-criticality.md` |
| `ADR-003` | `ADR-0008` | `docs/adr/0008-domain-owned-data-products-with-contracts.md` |
| `ADR-004` | `ADR-0009` | `docs/adr/0009-lakehouse-open-format-analytical-foundation.md` |
| `ADR-005` | `ADR-0010` | `docs/adr/0010-durable-workflow-execution-plane.md` |
| `ADR-006` | `ADR-0011` | `docs/adr/0011-ai-as-planner-over-typed-tools.md` |
| `ADR-007` | `ADR-0012` | `docs/adr/0012-layered-intelligence-behind-stable-retrieval.md` |
| `ADR-008` | `ADR-0013` | `docs/adr/0013-zero-trust-ai-governance-control-plane.md` |
| `ADR-009` | `ADR-0014` | `docs/adr/0014-observability-slos-and-error-budgets.md` |
| `ADR-010` | `ADR-0015` | `docs/adr/0015-gitops-and-policy-as-code-control-artifacts.md` |
| `ADR-011` | `ADR-0016` | `docs/adr/0016-finops-unit-economics-and-autonomy-budgets.md` |
| `ADR-012` | `ADR-0017` | `docs/adr/0017-replaceable-compute-and-governed-state.md` |

The supporting narrative and phase sequence live in:

- [First-Principles Systems Architecture Baseline](../architecture/first-principles-systems-architecture-report.md)
- [First-Principles Implementation Roadmap](../architecture/first-principles-implementation-roadmap.md)

## Authoring Rules

- Reference one or more source reports, baseline documents, or predecessor ADRs.
- Keep decisions additive-first and compatibility-preserving when possible.
- Do not hide rollout implications or audit expectations in prose outside the acceptance and
  traceability sections.
- If an ADR affects multiple domains or planes, the linked issue and PR must carry matching
  traceability fields.
- When an ADR is superseded, update both `supersedes` and `superseded_by` rather than deleting
  history.

## Templates

- [Adopted ADR Template](templates/adopted-adr-template.md)
- [Acceptance Check Template](templates/acceptance-check-template.md)
