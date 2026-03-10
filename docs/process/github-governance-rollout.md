# GitHub Governance Rollout

This repository is the pilot adopter for the `shortorigin` GitHub-native Scrumban model.

## Canonical Config

- GitHub governance config: [`.github/governance.toml`](../../.github/governance.toml)
- Issue forms: [`.github/ISSUE_TEMPLATE`](../../.github/ISSUE_TEMPLATE)
- PR template: [`.github/PULL_REQUEST_TEMPLATE.md`](../../.github/PULL_REQUEST_TEMPLATE.md)
- Development policy: [`DEVELOPMENT_MODEL.md`](../../DEVELOPMENT_MODEL.md)
- Contributor workflow: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
- Conflict-reduction playbook: [`pr-conflict-reduction-playbook.md`](pr-conflict-reduction-playbook.md)

## Required GitHub Workflow Protocol

Every material repository change follows the same GitHub lifecycle:

1. Create a GitHub issue with context, scope, and acceptance criteria.
   Issues must also include architectural references, owning subsystem, integration boundaries,
   impacted domains, affected consistency class, affected risk tier, rollback path, and validation
   artifacts.
2. Run `git fetch origin` before creating the working branch.
3. Create a dedicated issue branch from `origin/main` named `<type>/<issue-id>-<summary>`.
4. If the work is stacked, branch from the parent branch and target the child PR to that parent branch until it lands.
5. Rebase on the current target branch before requesting merge.
6. Open a pull request that references the issue and includes `Closes #<issue-id>` in the body.
   The PR body must also include ADR references, impacted domains, affected consistency class,
   affected risk tier, rollback path, validation artifacts, and execution artifact status.
7. Merge only after review and required checks pass so GitHub closes the linked issue automatically.

For multi-plane or `high` risk-class work, add `plans/<issue-id>-<slug>/task-contract.json` and
`plans/<issue-id>-<slug>/EXEC_PLAN.md` before opening the PR.

## Bootstrap Commands

Dry-run the organization and repository sync plans:

```bash
cargo xtask github sync org --config .github/governance.toml --dry-run
cargo xtask github sync repo --config .github/governance.toml --repository shortorigin/origin --dry-run
cargo xtask github audit-process
```

Apply the GitHub settings with `gh` once authenticated:

```bash
cargo xtask github sync org --config .github/governance.toml --apply
cargo xtask github sync repo --config .github/governance.toml --repository shortorigin/origin --apply
```

## Organization Setup

1. Create the public `shortorigin/.github` repository.
2. Copy the canonical community-health files, issue forms, PR template, and governance config into that repository.
3. Run `cargo xtask github sync org ... --apply` from the seeded checkout.

The org sync covers:

- ensuring the public `.github` repository exists
- ensuring the `Engineering Flow` organization Project exists
- ensuring the `Status` single-select field exists with the standard options
- linking configured repositories to the project

## Repository Setup

Run the repo sync for each repository that adopts the model.

The repo sync covers:

- repository merge settings
- labels
- milestones
- repository rulesets for `main` protection and branch naming
- required code owner review on `main`
- strict required status checks on `main`, which force PR heads to be current with the base branch before merge
- auto-merge enablement for protected-branch fallback when merge queue is unavailable

## Manual GitHub UI Steps

The GitHub CLI currently does not cover all project-view and workflow configuration. After the sync runs:

1. Create saved repository views for each linked repository.
2. Create saved milestone views for each standard milestone.
3. Enable built-in Project workflows so new issues land in `Backlog`.
4. Configure status transitions so linked pull requests move issues through `PR Open`, `Review`, and `Done`.
5. Enable merge queue for `main`.
6. Create GitHub environments `dev`, `stage`, and `production`.
7. Configure `production` to require `@shortorigin/core-maintainers` approval and disable self-approval.
8. Enable secret scanning, push protection, and private vulnerability reporting.

After applying or changing governance, run:

```bash
cargo verify-repo
```

Use the audit output to confirm that the live ruleset still matches `.github/governance.toml`, especially for required approvals, merge method restrictions, and status check enforcement.

## Required Checks

Configure the `main` ruleset to require these check names:

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

## Review And Merge Expectations

The checked-in governance source-of-truth for `main` expects:

- one approving review,
- code owner review,
- stale approvals dismissed on push,
- squash merge only,
- delete branch on merge.

Contributor workflow should match those protections:

- branch from fresh `origin/main`,
- use parent-targeted stacked PRs instead of opening stacked branches directly against `main`,
- rebase before merge when `main` has moved,
- regenerate derived outputs after rebasing if the PR changes shell or token inputs.

## Environment Secrets and Vars

Create the same secret names in each environment where needed:

- `AWS_ROLE_TO_ASSUME`
- `PULUMI_STATE_BUCKET`
- `CLOUDFLARE_API_TOKEN`

`dev` deploys automatically from pushes to `main`, `stage` is reserved for release candidates, and
`production` is reserved for approved final releases. See
[`github-workflow-migration.md`](github-workflow-migration.md) for the contributor-facing flow.
