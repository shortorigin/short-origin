# AGENTS

## Scope
- `.github/` owns repository governance templates, workflow enforcement, and contributor-facing automation entrypoints.
- Keep GitHub issue, branch, PR, and required-check policy aligned with `CONTRIBUTING.md`, `DEVELOPMENT_MODEL.md`, and `xtask/src/github.rs`.

## When Editing This Plane
- Update `.github/governance.toml`, templates, and workflow checks together when policy changes.
- Keep `Governance / validate` authoritative for process enforcement; prefer extending `xtask` over adding one-off shell scripts.
- Re-run `cargo xtask github audit-process` after any template or workflow change.

## Required Companion Updates
- If PR requirements change, update `.github/PULL_REQUEST_TEMPLATE.md`, contributor docs, and `xtask` PR validation tests.
- If issue traceability fields change, update `.github/ISSUE_TEMPLATE/*`, docs, and `xtask` process-audit expectations.
- If required checks change, update `.github/governance.toml`, workflow job names, and documentation together.

## Non-Goals
- Do not define product architecture or service behavior here.
- Do not bypass repository-owned validation with undocumented workflow-only policy.
