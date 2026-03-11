use std::fs;
use std::path::{Path, PathBuf};

use jsonschema::{validator_for, Validator};
use serde::Deserialize;
use serde_json::Value;

use crate::common::workspace_root;

#[derive(Debug, Deserialize)]
struct PluginManifest {
    schema_version: u32,
    plugin_id: String,
    app_id: String,
    display_name: String,
    version: String,
    platform_contract_version: String,
    runtime_contract_version: String,
    ui: PluginUiContribution,
    requested_capabilities: Vec<String>,
    required_platform_contracts: Vec<String>,
    service_dependencies: Vec<String>,
    workflow_dependencies: Vec<String>,
    host_requirements: Vec<String>,
    runtime_targets: Vec<String>,
    permissions: Vec<String>,
    single_instance: bool,
    suspend_policy: String,
    launcher: PluginLauncher,
    window_defaults: PluginWindowDefaults,
}

#[derive(Debug, Deserialize)]
struct PluginUiContribution {
    entry: String,
    routes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PluginLauncher {
    show_in_launcher: bool,
    show_on_desktop: bool,
}

#[derive(Debug, Deserialize)]
struct PluginWindowDefaults {
    width: i32,
    height: i32,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command] if command == "validate-manifests" => validate_manifests_for_workspace(),
        [command, path] if command == "validate-manifests" => {
            validate_manifest_paths(vec![PathBuf::from(path)])
        }
        _ => Err(help()),
    }
}

fn validate_manifests_for_workspace() -> Result<(), String> {
    let root = workspace_root()?;
    validate_manifest_paths(discover_plugin_manifest_paths(&root))
}

fn discover_plugin_manifest_paths(workspace_root: &Path) -> Vec<PathBuf> {
    ["control_center", "settings", "terminal"]
        .iter()
        .map(|name| {
            workspace_root
                .join("ui")
                .join("crates")
                .join("apps")
                .join(name)
                .join("app.manifest.toml")
        })
        .collect()
}

fn validate_manifest_paths(paths: Vec<PathBuf>) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let schema_path = workspace_root.join("schemas/contracts/v1/plugin-module-v1.json");
    let schema_raw = fs::read_to_string(&schema_path)
        .map_err(|error| format!("failed to read `{}`: {error}", schema_path.display()))?;
    let schema_json: Value = serde_json::from_str(&schema_raw)
        .map_err(|error| format!("failed to parse `{}`: {error}", schema_path.display()))?;
    let validator =
        validator_for(&schema_json).map_err(|error| format!("schema error: {error}"))?;

    let mut defects = Vec::new();
    for path in paths {
        defects.extend(validate_manifest_path(&path, &validator)?);
    }

    if defects.is_empty() {
        println!("plugin manifest validation passed");
        Ok(())
    } else {
        Err(format!(
            "plugin manifest validation found {} defect(s): {}",
            defects.len(),
            defects.join("; ")
        ))
    }
}

fn validate_manifest_path(path: &Path, validator: &Validator) -> Result<Vec<String>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let toml_value: toml::Value = toml::from_str(&raw)
        .map_err(|error| format!("failed to parse `{}`: {error}", path.display()))?;
    let json_value = serde_json::to_value(&toml_value)
        .map_err(|error| format!("TOML conversion failed: {error}"))?;

    let mut defects = validator
        .iter_errors(&json_value)
        .map(|error| {
            format!(
                "{}: schema violation at `{}`: {}",
                path.display(),
                error.instance_path(),
                error
            )
        })
        .collect::<Vec<_>>();

    match toml::from_str::<PluginManifest>(&raw) {
        Ok(manifest) => defects.extend(validate_manifest_semantics(path, &manifest)),
        Err(error) => defects.push(format!(
            "{}: manifest deserialization failed: {error}",
            path.display()
        )),
    }
    Ok(defects)
}

fn validate_manifest_semantics(path: &Path, manifest: &PluginManifest) -> Vec<String> {
    let mut defects = Vec::new();
    let path = path.display();

    if manifest.schema_version != 1 {
        defects.push(format!(
            "{path}: expected schema_version 1, found {}",
            manifest.schema_version
        ));
    }
    if manifest.plugin_id != manifest.app_id {
        defects.push(format!(
            "{path}: v1 plugin_id `{}` must match app_id `{}`",
            manifest.plugin_id, manifest.app_id
        ));
    }
    if manifest.display_name.trim().is_empty() {
        defects.push(format!("{path}: display_name must not be empty"));
    }
    if manifest.version.trim().is_empty() {
        defects.push(format!("{path}: version must not be empty"));
    }
    if !manifest.platform_contract_version.starts_with("1.") {
        defects.push(format!(
            "{path}: platform_contract_version must be 1.x, found `{}`",
            manifest.platform_contract_version
        ));
    }
    if !manifest.runtime_contract_version.starts_with("2.") {
        defects.push(format!(
            "{path}: runtime_contract_version must be 2.x, found `{}`",
            manifest.runtime_contract_version
        ));
    }
    if manifest.ui.entry.trim().is_empty() {
        defects.push(format!("{path}: ui.entry must not be empty"));
    }
    if manifest.ui.routes.is_empty() {
        defects.push(format!("{path}: ui.routes must not be empty"));
    }
    if manifest
        .ui
        .routes
        .iter()
        .any(|route| route.trim().is_empty())
    {
        defects.push(format!("{path}: ui.routes must not contain empty entries"));
    }
    if !manifest
        .required_platform_contracts
        .iter()
        .any(|contract| contract == "schemas/contracts/v1/plugin-module-v1.json")
    {
        defects.push(format!(
            "{path}: required_platform_contracts must include `schemas/contracts/v1/plugin-module-v1.json`"
        ));
    }
    if manifest
        .runtime_targets
        .iter()
        .any(|target| target != "pwa" && target != "tauri")
    {
        defects.push(format!(
            "{path}: runtime_targets may only contain `pwa` and `tauri`: {:?}",
            manifest.runtime_targets
        ));
    }
    if !manifest
        .runtime_targets
        .iter()
        .any(|target| target == "pwa")
    {
        defects.push(format!(
            "{path}: runtime_targets must include baseline `pwa`"
        ));
    }
    if !matches!(manifest.suspend_policy.as_str(), "never" | "on-minimize") {
        defects.push(format!(
            "{path}: suspend_policy must be `never` or `on-minimize`, found `{}`",
            manifest.suspend_policy
        ));
    }
    if manifest.window_defaults.width < 320 || manifest.window_defaults.height < 240 {
        defects.push(format!(
            "{path}: window_defaults must be at least 320x240, found {}x{}",
            manifest.window_defaults.width, manifest.window_defaults.height
        ));
    }

    let _ = (
        &manifest.requested_capabilities,
        &manifest.service_dependencies,
        &manifest.workflow_dependencies,
        &manifest.host_requirements,
        &manifest.permissions,
        manifest.single_instance,
        manifest.launcher.show_in_launcher,
        manifest.launcher.show_on_desktop,
    );

    defects
}

fn help() -> String {
    "usage: cargo xtask plugin validate-manifests [path-to-manifest]".to_string()
}

#[cfg(test)]
mod tests {
    use super::{validate_manifest_path, PluginManifest};
    use jsonschema::{validator_for, Validator};
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask-plugin-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock drift")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create temp dir");
        base
    }

    fn load_validator() -> Validator {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let schema_raw =
            fs::read_to_string(workspace_root.join("schemas/contracts/v1/plugin-module-v1.json"))
                .expect("schema raw");
        let schema_json: Value = serde_json::from_str(&schema_raw).expect("schema json");
        validator_for(&schema_json).expect("validator")
    }

    fn write_manifest(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write manifest");
    }

    const VALID_MANIFEST: &str = r#"
schema_version = 1
plugin_id = "system.test"
app_id = "system.test"
display_name = "System Test"
version = "0.1.0"
platform_contract_version = "1.0.0"
runtime_contract_version = "2.0.0"
requested_capabilities = ["window", "state"]
required_platform_contracts = ["schemas/contracts/v1/plugin-module-v1.json"]
service_dependencies = []
workflow_dependencies = []
host_requirements = ["browser-storage:required"]
runtime_targets = ["pwa", "tauri"]
permissions = ["shell.mount", "shell.window"]
single_instance = true
suspend_policy = "never"

[ui]
entry = "desktop_app_test"
routes = ["/apps/test"]

[launcher]
show_in_launcher = true
show_on_desktop = true

[window_defaults]
width = 640
height = 480
"#;

    #[test]
    fn validates_built_in_manifests() {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let validator = load_validator();
        for path in [
            workspace_root.join("ui/crates/apps/control_center/app.manifest.toml"),
            workspace_root.join("ui/crates/apps/settings/app.manifest.toml"),
            workspace_root.join("ui/crates/apps/terminal/app.manifest.toml"),
        ] {
            let defects = validate_manifest_path(&path, &validator).expect("validate manifest");
            assert!(
                defects.is_empty(),
                "expected no defects for {}: {defects:?}",
                path.display()
            );
        }
    }

    #[test]
    fn rejects_missing_required_fields() {
        let root = unique_temp_dir("missing-fields");
        let path = root.join("app.manifest.toml");
        write_manifest(
            &path,
            VALID_MANIFEST
                .replace("runtime_targets = [\"pwa\", \"tauri\"]\n", "")
                .as_str(),
        );
        let defects = validate_manifest_path(&path, &load_validator()).expect("validate");
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("schema violation")),
            "expected schema defect, got {defects:?}"
        );
    }

    #[test]
    fn rejects_invalid_runtime_targets() {
        let root = unique_temp_dir("bad-runtime");
        let path = root.join("app.manifest.toml");
        write_manifest(
            &path,
            &VALID_MANIFEST.replace(
                "runtime_targets = [\"pwa\", \"tauri\"]",
                "runtime_targets = [\"browser\", \"tauri\"]",
            ),
        );
        let defects = validate_manifest_path(&path, &load_validator()).expect("validate");
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("runtime_targets")),
            "expected runtime_targets defect, got {defects:?}"
        );
    }

    #[test]
    fn rejects_invalid_contract_version_reference() {
        let root = unique_temp_dir("bad-contract");
        let path = root.join("app.manifest.toml");
        write_manifest(
            &path,
            &VALID_MANIFEST.replace(
                "platform_contract_version = \"1.0.0\"",
                "platform_contract_version = \"2.0.0\"",
            ),
        );
        let defects = validate_manifest_path(&path, &load_validator()).expect("validate");
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("platform_contract_version must be 1.x")),
            "expected contract version defect, got {defects:?}"
        );
    }

    #[test]
    fn parses_valid_manifest_shape() {
        let manifest: PluginManifest = toml::from_str(VALID_MANIFEST).expect("parse manifest");
        assert_eq!(manifest.plugin_id, "system.test");
        assert_eq!(manifest.ui.entry, "desktop_app_test");
    }
}
