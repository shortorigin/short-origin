# GitHub-Native Development Model

## Purpose

This repository uses a GitHub-native Scrumban workflow built around issues, pull requests, GitHub Projects, and automated CI/CD. GitHub issues and milestones are the system of record for planning and delivery.

## Core Rules

1. Every material change starts with a GitHub issue.
2. No direct commits land on `main`.
3. Every merge flows through a reviewed pull request.
4. Required checks must pass before merge.
5. Work stays visible on the organization Project board.
6. Every pull request must retain a GitHub issue closing directive so merge closes the originating issue automatically.

## Flow

The standard flow is:

`Backlog -> Ready -> In Progress -> PR Open -> Review -> Done`

GitHub Projects owns the visible work queue. Milestones group issues into release objectives rather than fixed sprints.

## Issues

All work uses repository issue forms and the standard label taxonomy:

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

Every issue should capture:

- Summary
- Problem Statement
- Proposed Solution
- Acceptance Criteria
- Technical Notes
- Related Issues

Issue creation protocol:

1. Create the issue before starting implementation.
2. Confirm the issue title is specific enough to support branch and PR naming.
3. Record enough context that another contributor can execute the change without private side-channel knowledge.
4. Keep acceptance criteria concrete and testable.

## Branching and Commits

Short-lived trunk-based branches are required:

- `feature/<issue-id>-description`
- `fix/<issue-id>-description`
- `infra/<issue-id>-description`
- `docs/<issue-id>-description`
- `refactor/<issue-id>-description`
- `research/<issue-id>-description`

Branch protocol:

1. Branch from the latest `main`.
2. Include the GitHub issue identifier in the branch name.
3. Use one primary issue per branch.
4. Delete the branch after merge.

PR titles and squash-merge commit messages must use conventional commits:

`type(scope): description`

Examples:

- `feat(auth): add oauth provider`
- `fix(db): correct surrealdb transaction handling`
- `docs(api): update platform sdk examples`

## Pull Requests

Every PR must include:

- a linked issue
- a concise summary
- technical changes
- testing strategy
- deployment impact
- a closing directive in the PR body such as `Closes #123`
- repository-native language with no leaked Codex, OpenAI, ChatGPT, or other assistant/vendor branding unless required for an external reference or legal attribution

Merge policy:

- minimum reviewers: 1
- squash merge: required
- direct pushes to `main`: prohibited
- dismiss stale approvals when new commits are pushed
- code owner review: required
- auto-merge: enabled as the fallback path when merge queue is unavailable
- merging the PR closes the linked issue through the PR closing directive

## CI/CD Baseline

Required CI stages:

- governance
- ci gate
- security gate
- delivery promotion

Baseline checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo audit
```

CI optimization policy:

- Rust validation is path-scoped:
  backend-only Rust changes run the core validation profile, UI-only changes run the UI validation profile, and shared/root changes run the full workspace validation plus browser preview build.
- GitHub Actions reuses shared Rust dependency caches across CI, delivery, release, and security workflows with stable shared cache keys.
- Small cargo-installed tools are cached explicitly:
  `trunk` is cached for UI/browser preview builds and `cargo-audit` is cached for the security workflow.
- CI keeps incremental compilation disabled so cache storage is spent on reusable dependency artifacts instead of large incremental state that churns quickly on hosted runners.
- `sccache` and `target/` artifact reuse were evaluated but are not enabled in the current baseline:
  the workspace artifact footprint is too large for GitHub's cache budget to make those approaches efficient without an external cache backend.

Required status checks:

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

Documentation-to-automation drift is enforced by:

```bash
cargo xtask github audit-process
```

## Release Promotion

- `main` remains continuously mergeable after required checks pass.
- The `Delivery Dev` workflow runs automatically on `push` to `main`, publishes immutable
  `sha-<fullsha>` and `main-<shortsha>` tags to GHCR, and auto-deploys `dev`.
- The `Release Candidate` workflow rebuilds a chosen `main` SHA, publishes `vX.Y.Z-rc.N` tags,
  deploys `stage`, and creates a GitHub prerelease with generated notes.
- The `Promote Release` workflow retags the already-published GHCR digests to `vX.Y.Z`, deploys
  `production`, and creates the final GitHub Release without rebuilding.
- Rollback is manifest-based: redeploy the prior release manifest and OCI digests rather than
  rebuilding.

## Review Standards

Reviews evaluate:

- correctness
- safety
- maintainability
- performance implications
- alignment with repository architecture boundaries

Reviewers should confirm:

- the code compiles
- tests or relevant checks pass
- error handling is explicit
- contracts and docs are updated when needed
- assistant/vendor branding has not leaked into code, documentation, UI text, fixtures, commits, issues, or PR content

## Work-In-Progress Limits

- max active issues per engineer: 2
- max review queue per reviewer: 5
- target PR lifetime: 48 hours

## Repository Governance

Every repository in the organization should include:

- `README.md`
- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `LICENSE`
- `SECURITY.md`
- `ARCHITECTURE.md`
- `DEVELOPMENT_MODEL.md`

Organization rollout details and bootstrap commands live in
[`docs/process/github-governance-rollout.md`](docs/process/github-governance-rollout.md). See also
[`docs/process/github-workflow-migration.md`](docs/process/github-workflow-migration.md) for the
current contributor-facing CI/CD flow.
