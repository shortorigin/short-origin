# Local wasmCloud Development

This repository uses `wash` as the authoritative local entrypoint for wasmCloud development. The
local workflow is expected to support four activities:

1. Verify workstation prerequisites.
2. Start a local lattice named `institutional-lattice`.
3. Build and validate the workspace's wasmCloud-facing crates.
4. Render and inspect a development lattice manifest.

## Prerequisites

Confirm the following tools are available on the workstation:

- `rustc`
- `cargo`
- `rustup`
- `wash`
- `docker`

Confirm at least one Rust WebAssembly target is installed:

- `wasm32-wasip1`
- `wasm32-wasip2`
- `wasm32-unknown-unknown`

The repository provides a single readiness command:

```bash
cargo xtask wasmcloud doctor
```

That command verifies the toolchain, checks for an installed wasm target, runs
`cargo xtask components build`, and executes `cargo test -p wasmcloud-smoke-tests --all-targets`.

## Start and Stop the Local Lattice

Start the local lattice:

```bash
cargo xtask wasmcloud up
```

This uses `wash up` and defaults the lattice name to `institutional-lattice`.

Once the lattice has started successfully at least once on the workstation, detached startup is also
available:

```bash
cargo xtask wasmcloud up --detached
```

Inspect the local host inventory:

```bash
cargo xtask wasmcloud status --lattice institutional-lattice
```

Tear the environment down when finished:

```bash
cargo xtask wasmcloud down
```

If `wash down` reports that it could not contact NATS but a `wasmcloud_host` process is still
running, terminate the orphaned host process and rerun the command.

## Build and Validate Workspace Components

Run the workspace's wasmCloud-oriented compile checks:

```bash
cargo xtask components build
```

Run the wasmCloud smoke tests:

```bash
cargo test -p wasmcloud-smoke-tests --all-targets
```

## Wire Local SurrealDB Access

If SurrealDB is installed on the workstation, start it as the local durable store on loopback:

```bash
surreal start \
  --log info \
  --bind 127.0.0.1:8000 \
  --user root \
  --pass "<PASSWORD>" \
  surrealkv://$HOME/.local/share/origin/surrealdb.db
```

Export the runtime variables consumed by `shared/governed-storage::connect_from_env()`:

```bash
export ORIGIN_SURREALDB_ENDPOINT="ws://127.0.0.1:8000"
export ORIGIN_SURREALDB_USERNAME="root"
export ORIGIN_SURREALDB_PASSWORD="<PASSWORD>"
export ORIGIN_SURREALDB_NAMESPACE="short_origin"
export ORIGIN_SURREALDB_DATABASE="institutional"
```

Validate the governed storage path before starting other runtime components:

```bash
cargo test -p surrealdb-access -p governed-storage
```

The in-memory storage helper remains available for isolated tests, but local runtime verification
should prefer the durable host-installed SurrealDB path above.

## Render a Development Lattice Manifest

Render a development manifest with explicit component references and digests:

```bash
cargo xtask wasmcloud manifest \
  --environment dev \
  --finance-ref ghcr.io/shortorigin/finance-service:dev \
  --finance-digest sha256:test \
  --treasury-ref ghcr.io/shortorigin/treasury-disbursement:dev \
  --treasury-digest sha256:test \
  --output /tmp/origin-lattice-config.json
```

The manifest is rendered through the existing delivery path and should describe the
`institutional-lattice` rollout for the selected environment.

## Successful Local Verification

A local wasmCloud development environment is considered healthy when all of the following hold:

- `cargo xtask wasmcloud doctor` passes.
- `cargo xtask wasmcloud up` starts a local lattice successfully.
- `cargo xtask wasmcloud status --lattice institutional-lattice` reports at least one host.
- `cargo xtask wasmcloud manifest ...` renders a valid lattice manifest.
