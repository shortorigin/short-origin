# Contributing

## GitHub Workflow Protocol

All material changes are issue-driven and must follow the same GitHub workflow:

1. Create or refine a same-repository GitHub issue before writing code.
2. Ensure the issue captures:
   - the change, defect, or enhancement being proposed,
   - enough background and scope for another contributor to understand the request,
   - explicit acceptance criteria,
   - supporting technical notes, constraints, and related links where needed.
3. Move the issue onto the organization Project in the correct `Status` column.
4. Create a dedicated short-lived branch from the latest `main`.
5. Name the branch with the issue identifier using:
   - `feature/<issue-id>-description`
   - `fix/<issue-id>-description`
   - `infra/<issue-id>-description`
   - `docs/<issue-id>-description`
   - `refactor/<issue-id>-description`
   - `research/<issue-id>-description`
6. Implement the change on that issue branch and keep the change set small enough to review and merge quickly.
7. Open a pull request targeting `main` that:
   - references the originating issue,
   - explains the technical change,
   - documents the testing strategy,
   - includes a closing directive such as `Closes #<issue-id>`.
8. Merge only after review and required checks pass so GitHub automatically closes the linked issue.

Example commands:

```bash
gh issue create
git switch main
git pull --ff-only
git switch -c fix/123-runtime-error-contract
gh pr create --fill
```

## Pull Request Requirements

- No direct commits to `main`.
- PR titles must use conventional commits: `type(scope): description`.
- The PR body must reference a same-repository issue with `Closes #<issue-id>` or an equivalent issue URL.
- Source files, docs, UI strings, fixtures, generated assets, commit messages, PR text, and issue text must use repository/product terminology only; do not leak Codex, OpenAI, ChatGPT, or other assistant/vendor branding unless a third-party reference or legal attribution requires it.
- Required checks must pass before merge.
- At least one reviewer approval is required before merge.
- Squash merge is the default merge strategy.
- Keep the `Closes #<issue-id>` directive in the PR body through merge so the issue closes automatically.

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
cargo xtask github audit-process
```

If your change affects dependency security posture, validate `cargo audit` locally or rely on the `Security Scan` workflow in CI.

The required GitHub checks are:

- `Governance / validate`
- `CI / pr-gate`
- `Security / security-gate`

During self-review, also remove assistant-generated branding, boilerplate provenance notes, and tool-specific placeholder text before requesting review.
