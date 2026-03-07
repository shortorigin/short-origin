# PWA Runtime Notes

- Added `manifest.webmanifest` and linked it from the browser entry HTML.
- Added `sw.js` and service worker registration from `site/src/pwa.rs`.
- Offline strategy is intentionally conservative: cache shell root and manifest only.
- No background sync, push, or mutation-heavy offline workflows were introduced.
