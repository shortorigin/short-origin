//! Host-side runtime helpers for executing reducer effects and querying browser environment state.
//!
//! This module is the first extraction point for desktop shell side effects. It keeps reducer
//! semantics unchanged while moving effect execution and viewport/window queries behind a typed
//! boundary that can later be injected and mocked.

mod app_bus;
mod boot;
mod effects;
mod host_ui;
mod persistence_effects;
mod wallpaper_effects;

use std::rc::Rc;

use leptos::{logging, spawn_local, Callback};
use platform_host::{
    AppStateStore, ContentCache, ExplorerFsService, ExternalUrlService, HostCapabilities,
    HostServices, NotificationService, PrefsStore, TerminalProcessService, WallpaperAssetService,
};

use crate::{
    model::WindowRect, persistence, reducer::DesktopAction, runtime_context::DesktopRuntimeContext,
};

#[derive(Clone)]
/// Host service bundle for desktop runtime side effects.
pub struct DesktopHostContext {
    app_state: Rc<dyn AppStateStore>,
    prefs: Rc<dyn PrefsStore>,
    explorer: Rc<dyn ExplorerFsService>,
    cache: Rc<dyn ContentCache>,
    external_urls: Rc<dyn ExternalUrlService>,
    notifications: Rc<dyn NotificationService>,
    wallpaper: Rc<dyn WallpaperAssetService>,
    terminal_process: Option<Rc<dyn TerminalProcessService>>,
    capabilities: HostCapabilities,
    host_strategy_name: &'static str,
}

impl DesktopHostContext {
    /// Creates a runtime host context from an injected shared host bundle.
    pub fn new(services: HostServices) -> Self {
        Self {
            app_state: services.app_state,
            prefs: services.prefs,
            explorer: services.explorer,
            cache: services.cache,
            external_urls: services.external_urls,
            notifications: services.notifications,
            wallpaper: services.wallpaper,
            terminal_process: services.terminal_process,
            capabilities: services.capabilities,
            host_strategy_name: services.host_strategy.as_str(),
        }
    }

    /// Returns the configured app-state persistence service.
    pub fn app_state_store(&self) -> Rc<dyn AppStateStore> {
        self.app_state.clone()
    }

    /// Returns the configured lightweight preference service.
    pub fn prefs_store(&self) -> Rc<dyn PrefsStore> {
        self.prefs.clone()
    }

    /// Returns the configured explorer/filesystem service.
    pub fn explorer_fs_service(&self) -> Rc<dyn ExplorerFsService> {
        self.explorer.clone()
    }

    /// Returns the configured content cache service.
    pub fn content_cache(&self) -> Rc<dyn ContentCache> {
        self.cache.clone()
    }

    /// Returns the configured external URL service.
    pub fn external_url_service(&self) -> Rc<dyn ExternalUrlService> {
        self.external_urls.clone()
    }

    /// Returns the configured notification delivery service.
    pub fn notification_service(&self) -> Rc<dyn NotificationService> {
        self.notifications.clone()
    }

    /// Returns the configured wallpaper asset/library service.
    pub fn wallpaper_asset_service(&self) -> Rc<dyn WallpaperAssetService> {
        self.wallpaper.clone()
    }

    /// Returns the configured terminal-process backend when one is available.
    pub fn terminal_process_service(&self) -> Option<Rc<dyn TerminalProcessService>> {
        self.terminal_process.clone()
    }

    /// Returns the host capability snapshot for the active strategy.
    pub fn host_capabilities(&self) -> HostCapabilities {
        self.capabilities
    }

    /// Returns the stable name of the selected host strategy.
    pub fn host_strategy_name(&self) -> &'static str {
        self.host_strategy_name
    }

    /// Installs boot hydration/migration side effects for the desktop provider.
    ///
    /// This preserves the current boot sequence:
    /// 1. hydrate from compatibility snapshot first (if present)
    /// 2. asynchronously hydrate from durable storage if present
    /// 3. otherwise migrate the legacy snapshot into durable storage
    pub fn install_boot_hydration(&self, dispatch: Callback<DesktopAction>) {
        boot::install_boot_hydration(self.clone(), dispatch);
    }

    /// Executes a single [`crate::RuntimeEffect`] emitted by the reducer.
    pub fn run_runtime_effect(&self, runtime: DesktopRuntimeContext, effect: crate::RuntimeEffect) {
        effects::run_runtime_effect(self.clone(), runtime, effect);
    }

    /// Handles a request to focus the active window's primary input.
    ///
    /// The reducer emits this intent when a window opens or is focused. Apps opt in by rendering
    /// [`desktop_app_contract::window_primary_input_dom_id`] on their primary text field.
    pub fn focus_window_input(&self, window_id: crate::model::WindowId) {
        host_ui::focus_window_input(window_id);
    }

    /// Handles requests to open a URL outside the desktop shell.
    ///
    /// This is intentionally a host hook so browser integration can evolve independently of the UI
    /// reducer/effect pipeline.
    pub fn open_external_url(&self, url: &str) {
        host_ui::open_external_url(self.clone(), url);
    }

    fn persist_durable_snapshot(&self, state: crate::model::DesktopState, cause: &str) {
        let cause = cause.to_string();
        let host = self.clone();
        spawn_local(async move {
            if let Err(err) = persistence::persist_durable_layout_snapshot(&host, &state).await {
                logging::warn!("persist durable {cause} snapshot failed: {err}");
            }
        });
    }

    /// Returns the current desktop viewport rect available to the shell window manager.
    pub fn desktop_viewport_rect(&self, taskbar_height_px: i32) -> WindowRect {
        host_ui::desktop_viewport_rect(taskbar_height_px)
    }
}
