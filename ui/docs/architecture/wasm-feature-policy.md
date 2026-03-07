# WASM Feature Policy

## Required Foundation
- Stable Rust-to-WASM build.
- Standard DOM/CSS rendering.
- IndexedDB for structured browser persistence.

## Optional Features
- `BroadcastChannel`
- Service Worker
- Notifications API
- File System Access API

## Deferred Features
- OPFS as a required architectural dependency
- Navigation API
- URLPattern
- OffscreenCanvas
- WebGPU

## Fallback Rule
- Optional browser features must fail open to a functional shell, not a broken boot path.
