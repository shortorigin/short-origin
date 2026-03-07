# Browser Capability Matrix

| Capability | Detection | Status | Fallback |
| --- | --- | --- | --- |
| IndexedDB | `platform_host_web::persistence::indexed_db_supported()` | Adopted | Shell can still boot with degraded persistence if browser blocks storage |
| BroadcastChannel | `platform_host_web::broadcast_channel_supported()` | Adopted | Local-tab only behavior |
| Service Worker | `platform_host_web::pwa::service_worker_supported()` | Adopted | No offline/install enhancement |
| File System Access | `platform_host_web::file_access::directory_picker_supported()` | Optional | Origin-scoped storage only |
| Notifications | Host capability snapshot + browser permission | Optional | No notification delivery |
| OPFS | `platform_host_web::persistence::opfs_supported()` | Deferred | IndexedDB + Cache API remain authoritative |
