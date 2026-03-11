# GitHub Workflow Migration Note

The repository now enforces the documented GitHub flow directly in CI/CD instead of relying on
manual-only entrypoints.

## Required Pull Request Checks

- `Governance / governance-gate`
- `CI / pr-gate`
- `Security / security-gate`

These checks run automatically on pull requests. `Governance / governance-gate` also emits the generated
process audit artifacts from `cargo xtask github audit-process`, including ADR corpus and
traceability-field validation. `CI / pr-gate`, `Security / security-gate`, and `Governance / governance-gate`
all invoke repo-owned `cargo xtask validate ...` entrypoints so local and GitHub execution stay on
the same command surface.

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
cargo verify-repo
cargo xtask validate changed
```

Install the blocking local hook once per clone:

```bash
cargo xtask validate install-hooks
```

If the change touches `ui/`, also run:

```bash
cargo verify-ui
cargo xtask validate suite ui-hardening
```

For long or high-risk work, keep the matching `plans/<issue-id>-<slug>/` execution artifacts in
git as part of the same change set. The audit command writes JSON and Markdown evidence under
`target/process-audit/`. Validation commands write Markdown and JSON evidence under
`target/validation/`.
