# PR Conflict Reduction Playbook

This repository has recurring pull request merge conflicts because multiple short-lived branches are still converging on the same small set of `ui/` shell files while `main` is moving quickly. This document records the concrete failure pattern observed on March 8, 2026 and establishes the branch, rebase, and PR integration rules that reduce those conflicts.

## Observed Conflict Conditions

The current conflict pattern is not theoretical. It is tied to specific conditions already present in this repository:

- PR [#85](https://github.com/shortorigin/origin/pull/85) and PR [#86](https://github.com/shortorigin/origin/pull/86) were both opened on March 8, 2026 at `16:38:57Z` and both targeted `main`.
- PR [#82](https://github.com/shortorigin/origin/pull/82) merged into `main` on March 8, 2026 at `16:41:16Z`, less than three minutes later.
- PR `#85` was stacked on `#86`, but the child PR still targeted `main` instead of the parent branch.
- The local `origin/main` reference used during branch preparation lagged the actual remote `main` tip (`e14cd0f` locally versus `9a63e00` on GitHub after fetch).
- The affected PRs all touched the same conflict hot spots:
  - `ui/crates/desktop_runtime/src/components*`
  - `ui/crates/system_ui/*`
  - `ui/crates/site/src/generated/*`

Recent merged pull requests show repeated overlap in the same subsystem. The recurring conflict surface is the desktop shell/runtime boundary, especially shell components, generated CSS/tokens, and the system UI token/build pipeline.

## Root Causes

### 1. Stale branch bases

Branches were prepared from a local view of `main` that was already behind GitHub. That makes conflicts likely before review even starts.

### 2. Stacked PRs opened directly against `main`

This repository defaults to squash merge. When a child branch includes the parent branch's commits and still targets `main`, the child PR carries unrelated base work and becomes fragile as soon as the parent or another overlapping PR lands.

### 3. Parallel work in high-churn shell files

The same `ui/` shell/runtime files are being edited by multiple refactors and fixes in close succession. Even well-scoped branches conflict when they change the same layout, token, and generated output surfaces at the same time.

### 4. Mixed source changes and generated outputs

Shell PRs historically included both hand-authored source changes and regenerated outputs such as `ui/crates/site/src/generated/tailwind.css` and `ui/crates/site/src/generated/tokens.css`. Those generated files amplified conflicts because they were large, derived from shared inputs, and easy to invalidate after a rebase. The repo now treats those outputs as derived local artifacts and they should not be committed.

### 5. Governance drift between source-of-truth and live settings

The checked-in governance config requires one approval, code owner review, squash merge only, and strict status checks. The live repository ruleset was observed in a weaker state, which means contributors can follow a looser workflow than the repo intends unless governance is re-synced and audited.

## Required Branch Lifecycle

Use this lifecycle for every material change:

```bash
gh issue create
git fetch origin
git switch -c <type>/<issue-id>-<summary> origin/main
```

Rules:

1. Always fetch before creating the branch.
2. Create the branch from `origin/main`, not from a stale local `main`.
3. Keep the branch focused on one primary issue.
4. Install the blocking local hook with `cargo xtask validate install-hooks` so stale or failing pushes are rejected before GitHub.
5. Rebase on the current target branch before requesting merge.
6. If `main` moves while the PR is open, rebase again before merge.

For conflict hot spots, rebasing before merge is mandatory:

- `ui/crates/desktop_runtime`
- `ui/crates/system_ui`
- `ui/crates/site/src/generated`

## Stacked PR Rules

Stacked work is allowed, but only with explicit base-branch discipline:

1. Create the child branch from the parent branch.
2. Open the child PR against the parent branch, not against `main`.
3. After the parent merges, rebase the child onto fresh `origin/main`.
4. Retarget the child PR to `main` only after the rebase is complete.
5. Do not ask reviewers to review a child PR that still contains parent-only commits unrelated to the child change.

Under squash merge, this rule is non-optional. The child branch cannot rely on parent commit hashes surviving merge.

## High-Churn UI Shell Sequencing

When work touches the shell/runtime conflict hot spots:

1. Avoid running multiple parallel PRs that all modify shell layout, taskbar composition, token generation, or generated CSS outputs.
2. Split refactors from behavior fixes when practical.
3. Do not combine wallpaper, taskbar, shell layout, and token regeneration into one PR unless they are inseparable.
4. Regenerate derived assets only after rebasing onto the current target branch.
5. Treat `ui/crates/site/src/generated/*` and `ui/crates/site/tailwind.config.js` as derived artifacts that must be refreshed locally after shell or token changes, but do not commit them.

## Conflict Resolution Playbook

When a PR becomes stale or conflicting:

```bash
git fetch origin
git switch <branch>
git rebase origin/main
```

If the PR is stacked:

```bash
git fetch origin
git switch <child-branch>
git rebase <parent-branch-or-origin/main>
```

After the rebase:

1. Resolve conflicts in source files first.
2. Regenerate derived assets from the rebased sources.
3. Re-run validation.
4. Force-push the rebased branch with lease protection.
5. If the PR was stacked, retarget it only after the branch content matches the intended review scope.

Recommended validation:

```bash
cargo xtask validate changed
```

If the rebased change touches `ui/`, also run:

```bash
cargo verify-ui
cargo xtask validate suite ui-hardening
```

## Governance And Review Expectations

The checked-in governance model for `main` expects:

- one approving review,
- code owner review,
- stale approvals dismissed on push,
- strict required status checks,
- squash merge only,
- delete branch on merge.

Operational follow-through:

1. Re-sync repository governance after changing `.github/governance.toml`.
2. Run `cargo xtask github audit-process` regularly to detect drift.
3. Keep merge queue enabled for `main` in GitHub so conflicting or stale PRs are rebased through a single protected integration path.
