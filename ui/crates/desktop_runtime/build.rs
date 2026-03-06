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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WallpaperCatalog {
    schema_version: u32,
    wallpapers: Vec<WallpaperManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WallpaperManifest {
    wallpaper_id: String,
    display_name: String,
    note: String,
    media_kind: String,
    primary_path: String,
    poster_path: Option<String>,
    featured: bool,
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

    let wallpaper_catalog_path = crate_root
        .join("..")
        .join("..")
        .join("assets")
        .join("wallpapers")
        .join("catalog.toml");
    println!(
        "cargo:rerun-if-changed={}",
        wallpaper_catalog_path.display()
    );
    let raw = fs::read_to_string(&wallpaper_catalog_path).unwrap_or_else(|err| {
        panic!(
            "failed to read wallpaper catalog {}: {err}",
            wallpaper_catalog_path.display()
        )
    });
    let mut catalog: WallpaperCatalog = toml::from_str(&raw).unwrap_or_else(|err| {
        panic!(
            "failed to parse wallpaper catalog {}: {err}",
            wallpaper_catalog_path.display()
        )
    });
    if catalog.schema_version != 1 {
        panic!(
            "wallpaper catalog schema mismatch in {}: expected 1 found {}",
            wallpaper_catalog_path.display(),
            catalog.schema_version
        );
    }
    validate_wallpaper_catalog(&crate_root, &catalog);
    catalog
        .wallpapers
        .sort_by(|a, b| a.wallpaper_id.cmp(&b.wallpaper_id));
    let json =
        serde_json::to_string_pretty(&catalog.wallpapers).expect("serialize wallpaper catalog");
    let generated = format!(
        "/// Build-time generated built-in wallpaper catalog JSON.\n\
pub const BUILTIN_WALLPAPER_CATALOG_JSON: &str = r##\"{}\"##;\n",
        json
    );
    let out_file = out_dir.join("wallpaper_catalog_generated.rs");
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
            "wallpaper" => "AppCapability::Wallpaper",
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

fn validate_wallpaper_catalog(crate_root: &Path, catalog: &WallpaperCatalog) {
    let assets_root = crate_root
        .join("..")
        .join("..")
        .join("assets")
        .join("wallpapers");
    let mut seen = std::collections::BTreeSet::new();
    for entry in &catalog.wallpapers {
        if !seen.insert(entry.wallpaper_id.clone()) {
            panic!("duplicate wallpaper id: {}", entry.wallpaper_id);
        }
        let primary = assets_root.join(&entry.primary_path);
        if !primary.exists() {
            panic!("wallpaper asset missing: {}", primary.display());
        }
        println!("cargo:rerun-if-changed={}", primary.display());
        match entry.media_kind.as_str() {
            "static-image" | "animated-image" | "video" | "svg" => {}
            other => panic!("unsupported wallpaper media_kind `{other}`"),
        }
        if entry.media_kind == "video" && entry.poster_path.is_none() {
            panic!(
                "video wallpaper {} must declare poster_path",
                entry.wallpaper_id
            );
        }
        if let Some(poster_path) = &entry.poster_path {
            let poster = assets_root.join(poster_path);
            if !poster.exists() {
                panic!("wallpaper poster missing: {}", poster.display());
            }
            println!("cargo:rerun-if-changed={}", poster.display());
        }
    }
    if !catalog.wallpapers.iter().any(|entry| entry.featured) {
        panic!("wallpaper catalog must declare at least one featured entry");
    }
}
