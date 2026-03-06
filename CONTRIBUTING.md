# Contributing

This repository is issue-driven, pull-request reviewed, and release-managed through GitHub workflows. The goal is to keep `main` continuously releasable while preserving contract and operational auditability.

## Workflow

All material changes are issue-driven.

1. Open or refine a GitHub issue using the repository issue forms.
2. Move the issue onto the organization Project in the correct `Status` column.
3. Create a short-lived branch named `feature/<issue-id>-description`, `fix/<issue-id>-description`, `infra/<issue-id>-description`, `docs/<issue-id>-description`, `refactor/<issue-id>-description`, or `research/<issue-id>-description`.
4. Keep the change set small enough to review and merge quickly.
5. Open a pull request that links the issue, describes the technical changes, and includes a testing strategy.

## Standard Contributor Flow

1. Sync from `main` and create a short-lived branch for exactly one scoped change.
2. Implement the change in the owning module and update contracts or docs when boundaries change.
3. Run the required verification locally from the repository root.
4. Push the branch and open a PR with a conventional title and issue link.
5. Address review feedback with follow-up commits on the same branch.
6. Squash merge after approvals and required checks complete.
7. Confirm any follow-up operational task, release note, or migration note is captured before closing the thread.

## Pull Request Requirements

- No direct commits to `main`.
- PR titles must use conventional commits: `type(scope): description`.
- The PR body must reference a same-repository issue with `Closes #<issue-id>` or an equivalent issue URL.
- Required checks must pass before merge.
- Review requirements are defined by [`.github/governance.toml`](/Users/justinshort/short%20origin/.github/governance.toml) and enforced through repository rulesets.
- Squash merge is the default merge strategy.
- PRs must call out contract, schema, workflow, migration, or deployment impact when those areas change.
- If a PR changes `.github/workflows/`, `.github/actions/`, or release automation, update the related docs in `DEVELOPMENT_MODEL.md` or `docs/process/`.

## Git Conventions

- Branches must follow `feature/<issue-id>-description`, `fix/<issue-id>-description`, `infra/<issue-id>-description`, `docs/<issue-id>-description`, `refactor/<issue-id>-description`, or `research/<issue-id>-description`.
- Branch from `main` and keep branch lifetime short.
- Avoid force-push after review has started unless the PR conversation explicitly notes the rewrite.
- Use empty commits only when intentionally retriggering GitHub workflows or recording an operational checkpoint.
- Keep commits focused and readable even though merges squash; review quality depends on branch history being understandable.

## Labels and Milestones

Use the standard label taxonomy:

- `type:feature`
- `type:bug`
- `type:refactor`
- `type:docs`
- `type:infra`
- `type:research`
- `priority:low`
- `priority:medium`
- `priority:high`
- `priority:critical`

Use milestones to group issues into release objectives such as `v0.1 - MVP` and `v1.0 - Production`.

## Verification

Run the baseline checks from the repository root:

```bash
cargo verify-ci
```

For the narrower baseline that excludes UI/build parity checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

If your change affects dependency security posture, validate `cargo audit` locally or rely on the `Security Scan` workflow in CI.

Additional expectations:

- Changes affecting integration boundaries need integration tests.
- Schema or contract changes need fixture, compatibility, or migration updates.
- Workflow and release changes should be validated with targeted dry runs or documented runner-only validation steps.

## GitHub Operations

- Use the organization Project board status flow: `Backlog -> Ready -> In Progress -> PR Open -> Review -> Done`.
- Treat GitHub issues, PRs, releases, and workflow runs as the system of record for delivery history.
- Delivery to `dev`, `stage`, and `production` must flow through the defined GitHub Actions workflows and environment protections.
- Do not bypass GitHub environment approval boundaries with manual production changes unless there is an incident record and maintainer authorization.

See [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md), [AGENTS.md](/Users/justinshort/short%20origin/AGENTS.md), and [docs/process/github-governance-rollout.md](/Users/justinshort/short%20origin/docs/process/github-governance-rollout.md) for the governing policy and repository bootstrap details.
