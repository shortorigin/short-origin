# Contributing

## GitHub Workflow Protocol

All material changes are issue-driven and must follow the same GitHub workflow:

1. Create or refine a same-repository GitHub issue before writing code.
2. Ensure the issue captures:
   - the change, defect, or enhancement being proposed,
   - the architectural references, including governing ADRs and supporting architecture docs,
   - the owning subsystem or component responsible for implementation,
   - the integration boundaries, including allowed cross-plane touchpoints and explicit non-goals,
   - the impacted domains,
   - the affected consistency class,
   - the affected risk tier,
   - the primary architectural plane touched,
   - explicit scope in and scope out boundaries,
   - explicit acceptance criteria,
   - validation requirements,
   - validation artifacts,
   - rollback considerations for risky changes,
   - rollback path,
   - supporting technical notes, constraints, and related links where needed.
3. Move the issue onto the organization Project in the correct `Status` column.
4. Fetch the current remote branch tips before creating a working branch: `git fetch origin`.
5. Create a dedicated short-lived branch from `origin/main`, not from a stale local `main`.
6. Name the branch with the issue identifier using:
   - `feature/<issue-id>-description`
   - `fix/<issue-id>-description`
   - `infra/<issue-id>-description`
   - `docs/<issue-id>-description`
   - `refactor/<issue-id>-description`
   - `research/<issue-id>-description`
7. Implement the change on that issue branch and keep the change set small enough to review and
   merge quickly.
8. Keep each branch focused on one dominant subsystem or one explicitly sequenced cross-layer
   objective.
9. Install the blocking local hook once per clone with `cargo xtask validate install-hooks` so
   pushes run the repo-owned changed-scope validation gate before reaching GitHub.
10. For long, multi-step, multi-plane, or `high` risk-class work, add execution artifacts under
   `plans/<issue-id>-<slug>/` using the templates in `plans/templates/`.
11. If the work is stacked, create the child branch from its parent branch and open the child PR against the parent branch until the base PR lands.
12. Rebase on the current target branch before requesting merge, and rebase again if the target branch moves while the PR is open.
13. Open a pull request targeting `main` or the parent branch in a stack that:
   - references the originating issue,
   - records the execution artifact status or matching plan bundle path,
   - records ADR references and impacted domains,
   - records the affected consistency class and affected risk tier,
   - explains the technical change,
   - records the rollback path and validation artifacts,
   - documents the testing strategy,
   - records the risk class,
   - discloses any `git push --no-verify` bypass incident,
   - includes an `Architecture Delta` section when the PR spans multiple architectural planes,
   - includes a closing directive such as `Closes #<issue-id>`.
14. Merge only after review, merge-queue admission, and required checks pass so GitHub automatically closes the linked issue.

Example commands:

```bash
gh issue create
git fetch origin
git switch -c fix/123-runtime-error-contract origin/main
mkdir -p plans/123-runtime-error-contract
cp plans/templates/task-contract.template.json plans/123-runtime-error-contract/task-contract.json
cp plans/templates/EXEC_PLAN.template.md plans/123-runtime-error-contract/EXEC_PLAN.md
gh pr create --fill
```

## Pull Request Requirements

- No direct commits to `main`.
- PR titles must use conventional commits: `type(scope): description`.
- The PR body must reference a same-repository issue with `Closes #<issue-id>` or an equivalent issue URL.
- The PR body must include an `Execution Artifacts` section.
- The issue id in the branch name must match the issue closed in the PR body.
- Multi-plane or `high` risk-class PRs must include exactly one matching `plans/<issue-id>-<slug>/task-contract.json` and `plans/<issue-id>-<slug>/EXEC_PLAN.md`.
- The PR body must include ADR references, impacted domains, affected consistency class, affected risk tier, rollback path, and validation artifacts.
- Source files, docs, UI strings, fixtures, generated assets, commit messages, PR text, and issue text must use repository/product terminology only; do not leak Codex, OpenAI, ChatGPT, or other assistant/vendor branding unless a third-party reference or legal attribution requires it.
- Required checks must pass before merge.
- At least one reviewer approval is required before merge.
- Squash merge is the default merge strategy.
- Keep the `Closes #<issue-id>` directive in the PR body through merge so the issue closes automatically.
- If the PR is stacked, target the parent branch until the base PR merges.
- If the PR touches `ui/crates/desktop_runtime` or `ui/crates/system_ui`, rebase on the current target branch immediately before requesting merge.
- If the PR touches shell composition, `shared/`, `platform/`, `schemas/`, `.github/`, or
  `infrastructure/wasmcloud/manifests`, refresh from the latest target branch and rerun validation
  immediately before requesting merge.
- Regenerate derived assets after the last rebase and before review whenever the PR changes token, shell, or generated CSS inputs, but do not commit repo-generated UI CSS/token outputs.
- Do not mix unrelated shell refactors, generated asset churn, and behavioral fixes into one PR when they can be reviewed separately.
- Multi-plane PRs must explain why the change could not be split into a narrower sequence.

## PR Risk Classes

- `low`: one plane, additive or documentation-only changes, low rollback risk.
  Required depth: one approval, baseline validation, no special rollout note beyond normal PR body.
- `medium`: one dominant plane with contract, runtime, or generated output impact.
  Required depth: one approval, baseline validation, explicit deployment impact and rollback notes.
- `high`: multiple architectural planes, schema/runtime boundary changes, CI/governance changes, or
  environment-manifest changes.
  Required depth: code owner review, full baseline validation, explicit `Architecture Delta`,
  refresh-from-main confirmation, and rollback guidance in the PR body.

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
cargo verify-repo
```

Install the blocking local hook once per clone:

```bash
cargo xtask validate install-hooks
```

Use the changed-scope local gate while developing and before each push:

```bash
cargo xtask validate changed
cargo xtask github validate-pr-local --title "type(scope): summary" --body-file /tmp/pr-body.md
```

If the change touches `ui/`, also run:

```bash
cargo verify-ui
cargo xtask validate suite ui-hardening
```

Do not treat GitHub Actions as the first place to discover routine validation failures. `git push --no-verify` is an emergency escape hatch only and must be disclosed in the PR body.

`cargo verify-repo` now covers governance, security, and core validation through the repo-owned
`cargo xtask validate` framework.

The required GitHub checks are:

- `Governance / governance-gate`
- `CI / pr-gate`
- `Security / security-gate`

During self-review, also remove assistant-generated branding, boilerplate provenance notes, and tool-specific placeholder text before requesting review.

When resolving conflicts, validate semantic intent and plane boundaries, not just file cleanliness.
Use [docs/process/platform-regression-guardrails.md](docs/process/platform-regression-guardrails.md)
and [docs/process/pr-conflict-reduction-playbook.md](docs/process/pr-conflict-reduction-playbook.md)
for the current merge-fresh and conflict-reduction policy.
