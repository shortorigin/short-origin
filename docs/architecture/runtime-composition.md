# Runtime Composition and Delivery

This document defines how Origin composes at build time, at runtime, and across environments.

## Composition Layers

- `ui/`: presentation, routing, plugin mounting, and host-adapter composition.
- `shared/`: reusable primitives and governed data-access helpers.
- `platform/`: runtime abstraction layer, SDKs, wasmCloud bindings, and delivery metadata.
- `services/`: backend service components.
- `workflows/`: orchestration components for cross-service flows.
- `schemas/`: contracts, events, WIT packages, and schema definitions.
- `infrastructure/`: manifests and provider configuration for AWS and Cloudflare deployment.

## Runtime Roles

- Presentation: `ui/`
- Platform/runtime mediation: `platform/`
- Orchestration: `workflows/`
- Execution: `services/`
- Contract boundary: `schemas/`
- Deployment and ingress: `infrastructure/`

## Synchronous, Asynchronous, and Hosted Boundaries

- User interaction enters synchronously through the `ui/` shell.
- The shell calls typed platform APIs synchronously or through request/response boundaries.
- Cross-service coordination prefers asynchronous event-driven execution through contracts from
  `schemas/events`.
- Backend execution is cloud-hosted on AWS in wasmCloud/Wasmtime runtime environments.
- Public ingress, routing, DNS, and edge/network mediation are handled by Cloudflare.
- The PWA runs locally in the browser as the baseline host; the Tauri desktop host extends the same
  platform surface with additional local capabilities.

## Runtime Sequence

1. A user interacts with the Leptos/WebAssembly shell.
2. The shell resolves the active plugin module and invokes typed platform interfaces.
3. `platform/` routes requests across host boundaries and published service/workflow contracts.
4. wasmCloud services and workflows execute backend behavior on AWS-hosted runtime infrastructure.
5. Cloudflare mediates public network ingress and routing to AWS-hosted workloads.
6. Environment promotion uses digest-pinned manifests and release artifacts rather than rebuilds.

## Environments and Delivery

- Local development: workspace tooling, browser/PWA shell, optional local wasmCloud lattice.
- `dev`: automatic promotion from a green merge to `main`.
- `stage`: release-candidate deployment of a selected `main` SHA.
- `production`: final promotion of already-published digests without rebuild.

Release flow:

1. Merge to `main` after required checks pass.
2. `Delivery Dev` publishes immutable component descriptors and promotes `dev`.
3. `Release Candidate` rebuilds and verifies a selected `main` SHA, deploys `stage`, and emits
   release artifacts.
4. `Promote Release` retags the already-published digests, renders the production manifest, and
   deploys `production`.

Rollback expectation:

- redeploy a prior digest-pinned manifest and OCI references;
- do not rebuild a new artifact as the first rollback action.

The current production manifest artifact path remains `infrastructure/wasmcloud/manifests/prod/`.
Contributor-facing documentation refers to the same environment as `production`.
