# Origin

Origin is a cloud-native, WebAssembly-first enterprise application platform. It is a Rust-first,
contract-driven substrate for governed plugin applications that compose a shared Leptos/WebAssembly
shell, typed platform APIs, wasmCloud service workloads, and workflow-gated mutations.

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

## Platform Model

- Origin is a modular platform, not a single monolithic application.
- Platform concerns stay separated by plane:
  - `ui/` owns the Leptos/WebAssembly shell and host-facing presentation adapters.
  - `platform/` owns runtime, SDK, and execution abstractions shared by UI and services.
  - `services/` and `workflows/` own backend execution on wasmCloud/Wasmtime through published
    contracts from `schemas/`.
  - `infrastructure/` owns deployment and environment automation only.
- Plugin applications integrate through governed manifests and platform contracts rather than ad
  hoc imports into core shell code.

Authoritative supporting architecture documents:

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [First-Principles Systems Architecture Baseline](docs/architecture/first-principles-systems-architecture-report.md)
- [First-Principles Implementation Roadmap](docs/architecture/first-principles-implementation-roadmap.md)
- [ADR Catalog](docs/adr/README.md)
- [Layer Boundaries](docs/architecture/layer-boundaries.md)
- [Plugin Application Model](docs/architecture/plugin-application-model.md)
- [Runtime Composition and Delivery](docs/architecture/runtime-composition.md)

## Development

Origin uses a GitHub-native Scrumban model.

1. Start with a same-repository GitHub issue that defines context, scope, and acceptance criteria.
2. Work on a short-lived issue branch named `<type>/<issue-id>-description` using an approved prefix from [`DEVELOPMENT_MODEL.md`](DEVELOPMENT_MODEL.md).
3. Record ADR references, impacted domains, consistency class, risk tier, rollback path, and validation artifacts in the issue and PR traceability fields.
4. Open a pull request with a conventional title and a PR body that includes `Closes #<issue-id>`.
5. Merge to `main` only after review and required checks pass so the linked issue closes automatically.

Primary contributor docs:

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [DEVELOPMENT_MODEL.md](DEVELOPMENT_MODEL.md)
- [Layer Boundaries](docs/architecture/layer-boundaries.md)
- [Plugin Application Model](docs/architecture/plugin-application-model.md)
- [Runtime Composition and Delivery](docs/architecture/runtime-composition.md)
- [GitHub governance rollout](docs/process/github-governance-rollout.md)
- [GitHub workflow migration note](docs/process/github-workflow-migration.md)
- [Platform regression guardrails](docs/process/platform-regression-guardrails.md)
- [Local wasmCloud development](docs/process/wasmcloud-local-dev.md)
- [SECURITY.md](SECURITY.md)

## Verification

Run from the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

The GitHub CI baseline also includes `cargo audit`.

For local enforcement parity, also run:

```bash
cargo xtask architecture audit-boundaries
cargo xtask plugin validate-manifests
cargo xtask github audit-process
```

## Delivery and Release

Origin uses a GitHub-native, trunk-based delivery model:

- `main` is the only long-lived branch.
- `CI / pr-gate`, `Security / security-gate`, and `Governance / validate` are the required checks.
- The browser/PWA runtime is the baseline platform surface; the Tauri host extends the same surface
  with desktop-only capabilities instead of forking the product model.
- The `Delivery Dev` workflow runs automatically on pushes to `main`, publishes
  digest-addressable component descriptors to `ghcr.io/shortorigin/*`, and promotes `dev`.
- Release candidates are cut from a green `main` SHA, deployed to `stage`, and promoted to
  `production` only through the `Promote Release` workflow.
- Public traffic is mediated by Cloudflare DNS, routing, and edge/network policy. AWS hosts the
  wasmCloud/Wasmtime execution environment and supporting control-plane, runtime-plane, and
  data-plane workloads.

Digest-pinned environment manifests are rendered under
[`infrastructure/wasmcloud/manifests`](infrastructure/wasmcloud/manifests)
and attached to GitHub Releases for rollback and audit. The checked-in manifest path currently uses
`prod/` for the production environment artifact; contributor-facing docs refer to the same target as
`production`.
