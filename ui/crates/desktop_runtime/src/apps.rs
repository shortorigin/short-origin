//! Desktop app registry metadata and app-content mounting helpers.

use std::sync::OnceLock;

use crate::model::{DesktopState, OpenWindowRequest, DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use desktop_app_contract::{
    AppCapability, AppModule, AppMountContext, AppRegistration, ApplicationId,
    PluginLauncherRegistration, PluginUiRegistration, PluginWindowDefaults, SuspendPolicy,
};
use desktop_app_control_center::ControlCenterApp;
use desktop_app_settings::SettingsApp;
use desktop_app_terminal::TerminalApp;
use leptos::*;
use system_ui::primitives::IconName;

const APP_ID_CONTROL_CENTER: &str = "system.control-center";
const APP_ID_TERMINAL: &str = "system.terminal";
const APP_ID_SETTINGS: &str = "system.settings";
const ALL_APP_CAPABILITIES: &[AppCapability] = &[
    AppCapability::Window,
    AppCapability::State,
    AppCapability::Config,
    AppCapability::Theme,
    AppCapability::Notifications,
    AppCapability::Ipc,
    AppCapability::ExternalUrl,
    AppCapability::Commands,
];

#[derive(Debug, Clone, Copy)]
struct GeneratedAppManifestMetadata {
    plugin_id: &'static str,
    display_name: &'static str,
    version: &'static str,
    platform_contract_version: &'static str,
    runtime_contract_version: &'static str,
    ui_entry: &'static str,
    ui_routes: &'static [&'static str],
    requested_capabilities: &'static [AppCapability],
    required_platform_contracts: &'static [&'static str],
    service_dependencies: &'static [&'static str],
    workflow_dependencies: &'static [&'static str],
    host_requirements: &'static [&'static str],
    runtime_targets: &'static [&'static str],
    permissions: &'static [&'static str],
    single_instance: bool,
    suspend_policy: SuspendPolicy,
    show_in_launcher: bool,
    show_on_desktop: bool,
    window_defaults: (i32, i32),
}

include!(concat!(env!("OUT_DIR"), "/app_catalog_generated.rs"));

fn builtin_app_id(raw: &'static str) -> ApplicationId {
    ApplicationId::trusted(raw)
}

/// Returns the generated manifest catalog payload used for build-time discovery validation.
pub fn app_manifest_catalog_json() -> &'static str {
    APP_MANIFEST_CATALOG_JSON
}

#[derive(Debug, Clone)]
/// Metadata describing how an app appears in the launcher/desktop and how it is instantiated.
pub struct AppDescriptor {
    /// Stable runtime application identifier.
    pub app_id: ApplicationId,
    /// Governed plugin/app registration metadata.
    pub registration: AppRegistration,
    /// Label shown in the start/launcher menu.
    pub launcher_label: &'static str,
    /// Label shown under the desktop icon.
    pub desktop_icon_label: &'static str,
    /// Whether the app is listed in launcher menus.
    pub show_in_launcher: bool,
    /// Whether the app is rendered as a desktop icon.
    pub show_on_desktop: bool,
    /// Whether only one instance should be open at a time.
    pub single_instance: bool,
    /// Managed app module mount descriptor.
    pub module: AppModule,
    /// Suspend policy applied by the desktop window manager.
    pub suspend_policy: SuspendPolicy,
    /// Declared capability scopes requested by the app.
    pub requested_capabilities: &'static [AppCapability],
}

fn build_app_descriptor(
    app_id: &'static str,
    desktop_icon_label: &'static str,
    metadata: GeneratedAppManifestMetadata,
    module: AppModule,
) -> AppDescriptor {
    let app_id = builtin_app_id(app_id);
    AppDescriptor {
        app_id: app_id.clone(),
        registration: AppRegistration {
            plugin_id: metadata.plugin_id.to_string(),
            app_id,
            display_name: metadata.display_name.to_string(),
            version: metadata.version.to_string(),
            platform_contract_version: metadata.platform_contract_version.to_string(),
            runtime_contract_version: metadata.runtime_contract_version.to_string(),
            ui: PluginUiRegistration {
                entry: metadata.ui_entry.to_string(),
                routes: metadata
                    .ui_routes
                    .iter()
                    .map(|route| (*route).to_string())
                    .collect(),
            },
            requested_capabilities: metadata.requested_capabilities.to_vec(),
            required_platform_contracts: metadata
                .required_platform_contracts
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            service_dependencies: metadata
                .service_dependencies
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            workflow_dependencies: metadata
                .workflow_dependencies
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            host_requirements: metadata
                .host_requirements
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            runtime_targets: metadata
                .runtime_targets
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            permissions: metadata
                .permissions
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            single_instance: metadata.single_instance,
            suspend_policy: metadata.suspend_policy,
            launcher: PluginLauncherRegistration {
                show_in_launcher: metadata.show_in_launcher,
                show_on_desktop: metadata.show_on_desktop,
            },
            show_in_launcher: metadata.show_in_launcher,
            show_on_desktop: metadata.show_on_desktop,
            window_defaults: PluginWindowDefaults {
                width: metadata.window_defaults.0,
                height: metadata.window_defaults.1,
            },
        },
        launcher_label: metadata.display_name,
        desktop_icon_label,
        show_in_launcher: metadata.show_in_launcher,
        show_on_desktop: metadata.show_on_desktop,
        single_instance: metadata.single_instance,
        module,
        suspend_policy: metadata.suspend_policy,
        requested_capabilities: metadata.requested_capabilities,
    }
}

fn build_app_registry() -> Vec<AppDescriptor> {
    vec![
        build_app_descriptor(
            APP_ID_CONTROL_CENTER,
            "Control Center",
            SYSTEM_CONTROL_CENTER_MANIFEST,
            AppModule::new(mount_control_center_app),
        ),
        build_app_descriptor(
            APP_ID_TERMINAL,
            SYSTEM_TERMINAL_MANIFEST.display_name,
            SYSTEM_TERMINAL_MANIFEST,
            AppModule::new(mount_terminal_app),
        ),
        build_app_descriptor(
            APP_ID_SETTINGS,
            "Settings",
            SYSTEM_SETTINGS_MANIFEST,
            AppModule::new(mount_settings_app),
        ),
    ]
}

fn app_registry_storage() -> &'static OnceLock<Vec<AppDescriptor>> {
    static APP_REGISTRY: OnceLock<Vec<AppDescriptor>> = OnceLock::new();
    &APP_REGISTRY
}

const BUILTIN_PRIVILEGED_APP_IDS: &[&str] = &[APP_ID_SETTINGS, APP_ID_CONTROL_CENTER];
const LEGACY_BUILTIN_APP_ID_MAPPINGS: &[(&str, &str)] = &[
    ("Control Center", APP_ID_CONTROL_CENTER),
    ("Terminal", APP_ID_TERMINAL),
    ("Settings", APP_ID_SETTINGS),
];

/// Returns the static app registry used by the desktop shell.
pub fn app_registry() -> &'static [AppDescriptor] {
    app_registry_storage()
        .get_or_init(build_app_registry)
        .as_slice()
}

/// Returns app descriptors that should appear in launcher menus.
pub fn launcher_apps() -> Vec<AppDescriptor> {
    app_registry()
        .iter()
        .filter(|entry| entry.show_in_launcher)
        .cloned()
        .collect()
}

/// Returns app descriptors that should appear as desktop icons.
pub fn desktop_icon_apps() -> Vec<AppDescriptor> {
    app_registry()
        .iter()
        .filter(|entry| entry.show_on_desktop)
        .cloned()
        .collect()
}

/// Returns the descriptor for a canonical application id.
///
/// # Panics
///
/// Panics if the app id is not present in the registry.
pub fn app_descriptor_by_id(app_id: &ApplicationId) -> &'static AppDescriptor {
    app_registry()
        .iter()
        .find(|entry| &entry.app_id == app_id)
        .expect("app descriptor exists")
}

/// Returns the managed app module descriptor for one canonical app id.
pub fn app_module_by_id(app_id: &ApplicationId) -> AppModule {
    app_descriptor_by_id(app_id).module
}

/// Returns the window-manager suspend policy for one canonical app id.
pub fn app_suspend_policy_by_id(app_id: &ApplicationId) -> SuspendPolicy {
    app_descriptor_by_id(app_id).suspend_policy
}

/// Returns declared capability scopes for one canonical app id.
pub fn app_requested_capabilities_by_id(app_id: &ApplicationId) -> &'static [AppCapability] {
    app_descriptor_by_id(app_id).requested_capabilities
}

/// Returns the governed app registration for one canonical application id.
pub fn app_registration_by_id(app_id: &ApplicationId) -> AppRegistration {
    app_descriptor_by_id(app_id).registration.clone()
}

/// Returns whether `app_id` is privileged in shell policy.
pub fn app_is_privileged_by_id(app_id: &ApplicationId) -> bool {
    BUILTIN_PRIVILEGED_APP_IDS
        .iter()
        .any(|id| *id == app_id.as_str())
}

/// Returns whether `app_id` is privileged after applying the current runtime policy overlay.
pub fn app_is_privileged(state: &DesktopState, app_id: &ApplicationId) -> bool {
    app_is_privileged_by_id(app_id) || state.privileged_app_ids.contains(app_id.as_str())
}

/// Returns the runtime-effective capabilities for an app after applying privilege policy.
pub fn resolved_capabilities(state: &DesktopState, app_id: &ApplicationId) -> Vec<AppCapability> {
    if app_is_privileged(state, app_id) {
        ALL_APP_CAPABILITIES.to_vec()
    } else {
        app_requested_capabilities_by_id(app_id).to_vec()
    }
}

/// Parses a canonical or legacy serialized app id into an [`ApplicationId`].
pub fn parse_application_id_compat(raw: &str) -> Option<ApplicationId> {
    ApplicationId::new(raw.trim()).ok().or_else(|| {
        LEGACY_BUILTIN_APP_ID_MAPPINGS
            .iter()
            .find_map(|(legacy, canonical)| (*legacy == raw.trim()).then_some(*canonical))
            .map(ApplicationId::trusted)
    })
}

/// Returns the shell title for one canonical app id.
pub fn app_title_by_id(app_id: &ApplicationId) -> &'static str {
    app_descriptor_by_id(app_id).launcher_label
}

/// Returns the default icon id string for one canonical app id.
pub fn app_icon_id_by_id(app_id: &ApplicationId) -> &'static str {
    match app_id.as_str() {
        APP_ID_CONTROL_CENTER => "home",
        APP_ID_TERMINAL => "terminal",
        APP_ID_SETTINGS => "settings",
        _ => "window",
    }
}

/// Returns the semantic shell icon for one canonical app id.
pub fn app_icon_name_by_id(app_id: &ApplicationId) -> IconName {
    match app_id.as_str() {
        APP_ID_CONTROL_CENTER => IconName::Home,
        APP_ID_TERMINAL => IconName::Terminal,
        APP_ID_SETTINGS => IconName::Settings,
        _ => IconName::WindowMultiple,
    }
}

/// Returns the canonical system settings application id.
pub fn settings_application_id() -> ApplicationId {
    builtin_app_id(APP_ID_SETTINGS)
}

/// Returns the canonical pinned taskbar application ids in display order.
pub fn pinned_taskbar_app_ids() -> Vec<ApplicationId> {
    [APP_ID_CONTROL_CENTER, APP_ID_TERMINAL, APP_ID_SETTINGS]
        .into_iter()
        .map(builtin_app_id)
        .collect()
}

/// Builds the default [`OpenWindowRequest`] for a canonical application id.
pub fn default_open_request_by_id(
    app_id: &ApplicationId,
    viewport: Option<crate::model::WindowRect>,
) -> Option<OpenWindowRequest> {
    app_registry()
        .iter()
        .any(|entry| entry.app_id == *app_id)
        .then(|| {
            let mut req = OpenWindowRequest::new(app_id.clone());
            req.rect = Some(default_window_rect_for_app(app_id, viewport));
            req.viewport = viewport;
            req
        })
}

fn default_window_rect_for_app(
    app_id: &ApplicationId,
    viewport: Option<crate::model::WindowRect>,
) -> crate::model::WindowRect {
    let vp = viewport.unwrap_or(crate::model::WindowRect {
        x: 0,
        y: 0,
        w: 1280,
        h: 760,
    });

    let (min_w, min_h, max_w_ratio, max_h_ratio, default_w_ratio, default_h_ratio) =
        match app_id.as_str() {
            APP_ID_CONTROL_CENTER => (
                SYSTEM_CONTROL_CENTER_MANIFEST.window_defaults.0,
                SYSTEM_CONTROL_CENTER_MANIFEST.window_defaults.1,
                0.92,
                0.92,
                0.82,
                0.80,
            ),
            APP_ID_TERMINAL => (
                SYSTEM_TERMINAL_MANIFEST.window_defaults.0,
                SYSTEM_TERMINAL_MANIFEST.window_defaults.1,
                0.88,
                0.86,
                0.74,
                0.70,
            ),
            APP_ID_SETTINGS => (
                SYSTEM_SETTINGS_MANIFEST.window_defaults.0,
                SYSTEM_SETTINGS_MANIFEST.window_defaults.1,
                0.92,
                0.92,
                0.82,
                0.82,
            ),
            _ => (
                DEFAULT_WINDOW_WIDTH,
                DEFAULT_WINDOW_HEIGHT,
                0.80,
                0.80,
                0.70,
                0.70,
            ),
        };

    let max_w = ((vp.w as f32) * max_w_ratio) as i32;
    let max_h = ((vp.h as f32) * max_h_ratio) as i32;
    let w = (((vp.w as f32) * default_w_ratio) as i32).clamp(min_w, max_w.max(min_w));
    let h = (((vp.h as f32) * default_h_ratio) as i32).clamp(min_h, max_h.max(min_h));
    let x = vp.x + ((vp.w - w) / 2).max(10);
    let y = vp.y + ((vp.h - h) / 2).max(10);

    crate::model::WindowRect { x, y, w, h }
}

fn mount_control_center_app(context: AppMountContext) -> View {
    view! {
        <ControlCenterApp
            launch_params=context.launch_params.clone()
            restored_state=Some(context.restored_state.clone())
            services=Some(context.services)
        />
    }
    .into_view()
}

fn mount_terminal_app(context: AppMountContext) -> View {
    view! {
        <TerminalApp
            window_id=context.window_id
            launch_params=context.launch_params.clone()
            restored_state=Some(context.restored_state.clone())
            services=Some(context.services)
        />
    }
    .into_view()
}

fn mount_settings_app(context: AppMountContext) -> View {
    view! {
        <SettingsApp
            launch_params=context.launch_params.clone()
            restored_state=Some(context.restored_state.clone())
            services=Some(context.services)
        />
    }
    .into_view()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct CatalogUi {
        entry: String,
        routes: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct CatalogLauncher {
        show_in_launcher: bool,
        show_on_desktop: bool,
    }

    #[derive(Debug, Deserialize)]
    struct CatalogManifest {
        plugin_id: String,
        app_id: String,
        platform_contract_version: String,
        runtime_contract_version: String,
        ui: CatalogUi,
        required_platform_contracts: Vec<String>,
        runtime_targets: Vec<String>,
        launcher: CatalogLauncher,
    }

    #[test]
    fn launcher_and_desktop_registry_only_ship_supported_apps() {
        let shipped = app_registry()
            .iter()
            .filter(|entry| entry.show_in_launcher || entry.show_on_desktop)
            .map(|entry| entry.app_id.to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            shipped,
            vec![
                APP_ID_CONTROL_CENTER.to_string(),
                APP_ID_TERMINAL.to_string(),
                APP_ID_SETTINGS.to_string(),
            ]
        );
    }

    #[test]
    fn default_open_request_by_id_preserves_authored_defaults_for_reducer_clamping() {
        let viewport = crate::model::WindowRect {
            x: 0,
            y: 0,
            w: 900,
            h: 620,
        };
        let req =
            default_open_request_by_id(&builtin_app_id(APP_ID_CONTROL_CENTER), Some(viewport))
                .expect("default request");
        let rect = req.rect.expect("default rect");

        assert_eq!(req.viewport, Some(viewport));
        assert_eq!(rect.w, SYSTEM_CONTROL_CENTER_MANIFEST.window_defaults.0);
        assert_eq!(rect.h, SYSTEM_CONTROL_CENTER_MANIFEST.window_defaults.1);
    }

    #[test]
    fn control_center_defaults_are_larger_than_terminal() {
        let viewport = crate::model::WindowRect {
            x: 0,
            y: 0,
            w: 1280,
            h: 760,
        };
        let control_center =
            default_open_request_by_id(&builtin_app_id(APP_ID_CONTROL_CENTER), Some(viewport))
                .expect("control center request")
                .rect
                .expect("control center rect");
        let terminal = default_open_request_by_id(&builtin_app_id(APP_ID_TERMINAL), Some(viewport))
            .expect("terminal request")
            .rect
            .expect("terminal rect");

        assert!(control_center.w > terminal.w);
        assert!(control_center.h >= terminal.h);
    }

    #[test]
    fn built_in_apps_expose_governed_registration_metadata() {
        let registration = app_registration_by_id(&builtin_app_id(APP_ID_CONTROL_CENTER));
        assert_eq!(registration.plugin_id, APP_ID_CONTROL_CENTER);
        assert_eq!(registration.ui.entry, "desktop_app_control_center");
        assert_eq!(
            registration.required_platform_contracts,
            vec![
                "schemas/contracts/v1/plugin-module-v1.json".to_string(),
                "platform/sdk/v1/dashboard".to_string()
            ]
        );
        assert_eq!(registration.runtime_targets, vec!["pwa", "tauri"]);
        assert!(registration.launcher.show_on_desktop);
    }

    #[test]
    fn manifest_catalog_includes_required_plugin_fields() {
        let catalog: Vec<CatalogManifest> =
            serde_json::from_str(app_manifest_catalog_json()).expect("manifest catalog json");
        let control_center = catalog
            .iter()
            .find(|manifest| manifest.app_id == APP_ID_CONTROL_CENTER)
            .expect("control center manifest");

        assert_eq!(control_center.plugin_id, APP_ID_CONTROL_CENTER);
        assert_eq!(control_center.platform_contract_version, "1.0.0");
        assert_eq!(control_center.runtime_contract_version, "2.0.0");
        assert_eq!(control_center.ui.entry, "desktop_app_control_center");
        assert!(control_center
            .ui
            .routes
            .contains(&"/apps/control-center".to_string()));
        assert!(control_center
            .required_platform_contracts
            .contains(&"schemas/contracts/v1/plugin-module-v1.json".to_string()));
        assert_eq!(control_center.runtime_targets, vec!["pwa", "tauri"]);
        assert!(control_center.launcher.show_in_launcher);
        assert!(control_center.launcher.show_on_desktop);
    }
}
