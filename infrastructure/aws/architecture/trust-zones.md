# AWS Trust Zones

- `public-edge`: no AWS ingress; Cloudflare terminates public traffic.
- `control-plane`: policy, approval, audit, evidence, and CI/CD services.
- `runtime-plane`: Nomad clients and wasmCloud hosts.
- `data-plane`: SurrealDB, backups, and evidence storage.
- `management-plane`: Pulumi backend, observability, and break-glass recovery.
