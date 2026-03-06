# wasmCloud Deployment Manifests

This directory is reserved for digest-pinned `lattice-config.json` files rendered by the GitHub
delivery workflows for each promoted environment:

- `dev/lattice-config.json`
- `stage/lattice-config.json`
- `prod/lattice-config.json`

The checked-in tree keeps the environment directories stable. Delivery workflows render concrete
manifests with `cargo xtask delivery render-manifest ...`, upload them as workflow artifacts, and
attach release-candidate or production manifests to GitHub Releases for rollback and audit.
