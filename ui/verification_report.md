# UI Shell Verification Report

## Commands Executed

| Command | Result |
| --- | --- |
| `cargo check -p desktop_runtime` | PASS |
| `cargo check -p site` | PASS |
| `cargo check -p desktop_tauri` | PASS |
| `cargo test -p desktop_runtime` | PASS |
| `cargo test -p platform_host_web` | PASS |
| `cargo test -p site` | PASS |
| `cargo ui-build` | PASS |
| `cargo verify-ui` | PASS |
| `cargo xtask ui-hardening` | PASS |

## Command Transcript Summary

### `cargo check -p desktop_runtime`
- Result: pass
- Notable output:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 1.22s`

### `cargo check -p site`
- Result: pass
- Notable output:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 0.69s`

### `cargo check -p desktop_tauri`
- Result: pass
- Notable output:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 31.14s`

### `cargo test -p desktop_runtime`
- Result: pass
- Coverage relevant to this remediation:
  - boot planner precedence and migration
  - orphaned modal restore rejection
  - modal reconstruction
  - stale sync snapshot rejection
  - deterministic boot deep-link augmentation

### `cargo test -p platform_host_web`
- Result: pass
- Coverage relevant to this remediation:
  - self-originated sync ignore
  - stale sync ignore
  - newer sync acceptance
  - typed event round-trip

### `cargo test -p site`
- Result: pass
- Coverage relevant to this remediation:
  - browser route parsing
  - compatibility deep-link routing
  - browser asset-path handling

### `cargo ui-build`
- Result: pass
- Notable output:
  - `Running 'target/debug/xtask ui build'`
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s) in 7.56s`

### `cargo verify-ui`
- Result: pass
- Notable output:
  - `Running 'target/debug/xtask verify profile ui'`
  - package checks, tests, preview smoke path, and browser build completed without failures

### `cargo xtask ui-hardening`
- Result: pass
- Generated report:
  - `build/wasm-hardening/remediation-report.md`
- Final status:
  - `pipeline status: HARDENED`
  - `byte-identical result: PASS`
  - browser matrix `PASS`

## Artifact Inventory

### Canonical Hardened Browser Artifact Set
- Path: `build/wasm-hardening/build-b`
- Files:
  - `icon.png`
  - `index.html`
  - `manifest.webmanifest`
  - `site_app-4a493b95f643e7b.js`
  - `site_app-4a493b95f643e7b_bg.wasm`
  - `snippets/platform_host_web-a3dd707719cd2672/inline0.js`
  - `sw.js`
  - `wallpapers/aurora-flow-poster.svg`
  - `wallpapers/aurora-flow.svg`
  - `wallpapers/catalog.toml`
  - `wallpapers/city-lights.svg`
  - `wallpapers/cloud-bands.svg`
  - `wallpapers/green-hills.svg`
  - `wallpapers/paper-grid.svg`
  - `wallpapers/rain-window-poster.svg`
  - `wallpapers/rain-window.svg`
  - `wallpapers/sunset-lake.svg`
  - `wallpapers/teal-grid.svg`
  - `wallpapers/teal-solid.svg`
  - `wallpapers/woven-steel.svg`

### Integrity Attributes
- Verified from the hardening report:
  - `/site_app-4a493b95f643e7b.js`
  - `/snippets/platform_host_web-a3dd707719cd2672/inline0.js`
  - `/site_app-4a493b95f643e7b_bg.wasm`
- Result: all emitted integrity attributes matched independently computed digests.

## Determinism Notes

- `cargo xtask ui-hardening` performed two clean release browser builds and compared the full deployable artifact graph.
- Clean build A: `build/wasm-hardening/build-a`
- Clean build B: `build/wasm-hardening/build-b`
- Result: byte-identical artifacts and identical generated HTML.
- Output hash examples from the hardening report:
  - `index.html`: `c996e370c4dfd59c7db3d4a8ba858040cddbfa4bb1bcdcae82b23aae5f088d5c`
  - `site_app-4a493b95f643e7b.js`: `d953969d7c00cd3fb1c38778b436b60f02cbc1aed5e67aef5f6b31dabd10887f`
  - `site_app-4a493b95f643e7b_bg.wasm`: `cdfddf6e3b1c59f820f251eb59cc8a8006ccecf2b1e4078803ab517f7fc9a15d`

## Browser Validation

### Automated
- Source: `cargo xtask ui-hardening`
- Node: `v25.8.0`
- Results:
  - Chromium: pass, no console errors, no request failures, no integrity failures, no WASM/module failures
  - Firefox: pass, no console errors, no request failures, no integrity failures, no WASM/module failures
  - WebKit: pass, no console errors, no request failures, no integrity failures, no WASM/module failures

### Native Safari
- Status: not separately instrumented in this environment
- Note:
  WebKit automation passed, which is a strong compatibility signal, but native Safari itself was not independently driven or observed through a dedicated automation surface in this run.

## Defect Coverage Confirmation

- Boot restore occurs exactly once per session:
  verified by atomic boot hydration design and reducer tests.
- Durable precedence and legacy migration:
  verified by boot planner tests and durable revision loading.
- Deep-link ordering:
  verified by `CompleteBootHydration` reducer tests.
- Same-context sync loop prevention:
  verified by typed sender-aware `ShellSyncEvent` tests.
- Stale sync rejection:
  verified by reducer stale snapshot test and cross-context stale event test.
- Restore invariants:
  verified by snapshot normalization tests for modal and focus reconstruction.

## Residual Risk Notes

- Theme and wallpaper sync revisions are monotonic runtime-generated values rather than durable envelope timestamps because those values remain stored in typed prefs, not separate app-state envelopes.
- Native Safari itself was not separately automated. WebKit passed, but a manual Safari confirmation remains advisable before a release that explicitly targets Safari.
