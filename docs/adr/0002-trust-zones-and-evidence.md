# ADR 0002: Trust Zones and Evidence

## Status
Accepted

## Decision
The platform is partitioned into five trust zones: public edge, institutional control plane, runtime plane, data plane, and management plane. Every material decision and side effect produces an evidence manifest and a versioned decision or side-effect record.

## Consequences
- Cloudflare terminates public ingress and enforces edge policy.
- Nomad and wasmCloud workloads run in the runtime plane with explicit bindings.
- SurrealDB and evidence storage remain isolated in the data plane.
- Audit reconstruction reads from envelope-linked events, approvals, risks, and evidence manifests only.
