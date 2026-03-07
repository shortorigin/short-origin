# WASM Standards Audit

## Current Implementation Summary
- The browser entrypoint lived in `site/src/web_app.rs` and manually parsed `window.location` query/hash values into desktop deep-link state.
- Browser host services were already split by capability at the adapter level, but browser semantics were still surfaced through desktop-shaped names such as explorer/filesystem.
- Browser persistence was mixed: app state in IndexedDB, prefs in `localStorage`, cache in Cache API, and a browser VFS inside the bridge layer.
- The shell shipped multiple competing theme layers, which caused visual drift and unclear design-system ownership.

## Browser Capability Inventory
| Capability | Previous State | Current State | Decision |
| --- | --- | --- | --- |
| Navigation | Manual query/hash parsing in site entrypoint | Typed browser route adapter in `site/src/browser_navigation.rs` | Standards-aligned, refined |
| App state | IndexedDB | IndexedDB | Keep |
| Preferences | `localStorage` | IndexedDB-backed prefs through bridge | Replace |
| Cache | Cache API | Cache API | Keep |
| Notifications | Web Notifications API | Web Notifications API | Keep |
| File-like access | File System Access + IndexedDB VFS | File System Access remains optional, documented as progressive enhancement | Custom but justified |
| Cross-context sync | None | `BroadcastChannel` shell sync | Replace |
| Installability | None | manifest + service worker wiring | Replace |

## Keep / Replace Decisions
- Keep IndexedDB as the structured browser persistence layer.
- Keep Cache API for derived/static browser cache content.
- Keep DOM/CSS rendering as the primary shell UI path.
- Replace browser prefs storage from `localStorage` to IndexedDB-backed storage.
- Replace manual route/deep-link parsing in the site root with a dedicated browser navigation module.
- Replace ad hoc same-tab-only state behavior with standards-based `BroadcastChannel` coordination.
- Keep browser file access capability optional; do not promote the browser VFS as canonical browser semantics.

## Performance Issues
- Boot hydration previously loaded compatibility snapshot, theme, wallpaper, durable snapshot, and wallpaper library in one async fan-out path.
- Window shell code still contains large state reads and monolithic component structure in `desktop_runtime::components`.
- Browser target previously paid extra cost to preserve desktop-preview behavior on small viewports rather than adapting layout structurally.

## Architectural Issues
- `site/src/web_app.rs` previously owned routing, deep-link parsing, and browser bootstrap logic directly.
- `platform_host_web` exposed browser behavior primarily through desktop-oriented adapter names.
- Theme ownership was fragmented across multiple theme packs instead of one authoritative design system.
- Shell layout and visual behavior were split between reusable primitives and large runtime-owned shell components, which made fixes harder to localize.

## Prioritized Refactor Targets
1. Browser route handling and shell boot.
2. IndexedDB-backed prefs and browser persistence normalization.
3. Cross-tab browser synchronization.
4. Single-theme visual baseline and responsive shell stabilization.
5. PWA/installability wiring and explicit browser capability documentation.
