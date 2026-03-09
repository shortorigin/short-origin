# Platform Regression Guardrails

This document records the structural causes of recurrent regressions and the required safeguards
that Origin uses to reduce merge conflicts, narrow change blast radius, and preserve architecture
integrity.

## Regression Classes and Required Safeguards

### Broad cross-cutting branches

Cause:
- one branch edits several architectural planes without a dominant subsystem or explicit sequence.

Safeguard:
- each issue must declare a primary architectural plane, scope in, and scope out;
- each branch must target one dominant subsystem or one explicitly sequenced cross-layer objective;
- multi-plane PRs must include an `Architecture Delta` section and risk classification.

### Overlapping edits to shared shell or runtime files

Cause:
- parallel work lands in the same `ui/`, `shared/`, `platform/`, or schema hot spots.

Safeguard:
- changes touching shell composition, shared primitives, platform runtime interfaces, contracts, or
  environment manifests must refresh from the latest target branch and rerun validation immediately
  before review;
- split refactors from behavior changes whenever the work can be sequenced safely.

### Stale branches merged after base drift

Cause:
- a PR is reviewed on an out-of-date branch and merged after `main` or the target branch has moved.

Safeguard:
- contributors must branch from fresh `origin/main`;
- contributors must rebase or otherwise refresh from the latest target branch before requesting
  merge;
- GitHub branch protection remains strict on required checks so stale heads cannot merge cleanly by
  policy drift alone.

### File-clean conflict resolution without architectural validation

Cause:
- conflicts are resolved syntactically, but the result violates boundaries or platform intent.

Safeguard:
- conflict resolution must validate semantic intent, not only merged file contents;
- contributors must rerun `cargo xtask architecture audit-boundaries` and
  `cargo xtask github audit-process` after resolving conflicts in shared planes.

### Insufficient integration coverage for cross-layer behavior

Cause:
- compilation passes while runtime, contract, or shell-composition behavior drifts.

Safeguard:
- changes affecting integration boundaries must add or update integration tests;
- shell/runtime registration, contract compatibility, and plugin manifest validation are validated
  through repository-owned tests before merge.

### Tool-assisted changes without acceptance constraints

Cause:
- broad automated edits land without explicit scope, rollback, or verification limits.

Safeguard:
- issue forms require acceptance criteria, validation requirements, and rollback considerations;
- PRs must record contracts changed, tests added or updated, and risk class;
- repository-owned automation audits contributor templates and enforcement commands for drift.

## Merge-Fresh Validation Hot Spots

Refreshing from the latest target branch and rerunning validation is mandatory when a change touches:

- `ui/`
- `shared/`
- `platform/`
- `schemas/`
- `.github/`
- `infrastructure/wasmcloud/manifests/`

Recommended commands:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo xtask architecture audit-boundaries
cargo xtask plugin validate-manifests
cargo xtask github audit-process
```
