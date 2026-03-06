# Contributing

## Workflow

All material changes are issue-driven.

1. Open or refine a GitHub issue using the repository issue forms.
2. Move the issue onto the organization Project in the correct `Status` column.
3. Create a short-lived branch named `feature/<issue-id>-description`, `fix/<issue-id>-description`, `infra/<issue-id>-description`, `docs/<issue-id>-description`, `refactor/<issue-id>-description`, or `research/<issue-id>-description`.
4. Keep the change set small enough to review and merge quickly.
5. Open a pull request that links the issue, describes the technical changes, and includes a testing strategy.

## Pull Request Requirements

- No direct commits to `main`.
- PR titles must use conventional commits: `type(scope): description`.
- The PR body must reference a same-repository issue with `Closes #<issue-id>` or an equivalent issue URL.
- Required checks must pass before merge.
- At least one reviewer approval is required before merge.
- Squash merge is the default merge strategy.

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
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

If your change affects dependency security posture, validate `cargo audit` locally or rely on the `Security Scan` workflow in CI.
