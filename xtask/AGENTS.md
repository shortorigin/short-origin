# AGENTS

## Scope
- `xtask/` owns repository-native validation, governance audits, and delivery/tooling composition.
- Prefer `xtask` subcommands for repeatable repository checks instead of ad hoc scripts.

## When Editing This Plane
- Keep help output, tests, and contributor docs in sync with any new command or profile.
- Validation logic should be deterministic, workspace-root relative, and safe on clean CI runners.
- Extend existing subcommands before introducing parallel command surfaces.

## Required Companion Updates
- New governance rules belong in `xtask/src/github.rs` and must be covered by tests.
- New validation profiles belong in `xtask/src/main.rs`, `.cargo/config.toml`, and contributor docs.
- Path-classification changes must update `xtask/src/architecture.rs` tests.

## Non-Goals
- Do not push process policy into external scripts when `xtask` can own it.
- Do not add dependencies unless the repository-owned validation surface clearly needs them.
