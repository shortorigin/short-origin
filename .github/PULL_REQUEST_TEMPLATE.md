## Summary

## Linked Issue

Closes #

## Execution Artifacts

- Task contract: `plans/<issue-id>-<slug>/task-contract.json` | `Not required`
- Exec plan: `plans/<issue-id>-<slug>/EXEC_PLAN.md` | `Not required`

## ADR References

-

## Impacted Domains

-

## Layers Touched

- [ ] `enterprise`
- [ ] `schemas`
- [ ] `shared`
- [ ] `platform`
- [ ] `services`
- [ ] `workflows`
- [ ] `ui`
- [ ] `infrastructure`
- [ ] `agents`
- [ ] `testing`
- [ ] `docs`
- [ ] `.github` / delivery tooling

## Contracts Changed

- None.

## Tests Added or Updated

- None.

## Refreshed from Main

- Branch refreshed from the latest target branch before review: yes/no
- Validation rerun after refresh: yes/no

## Risk Class

- low | medium | high

## Affected Consistency Class

- Class A | Class B | Class C | Not Applicable

## Affected Risk Tier

- low | medium | high | not applicable

## Architecture Delta

- Single-plane change or why a multi-plane sequence was required.

## Workflow Checklist

- [ ] This branch is based on the current target branch (`origin/main` for normal PRs, the parent branch for stacked PRs).
- [ ] If this PR is stacked, the PR base points to the parent branch until that parent work merges.
- [ ] If this PR touches `ui/crates/desktop_runtime`, `ui/crates/system_ui`, `shared/`, `platform/`, `schemas/`, `.github/`, or `infrastructure/wasmcloud/manifests`, I refreshed from the latest target branch and reran validation immediately before requesting merge.
- [ ] If this PR changes shell, token, or Tailwind inputs, I regenerated the local derived UI outputs after the last rebase and did not commit repo-generated CSS/token files.
- [ ] The repository pre-push hook is installed locally, or I am disclosing below why it was bypassed.

## Local Validation

- `cargo xtask validate changed`: pass/fail
- `cargo xtask github validate-pr-local`: pass/fail
- `git push --no-verify` used: no/yes
- If `git push --no-verify` was used, document the incident, rationale, and follow-up issue here.

## Technical Changes

## Testing Strategy

## Rollback Path

- Describe the rollback path or write `Not applicable`.

## Validation Artifacts

- List commands, reports, or review evidence.

## Deployment Impact
