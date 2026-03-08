# GitHub Governance Rollout

This repository is the pilot adopter for the `shortorigin` GitHub-native Scrumban model.

## Canonical Config

- GitHub governance config: [`.github/governance.toml`](../../.github/governance.toml)
- Issue forms: [`.github/ISSUE_TEMPLATE`](../../.github/ISSUE_TEMPLATE)
- PR template: [`.github/PULL_REQUEST_TEMPLATE.md`](../../.github/PULL_REQUEST_TEMPLATE.md)
- Development policy: [`DEVELOPMENT_MODEL.md`](../../DEVELOPMENT_MODEL.md)
- Contributor workflow: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)

## Required GitHub Workflow Protocol

Every material repository change follows the same GitHub lifecycle:

1. Create a GitHub issue with context, scope, and acceptance criteria.
2. Create a dedicated issue branch from `main` named `<type>/<issue-id>-<summary>`.
3. Implement the change on that branch.
4. Open a pull request that references the issue and includes `Closes #<issue-id>` in the body.
5. Merge only after review and required checks pass so GitHub closes the linked issue automatically.

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
- auto-merge enablement for protected-branch fallback

## Manual GitHub UI Steps

The GitHub CLI currently does not cover all project-view and workflow configuration. After the sync runs:

1. Create saved repository views for each linked repository.
2. Create saved milestone views for each standard milestone.
3. Enable built-in Project workflows so new issues land in `Backlog`.
4. Configure status transitions so linked pull requests move issues through `PR Open`, `Review`, and `Done`.
5. Enable merge queue for `main` if the plan supports it.
6. Create GitHub environments `dev`, `stage`, and `production`.
7. Configure `production` to require `@shortorigin/core-maintainers` approval and disable self-approval.
8. Enable secret scanning, push protection, and private vulnerability reporting.

## Required Checks

Configure the `main` ruleset to require these check names:

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

## Environment Secrets and Vars

Create the same secret names in each environment where needed:

- `AWS_ROLE_TO_ASSUME`
- `PULUMI_STATE_BUCKET`
- `CLOUDFLARE_API_TOKEN`

`dev` deploys automatically from pushes to `main`, `stage` is reserved for release candidates, and
`production` is reserved for approved final releases. See
[`github-workflow-migration.md`](github-workflow-migration.md) for the contributor-facing flow.
