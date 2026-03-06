//! Typed host-domain contracts and shared models used across runtime and browser adapters.
//!
//! This crate is the API-first boundary for platform services. It exposes shared
//! persistence/explorer models, time/session helpers, and app-state service traits while concrete
//! browser adapters live in `platform_host_web` and desktop transport remains behind
//! `desktop_tauri`.
//!
//! The contracts here are intentionally implementation-agnostic: runtime and app crates depend on
//! these traits and models, while browser and desktop layers choose the concrete adapters and
//! compatibility behavior.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

pub mod cache;
pub mod external_url;
pub mod fs;
pub mod host;
pub mod notifications;
pub mod session;
pub mod storage;
pub mod terminal_process;
pub mod time;
pub mod wallpaper;

pub use cache::{
    cache_get_json_with, cache_put_json_with, ContentCache, ContentCacheFuture, MemoryContentCache,
    NoopContentCache,
};
pub use external_url::{ExternalUrlFuture, ExternalUrlService, NoopExternalUrlService};
pub use fs::path::normalize_virtual_path;
pub use fs::service::{ExplorerFsFuture, ExplorerFsService, NoopExplorerFsService};
pub use fs::types::{
    explorer_preview_cache_key, ExplorerBackend, ExplorerBackendStatus, ExplorerEntry,
    ExplorerEntryKind, ExplorerFileReadResult, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState, ExplorerPrefs, EXPLORER_CACHE_NAME,
    EXPLORER_PREFS_KEY,
};
pub use host::{CapabilityError, CapabilityStatus, HostCapabilities, HostServices, HostStrategy};
pub use notifications::{NoopNotificationService, NotificationFuture, NotificationService};
pub use session::{session_store, MemorySessionStore};
pub use storage::app_state::{
    build_app_state_envelope, load_app_state_typed_with, load_app_state_with_migration,
    migrate_envelope_payload, save_app_state_with, AppStateEnvelope, AppStateSchemaPolicy,
    AppStateStore, AppStateStoreFuture, MemoryAppStateStore, NoopAppStateStore,
    APP_STATE_ENVELOPE_VERSION, CALCULATOR_STATE_NAMESPACE, DESKTOP_STATE_NAMESPACE,
    EXPLORER_STATE_NAMESPACE, NOTEPAD_STATE_NAMESPACE, PAINT_STATE_NAMESPACE,
    TERMINAL_STATE_NAMESPACE,
};
pub use storage::prefs::{
    load_pref_with, save_pref_with, MemoryPrefsStore, NoopPrefsStore, PrefsStore, PrefsStoreFuture,
};
pub use terminal_process::{
    NoopTerminalProcessService, TerminalEvent, TerminalProcessFuture, TerminalProcessService,
    TerminalResizeRequest, TerminalSessionId, TerminalWriteRequest,
};
pub use time::{next_monotonic_timestamp_ms, unix_time_ms_now};
pub use wallpaper::{
    NoopWallpaperAssetService, ResolvedWallpaperSource, WallpaperAnimationPolicy,
    WallpaperAssetDeleteResult, WallpaperAssetFuture, WallpaperAssetMetadataPatch,
    WallpaperAssetRecord, WallpaperAssetService, WallpaperCollection,
    WallpaperCollectionDeleteResult, WallpaperConfig, WallpaperDisplayMode, WallpaperImportRequest,
    WallpaperImportResult, WallpaperLibrarySnapshot, WallpaperMediaKind, WallpaperPosition,
    WallpaperSelection, WallpaperSourceKind,
};
