# WASM Refactor Plan

## Per-Crate Refactor Plan
- `site`: add browser navigation module, service worker registration, browser sync listener, manifest/service-worker assets, and modern-only theme bundle.
- `platform_host_web`: add browser capability modules, move prefs storage to IndexedDB, add `BroadcastChannel` publication helpers.
- `desktop_runtime`: export durable boot snapshot loading, stage wallpaper-library hydration after critical boot hydration, and default new themes to modern adaptive.

## Interface Changes
- `desktop_runtime` re-exports `load_durable_boot_snapshot`.
- `platform_host_web` exports `cross_context`, `navigation`, `file_access`, `persistence`, and `pwa` helpers.
- Browser prefs semantics change from `localStorage` to IndexedDB-backed persistence.

## Migration Sequencing
1. Normalize browser persistence and browser capability helpers.
2. Replace site route parsing and add PWA/runtime enhancements.
3. Move shell defaults and responsive CSS to the modern baseline.
4. Add docs and verification artifacts.

## Risk Notes
- Browser sync currently targets theme, wallpaper, and layout hydration only.
- Browser file-access semantics remain optional; the VFS path still exists for compatibility.
- Legacy skin enums remain in the runtime for compatibility, but the browser shell ships one authoritative theme bundle.

## Compatibility Notes
- Existing `?open=` deep links still work.
- Canonical `/notes/:slug` and `/projects/:slug` routes remain browser-first.
- Desktop/Tauri behavior is preserved.

## Verification Strategy
- Check `site`, `desktop_runtime`, and `platform_host_web` compilation together.
- Run focused tests for route parsing and existing host bridge behavior.

## UI Coherence and Design-System Refactor
- Removed legacy and neumorphic theme CSS from the browser bundle.
- Updated foundational tokens to a single modern-adaptive palette and type direction.
- Added window body overflow guards, viewport-constrained window sizing, responsive window stacking, and taskbar overflow handling.
