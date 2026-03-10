# Execution Artifacts

Execution artifacts are repo-linked companion files used for long, multi-step, or high-risk work.
They tighten implementation bounds without replacing GitHub issues or pull requests as the system
of record.

## Required Artifacts

Active plan bundles live under `plans/<issue-id>-<slug>/` and contain:

- `task-contract.json`
- `EXEC_PLAN.md`

Templates live under `plans/templates/`.

## When They Are Required

Execution artifacts are required when either of these is true:

- the PR spans more than one non-docs architectural plane;
- the PR risk class is `high`.

They are recommended for other long-running single-plane work where a second implementer would need
an explicit execution contract.

## Task Contract

`task-contract.json` is machine-validated against
`schemas/contracts/v1/task-contract-v1.json`. It records:

- the GitHub issue id and URL;
- the issue-derived branch name;
- the primary architectural plane and owning subsystem;
- architectural references, allowed touchpoints, and explicit non-goals;
- scope, target paths, acceptance criteria, validation commands, and validation artifacts;
- rollback path;
- whether an ExecPlan is required and where it lives.

## ExecPlan

`EXEC_PLAN.md` uses fixed `##` headings in this exact order:

1. `Summary`
2. `Task Contract`
3. `Scope Boundaries`
4. `Implementation Slices`
5. `Validation Plan`
6. `Rollout and Rollback`
7. `Open Questions`

The file is meant for implementer handoff and review. Keep it decision-complete and repository
local.

## Validation

Repository-owned validation runs through:

```bash
cargo xtask github validate-execution-artifacts --issue-id <id> --branch <branch> --risk-class <low|medium|high>
cargo xtask github validate-pr --event-path "$GITHUB_EVENT_PATH" --config .github/governance.toml
```

Use `cargo verify-repo` for the canonical non-UI local validation surface before opening or
refreshing a PR.
