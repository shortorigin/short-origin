use std::rc::Rc;

use platform_host::{
    AppStateEnvelope, AppStateStore, AppStateStoreFuture, ContentCache, ContentCacheFuture,
    ExplorerBackendStatus, ExplorerFileReadResult, ExplorerFsFuture, ExplorerFsService,
    ExplorerListResult, ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
    ExternalUrlFuture, ExternalUrlService, HostCapabilities, HostServices, HostStrategy,
    NoopAppStateStore, NoopContentCache, NoopExplorerFsService, NoopExternalUrlService,
    NoopNotificationService, NoopPrefsStore, NoopWallpaperAssetService, NotificationFuture,
    NotificationService, PrefsStore, PrefsStoreFuture, ResolvedWallpaperSource,
    WallpaperAssetDeleteResult, WallpaperAssetFuture, WallpaperAssetMetadataPatch,
    WallpaperAssetRecord, WallpaperAssetService, WallpaperCollection,
    WallpaperCollectionDeleteResult, WallpaperImportRequest, WallpaperImportResult,
    WallpaperLibrarySnapshot, WallpaperSelection,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    TauriAppStateStore, TauriContentCache, TauriExplorerFsService, TauriExternalUrlService,
    TauriNotificationService, TauriPrefsStore, WebAppStateStore, WebContentCache,
    WebExplorerFsService, WebExternalUrlService, WebNotificationService, WebPrefsStore,
    WebWallpaperAssetService,
};

/// Returns the compile-time selected host strategy for the active build.
///
/// Product builds should enable at most one desktop-host feature at a time. The verification
/// matrix also exercises `--all-features`, which enables both desktop variants simultaneously;
/// in that case this selector gives `desktop-host-tauri` precedence so the crate still compiles
/// under Cargo's all-features expansion without introducing ambiguous adapter wiring.
pub const fn selected_host_strategy() -> HostStrategy {
    if cfg!(feature = "desktop-host-tauri") {
        HostStrategy::DesktopTauri
    } else if cfg!(feature = "desktop-host-stub") {
        HostStrategy::DesktopStub
    } else {
        HostStrategy::Browser
    }
}

/// Returns the selected host strategy as a stable string token.
pub fn host_strategy_name() -> &'static str {
    selected_host_strategy().as_str()
}

/// Adapter enum that erases the concrete app-state backend behind [`AppStateStore`].
#[derive(Debug, Clone, Copy)]
pub enum AppStateStoreAdapter {
    /// Browser-backed IndexedDB app-state persistence.
    Browser(WebAppStateStore),
    /// Native desktop transport app-state persistence.
    DesktopTauri(TauriAppStateStore),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopAppStateStore),
}

impl AppStateStore for AppStateStoreAdapter {
    fn load_app_state_envelope<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<Option<AppStateEnvelope>, String>> {
        match self {
            Self::Browser(store) => store.load_app_state_envelope(namespace),
            Self::DesktopTauri(store) => store.load_app_state_envelope(namespace),
            Self::DesktopStub(store) => store.load_app_state_envelope(namespace),
        }
    }

    fn save_app_state_envelope<'a>(
        &'a self,
        envelope: &'a AppStateEnvelope,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.save_app_state_envelope(envelope),
            Self::DesktopTauri(store) => store.save_app_state_envelope(envelope),
            Self::DesktopStub(store) => store.save_app_state_envelope(envelope),
        }
    }

    fn delete_app_state<'a>(
        &'a self,
        namespace: &'a str,
    ) -> AppStateStoreFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.delete_app_state(namespace),
            Self::DesktopTauri(store) => store.delete_app_state(namespace),
            Self::DesktopStub(store) => store.delete_app_state(namespace),
        }
    }

    fn list_app_state_namespaces<'a>(
        &'a self,
    ) -> AppStateStoreFuture<'a, Result<Vec<String>, String>> {
        match self {
            Self::Browser(store) => store.list_app_state_namespaces(),
            Self::DesktopTauri(store) => store.list_app_state_namespaces(),
            Self::DesktopStub(store) => store.list_app_state_namespaces(),
        }
    }
}

/// Adapter enum that erases the concrete content-cache backend behind [`ContentCache`].
#[derive(Debug, Clone, Copy)]
pub enum ContentCacheAdapter {
    /// Browser Cache API-backed content cache.
    Browser(WebContentCache),
    /// Native desktop transport content cache.
    DesktopTauri(TauriContentCache),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopContentCache),
}

impl ContentCache for ContentCacheAdapter {
    fn put_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
        value: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.put_text(cache_name, key, value),
            Self::DesktopTauri(store) => store.put_text(cache_name, key, value),
            Self::DesktopStub(store) => store.put_text(cache_name, key, value),
        }
    }

    fn get_text<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<Option<String>, String>> {
        match self {
            Self::Browser(store) => store.get_text(cache_name, key),
            Self::DesktopTauri(store) => store.get_text(cache_name, key),
            Self::DesktopStub(store) => store.get_text(cache_name, key),
        }
    }

    fn delete<'a>(
        &'a self,
        cache_name: &'a str,
        key: &'a str,
    ) -> ContentCacheFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.delete(cache_name, key),
            Self::DesktopTauri(store) => store.delete(cache_name, key),
            Self::DesktopStub(store) => store.delete(cache_name, key),
        }
    }
}

/// Adapter enum that erases the concrete explorer/filesystem backend behind
/// [`ExplorerFsService`].
#[derive(Debug, Clone, Copy)]
pub enum ExplorerFsServiceAdapter {
    /// Browser-backed virtual/native filesystem integration.
    Browser(WebExplorerFsService),
    /// Native desktop explorer/filesystem transport.
    DesktopTauri(TauriExplorerFsService),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopExplorerFsService),
}

impl ExplorerFsService for ExplorerFsServiceAdapter {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        match self {
            Self::Browser(store) => store.status(),
            Self::DesktopTauri(store) => store.status(),
            Self::DesktopStub(store) => store.status(),
        }
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        match self {
            Self::Browser(store) => store.pick_native_directory(),
            Self::DesktopTauri(store) => store.pick_native_directory(),
            Self::DesktopStub(store) => store.pick_native_directory(),
        }
    }

    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, Result<ExplorerPermissionState, String>> {
        match self {
            Self::Browser(store) => store.request_permission(mode),
            Self::DesktopTauri(store) => store.request_permission(mode),
            Self::DesktopStub(store) => store.request_permission(mode),
        }
    }

    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerListResult, String>> {
        match self {
            Self::Browser(store) => store.list_dir(path),
            Self::DesktopTauri(store) => store.list_dir(path),
            Self::DesktopStub(store) => store.list_dir(path),
        }
    }

    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerFileReadResult, String>> {
        match self {
            Self::Browser(store) => store.read_text_file(path),
            Self::DesktopTauri(store) => store.read_text_file(path),
            Self::DesktopStub(store) => store.read_text_file(path),
        }
    }

    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        match self {
            Self::Browser(store) => store.write_text_file(path, text),
            Self::DesktopTauri(store) => store.write_text_file(path, text),
            Self::DesktopStub(store) => store.write_text_file(path, text),
        }
    }

    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        match self {
            Self::Browser(store) => store.create_dir(path),
            Self::DesktopTauri(store) => store.create_dir(path),
            Self::DesktopStub(store) => store.create_dir(path),
        }
    }

    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        match self {
            Self::Browser(store) => store.create_file(path, text),
            Self::DesktopTauri(store) => store.create_file(path, text),
            Self::DesktopStub(store) => store.create_file(path, text),
        }
    }

    fn delete<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
    ) -> ExplorerFsFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.delete(path, recursive),
            Self::DesktopTauri(store) => store.delete(path, recursive),
            Self::DesktopStub(store) => store.delete(path, recursive),
        }
    }

    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        match self {
            Self::Browser(store) => store.stat(path),
            Self::DesktopTauri(store) => store.stat(path),
            Self::DesktopStub(store) => store.stat(path),
        }
    }
}

/// Adapter enum that erases the concrete external URL backend behind [`ExternalUrlService`].
#[derive(Debug, Clone, Copy)]
pub enum ExternalUrlServiceAdapter {
    /// Browser-backed external URL opening.
    Browser(WebExternalUrlService),
    /// Native desktop external URL opening through Tauri transport.
    DesktopTauri(TauriExternalUrlService),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopExternalUrlService),
}

impl ExternalUrlService for ExternalUrlServiceAdapter {
    fn open_url<'a>(&'a self, url: &'a str) -> ExternalUrlFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(service) => service.open_url(url),
            Self::DesktopTauri(service) => service.open_url(url),
            Self::DesktopStub(service) => service.open_url(url),
        }
    }
}

/// Adapter enum that erases the concrete preferences backend behind [`PrefsStore`].
#[derive(Debug, Clone, Copy)]
pub enum PrefsStoreAdapter {
    /// Browser-backed preference storage.
    Browser(WebPrefsStore),
    /// Native desktop transport preference storage.
    DesktopTauri(TauriPrefsStore),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopPrefsStore),
}

impl PrefsStoreAdapter {
    /// Loads a browser-local typed preference value.
    ///
    /// Returns `None` for transport-backed desktop strategies because typed local helpers are only
    /// available on the browser implementation.
    pub fn load_typed<T: DeserializeOwned>(self, key: &str) -> Option<T> {
        match self {
            Self::Browser(store) => store.load_typed(key),
            Self::DesktopTauri(_) | Self::DesktopStub(_) => None,
        }
    }

    /// Saves a browser-local typed preference value.
    ///
    /// Desktop transport strategies accept the call as a no-op because callers should use the
    /// trait-based async [`PrefsStore`] path for cross-host persistence.
    pub fn save_typed<T: Serialize>(self, key: &str, value: &T) -> Result<(), String> {
        match self {
            Self::Browser(store) => store.save_typed(key, value),
            Self::DesktopTauri(_) | Self::DesktopStub(_) => {
                let _ = (key, value);
                Ok(())
            }
        }
    }
}

impl PrefsStore for PrefsStoreAdapter {
    fn load_pref<'a>(
        &'a self,
        key: &'a str,
    ) -> PrefsStoreFuture<'a, Result<Option<String>, String>> {
        match self {
            Self::Browser(store) => store.load_pref(key),
            Self::DesktopTauri(store) => store.load_pref(key),
            Self::DesktopStub(store) => store.load_pref(key),
        }
    }

    fn save_pref<'a>(
        &'a self,
        key: &'a str,
        raw_json: &'a str,
    ) -> PrefsStoreFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.save_pref(key, raw_json),
            Self::DesktopTauri(store) => store.save_pref(key, raw_json),
            Self::DesktopStub(store) => store.save_pref(key, raw_json),
        }
    }

    fn delete_pref<'a>(&'a self, key: &'a str) -> PrefsStoreFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(store) => store.delete_pref(key),
            Self::DesktopTauri(store) => store.delete_pref(key),
            Self::DesktopStub(store) => store.delete_pref(key),
        }
    }
}

/// Adapter enum that erases the concrete notification backend behind
/// [`NotificationService`].
#[derive(Debug, Clone, Copy)]
pub enum NotificationServiceAdapter {
    /// Browser Notification API-backed delivery.
    Browser(WebNotificationService),
    /// Native desktop transport-backed delivery.
    DesktopTauri(TauriNotificationService),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopNotificationService),
}

impl NotificationService for NotificationServiceAdapter {
    fn notify<'a>(
        &'a self,
        title: &'a str,
        body: &'a str,
    ) -> NotificationFuture<'a, Result<(), String>> {
        match self {
            Self::Browser(service) => service.notify(title, body),
            Self::DesktopTauri(service) => service.notify(title, body),
            Self::DesktopStub(service) => service.notify(title, body),
        }
    }
}

/// Adapter enum that erases the concrete wallpaper-library backend behind
/// [`WallpaperAssetService`].
#[derive(Debug, Clone, Copy)]
pub enum WallpaperAssetServiceAdapter {
    /// Browser-backed wallpaper library implementation.
    Browser(WebWallpaperAssetService),
    /// Desktop transport build that currently reuses the browser wallpaper implementation.
    DesktopTauri(WebWallpaperAssetService),
    /// No-op fallback used when desktop transport is intentionally stubbed.
    DesktopStub(NoopWallpaperAssetService),
}

impl WallpaperAssetService for WallpaperAssetServiceAdapter {
    fn import_from_picker<'a>(
        &'a self,
        request: WallpaperImportRequest,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperImportResult, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.import_from_picker(request)
            }
            Self::DesktopStub(service) => service.import_from_picker(request),
        }
    }

    fn list_library<'a>(
        &'a self,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperLibrarySnapshot, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => service.list_library(),
            Self::DesktopStub(service) => service.list_library(),
        }
    }

    fn update_asset_metadata<'a>(
        &'a self,
        asset_id: &'a str,
        patch: WallpaperAssetMetadataPatch,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetRecord, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.update_asset_metadata(asset_id, patch)
            }
            Self::DesktopStub(service) => service.update_asset_metadata(asset_id, patch),
        }
    }

    fn create_collection<'a>(
        &'a self,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.create_collection(display_name)
            }
            Self::DesktopStub(service) => service.create_collection(display_name),
        }
    }

    fn rename_collection<'a>(
        &'a self,
        collection_id: &'a str,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.rename_collection(collection_id, display_name)
            }
            Self::DesktopStub(service) => service.rename_collection(collection_id, display_name),
        }
    }

    fn delete_collection<'a>(
        &'a self,
        collection_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollectionDeleteResult, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.delete_collection(collection_id)
            }
            Self::DesktopStub(service) => service.delete_collection(collection_id),
        }
    }

    fn delete_asset<'a>(
        &'a self,
        asset_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetDeleteResult, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => service.delete_asset(asset_id),
            Self::DesktopStub(service) => service.delete_asset(asset_id),
        }
    }

    fn resolve_source<'a>(
        &'a self,
        selection: WallpaperSelection,
    ) -> WallpaperAssetFuture<'a, Result<Option<ResolvedWallpaperSource>, String>> {
        match self {
            Self::Browser(service) | Self::DesktopTauri(service) => {
                service.resolve_source(selection)
            }
            Self::DesktopStub(service) => service.resolve_source(selection),
        }
    }
}

/// Builds the app-state adapter for the compile-time selected host strategy.
pub fn app_state_store() -> AppStateStoreAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => AppStateStoreAdapter::Browser(WebAppStateStore),
        HostStrategy::DesktopTauri => AppStateStoreAdapter::DesktopTauri(TauriAppStateStore),
        HostStrategy::DesktopStub => AppStateStoreAdapter::DesktopStub(NoopAppStateStore),
    }
}

/// Builds the content-cache adapter for the compile-time selected host strategy.
pub fn content_cache() -> ContentCacheAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => ContentCacheAdapter::Browser(WebContentCache),
        HostStrategy::DesktopTauri => ContentCacheAdapter::DesktopTauri(TauriContentCache),
        HostStrategy::DesktopStub => ContentCacheAdapter::DesktopStub(NoopContentCache),
    }
}

/// Builds the explorer/filesystem adapter for the compile-time selected host strategy.
pub fn explorer_fs_service() -> ExplorerFsServiceAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => ExplorerFsServiceAdapter::Browser(WebExplorerFsService),
        HostStrategy::DesktopTauri => {
            ExplorerFsServiceAdapter::DesktopTauri(TauriExplorerFsService)
        }
        HostStrategy::DesktopStub => ExplorerFsServiceAdapter::DesktopStub(NoopExplorerFsService),
    }
}

/// Builds the preferences adapter for the compile-time selected host strategy.
pub fn prefs_store() -> PrefsStoreAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => PrefsStoreAdapter::Browser(WebPrefsStore),
        HostStrategy::DesktopTauri => PrefsStoreAdapter::DesktopTauri(TauriPrefsStore),
        HostStrategy::DesktopStub => PrefsStoreAdapter::DesktopStub(NoopPrefsStore),
    }
}

/// Builds the notification adapter for the compile-time selected host strategy.
pub fn notification_service() -> NotificationServiceAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => NotificationServiceAdapter::Browser(WebNotificationService),
        HostStrategy::DesktopTauri => {
            NotificationServiceAdapter::DesktopTauri(TauriNotificationService)
        }
        HostStrategy::DesktopStub => {
            NotificationServiceAdapter::DesktopStub(NoopNotificationService)
        }
    }
}

/// Builds the external-URL adapter for the compile-time selected host strategy.
pub fn external_url_service() -> ExternalUrlServiceAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => ExternalUrlServiceAdapter::Browser(WebExternalUrlService),
        HostStrategy::DesktopTauri => {
            ExternalUrlServiceAdapter::DesktopTauri(TauriExternalUrlService)
        }
        HostStrategy::DesktopStub => ExternalUrlServiceAdapter::DesktopStub(NoopExternalUrlService),
    }
}

/// Builds the wallpaper-library adapter for the compile-time selected host strategy.
pub fn wallpaper_asset_service() -> WallpaperAssetServiceAdapter {
    match selected_host_strategy() {
        HostStrategy::Browser => WallpaperAssetServiceAdapter::Browser(WebWallpaperAssetService),
        HostStrategy::DesktopTauri => {
            WallpaperAssetServiceAdapter::DesktopTauri(WebWallpaperAssetService)
        }
        HostStrategy::DesktopStub => {
            WallpaperAssetServiceAdapter::DesktopStub(NoopWallpaperAssetService)
        }
    }
}

/// Returns the host capability snapshot for the selected host strategy.
pub const fn host_capabilities() -> HostCapabilities {
    match selected_host_strategy() {
        HostStrategy::Browser => HostCapabilities::browser(),
        HostStrategy::DesktopTauri => HostCapabilities::desktop_tauri(),
        HostStrategy::DesktopStub => HostCapabilities::desktop_stub(),
    }
}

/// Builds the runtime host bundle for the selected browser or desktop host strategy.
pub fn build_host_services() -> HostServices {
    HostServices {
        app_state: Rc::new(app_state_store()),
        prefs: Rc::new(prefs_store()),
        explorer: Rc::new(explorer_fs_service()),
        cache: Rc::new(content_cache()),
        external_urls: Rc::new(external_url_service()),
        notifications: Rc::new(notification_service()),
        wallpaper: Rc::new(wallpaper_asset_service()),
        terminal_process: None,
        capabilities: host_capabilities(),
        host_strategy: selected_host_strategy(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_host_strategy_matches_compiled_feature_set() {
        let expected = if cfg!(feature = "desktop-host-tauri") {
            HostStrategy::DesktopTauri
        } else if cfg!(feature = "desktop-host-stub") {
            HostStrategy::DesktopStub
        } else {
            HostStrategy::Browser
        };

        assert_eq!(selected_host_strategy(), expected);
        assert_eq!(host_strategy_name(), expected.as_str());
    }

    #[test]
    fn all_features_prefers_tauri_over_stub() {
        if cfg!(all(
            feature = "desktop-host-tauri",
            feature = "desktop-host-stub"
        )) {
            assert_eq!(selected_host_strategy(), HostStrategy::DesktopTauri);
        }
    }
}
