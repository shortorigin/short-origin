# Origin

Origin is a Rust-first, contract-driven platform organized around explicit module boundaries, typed contracts, and governed delivery workflows. The repository is structured for wasmCloud and Wasmtime deployment, SurrealDB-backed data contracts, and Leptos/Tauri operator surfaces.

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

Origin uses a GitHub-native Scrumban model.

1. Start with a same-repository GitHub issue that defines context, scope, and acceptance criteria.
2. Work on a short-lived issue branch named `<type>/<issue-id>-description` using an approved prefix from [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md).
3. Open a pull request with a conventional title and a PR body that includes `Closes #<issue-id>`.
4. Merge to `main` only after review and required checks pass so the linked issue closes automatically.

Primary contributor docs:

- [ARCHITECTURE.md](/Users/justinshort/short%20origin/ARCHITECTURE.md)
- [CONTRIBUTING.md](/Users/justinshort/short%20origin/CONTRIBUTING.md)
- [DEVELOPMENT_MODEL.md](/Users/justinshort/short%20origin/DEVELOPMENT_MODEL.md)
- [GitHub governance rollout](/Users/justinshort/short%20origin/docs/process/github-governance-rollout.md)
- [Local wasmCloud development](/Users/justinshort/short%20origin/docs/process/wasmcloud-local-dev.md)
- [SECURITY.md](/Users/justinshort/short%20origin/SECURITY.md)

## Verification

Run from the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

The GitHub CI baseline also includes `cargo audit`.

## Delivery and Release

Origin uses a GitHub-native, trunk-based delivery model:

- `main` is the only long-lived branch.
- `CI / pr-gate`, `Security / security-gate`, and `Governance / validate` are the required checks.
- Merges to `main` publish digest-addressable component descriptors to `ghcr.io/shortorigin/*` and
  auto-promote the `dev` environment.
- Release candidates are cut from a green `main` SHA, deployed to `stage`, and promoted to
  `production` only through the `Promote Release` workflow.

Digest-pinned environment manifests are rendered under
[`infrastructure/wasmcloud/manifests`](/Users/justinshort/short%20origin/infrastructure/wasmcloud/manifests)
and attached to GitHub Releases for rollback and audit.
