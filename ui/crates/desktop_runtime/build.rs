use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowDefaults {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiContribution {
    entry: String,
    routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Launcher {
    show_in_launcher: bool,
    show_on_desktop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppManifest {
    schema_version: u32,
    plugin_id: String,
    app_id: String,
    display_name: String,
    version: String,
    platform_contract_version: String,
    runtime_contract_version: String,
    ui: UiContribution,
    requested_capabilities: Vec<String>,
    required_platform_contracts: Vec<String>,
    service_dependencies: Vec<String>,
    workflow_dependencies: Vec<String>,
    host_requirements: Vec<String>,
    runtime_targets: Vec<String>,
    permissions: Vec<String>,
    single_instance: bool,
    suspend_policy: String,
    launcher: Launcher,
    window_defaults: WindowDefaults,
}

fn app_manifest_paths(root: &Path) -> Vec<PathBuf> {
    ["control_center", "terminal", "settings"]
        .iter()
        .map(|name| {
            root.join("..")
                .join("apps")
                .join(name)
                .join("app.manifest.toml")
        })
        .collect()
}

fn main() {
    let crate_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let mut manifests = Vec::<AppManifest>::new();

    for path in app_manifest_paths(&crate_root) {
        println!("cargo:rerun-if-changed={}", path.display());
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let manifest: AppManifest = toml::from_str(&raw)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
        if manifest.schema_version != 1 {
            panic!(
                "manifest schema mismatch in {}: expected 1 found {}",
                path.display(),
                manifest.schema_version
            );
        }
        if manifest.plugin_id != manifest.app_id {
            panic!(
                "plugin/app id mismatch in {}: expected identical v1 ids, found `{}` vs `{}`",
                path.display(),
                manifest.plugin_id,
                manifest.app_id
            );
        }
        if !manifest.platform_contract_version.starts_with("1.") {
            panic!(
                "platform contract mismatch in {}: expected 1.x found {}",
                path.display(),
                manifest.platform_contract_version
            );
        }
        if !manifest.runtime_contract_version.starts_with("2.") {
            panic!(
                "runtime contract mismatch in {}: expected 2.x found {}",
                path.display(),
                manifest.runtime_contract_version
            );
        }
        if manifest.ui.entry.trim().is_empty() || manifest.ui.routes.is_empty() {
            panic!("ui entry and routes must be declared in {}", path.display());
        }
        if manifest
            .ui
            .routes
            .iter()
            .any(|route| route.trim().is_empty())
        {
            panic!("ui routes must not be empty in {}", path.display());
        }
        if !manifest
            .required_platform_contracts
            .iter()
            .any(|contract| contract == "schemas/contracts/v1/plugin-module-v1.json")
        {
            panic!("plugin contract reference missing in {}", path.display());
        }
        if manifest
            .runtime_targets
            .iter()
            .any(|target| target != "pwa" && target != "tauri")
        {
            panic!(
                "unsupported runtime target in {}: {:?}",
                path.display(),
                manifest.runtime_targets
            );
        }
        if !manifest
            .runtime_targets
            .iter()
            .any(|target| target == "pwa")
        {
            panic!("baseline PWA target must be declared in {}", path.display());
        }
        manifests.push(manifest);
    }

    manifests.sort_by(|a, b| a.app_id.cmp(&b.app_id));
    let json = serde_json::to_string_pretty(&manifests).expect("serialize app manifest catalog");
    let manifest_metadata_consts = manifests
        .iter()
        .map(render_manifest_metadata_const)
        .collect::<Vec<_>>()
        .join("\n\n");
    let generated = format!(
        "/// Build-time generated app manifest catalog JSON.\n\
pub const APP_MANIFEST_CATALOG_JSON: &str = r##\"{}\"##;\n\n{}\n",
        json, manifest_metadata_consts
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let out_file = out_dir.join("app_catalog_generated.rs");
    fs::write(&out_file, generated)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_file.display()));
}

fn render_manifest_metadata_const(manifest: &AppManifest) -> String {
    let ident = manifest
        .app_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let requested_capabilities = manifest
        .requested_capabilities
        .iter()
        .map(|capability| match capability.as_str() {
            "window" => "AppCapability::Window",
            "state" => "AppCapability::State",
            "config" => "AppCapability::Config",
            "theme" => "AppCapability::Theme",
            "notifications" => "AppCapability::Notifications",
            "ipc" => "AppCapability::Ipc",
            "external-url" => "AppCapability::ExternalUrl",
            "commands" => "AppCapability::Commands",
            other => panic!(
                "unsupported requested capability `{other}` in manifest {}",
                manifest.app_id
            ),
        })
        .collect::<Vec<_>>()
        .join(", ");
    let suspend_policy = match manifest.suspend_policy.as_str() {
        "never" => "SuspendPolicy::Never",
        "on-minimize" => "SuspendPolicy::OnMinimize",
        other => panic!(
            "unsupported suspend policy `{other}` in manifest {}",
            manifest.app_id
        ),
    };
    let ui_routes = render_string_array(&manifest.ui.routes);
    let required_platform_contracts = render_string_array(&manifest.required_platform_contracts);
    let service_dependencies = render_string_array(&manifest.service_dependencies);
    let workflow_dependencies = render_string_array(&manifest.workflow_dependencies);
    let host_requirements = render_string_array(&manifest.host_requirements);
    let runtime_targets = render_string_array(&manifest.runtime_targets);
    let permissions = render_string_array(&manifest.permissions);
    let plugin_id = format!("{:?}", manifest.plugin_id);
    let version = format!("{:?}", manifest.version);
    let platform_contract_version = format!("{:?}", manifest.platform_contract_version);
    let runtime_contract_version = format!("{:?}", manifest.runtime_contract_version);
    let ui_entry = format!("{:?}", manifest.ui.entry);

    format!(
        "const {ident}_MANIFEST: GeneratedAppManifestMetadata = GeneratedAppManifestMetadata {{
    plugin_id: {plugin_id},
    display_name: \"{display_name}\",
    version: {version},
    platform_contract_version: {platform_contract_version},
    runtime_contract_version: {runtime_contract_version},
    ui_entry: {ui_entry},
    ui_routes: &{ui_routes},
    requested_capabilities: &[{requested_capabilities}],
    required_platform_contracts: &{required_platform_contracts},
    service_dependencies: &{service_dependencies},
    workflow_dependencies: &{workflow_dependencies},
    host_requirements: &{host_requirements},
    runtime_targets: &{runtime_targets},
    permissions: &{permissions},
    single_instance: {single_instance},
    suspend_policy: {suspend_policy},
    show_in_launcher: {show_in_launcher},
    show_on_desktop: {show_on_desktop},
    window_defaults: ({window_width}, {window_height}),
}};",
        ident = ident,
        plugin_id = plugin_id,
        display_name = manifest.display_name,
        version = version,
        platform_contract_version = platform_contract_version,
        runtime_contract_version = runtime_contract_version,
        ui_entry = ui_entry,
        ui_routes = ui_routes,
        requested_capabilities = requested_capabilities,
        required_platform_contracts = required_platform_contracts,
        service_dependencies = service_dependencies,
        workflow_dependencies = workflow_dependencies,
        host_requirements = host_requirements,
        runtime_targets = runtime_targets,
        permissions = permissions,
        single_instance = manifest.single_instance,
        suspend_policy = suspend_policy,
        show_in_launcher = manifest.launcher.show_in_launcher,
        show_on_desktop = manifest.launcher.show_on_desktop,
        window_width = manifest.window_defaults.width,
        window_height = manifest.window_defaults.height,
    )
}

fn render_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}
