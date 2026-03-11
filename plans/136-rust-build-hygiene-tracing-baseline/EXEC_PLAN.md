# Execution Plan

## Summary
- Establish a repo-owned Rust build hygiene and tracing baseline by tightening workspace profile policy, adding repeatable `cargo xtask rust` workflows, and closing the documented UI tracing bootstrap gap without changing intended product behavior.

## Task Contract
- Task contract: `plans/136-rust-build-hygiene-tracing-baseline/task-contract.json`
- GitHub issue: `#136`
- Branch: `refactor/136-rust-build-hygiene-tracing-baseline`

## Scope Boundaries
- Allowed touchpoints are the root workspace manifest and cargo config, `xtask/`, `shared/telemetry/`, `ui/crates/site/`, `ui/crates/desktop_tauri/`, contributor docs, and the matching `plans/` bundle.
- Non-goals are schema/runtime contract redesign, external cache backend adoption, and repository-wide migration of existing diagnostics callsites to `tracing`.

## Implementation Slices
- Add the issue-linked execution artifacts and keep them updated with the bounded implementation surface.
- Add root Cargo profile policy and cargo aliases for Rust build hygiene workflows.
- Extend `xtask` with `rust audit`, `rust clean`, and `rust trace` plus deterministic tests and ignored-output reporting.
- Extend `shared/telemetry` with runtime/environment types and feature-gated tracing bootstrap helpers for browser and native entrypoints.
- Wire `site` and `desktop_tauri` to initialize tracing bootstrap and emit bootstrap lifecycle events while preserving existing visible warnings in `desktop_runtime`.
- Update contributor docs to describe the new profile policy, command surface, and tracing bootstrap status.

## Validation Plan
- Run `cargo test -p xtask`.
- Run `cargo test -p telemetry`.
- Run `cargo check -p desktop_tauri`.
- Run `cargo xtask verify profile ui`.
- Run `cargo verify-repo`.
- Run `cargo xtask ui-hardening`.
- Smoke `cargo xtask rust audit`, `cargo xtask rust clean incremental`, and `cargo xtask rust trace desktop --dry-run`.

## Rollout and Rollback
- Roll out as a dedicated PR tied to issue `#136` with the matching execution artifact bundle committed in the same branch.
- Roll back the workspace profile policy, xtask surface, telemetry bootstrap, UI entrypoint wiring, and docs together to avoid partial command/documentation or tracing-baseline drift.

## Open Questions
- None.
