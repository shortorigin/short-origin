# GitHub-Native Development Model

## Purpose

This repository uses a GitHub-native Scrumban workflow built around issues, pull requests, GitHub Projects, and automated CI/CD. GitHub issues and milestones are the system of record for planning and delivery.

## Core Rules

1. Every material change starts with a GitHub issue.
2. No direct commits land on `main`.
3. Every merge flows through a reviewed pull request.
4. Required checks must pass before merge.
5. Work stays visible on the organization Project board.

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

## Branching and Commits

Short-lived trunk-based branches are required:

- `feature/<issue-id>-description`
- `fix/<issue-id>-description`
- `infra/<issue-id>-description`
- `docs/<issue-id>-description`
- `refactor/<issue-id>-description`
- `research/<issue-id>-description`

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

Merge policy:

- minimum reviewers: 1
- squash merge: required
- direct pushes to `main`: prohibited
- dismiss stale approvals when new commits are pushed
- code owner review: required
- auto-merge: enabled as the fallback path when merge queue is unavailable

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

Required status checks:

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

## Release Promotion

- `main` remains continuously mergeable after required checks pass.
- The `Delivery Dev` workflow publishes immutable `sha-<fullsha>` and `main-<shortsha>` tags to
  GHCR and auto-deploys `dev`.
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

Organization rollout details and bootstrap commands live in [docs/process/github-governance-rollout.md](/Users/justinshort/short%20origin/docs/process/github-governance-rollout.md).
