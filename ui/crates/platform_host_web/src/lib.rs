//! Browser (`wasm32`) implementations of [`platform_host`] service contracts.
//!
//! This crate is the concrete browser-side host wiring layer for app-state, cache, prefs,
//! explorer/filesystem, notifications, and external URL opening services.
//!
//! Bridge bindings are split by domain under `bridge/`:
//! - `bridge::app_state`
//! - `bridge::cache`
//! - `bridge::fs`
//! - `bridge::prefs`
//! - `bridge::interop` (shared wasm/non-wasm transport glue)
//!
//! Use the adapter factories re-exported from [`adapters`] when wiring the runtime. They hide the
//! concrete browser-vs-desktop-webview transport choice behind the typed [`platform_host`] traits
//! and capability snapshot.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

/// Compile-time host-strategy selection and concrete adapter factories for runtime wiring.
pub mod adapters;
mod bridge;
pub mod cache;
pub mod cross_context;
pub mod external_url;
pub mod file_access;
pub mod fs;
pub mod navigation;
pub mod notifications;
pub mod persistence;
pub mod pwa;
pub mod storage;

pub use adapters::{
    AppStateStoreAdapter, ContentCacheAdapter, ExplorerFsServiceAdapter, ExternalUrlServiceAdapter,
    NotificationServiceAdapter, PrefsStoreAdapter, app_state_store, build_host_services,
    content_cache, explorer_fs_service, external_url_service, host_capabilities,
    host_strategy_name, notification_service, prefs_store, selected_host_strategy,
};
pub use cache::cache_api::WebContentCache;
pub use cache::tauri_cache_api::TauriContentCache;
pub use cross_context::{
    ShellSyncEvent, ShellSyncKind, broadcast_channel_supported, decode_shell_sync_event,
    publish_shell_sync_event, shell_sync_sender_id, should_apply_shell_sync_event,
};
pub use external_url::{TauriExternalUrlService, WebExternalUrlService};
pub use fs::explorer::{TauriExplorerFsService, WebExplorerFsService};
pub use notifications::{TauriNotificationService, WebNotificationService};
pub use storage::indexed_db::WebAppStateStore;
pub use storage::local_prefs::WebPrefsStore;
pub use storage::tauri_app_state::TauriAppStateStore;
pub use storage::tauri_prefs::TauriPrefsStore;
