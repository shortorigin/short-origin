//! Shared contract types between the desktop window manager runtime and managed apps.
//!
//! v2 introduces a capability-scoped service injection model (`AppServices`) and
//! canonical string application identifiers (`ApplicationId`) while keeping stable
//! lifecycle semantics for runtime-managed windows.
//!
//! Host-owned wallpaper models now live in [`platform_host`]. This crate exposes the app-facing
//! service surface that consumes those models without redefining the wallpaper domain itself.
//!
//! Runtime composition code constructs [`AppMountContext`] values per window instance and injects
//! [`AppServices`] so application crates can persist state, query capabilities, use explorer/cache
//! helpers, and register structured shell commands without importing environment-specific host
//! implementations directly.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use std::{cell::Cell, rc::Rc};

use futures::future::LocalBoxFuture;
use leptos::{Callable, Callback, ReadSignal, RwSignal, View};
use platform_host::{
    load_app_state_with_migration, load_pref_with, save_app_state_with, save_pref_with,
    AppStateEnvelope, AppStateStore, CapabilityStatus, ContentCache, ExplorerBackendStatus,
    ExplorerFileReadResult, ExplorerFsService, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState, HostCapabilities, PrefsStore, WallpaperConfig,
    WallpaperImportRequest, WallpaperLibrarySnapshot,
};
use sdk_rs::UiDashboardSnapshotV1;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_shell_contract::{
    CommandDescriptor, CommandNotice, CommandNoticeLevel, CommandResult, CompletionItem,
    CompletionRequest, DisplayPreference, ExecutionId, ParsedInvocation, ShellError, ShellRequest,
    ShellStreamEvent, StructuredData,
};

/// Stable identifier for a runtime-managed window.
pub type WindowRuntimeId = u64;

/// Returns the canonical DOM id for a window's primary keyboard input.
///
/// Runtime-managed apps can render this id on their preferred text field so the desktop host can
/// restore keyboard focus when a window opens or becomes focused again.
pub fn window_primary_input_dom_id(window_id: WindowRuntimeId) -> String {
    format!("window-primary-input-{window_id}")
}

/// Stable identifier for an app package/module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationId(String);

impl ApplicationId {
    /// Returns an app identifier when `raw` conforms to the `segment.segment...` policy.
    ///
    /// Each segment must start with an ASCII lowercase letter, contain only ASCII lowercase
    /// letters, digits, or `-`, and be non-empty. The full identifier must contain at least two
    /// segments and remain within the runtime length limits used by manifests, deep links, and IPC
    /// topic prefixes.
    ///
    /// # Errors
    ///
    /// Returns a human-readable validation message when the identifier violates the runtime naming
    /// contract.
    pub fn new(raw: impl Into<String>) -> Result<Self, String> {
        let raw = raw.into();
        if is_valid_application_id(&raw) {
            Ok(Self(raw))
        } else {
            Err(format!(
                "invalid application id `{raw}`; expected namespaced dotted segments"
            ))
        }
    }

    /// Returns the string form of the identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Creates an id without validation for compile-time/runtime trusted constants.
    pub fn trusted(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }
}

impl std::fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn is_valid_application_id(raw: &str) -> bool {
    if raw.is_empty() || raw.len() > 120 {
        return false;
    }

    let mut count = 0usize;
    for part in raw.split('.') {
        count += 1;
        if part.is_empty() || part.len() > 32 {
            return false;
        }
        let bytes = part.as_bytes();
        if !bytes[0].is_ascii_lowercase() {
            return false;
        }
        if !bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        {
            return false;
        }
        if part.ends_with('-') {
            return false;
        }
    }

    count >= 2
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Declared app capability scopes enforced by runtime policy.
pub enum AppCapability {
    /// Window title/focus actions.
    Window,
    /// Window-scoped and app-shared state persistence APIs.
    State,
    /// Config key/value access.
    Config,
    /// Theme/accessibility shell controls.
    Theme,
    /// Wallpaper selection, preview, and library-management controls.
    Wallpaper,
    /// Host notification APIs.
    Notifications,
    /// Inter-application pub/sub and request/reply channels.
    Ipc,
    /// Requests for opening external URLs.
    ExternalUrl,
    /// Dynamic system terminal command registration.
    Commands,
}

/// Runtime-granted app capabilities paired with host availability for optional domains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySet {
    granted: Vec<AppCapability>,
    host: HostCapabilities,
}

impl CapabilitySet {
    /// Creates a capability set from manifest/policy grants and a host availability snapshot.
    pub fn new(granted: impl Into<Vec<AppCapability>>, host: HostCapabilities) -> Self {
        Self {
            granted: granted.into(),
            host,
        }
    }

    /// Returns all runtime-granted app capabilities.
    pub fn granted(&self) -> &[AppCapability] {
        &self.granted
    }

    /// Returns the host capability snapshot.
    pub fn host(&self) -> HostCapabilities {
        self.host
    }

    /// Returns whether the runtime granted `capability` to the mounted app.
    pub fn is_granted(&self, capability: AppCapability) -> bool {
        self.granted.contains(&capability)
    }

    /// Returns host availability for a capability after runtime grant evaluation.
    pub fn status(&self, capability: AppCapability) -> CapabilityStatus {
        if !self.is_granted(capability) {
            return CapabilityStatus::Unavailable;
        }

        match capability {
            AppCapability::Commands => self.host.structured_commands,
            AppCapability::Wallpaper => self.host.wallpaper_library,
            AppCapability::Notifications => self.host.notifications,
            AppCapability::ExternalUrl => self.host.external_urls,
            AppCapability::Window
            | AppCapability::State
            | AppCapability::Config
            | AppCapability::Theme
            | AppCapability::Ipc => CapabilityStatus::Available,
        }
    }

    /// Returns whether a runtime capability is both granted and immediately available.
    pub fn can_use(&self, capability: AppCapability) -> bool {
        self.status(capability).is_available()
    }

    /// Returns whether the active host supports a native terminal-process backend.
    pub fn supports_terminal_process(&self) -> bool {
        self.host.terminal_process.is_available()
    }

    /// Returns whether the active host exposes native explorer integration.
    pub fn supports_native_explorer(&self) -> bool {
        self.host.native_explorer.is_available()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Lifecycle events emitted by the desktop window manager.
pub enum AppLifecycleEvent {
    /// App view has been mounted into a managed window.
    Mounted,
    /// Window became focused.
    Focused,
    /// Window lost focus.
    Blurred,
    /// Window was minimized.
    Minimized,
    /// Window was restored from minimized/maximized/suspended state.
    Restored,
    /// App is suspended by the manager.
    Suspended,
    /// App resumed from a suspended state.
    Resumed,
    /// Window close sequence started.
    Closing,
    /// Window close sequence completed.
    Closed,
}

impl AppLifecycleEvent {
    /// Returns a stable string token for persistence/debugging hooks.
    pub const fn token(self) -> &'static str {
        match self {
            Self::Mounted => "mounted",
            Self::Focused => "focused",
            Self::Blurred => "blurred",
            Self::Minimized => "minimized",
            Self::Restored => "restored",
            Self::Suspended => "suspended",
            Self::Resumed => "resumed",
            Self::Closing => "closing",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Typed IPC envelope delivered through runtime-managed app inbox channels.
pub struct AppEvent {
    /// Envelope schema version.
    pub schema_version: u32,
    /// Topic identifier (`app.<app_id>.<channel>.v1`).
    pub topic: String,
    /// JSON payload for the event.
    pub payload: Value,
    /// Optional request/reply correlation id.
    pub correlation_id: Option<String>,
    /// Optional reply target topic.
    pub reply_to: Option<String>,
    /// Source app id when known.
    pub source_app_id: Option<String>,
    /// Source window id when known.
    pub source_window_id: Option<WindowRuntimeId>,
    /// Timestamp in unix milliseconds when known.
    pub timestamp_unix_ms: Option<u64>,
}

impl AppEvent {
    /// Creates a v1 app event from topic/payload/source window id.
    pub fn new(topic: impl Into<String>, payload: Value, source_window_id: Option<u64>) -> Self {
        Self {
            schema_version: 1,
            topic: topic.into(),
            payload,
            correlation_id: None,
            reply_to: None,
            source_app_id: None,
            source_window_id,
            timestamp_unix_ms: None,
        }
    }

    /// Adds request/reply metadata to the envelope.
    pub fn with_correlation(
        mut self,
        correlation_id: Option<String>,
        reply_to: Option<String>,
    ) -> Self {
        self.correlation_id = correlation_id;
        self.reply_to = reply_to;
        self
    }
}

/// Alias for v2 naming in runtime/app APIs.
pub type IpcEnvelope = AppEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Transport commands emitted by app services to the desktop runtime.
pub enum AppCommand {
    /// Request a title update for the current window.
    SetWindowTitle {
        /// New title text.
        title: String,
    },
    /// Persist manager-owned app state for the current window.
    PersistState {
        /// Serialized app state payload.
        state: Value,
    },
    /// Persist app-shared state scoped by key.
    PersistSharedState {
        /// Shared state key.
        key: String,
        /// Shared state payload.
        state: Value,
    },
    /// Save a config value under a namespace/key pair.
    SaveConfig {
        /// Config namespace.
        namespace: String,
        /// Config key.
        key: String,
        /// Config payload.
        value: Value,
    },
    /// Request opening a URL through the host boundary.
    OpenExternalUrl {
        /// Target URL.
        url: String,
    },
    /// Subscribe current window to an app-bus topic.
    Subscribe {
        /// Topic name.
        topic: String,
    },
    /// Remove current window subscription for an app-bus topic.
    Unsubscribe {
        /// Topic name.
        topic: String,
    },
    /// Publish an event to all topic subscribers.
    PublishEvent {
        /// Topic name.
        topic: String,
        /// Event payload.
        payload: Value,
        /// Optional correlation id.
        correlation_id: Option<String>,
        /// Optional reply target.
        reply_to: Option<String>,
    },
    /// Preview a wallpaper configuration without committing it.
    PreviewWallpaper {
        /// Wallpaper preview configuration.
        config: WallpaperConfig,
    },
    /// Commit the active wallpaper preview as the current wallpaper.
    ApplyWallpaperPreview,
    /// Set the active wallpaper configuration immediately.
    SetCurrentWallpaper {
        /// Wallpaper configuration to apply.
        config: WallpaperConfig,
    },
    /// Clear the active wallpaper preview.
    ClearWallpaperPreview,
    /// Import a wallpaper asset through the host picker flow.
    ImportWallpaperFromPicker {
        /// Import policy and defaults for the new asset.
        request: WallpaperImportRequest,
    },
    /// Rename a managed wallpaper asset.
    RenameWallpaperAsset {
        /// Managed asset identifier.
        asset_id: String,
        /// New human-readable label.
        display_name: String,
    },
    /// Toggle the favorite flag for a managed wallpaper asset.
    SetWallpaperFavorite {
        /// Managed asset identifier.
        asset_id: String,
        /// Updated favorite state.
        favorite: bool,
    },
    /// Replace tags for a managed wallpaper asset.
    SetWallpaperTags {
        /// Managed asset identifier.
        asset_id: String,
        /// Tags associated with the asset.
        tags: Vec<String>,
    },
    /// Replace collection memberships for a managed wallpaper asset.
    SetWallpaperCollections {
        /// Managed asset identifier.
        asset_id: String,
        /// Collection identifiers.
        collection_ids: Vec<String>,
    },
    /// Create a new wallpaper collection.
    CreateWallpaperCollection {
        /// New collection label.
        display_name: String,
    },
    /// Rename an existing wallpaper collection.
    RenameWallpaperCollection {
        /// Collection identifier.
        collection_id: String,
        /// Updated collection label.
        display_name: String,
    },
    /// Delete a wallpaper collection and remove memberships.
    DeleteWallpaperCollection {
        /// Collection identifier.
        collection_id: String,
    },
    /// Delete a managed wallpaper asset.
    DeleteWallpaperAsset {
        /// Managed asset identifier.
        asset_id: String,
    },
    /// Toggle desktop high-contrast rendering.
    SetDesktopHighContrast {
        /// Whether high contrast should be enabled.
        enabled: bool,
    },
    /// Toggle desktop reduced-motion rendering.
    SetDesktopReducedMotion {
        /// Whether reduced motion should be enabled.
        enabled: bool,
    },
    /// Emit a host notification.
    Notify {
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Manager policy controlling app suspension behavior.
pub enum SuspendPolicy {
    /// Minimized windows are suspended until restored.
    #[default]
    OnMinimize,
    /// Windows are never manager-suspended.
    Never,
}

#[derive(Clone, Copy)]
/// Window-scoped app service for shell window integration APIs.
pub struct WindowService {
    sender: Callback<AppCommand>,
}

impl WindowService {
    /// Requests a title change for the current window.
    pub fn set_title(&self, title: impl Into<String>) {
        self.sender.call(AppCommand::SetWindowTitle {
            title: title.into(),
        });
    }
}

#[derive(Clone, Copy)]
/// State persistence service for window and app-shared state channels.
pub struct StateService {
    sender: Callback<AppCommand>,
}

impl StateService {
    /// Persists manager-owned state for this window instance.
    pub fn persist_window_state(&self, state: Value) {
        self.sender.call(AppCommand::PersistState { state });
    }

    /// Persists app-shared state under `key`.
    pub fn persist_shared_state(&self, key: impl Into<String>, state: Value) {
        self.sender.call(AppCommand::PersistSharedState {
            key: key.into(),
            state,
        });
    }
}

#[derive(Clone)]
/// Namespaced app config service.
pub struct ConfigService {
    sender: Callback<AppCommand>,
    prefs: Rc<dyn PrefsStore>,
}

impl ConfigService {
    fn pref_key(namespace: &str, key: &str) -> String {
        format!("{namespace}.{key}")
    }

    /// Loads a typed namespaced config value from the runtime-selected host preference store.
    pub async fn load<T: serde::de::DeserializeOwned>(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<Option<T>, String> {
        load_pref_with(self.prefs.as_ref(), &Self::pref_key(namespace, key)).await
    }

    /// Saves a namespaced config key/value payload.
    pub fn save(&self, namespace: impl Into<String>, key: impl Into<String>, value: Value) {
        let namespace = namespace.into();
        let key = key.into();
        self.sender.call(AppCommand::SaveConfig {
            namespace,
            key,
            value,
        });
    }
}

#[derive(Clone)]
/// Typed app-state persistence service backed by the runtime-selected host strategy.
pub struct AppStateHostService {
    store: Rc<dyn AppStateStore>,
}

impl AppStateHostService {
    /// Creates an app-state host service from a concrete adapter object.
    pub fn new(store: Rc<dyn AppStateStore>) -> Self {
        Self { store }
    }

    /// Loads typed app state and applies an explicit legacy migration hook.
    pub async fn load_with_migration<T, F>(
        &self,
        namespace: &str,
        current_schema_version: u32,
        migrate_legacy: F,
    ) -> Result<Option<T>, String>
    where
        T: serde::de::DeserializeOwned,
        F: Fn(u32, &AppStateEnvelope) -> Result<Option<T>, String>,
    {
        load_app_state_with_migration(
            self.store.as_ref(),
            namespace,
            current_schema_version,
            migrate_legacy,
        )
        .await
    }

    /// Persists typed app state under the provided namespace and schema version.
    pub async fn save<T: Serialize>(
        &self,
        namespace: &str,
        schema_version: u32,
        payload: &T,
    ) -> Result<(), String> {
        save_app_state_with(self.store.as_ref(), namespace, schema_version, payload).await
    }
}

#[derive(Clone)]
/// Typed preference service backed by the runtime-selected host strategy.
pub struct PrefsHostService {
    store: Rc<dyn PrefsStore>,
}

impl PrefsHostService {
    /// Creates a preference host service from a concrete adapter object.
    pub fn new(store: Rc<dyn PrefsStore>) -> Self {
        Self { store }
    }

    /// Loads a typed preference value.
    pub async fn load<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, String> {
        load_pref_with(self.store.as_ref(), key).await
    }

    /// Saves a typed preference value.
    pub async fn save<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        save_pref_with(self.store.as_ref(), key, value).await
    }

    /// Deletes a stored preference key.
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        self.store.delete_pref(key).await
    }
}

#[derive(Clone)]
/// Explorer/filesystem service backed by the runtime-selected host strategy.
pub struct ExplorerHostService {
    service: Rc<dyn ExplorerFsService>,
}

impl ExplorerHostService {
    /// Creates an explorer host service from a concrete adapter object.
    pub fn new(service: Rc<dyn ExplorerFsService>) -> Self {
        Self { service }
    }

    /// Returns active backend status.
    pub async fn status(&self) -> Result<ExplorerBackendStatus, String> {
        self.service.status().await
    }

    /// Opens the native-directory picker.
    pub async fn pick_native_directory(&self) -> Result<ExplorerBackendStatus, String> {
        self.service.pick_native_directory().await
    }

    /// Requests backend permissions.
    pub async fn request_permission(
        &self,
        mode: ExplorerPermissionMode,
    ) -> Result<ExplorerPermissionState, String> {
        self.service.request_permission(mode).await
    }

    /// Lists a directory.
    pub async fn list_dir(&self, path: &str) -> Result<ExplorerListResult, String> {
        self.service.list_dir(path).await
    }

    /// Reads a text file.
    pub async fn read_text_file(&self, path: &str) -> Result<ExplorerFileReadResult, String> {
        self.service.read_text_file(path).await
    }

    /// Writes a text file.
    pub async fn write_text_file(
        &self,
        path: &str,
        text: &str,
    ) -> Result<ExplorerMetadata, String> {
        self.service.write_text_file(path, text).await
    }

    /// Creates a directory.
    pub async fn create_dir(&self, path: &str) -> Result<ExplorerMetadata, String> {
        self.service.create_dir(path).await
    }

    /// Creates a text file.
    pub async fn create_file(&self, path: &str, text: &str) -> Result<ExplorerMetadata, String> {
        self.service.create_file(path, text).await
    }

    /// Deletes a path.
    pub async fn delete(&self, path: &str, recursive: bool) -> Result<(), String> {
        self.service.delete(path, recursive).await
    }

    /// Retrieves metadata for a path.
    pub async fn stat(&self, path: &str) -> Result<ExplorerMetadata, String> {
        self.service.stat(path).await
    }
}

#[derive(Clone)]
/// Content-cache service backed by the runtime-selected host strategy.
pub struct CacheHostService {
    cache: Rc<dyn ContentCache>,
}

impl CacheHostService {
    /// Creates a content-cache host service from a concrete adapter object.
    pub fn new(cache: Rc<dyn ContentCache>) -> Self {
        Self { cache }
    }

    /// Stores cached text content.
    pub async fn put_text(&self, cache_name: &str, key: &str, value: &str) -> Result<(), String> {
        self.cache.put_text(cache_name, key, value).await
    }

    /// Loads cached text content.
    pub async fn get_text(&self, cache_name: &str, key: &str) -> Result<Option<String>, String> {
        self.cache.get_text(cache_name, key).await
    }

    /// Deletes cached text content.
    pub async fn delete(&self, cache_name: &str, key: &str) -> Result<(), String> {
        self.cache.delete(cache_name, key).await
    }
}

#[derive(Clone, Copy)]
/// Theme service for shell appearance/accessibility actions.
pub struct ThemeService {
    sender: Callback<AppCommand>,
    /// Current high-contrast flag.
    pub high_contrast: ReadSignal<bool>,
    /// Current reduced-motion flag.
    pub reduced_motion: ReadSignal<bool>,
}

impl ThemeService {
    /// Requests high contrast toggle.
    pub fn set_high_contrast(&self, enabled: bool) {
        self.sender
            .call(AppCommand::SetDesktopHighContrast { enabled });
    }

    /// Requests reduced motion toggle.
    pub fn set_reduced_motion(&self, enabled: bool) {
        self.sender
            .call(AppCommand::SetDesktopReducedMotion { enabled });
    }
}

#[derive(Clone, Copy)]
/// Wallpaper service for desktop background query, preview, and library operations.
pub struct WallpaperService {
    sender: Callback<AppCommand>,
    /// Current committed wallpaper configuration.
    pub current: ReadSignal<WallpaperConfig>,
    /// Current wallpaper preview when one exists.
    pub preview: ReadSignal<Option<WallpaperConfig>>,
    /// Current wallpaper library snapshot.
    pub library: ReadSignal<WallpaperLibrarySnapshot>,
}

impl WallpaperService {
    /// Starts a wallpaper preview.
    pub fn preview(&self, config: WallpaperConfig) {
        self.sender.call(AppCommand::PreviewWallpaper { config });
    }

    /// Commits the active wallpaper preview.
    pub fn apply_preview(&self) {
        self.sender.call(AppCommand::ApplyWallpaperPreview);
    }

    /// Replaces the current wallpaper immediately.
    pub fn set_current(&self, config: WallpaperConfig) {
        self.sender.call(AppCommand::SetCurrentWallpaper { config });
    }

    /// Clears the active wallpaper preview.
    pub fn clear_preview(&self) {
        self.sender.call(AppCommand::ClearWallpaperPreview);
    }

    /// Starts host import flow for a new wallpaper asset.
    pub fn import_from_picker(&self, request: WallpaperImportRequest) {
        self.sender
            .call(AppCommand::ImportWallpaperFromPicker { request });
    }

    /// Renames a managed wallpaper asset.
    pub fn rename_asset(&self, asset_id: impl Into<String>, display_name: impl Into<String>) {
        self.sender.call(AppCommand::RenameWallpaperAsset {
            asset_id: asset_id.into(),
            display_name: display_name.into(),
        });
    }

    /// Updates the favorite flag for a managed wallpaper asset.
    pub fn set_favorite(&self, asset_id: impl Into<String>, favorite: bool) {
        self.sender.call(AppCommand::SetWallpaperFavorite {
            asset_id: asset_id.into(),
            favorite,
        });
    }

    /// Replaces tags for a managed wallpaper asset.
    pub fn set_tags(&self, asset_id: impl Into<String>, tags: Vec<String>) {
        self.sender.call(AppCommand::SetWallpaperTags {
            asset_id: asset_id.into(),
            tags,
        });
    }

    /// Replaces collection memberships for a managed wallpaper asset.
    pub fn set_collections(&self, asset_id: impl Into<String>, collection_ids: Vec<String>) {
        self.sender.call(AppCommand::SetWallpaperCollections {
            asset_id: asset_id.into(),
            collection_ids,
        });
    }

    /// Creates a new wallpaper collection.
    pub fn create_collection(&self, display_name: impl Into<String>) {
        self.sender.call(AppCommand::CreateWallpaperCollection {
            display_name: display_name.into(),
        });
    }

    /// Renames an existing wallpaper collection.
    pub fn rename_collection(
        &self,
        collection_id: impl Into<String>,
        display_name: impl Into<String>,
    ) {
        self.sender.call(AppCommand::RenameWallpaperCollection {
            collection_id: collection_id.into(),
            display_name: display_name.into(),
        });
    }

    /// Deletes a wallpaper collection.
    pub fn delete_collection(&self, collection_id: impl Into<String>) {
        self.sender.call(AppCommand::DeleteWallpaperCollection {
            collection_id: collection_id.into(),
        });
    }

    /// Deletes a managed wallpaper asset.
    pub fn delete_asset(&self, asset_id: impl Into<String>) {
        self.sender.call(AppCommand::DeleteWallpaperAsset {
            asset_id: asset_id.into(),
        });
    }
}

#[derive(Clone, Copy)]
/// Notification service routed through host capabilities.
pub struct NotificationService {
    sender: Callback<AppCommand>,
}

impl NotificationService {
    /// Emits a host notification request.
    pub fn notify(&self, title: impl Into<String>, body: impl Into<String>) {
        self.sender.call(AppCommand::Notify {
            title: title.into(),
            body: body.into(),
        });
    }
}

#[derive(Clone, Copy)]
/// Inter-app IPC service for topic subscriptions and pub/sub request-reply envelopes.
pub struct IpcService {
    sender: Callback<AppCommand>,
}

impl IpcService {
    /// Subscribes this window to a topic.
    pub fn subscribe(&self, topic: impl Into<String>) {
        self.sender.call(AppCommand::Subscribe {
            topic: topic.into(),
        });
    }

    /// Unsubscribes this window from a topic.
    pub fn unsubscribe(&self, topic: impl Into<String>) {
        self.sender.call(AppCommand::Unsubscribe {
            topic: topic.into(),
        });
    }

    /// Publishes a one-way event payload.
    pub fn publish(&self, topic: impl Into<String>, payload: Value) {
        self.sender.call(AppCommand::PublishEvent {
            topic: topic.into(),
            payload,
            correlation_id: None,
            reply_to: None,
        });
    }

    /// Publishes a request payload carrying correlation metadata.
    pub fn request(
        &self,
        topic: impl Into<String>,
        payload: Value,
        correlation_id: impl Into<String>,
        reply_to: impl Into<String>,
    ) {
        self.sender.call(AppCommand::PublishEvent {
            topic: topic.into(),
            payload,
            correlation_id: Some(correlation_id.into()),
            reply_to: Some(reply_to.into()),
        });
    }
}

/// Async completion provider used by command registrations.
pub type AppCommandCompletion = Rc<
    dyn Fn(CompletionRequest) -> LocalBoxFuture<'static, Result<Vec<CompletionItem>, ShellError>>,
>;

/// Async command handler used by app command registrations.
pub type AppCommandHandler =
    Rc<dyn Fn(AppCommandContext) -> LocalBoxFuture<'static, Result<CommandResult, ShellError>>>;

/// Execution context supplied to app-registered command handlers.
#[derive(Clone)]
pub struct AppCommandContext {
    /// Execution identifier for the current command.
    pub execution_id: ExecutionId,
    /// Parsed invocation metadata for the current pipeline stage.
    pub invocation: ParsedInvocation,
    /// Full parsed argv payload.
    pub argv: Vec<String>,
    /// Positional argument tokens after command-path resolution.
    pub args: Vec<String>,
    /// Current logical cwd.
    pub cwd: String,
    /// Structured input passed from the previous pipeline stage.
    pub input: StructuredData,
    /// Optional source window identifier.
    pub source_window_id: Option<WindowRuntimeId>,
    emit: Rc<dyn Fn(ShellStreamEvent)>,
    set_cwd: Rc<dyn Fn(String)>,
    is_cancelled: Rc<dyn Fn() -> bool>,
}

type ShellEventEmitter = Rc<dyn Fn(ShellStreamEvent)>;
type ShellCwdSetter = Rc<dyn Fn(String)>;
type CancellationProbe = Rc<dyn Fn() -> bool>;
type ShellSessionFactory = Rc<dyn Fn(String) -> Result<ShellSessionHandle, String>>;
type CommandRegistrar =
    Rc<dyn Fn(AppCommandRegistration) -> Result<CommandRegistrationHandle, String>>;
type ProviderRegistrar =
    Rc<dyn Fn(Rc<dyn AppCommandProvider>) -> Result<CommandRegistrationHandle, String>>;

impl AppCommandContext {
    /// Emits an informational notice for the current execution.
    pub fn info(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Info, message);
    }

    /// Emits a warning notice for the current execution.
    pub fn warn(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Warning, message);
    }

    /// Emits an error notice for the current execution.
    pub fn error(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Error, message);
    }

    /// Emits a structured notice for the current execution.
    pub fn notice(&self, level: CommandNoticeLevel, message: impl Into<String>) {
        self.emit(ShellStreamEvent::Notice {
            execution_id: self.execution_id,
            notice: CommandNotice {
                level,
                message: message.into(),
            },
        });
    }

    /// Emits a progress update for the current execution.
    pub fn progress(&self, value: Option<f32>, label: Option<String>) {
        self.emit(ShellStreamEvent::Progress {
            execution_id: self.execution_id,
            value,
            label,
        });
    }

    /// Emits a structured data frame for the current execution.
    pub fn data(&self, data: StructuredData, display: DisplayPreference) {
        self.emit(ShellStreamEvent::Data {
            execution_id: self.execution_id,
            data,
            display,
        });
    }

    /// Emits an incremental shell stream event.
    pub fn emit(&self, event: ShellStreamEvent) {
        (self.emit)(event);
    }

    /// Updates the logical cwd for the current session.
    pub fn set_cwd(&self, cwd: impl Into<String>) {
        (self.set_cwd)(cwd.into());
    }

    /// Returns whether the active execution has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        (self.is_cancelled)()
    }

    /// Creates a new command context from runtime-provided callbacks.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        execution_id: ExecutionId,
        invocation: ParsedInvocation,
        argv: Vec<String>,
        args: Vec<String>,
        cwd: String,
        input: StructuredData,
        source_window_id: Option<WindowRuntimeId>,
        emit: ShellEventEmitter,
        set_cwd: ShellCwdSetter,
        is_cancelled: CancellationProbe,
    ) -> Self {
        Self {
            execution_id,
            invocation,
            argv,
            args,
            cwd,
            input,
            source_window_id,
            emit,
            set_cwd,
            is_cancelled,
        }
    }
}

/// One command registration payload exposed by an app/provider.
#[derive(Clone)]
pub struct AppCommandRegistration {
    /// Static command metadata.
    pub descriptor: CommandDescriptor,
    /// Optional completion provider.
    pub completion: Option<AppCommandCompletion>,
    /// Async command handler.
    pub handler: AppCommandHandler,
}

/// Dynamic command provider implemented by apps that expose multiple commands.
pub trait AppCommandProvider {
    /// Returns all command registrations owned by this provider.
    fn commands(&self) -> Vec<AppCommandRegistration>;
}

/// Drop-based registration handle for dynamically registered commands.
#[derive(Clone)]
pub struct CommandRegistrationHandle {
    unregister: Rc<dyn Fn()>,
    active: Rc<Cell<bool>>,
}

impl CommandRegistrationHandle {
    /// Creates a new registration handle from an unregister callback.
    pub fn new(unregister: Rc<dyn Fn()>) -> Self {
        Self {
            unregister,
            active: Rc::new(Cell::new(true)),
        }
    }

    /// Creates a no-op registration handle.
    pub fn noop() -> Self {
        Self::new(Rc::new(|| {}))
    }

    /// Unregisters the command(s) if still active.
    pub fn unregister(&self) {
        if self.active.replace(false) {
            (self.unregister)();
        }
    }
}

impl Drop for CommandRegistrationHandle {
    fn drop(&mut self) {
        self.unregister();
    }
}

/// Live shell session bridge exposed to the terminal UI.
#[derive(Clone)]
pub struct ShellSessionHandle {
    /// Reactive shell event stream for this session.
    pub events: ReadSignal<Vec<ShellStreamEvent>>,
    /// Reactive active execution id when one exists.
    pub active_execution: ReadSignal<Option<ExecutionId>>,
    /// Reactive current cwd value.
    pub cwd: ReadSignal<String>,
    submit: Rc<dyn Fn(ShellRequest)>,
    cancel: Rc<dyn Fn()>,
    complete: AppCommandCompletion,
}

impl ShellSessionHandle {
    /// Creates a new shell session handle.
    pub fn new(
        events: ReadSignal<Vec<ShellStreamEvent>>,
        active_execution: ReadSignal<Option<ExecutionId>>,
        cwd: ReadSignal<String>,
        submit: Rc<dyn Fn(ShellRequest)>,
        cancel: Rc<dyn Fn()>,
        complete: AppCommandCompletion,
    ) -> Self {
        Self {
            events,
            active_execution,
            cwd,
            submit,
            cancel,
            complete,
        }
    }

    /// Submits a shell request to the active session.
    pub fn submit(&self, request: ShellRequest) {
        (self.submit)(request);
    }

    /// Cancels the active foreground execution.
    pub fn cancel(&self) {
        (self.cancel)();
    }

    /// Resolves completion candidates for the current request.
    pub async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Vec<CompletionItem>, ShellError> {
        (self.complete)(request).await
    }
}

/// Command service bridging shell sessions and dynamic registration.
#[derive(Clone)]
pub struct CommandService {
    /// Reactive global terminal history maintained by the desktop runtime.
    pub history: ReadSignal<Vec<String>>,
    create_session: ShellSessionFactory,
    register_command: CommandRegistrar,
    register_provider: ProviderRegistrar,
}

impl CommandService {
    /// Creates a command service from runtime-provided callbacks.
    pub fn new(
        history: ReadSignal<Vec<String>>,
        create_session: ShellSessionFactory,
        register_command: CommandRegistrar,
        register_provider: ProviderRegistrar,
    ) -> Self {
        Self {
            history,
            create_session,
            register_command,
            register_provider,
        }
    }

    /// Creates a disabled command service that rejects all requests deterministically.
    pub fn disabled() -> Self {
        Self::new(
            leptos::create_rw_signal(Vec::new()).read_only(),
            Rc::new(|_| Err("command sessions are unavailable".to_string())),
            Rc::new(|_| Err("command registration is unavailable".to_string())),
            Rc::new(|_| Err("command registration is unavailable".to_string())),
        )
    }

    /// Creates a new shell session for the current app window.
    pub fn create_session(&self, cwd: impl Into<String>) -> Result<ShellSessionHandle, String> {
        (self.create_session)(cwd.into())
    }

    /// Registers one command dynamically.
    pub fn register_command(
        &self,
        registration: AppCommandRegistration,
    ) -> Result<CommandRegistrationHandle, String> {
        (self.register_command)(registration)
    }

    /// Registers a multi-command provider dynamically.
    pub fn register_provider(
        &self,
        provider: Rc<dyn AppCommandProvider>,
    ) -> Result<CommandRegistrationHandle, String> {
        (self.register_provider)(provider)
    }
}

/// Typed platform dashboard state injected from the shared SDK layer.
#[derive(Clone)]
pub struct PlatformService {
    /// Reactive platform dashboard snapshot owned by the runtime shell.
    pub dashboard: ReadSignal<UiDashboardSnapshotV1>,
}

impl PlatformService {
    /// Creates a platform dashboard service.
    pub fn new(dashboard: ReadSignal<UiDashboardSnapshotV1>) -> Self {
        Self { dashboard }
    }
}

#[derive(Clone)]
/// Injected app services bundle.
///
/// This is the main app-facing service surface. It combines runtime-mediated command callbacks
/// with host-selected persistence, explorer, cache, wallpaper, notification, and command-session
/// adapters, while [`CapabilitySet`] exposes which optional domains are currently granted and
/// available.
pub struct AppServices {
    capabilities: CapabilitySet,
    /// Window integration service.
    pub window: WindowService,
    /// State persistence service.
    pub state: StateService,
    /// Namespaced config service.
    pub config: ConfigService,
    /// Typed app-state persistence service.
    pub app_state: AppStateHostService,
    /// Typed preference service.
    pub prefs: PrefsHostService,
    /// Explorer/filesystem service.
    pub explorer: ExplorerHostService,
    /// Content-cache service.
    pub cache: CacheHostService,
    /// Theme/accessibility service.
    pub theme: ThemeService,
    /// Wallpaper query/preview/library service.
    pub wallpaper: WallpaperService,
    /// Notification service.
    pub notifications: NotificationService,
    /// IPC service.
    pub ipc: IpcService,
    /// Typed platform dashboard surface from `platform/sdk`.
    pub platform: PlatformService,
    /// Shell command registration and session service.
    pub commands: CommandService,
}

impl AppServices {
    /// Creates service handles from the runtime command callback and host-selected adapters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sender: Callback<AppCommand>,
        capabilities: CapabilitySet,
        app_state: Rc<dyn AppStateStore>,
        prefs: Rc<dyn PrefsStore>,
        explorer: Rc<dyn ExplorerFsService>,
        cache: Rc<dyn ContentCache>,
        theme_high_contrast: ReadSignal<bool>,
        theme_reduced_motion: ReadSignal<bool>,
        wallpaper_current: ReadSignal<WallpaperConfig>,
        wallpaper_preview: ReadSignal<Option<WallpaperConfig>>,
        wallpaper_library: ReadSignal<WallpaperLibrarySnapshot>,
        platform_dashboard: ReadSignal<UiDashboardSnapshotV1>,
        commands: CommandService,
    ) -> Self {
        Self {
            capabilities,
            window: WindowService { sender },
            state: StateService { sender },
            config: ConfigService {
                sender,
                prefs: prefs.clone(),
            },
            app_state: AppStateHostService::new(app_state),
            prefs: PrefsHostService::new(prefs),
            explorer: ExplorerHostService::new(explorer),
            cache: CacheHostService::new(cache),
            theme: ThemeService {
                sender,
                high_contrast: theme_high_contrast,
                reduced_motion: theme_reduced_motion,
            },
            wallpaper: WallpaperService {
                sender,
                current: wallpaper_current,
                preview: wallpaper_preview,
                library: wallpaper_library,
            },
            notifications: NotificationService { sender },
            ipc: IpcService { sender },
            platform: PlatformService::new(platform_dashboard),
            commands,
        }
    }

    /// Returns the runtime-granted and host-available capability snapshot for the mounted app.
    pub fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }
}

#[derive(Clone)]
/// App mount context injected by the desktop runtime per window instance.
///
/// One value is created for each mounted window. The context carries immutable launch/restoration
/// payloads, reactive lifecycle and inbox signals, a reactive capability snapshot, and the shared
/// [`AppServices`] bundle for host/runtime operations.
pub struct AppMountContext {
    /// Stable app id from the runtime catalog.
    pub app_id: ApplicationId,
    /// Stable runtime window id.
    pub window_id: WindowRuntimeId,
    /// Launch params supplied at window-open time.
    pub launch_params: Value,
    /// Manager-restored app state payload.
    pub restored_state: Value,
    /// Reactive lifecycle signal for this window/app.
    pub lifecycle: ReadSignal<AppLifecycleEvent>,
    /// Reactive inbox signal populated by the app-bus.
    pub inbox: RwSignal<Vec<IpcEnvelope>>,
    /// Reactive capability snapshot for this mounted app.
    pub capabilities: ReadSignal<CapabilitySet>,
    /// Runtime service bundle.
    pub services: AppServices,
}

/// Static app mount function used by the runtime registry.
pub type AppMountFn = fn(AppMountContext) -> View;

#[derive(Debug, Clone, Copy)]
/// Mounted app module descriptor used by the runtime app registry.
pub struct AppModule {
    mount_fn: AppMountFn,
}

impl AppModule {
    /// Creates a module from a mount function.
    pub const fn new(mount_fn: AppMountFn) -> Self {
        Self { mount_fn }
    }

    /// Mounts the app view with a runtime-provided context.
    pub fn mount(self, context: AppMountContext) -> View {
        (self.mount_fn)(context)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Manifest-backed registration metadata for a runtime app entry.
pub struct AppRegistration {
    /// Canonical app id.
    pub app_id: ApplicationId,
    /// Human-readable display name.
    pub display_name: String,
    /// Package semantic version.
    pub version: String,
    /// Runtime contract version string.
    pub runtime_contract_version: String,
    /// Declared requested capabilities.
    pub requested_capabilities: Vec<AppCapability>,
    /// Whether only one instance should be active.
    pub single_instance: bool,
    /// Suspend policy for minimized windows.
    pub suspend_policy: SuspendPolicy,
    /// Launcher visibility flag.
    pub show_in_launcher: bool,
    /// Desktop icon visibility flag.
    pub show_on_desktop: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_id_requires_dotted_namespaces() {
        assert!(ApplicationId::new("system.control-center").is_ok());
        assert!(ApplicationId::new("system.settings").is_ok());
        assert!(ApplicationId::new("calculator").is_err());
        assert!(ApplicationId::new("System.calc").is_err());
        assert!(ApplicationId::new("system..calc").is_err());
    }

    #[test]
    fn publish_event_request_metadata_is_attached() {
        let envelope = AppEvent::new("app.system.calc.events.v1", Value::Null, Some(3))
            .with_correlation(
                Some("req-1".to_string()),
                Some("app.system.calc.reply.v1".to_string()),
            );
        assert_eq!(envelope.schema_version, 1);
        assert_eq!(envelope.correlation_id.as_deref(), Some("req-1"));
        assert_eq!(
            envelope.reply_to.as_deref(),
            Some("app.system.calc.reply.v1")
        );
    }

    #[test]
    fn capability_set_combines_runtime_grant_with_host_availability() {
        let capabilities = CapabilitySet::new(
            vec![AppCapability::Commands, AppCapability::Notifications],
            HostCapabilities::browser(),
        );

        assert!(capabilities.is_granted(AppCapability::Commands));
        assert!(capabilities.can_use(AppCapability::Commands));
        assert_eq!(
            capabilities.status(AppCapability::Notifications),
            CapabilityStatus::RequiresUserActivation
        );
        assert!(!capabilities.can_use(AppCapability::Notifications));
        assert!(!capabilities.supports_terminal_process());
    }

    #[test]
    fn primary_input_dom_id_uses_window_id() {
        assert_eq!(window_primary_input_dom_id(42), "window-primary-input-42");
    }
}
