# WASM Target Architecture

## Overview
- Rust + WebAssembly remains the application core.
- Browser routing, persistence, installability, and cross-context coordination now use standard browser APIs first.
- Desktop/Tauri remains the richer host for platform-native capabilities.

## Capability Mapping
| Domain | Adopted API | Policy |
| --- | --- | --- |
| Navigation | `window.location`, router paths, typed route adapter | Required |
| Structured persistence | IndexedDB | Required |
| Lightweight prefs | IndexedDB | Required |
| Cache | Cache API | Required when cache is used |
| Cross-context sync | `BroadcastChannel` | Optional with no-op fallback |
| Notifications | Notifications API | Optional with capability gating |
| File access | File System Access API | Optional progressive enhancement |
| Installability | Web App Manifest + Service Worker | Optional enhancement for browser builds |

## Desktop vs Browser Split
- Browser: routing, web installability, origin-scoped persistence, optional file access, standard URL opening.
- Desktop: Tauri transport, richer native file access, platform-owned notifications, native shell affordances.

## Progressive Enhancement Policy
- `BroadcastChannel`, notifications, file access, and service worker support are capability-detected.
- No advanced feature is required to boot the shell.
- OPFS and WebGPU remain deferred.

## Fallback Policy
- No `BroadcastChannel`: local-tab behavior only.
- No service worker: browser shell still runs online.
- No notifications permission/support: notification requests remain best-effort.
- No File System Access: browser shell retains origin-scoped storage behavior only.

## API Adoption Decisions
- Adopted: typed route adapter, IndexedDB prefs, `BroadcastChannel`, manifest/service worker wiring.
- Deferred: OPFS, Navigation API, URLPattern, OffscreenCanvas, WebGPU.

## Non-Goals
- No JS-first rewrite.
- No mandatory worker architecture.
- No speculative browser filesystem layer beyond optional file access support.
