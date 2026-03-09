# GitHub Workflow Migration Note

The repository now enforces the documented GitHub flow directly in CI/CD instead of relying on
manual-only entrypoints.

## Required Pull Request Checks

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

These checks run automatically on pull requests. `Governance / validate` also emits the generated
process audit artifacts from `cargo xtask github audit-process`, including ADR corpus and
traceability-field validation.

## Main-Branch Delivery

- `Delivery Dev` runs automatically on `push` to `main`.
- The workflow publishes immutable GHCR descriptor tags and promotes the `dev` environment.
- Contributor-facing docs and branch protection should treat this as the canonical merge-to-dev
  path.

## Operator-Only Release Actions

- `Release Candidate` remains a manual workflow for rebuilding a selected `main` SHA and deploying
  `stage`.
- `Promote Release` remains a manual workflow for promoting the release-candidate artifacts into
  `production` without rebuilding.

## Local Commands

Run these from the repository root when you want local parity with the enforced automation:

```bash
cargo xtask architecture audit-boundaries
cargo xtask plugin validate-manifests
cargo xtask github audit-process
cargo xtask verify profile core
cargo xtask verify profile ui
```

The audit command writes JSON and Markdown evidence under `target/process-audit/`.
