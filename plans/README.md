# Plans

`plans/` contains active execution artifacts for long, multi-step, or high-risk repository work.
GitHub issues and pull requests remain the system of record. These files are companion execution
artifacts for bounded implementation, not a replacement planning system.

## When To Use

Create a plan bundle when any of the following is true:

- the pull request spans more than one non-docs architectural plane;
- the PR risk class is `high`;
- the work is long-running enough that a second implementer would need an explicit execution plan.

For smaller single-plane work, plan artifacts are optional.

## Layout

```text
plans/
  README.md
  templates/
    task-contract.template.json
    EXEC_PLAN.template.md
  <issue-id>-<slug>/
    task-contract.json
    EXEC_PLAN.md
```

## Rules

- Directory names must start with the GitHub issue id and use kebab-case after the first dash.
- `task-contract.json` must validate against `schemas/contracts/v1/task-contract-v1.json`.
- `EXEC_PLAN.md` must use the fixed section headings documented in
  [`docs/process/execution-artifacts.md`](../docs/process/execution-artifacts.md).
- Active execution artifacts are repo-local and reviewable. Do not create new `work-items/`.
