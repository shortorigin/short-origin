# AGENTS

## Scope
- `schemas/` is the source of truth for versioned contracts, events, WIT interfaces, and schema semantics.
- Process and execution artifacts belong here only as typed contracts, never as implementation policy hidden in services or workflows.

## When Editing This Plane
- Prefer additive contract evolution and keep compatibility implications explicit.
- Update Rust contract types, embedded schema registry coverage, and contract tests in the same change set.
- Keep schema names, file names, and exported Rust types aligned.

## Required Companion Updates
- New or changed contract files under `schemas/contracts/v1/` must be reflected in `schemas/crates/contracts/src/lib.rs`.
- Embedded schema registry tests and any contract round-trip tests must be updated when the contract catalog changes.
- If a schema changes contributor workflow, update the relevant docs and `xtask` validators that consume it.

## Non-Goals
- Do not move canonical workflow enforcement into schema-only prose.
- Do not duplicate service-local types as contract definitions.
