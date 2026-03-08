use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WindowDefaults {
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppManifest {
    schema_version: u32,
    app_id: String,
    display_name: String,
    version: String,
    runtime_contract_version: String,
    requested_capabilities: Vec<String>,
    single_instance: bool,
    suspend_policy: String,
    show_in_launcher: bool,
    show_on_desktop: bool,
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
        if !manifest.runtime_contract_version.starts_with("2.") {
            panic!(
                "runtime contract mismatch in {}: expected 2.x found {}",
                path.display(),
                manifest.runtime_contract_version
            );
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

    format!(
        "const {ident}_MANIFEST: GeneratedAppManifestMetadata = GeneratedAppManifestMetadata {{
    display_name: \"{display_name}\",
    requested_capabilities: &[{requested_capabilities}],
    single_instance: {single_instance},
    suspend_policy: {suspend_policy},
    show_in_launcher: {show_in_launcher},
    show_on_desktop: {show_on_desktop},
    window_defaults: ({window_width}, {window_height}),
}};",
        ident = ident,
        display_name = manifest.display_name,
        requested_capabilities = requested_capabilities,
        single_instance = manifest.single_instance,
        suspend_policy = suspend_policy,
        show_in_launcher = manifest.show_in_launcher,
        show_on_desktop = manifest.show_on_desktop,
        window_width = manifest.window_defaults.width,
        window_height = manifest.window_defaults.height,
    )
}
