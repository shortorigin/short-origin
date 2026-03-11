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

/// Canonical host-layer result type used by runtime and adapter boundaries.
pub type HostResult<T> = Result<T, String>;

pub mod cache;
pub mod external_url;
pub mod fs;
pub mod host;
pub mod notifications;
pub mod session;
pub mod storage;
pub mod terminal_process;
pub mod time;

pub use cache::{
    ContentCache, ContentCacheFuture, MemoryContentCache, NoopContentCache, cache_get_json_with,
    cache_put_json_with,
};
pub use external_url::{ExternalUrlFuture, ExternalUrlService, NoopExternalUrlService};
pub use fs::path::normalize_virtual_path;
pub use fs::service::{ExplorerFsFuture, ExplorerFsService, NoopExplorerFsService};
pub use fs::types::{
    EXPLORER_CACHE_NAME, EXPLORER_PREFS_KEY, ExplorerBackend, ExplorerBackendStatus, ExplorerEntry,
    ExplorerEntryKind, ExplorerFileReadResult, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState, ExplorerPrefs, explorer_preview_cache_key,
};
pub use host::{CapabilityError, CapabilityStatus, HostCapabilities, HostServices, HostStrategy};
pub use notifications::{NoopNotificationService, NotificationFuture, NotificationService};
pub use session::{MemorySessionStore, session_store};
pub use storage::app_state::{
    APP_STATE_ENVELOPE_VERSION, AppStateEnvelope, AppStateSchemaPolicy, AppStateStore,
    AppStateStoreFuture, CALCULATOR_STATE_NAMESPACE, DESKTOP_STATE_NAMESPACE,
    EXPLORER_STATE_NAMESPACE, MemoryAppStateStore, NOTEPAD_STATE_NAMESPACE, NoopAppStateStore,
    PAINT_STATE_NAMESPACE, TERMINAL_STATE_NAMESPACE, build_app_state_envelope,
    load_app_state_typed_with, load_app_state_with_migration, migrate_envelope_payload,
    save_app_state_with,
};
pub use storage::prefs::{
    MemoryPrefsStore, NoopPrefsStore, PrefsStore, PrefsStoreFuture, load_pref_with, save_pref_with,
};
pub use terminal_process::{
    NoopTerminalProcessService, TerminalEvent, TerminalProcessFuture, TerminalProcessService,
    TerminalResizeRequest, TerminalSessionId, TerminalWriteRequest,
};
pub use time::{next_monotonic_timestamp_ms, unix_time_ms_now};
