# Infrastructure

## Purpose
`infrastructure/` defines deployment, runtime, and platform-environment automation for AWS,
Cloudflare, Nomad, and Pulumi targets. It standardizes how AWS-hosted wasmCloud workloads,
supporting lattice services, and Cloudflare-mediated ingress/network policy are provisioned.

## Scope
In scope:
- Environment definitions under `infrastructure/aws`, `infrastructure/cloudflare`, `infrastructure/nomad`, and `infrastructure/pulumi`.
- Provisioning workflows and runtime wiring.
- Environment-level secrets and policy integration patterns.
- Digest-pinned deployment manifests consumed by the delivery and release workflows.

Out of scope:
- Business-domain logic.
- Application schema definitions.
- UI code.

## Interfaces
- Deployment interface from CI/CD pipelines into target environments.
- Runtime interface for wasmCloud hosts, lattice control-plane services, and Wasmtime-compatible execution settings.
- Configuration interface for service discovery, secrets, and policy enforcement.

## Dependencies
- Depends on `platform/` runtime contracts to configure host and execution assumptions.
- Depends on `services/` deployment artifacts and version metadata.
- Depends on `workflows/` for operational runbook expectations.

## Development Workflow
1. Define environment changes as code in provider-specific subdirectories.
2. Validate plans before apply in non-production targets.
3. Ensure lattice/runtime configuration remains contract-compatible with `platform/`.
4. Promote changes through staged environments with rollback instructions.
5. Start with a GitHub issue, link it from the PR, and capture policy-leakage review notes in the issue or PR when infrastructure posture changes.

## Build/Test Commands
Run from repo root:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```
Provider/tool-specific validations should be added in each infrastructure submodule (for example `pulumi preview`, `nomad job validate`).

## Integration Patterns
- Infrastructure publishes environment configuration consumed by wasmCloud hosts and supporting lattice services.
- Runtime settings are versioned and tied to platform interface compatibility.
- Observability standards must emit telemetry consumable by agents and operations workflows.
- Nomad job definitions should schedule lattice infrastructure, not raw service binaries.

## Reuse Opportunities
- Consolidate reusable infrastructure modules for networking, secrets, and policy controls.
- Reuse deployment templates across environments with provider-specific adapters.

## Out of Scope
- Domain ontology changes.
- Service business rule implementation.
- Contract schema authorship.
