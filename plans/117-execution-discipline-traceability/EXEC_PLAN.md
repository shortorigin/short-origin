# Execution Plan

## Summary
- Harden repository execution workflow by adding repo-linked planning artifacts, task-contract validation, tighter issue-branch-PR traceability, and clearer local instructions without changing product architecture.

## Task Contract
- Task contract: `plans/117-execution-discipline-traceability/task-contract.json`
- GitHub issue: `#117`
- Branch: `infra/117-execution-discipline-traceability`

## Scope Boundaries
- Allowed touchpoints are limited to governance templates, `xtask`, schema contracts, `plans/`, and contributor-facing documentation.
- Non-goals are reactivating archived `work-items/`, redesigning platform architecture, or weakening existing checks.

## Implementation Slices
- Add `plans/` conventions, templates, and the active issue artifact bundle.
- Add `task-contract-v1` in `schemas/contracts/v1` plus Rust type and registry coverage.
- Extend `xtask` governance validation and repo verification profiles.
- Update AGENTS/docs/template surfaces so the workflow is locally discoverable.

## Validation Plan
- Run `cargo xtask verify profile repo`.
- Run `cargo xtask architecture audit-boundaries`.
- Run `cargo xtask plugin validate-manifests`.
- Run `cargo xtask github audit-process`.
- Run `cargo test -p xtask --all-targets`.
- Run `cargo test -p contracts --all-targets`.

## Rollout and Rollback
- Rollout is repository-only: merge after required checks pass and keep the new execution artifacts reviewable in git.
- Rollback is a full revert of the governance, `xtask`, schema, `plans/`, and docs changes to avoid partial workflow drift.

## Open Questions
- None.
