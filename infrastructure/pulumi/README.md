# Pulumi Infrastructure Workspace

This workspace provides a modular Pulumi TypeScript codebase for provisioning AWS infrastructure and Cloudflare edge networking with explicit AWS↔Cloudflare wiring.

## Layout

- `bootstrap-state/`: one-time Pulumi stack that provisions S3 state storage and DynamoDB locking.
- `live/`: environment stacks (`dev`, `stage`, `prod`) for AWS + Cloudflare infrastructure.
- `scripts/`: operator-friendly wrappers for preview and apply flows.

## Design Goals

- Reproducible deployments with environment isolation.
- Stage-gated promotion between continuous delivery (`dev`) and manual production approval.
- Strict module boundaries between AWS resources, Cloudflare resources, and cross-cloud wiring.
- Private AWS origin exposure through Cloudflare Tunnel.
- Consistent defaults aligned to:
  - Region: `us-west-2`
  - Environments: `dev`, `stage`, `prod`
  - Instance strategy: `dev=t4g.small`, `stage=m7gd.medium`, `prod=m7gd.medium`

## Quick Start

1. Bootstrap state backend:
```bash
./scripts/bootstrap-state.sh
```

2. Configure secrets for live stacks:
```bash
cd live
pulumi stack select dev
pulumi config set --secret origin:tunnelSecret "<BASE64_32_BYTE_SECRET>"
pulumi config set --secret origin:surrealdbRootPassword "<PASSWORD>"
pulumi stack select stage
pulumi config set --secret origin:tunnelSecret "<BASE64_32_BYTE_SECRET>"
pulumi config set --secret origin:surrealdbRootPassword "<PASSWORD>"
```

3. Preview and deploy:
```bash
./scripts/preview.sh dev
./scripts/preview.sh stage
./scripts/deploy.sh dev
```

## Runtime Client Contract

Pulumi-configured hosts install and start SurrealDB on `127.0.0.1:8000` with root credentials
from `origin:surrealdbRootPassword`. Runtime consumers in the repository connect through the
governed shared storage layer and should receive these environment variables:

```bash
export ORIGIN_SURREALDB_ENDPOINT="ws://127.0.0.1:8000"
export ORIGIN_SURREALDB_USERNAME="root"
export ORIGIN_SURREALDB_PASSWORD="<origin:surrealdbRootPassword>"
export ORIGIN_SURREALDB_NAMESPACE="short_origin"
export ORIGIN_SURREALDB_DATABASE="institutional"
```

Do not add direct SurrealDB client usage outside `shared/surrealdb-access`; services and workflows
should consume the governed connection helpers exposed by `shared/governed-storage`.

## Required Environment Variables

- `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION` (or OIDC in CI)
- `CLOUDFLARE_API_TOKEN`

## CI/CD

GitHub Actions workflows:
- `.github/workflows/ci.yml`
- `.github/workflows/delivery-dev.yml`
- `.github/workflows/release-candidate.yml`
- `.github/workflows/promote-release.yml`

These workflows run stack-specific previews/applies against the S3 Pulumi backend and promote
digest-pinned manifests across `dev`, `stage`, and `production`.
