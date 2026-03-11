# Execution Plan

## Summary
- Establish a repo-owned local-first validation framework that makes local execution the authoritative path for required checks, aligns GitHub workflows to the same suite-selection and execution logic, blocks stale or failing branches before push, and removes tracked generated UI styling artifacts from the main merge-conflict surface.

## Task Contract
- Task contract: `plans/139-local-first-validation-framework/task-contract.json`
- GitHub issue: `#139`
- Branch: `infra/139-local-first-validation-framework`

## Scope Boundaries
- Allowed touchpoints: `xtask/`, `.github/`, `.cargo/`, `ui/`, `docs/`, and `plans/`.
- Non-goals: no unrelated runtime redesign, no container-first workflow replacement, and no bypass of merge-queue or review protections.

## Implementation Slices
- Add issue-linked execution artifacts and create the new `xtask validate` command family with shared suite selection, base resolution, reporting, and hook installation.
- Introduce repo-owned security validation policy and move workflow-only validation concerns into `xtask`, including local PR validation and CI parity entrypoints.
- Stop tracking generated UI styling artifacts, make them derived outputs, and update build/validation flows to regenerate them automatically.
- Refactor GitHub workflows, Cargo aliases, docs, templates, and governance guidance to call the repo-owned validation surfaces and document the new local-first workflow.

## Validation Plan
- Run `cargo test -p xtask`.
- Run `cargo xtask validate doctor`.
- Run `cargo xtask validate suite security`.
- Run `cargo xtask validate changed --base origin/main`.
- Run `cargo verify-repo`.
- Run `cargo verify-ui`.
- Run `cargo xtask ui-hardening`.

## Rollout and Rollback
- Roll forward in one issue-linked branch so the validation framework, workflow parity, and generated-asset governance land together.
- Roll back by reverting the change set as one unit, restoring the prior workflow entrypoints and tracked generated artifacts only if needed to reestablish a coherent validation path.

## Open Questions
- None.
