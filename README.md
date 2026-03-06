# Short Origin

Short Origin is a Rust-first, contract-driven platform organized around explicit module boundaries, typed contracts, and governed delivery workflows. The repository is structured for wasmCloud and Wasmtime deployment, SurrealDB-backed data contracts, and Leptos/Tauri operator surfaces.

## Top-Level Modules

- `enterprise/`: ontology, policy semantics, and domain language.
- `schemas/`: canonical contracts, events, WIT packages, and SurrealDB schema definitions.
- `services/`: Rust service implementations deployed through wasmCloud-compatible adapters.
- `workflows/`: cross-service orchestration and policy-gated mutation paths.
- `platform/`: runtime, SDK, and integration layers.
- `ui/`: Leptos/Tauri presentation, shell composition, and host adapters.
- `shared/`: reusable Rust crates for telemetry, validation, error models, and data access.
- `infrastructure/`: deployment and environment automation.
- `agents/`: governed agent procedures and role definitions.
- `testing/`: fixtures and integration-test support.
- `docs/`: ADRs, process guidance, and rollout documentation.

## Development

Short Origin uses a GitHub-native Scrumban model.

1. Start with a GitHub issue.
2. Work on a short-lived branch named `feature/<issue-id>-description`, `fix/<issue-id>-description`, or the matching approved prefix from [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md).
3. Open a pull request with a conventional title such as `feat(auth): add oauth provider`.
4. Merge to `main` through squash merge after review and required checks pass.

Primary contributor docs:

- [ARCHITECTURE.md](/Users/justinshort/short%20origin/ARCHITECTURE.md)
- [CONTRIBUTING.md](/Users/justinshort/short%20origin/CONTRIBUTING.md)
- [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md)
- [SECURITY.md](/Users/justinshort/short%20origin/SECURITY.md)
- [Security Rust Initiative](/Users/justinshort/short%20origin/docs/security-rust/README.md)

## Bootstrap

Run these from the repository root when setting up or diagnosing a local environment:

```bash
cargo doctor --domain all
cargo tasks
cargo verify
cargo dx
```

Cargo aliases in [.cargo/config.toml](/Users/justinshort/short%20origin/.cargo/config.toml) provide a short front door for the `xtask` substrate. Use `cargo x ...` as the escape hatch for any full `xtask` command, `cargo tasks` to list registered workflows, `cargo task <task-id>` to run a named task, and `cargo doctor --domain <domain>` to inspect prerequisites.
`cargo dx` launches the Ratatui developer console for Git state, issue/PR workflow state, prerequisite checks, task execution, workflow runs, and guarded release dispatch.

## Verification

Run from the repository root:

```bash
cargo verify
cargo ui-verify
cargo ui-e2e --all-scenes
cargo infra-verify
cargo infra-preview --stack dev
cargo security-audit
cargo xtask run verify-full
cargo xtask workspace verify --profile full
```

The aliases are ergonomic wrappers over the same `xtask` surface. `cargo verify` maps to the canonical `cargo xtask run verify-full`, and the existing `workspace verify --profile full` alias remains supported for compatibility.

Other common task entrypoints:

```bash
cargo xtask run components-build
cargo xtask run ui-verify
cargo xtask ui e2e --all-scenes
cargo xtask run docs-security-book-test
cargo xtask run infra-verify
cargo xtask infra preview --stack dev
cargo xtask run security-audit
cargo xtask artifacts schemas export --output-dir target/generated/schemas
cargo xtask cache status
```

Security book contributors can also use the direct docs subcommands:

```bash
cargo xtask docs security-book build
cargo xtask docs security-book preview
cargo xtask docs security-book test
```

## Delivery and Release

Short Origin uses a GitHub-native, trunk-based delivery model:

- `main` is the only long-lived branch.
- `CI / pr-gate`, `Security / security-gate`, and `Governance / validate` are the required checks.
- Merges to `main` publish digest-addressable component descriptors to `ghcr.io/shortorigin/*` and
  auto-promote the `dev` environment.
- Release candidates are cut from a green `main` SHA, deployed to `stage`, and promoted to
  `production` only through the `Promote Release` workflow.

Digest-pinned environment manifests are rendered under
[`infrastructure/wasmcloud/manifests`](/Users/justinshort/short%20origin/infrastructure/wasmcloud/manifests)
and attached to GitHub Releases for rollback and audit.
