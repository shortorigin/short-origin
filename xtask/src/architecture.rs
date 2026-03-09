use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::common::workspace_root;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Plane {
    Enterprise,
    Schemas,
    Shared,
    Platform,
    Services,
    Workflows,
    Ui,
    Infrastructure,
    Agents,
    Testing,
    Docs,
    Github,
    Xtask,
    WorkItems,
    Root,
    Unknown,
}

impl Plane {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Enterprise => "enterprise",
            Self::Schemas => "schemas",
            Self::Shared => "shared",
            Self::Platform => "platform",
            Self::Services => "services",
            Self::Workflows => "workflows",
            Self::Ui => "ui",
            Self::Infrastructure => "infrastructure",
            Self::Agents => "agents",
            Self::Testing => "testing",
            Self::Docs => "docs",
            Self::Github => ".github",
            Self::Xtask => "xtask",
            Self::WorkItems => "work-items",
            Self::Root => "root",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
struct ManifestDependency {
    section: String,
    name: String,
    target_path: String,
    plane: Plane,
}

#[derive(Debug)]
struct MemberAudit {
    member_path: String,
    plane: Plane,
    dependencies: Vec<ManifestDependency>,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command] if command == "audit-boundaries" => audit_boundaries(),
        _ => Err(help()),
    }
}

fn audit_boundaries() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let audits = audit_workspace_members(&workspace_root)?;
    let defects = collect_boundary_defects(&workspace_root, &audits)?;

    if defects.is_empty() {
        println!("architecture boundary audit passed");
        Ok(())
    } else {
        Err(format!(
            "architecture boundary audit found {} defect(s): {}",
            defects.len(),
            defects.join("; ")
        ))
    }
}

fn audit_workspace_members(workspace_root: &Path) -> Result<Vec<MemberAudit>, String> {
    let root_manifest = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .map_err(|error| format!("failed to read workspace Cargo.toml: {error}"))?;
    let root_value: toml::Value = toml::from_str(&root_manifest)
        .map_err(|error| format!("failed to parse workspace Cargo.toml: {error}"))?;
    let members = root_value
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "workspace members are missing from root Cargo.toml".to_string())?;

    members
        .iter()
        .filter_map(toml::Value::as_str)
        .map(|member| audit_member(workspace_root, member))
        .collect()
}

fn audit_member(workspace_root: &Path, member: &str) -> Result<MemberAudit, String> {
    let manifest_path = workspace_root.join(member).join("Cargo.toml");
    let manifest_raw = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest: toml::Value = toml::from_str(&manifest_raw)
        .map_err(|error| format!("failed to parse `{}`: {error}", manifest_path.display()))?;
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| format!("manifest path `{}` has no parent", manifest_path.display()))?;

    Ok(MemberAudit {
        member_path: member.to_string(),
        plane: classify_member_path(member),
        dependencies: collect_manifest_dependencies(workspace_root, manifest_dir, &manifest)?,
    })
}

fn collect_manifest_dependencies(
    workspace_root: &Path,
    manifest_dir: &Path,
    manifest: &toml::Value,
) -> Result<Vec<ManifestDependency>, String> {
    let mut dependencies = Vec::new();

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = manifest.get(section).and_then(toml::Value::as_table) {
            dependencies.extend(parse_dependency_table(
                workspace_root,
                manifest_dir,
                section,
                table,
            )?);
        }
    }

    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for (target_name, value) in targets {
            if let Some(target_table) = value.as_table() {
                for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(table) = target_table.get(section).and_then(toml::Value::as_table) {
                        let scoped = format!("target.{target_name}.{section}");
                        dependencies.extend(parse_dependency_table(
                            workspace_root,
                            manifest_dir,
                            &scoped,
                            table,
                        )?);
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

fn parse_dependency_table(
    workspace_root: &Path,
    manifest_dir: &Path,
    section: &str,
    table: &toml::Table,
) -> Result<Vec<ManifestDependency>, String> {
    let mut dependencies = Vec::new();
    let canonical_workspace_root =
        fs::canonicalize(workspace_root).unwrap_or_else(|_| workspace_root.to_path_buf());

    for (name, value) in table {
        let Some(path_value) = dependency_path(value) else {
            continue;
        };
        let candidate = manifest_dir.join(path_value);
        let resolved = fs::canonicalize(&candidate).map_err(|error| {
            format!(
                "failed to resolve dependency path `{}` from `{}`: {error}",
                candidate.display(),
                manifest_dir.display()
            )
        })?;
        if !resolved.starts_with(&canonical_workspace_root) {
            continue;
        }
        let relative = resolved
            .strip_prefix(&canonical_workspace_root)
            .map_err(|error| format!("failed to strip workspace prefix: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        dependencies.push(ManifestDependency {
            section: section.to_string(),
            name: name.clone(),
            plane: classify_member_path(&relative),
            target_path: relative,
        });
    }

    Ok(dependencies)
}

fn dependency_path(value: &toml::Value) -> Option<&str> {
    match value {
        toml::Value::Table(table) => table.get("path").and_then(toml::Value::as_str),
        _ => None,
    }
}

fn collect_boundary_defects(
    workspace_root: &Path,
    audits: &[MemberAudit],
) -> Result<Vec<String>, String> {
    let mut defects = Vec::new();

    for audit in audits {
        let allowed = allowed_planes(audit.plane);
        for dependency in &audit.dependencies {
            if !allowed.contains(&dependency.plane) {
                defects.push(format!(
                    "member `{}` in plane `{}` has disallowed {} dependency `{}` -> `{}` ({})",
                    audit.member_path,
                    audit.plane.as_str(),
                    dependency.section,
                    dependency.name,
                    dependency.target_path,
                    dependency.plane.as_str()
                ));
            }
        }
    }

    defects.extend(scan_for_direct_surreal_usage(workspace_root)?);
    Ok(defects)
}

fn scan_for_direct_surreal_usage(workspace_root: &Path) -> Result<Vec<String>, String> {
    let allowed_root = workspace_root.join("shared").join("surrealdb-access");
    let mut files = Vec::new();
    collect_rust_files(workspace_root, &mut files)?;

    let defects = files
        .into_iter()
        .filter(|path| !path.starts_with(&allowed_root))
        .filter(|path| {
            path.strip_prefix(workspace_root)
                .ok()
                .is_none_or(|relative| relative != Path::new("xtask/src/architecture.rs"))
        })
        .filter_map(|path| {
            let raw = fs::read_to_string(&path).ok()?;
            let surreal_marker = ["surreal", "db::"].concat();
            let query_marker = ['.', 'q', 'u', 'e', 'r', 'y', '(']
                .into_iter()
                .collect::<String>();
            if raw.contains(&surreal_marker) || raw.contains(&query_marker) {
                let relative = path
                    .strip_prefix(workspace_root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                Some(format!(
                    "direct SurrealDB usage must stay inside shared/surrealdb-access: `{relative}`"
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(defects)
}

fn collect_rust_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root)
        .map_err(|error| format!("failed to read directory `{}`: {error}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read dir entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if matches!(name, "target" | "node_modules" | ".git") {
                continue;
            }
            collect_rust_files(&path, output)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn allowed_planes(plane: Plane) -> &'static [Plane] {
    match plane {
        Plane::Enterprise => &[Plane::Enterprise, Plane::Schemas, Plane::Shared],
        Plane::Schemas => &[Plane::Schemas, Plane::Enterprise, Plane::Shared],
        Plane::Shared => &[Plane::Shared, Plane::Schemas, Plane::Enterprise],
        Plane::Platform => &[Plane::Platform, Plane::Schemas, Plane::Shared],
        Plane::Services => &[
            Plane::Services,
            Plane::Platform,
            Plane::Schemas,
            Plane::Shared,
            Plane::Enterprise,
        ],
        Plane::Workflows => &[
            Plane::Workflows,
            Plane::Services,
            Plane::Platform,
            Plane::Schemas,
            Plane::Shared,
            Plane::Enterprise,
        ],
        Plane::Ui => &[Plane::Ui, Plane::Platform, Plane::Schemas, Plane::Shared],
        Plane::Infrastructure => &[Plane::Infrastructure],
        Plane::Agents => &[
            Plane::Agents,
            Plane::Platform,
            Plane::Schemas,
            Plane::Shared,
            Plane::Enterprise,
            Plane::Workflows,
        ],
        Plane::Testing => &[
            Plane::Enterprise,
            Plane::Schemas,
            Plane::Shared,
            Plane::Platform,
            Plane::Services,
            Plane::Workflows,
            Plane::Ui,
            Plane::Infrastructure,
            Plane::Agents,
            Plane::Testing,
            Plane::Xtask,
        ],
        Plane::Docs => &[Plane::Docs],
        Plane::Github => &[Plane::Github],
        Plane::Xtask => &[Plane::Xtask, Plane::Platform, Plane::Schemas, Plane::Shared],
        Plane::WorkItems => &[Plane::WorkItems],
        Plane::Root => &[Plane::Root],
        Plane::Unknown => &[],
    }
}

fn classify_member_path(member: &str) -> Plane {
    if member == "xtask" {
        return Plane::Xtask;
    }
    if member.starts_with("platform/wasmcloud/smoke-tests") {
        return Plane::Testing;
    }

    match member.split('/').next().unwrap_or_default() {
        "enterprise" => Plane::Enterprise,
        "schemas" => Plane::Schemas,
        "shared" => Plane::Shared,
        "platform" => Plane::Platform,
        "services" => Plane::Services,
        "workflows" => Plane::Workflows,
        "ui" => Plane::Ui,
        "infrastructure" => Plane::Infrastructure,
        "agents" => Plane::Agents,
        "testing" => Plane::Testing,
        _ => Plane::Unknown,
    }
}

pub(crate) fn classify_repo_path(path: &str) -> Plane {
    if path.is_empty() {
        return Plane::Unknown;
    }
    if path.starts_with(".github/") {
        return Plane::Github;
    }
    if path.starts_with("docs/")
        || matches!(
            path,
            "README.md"
                | "ARCHITECTURE.md"
                | "CONTRIBUTING.md"
                | "DEVELOPMENT_MODEL.md"
                | "AGENTS.md"
        )
    {
        return Plane::Docs;
    }
    if path.starts_with("testing/") {
        return Plane::Testing;
    }
    if path.starts_with("work-items/") {
        return Plane::WorkItems;
    }
    if path.starts_with("xtask/") {
        return Plane::Xtask;
    }
    if path.starts_with("platform/wasmcloud/smoke-tests/") {
        return Plane::Testing;
    }
    if path.starts_with("infrastructure/") {
        return Plane::Infrastructure;
    }

    let mut components = Path::new(path).components();
    match components.next() {
        Some(Component::Normal(component)) => match component.to_string_lossy().as_ref() {
            "enterprise" => Plane::Enterprise,
            "schemas" => Plane::Schemas,
            "shared" => Plane::Shared,
            "platform" => Plane::Platform,
            "services" => Plane::Services,
            "workflows" => Plane::Workflows,
            "ui" => Plane::Ui,
            "agents" => Plane::Agents,
            _ => Plane::Root,
        },
        _ => Plane::Root,
    }
}

pub(crate) fn planes_for_paths<'a>(paths: impl IntoIterator<Item = &'a str>) -> BTreeSet<Plane> {
    paths
        .into_iter()
        .map(classify_repo_path)
        .filter(|plane| *plane != Plane::Unknown)
        .collect()
}

fn help() -> String {
    "usage: cargo xtask architecture audit-boundaries".to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        audit_workspace_members, classify_member_path, classify_repo_path, planes_for_paths, Plane,
    };
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask-architecture-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock drift")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create temp dir");
        base
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn classify_member_path_treats_smoke_tests_as_testing() {
        assert_eq!(
            classify_member_path("platform/wasmcloud/smoke-tests"),
            Plane::Testing
        );
        assert_eq!(classify_member_path("ui/crates/site"), Plane::Ui);
    }

    #[test]
    fn classify_repo_path_maps_docs_and_github_files() {
        assert_eq!(classify_repo_path("README.md"), Plane::Docs);
        assert_eq!(
            classify_repo_path(".github/workflows/governance.yml"),
            Plane::Github
        );
        assert_eq!(
            classify_repo_path("services/finance-service/src/lib.rs"),
            Plane::Services
        );
    }

    #[test]
    fn planes_for_paths_collects_unique_planes() {
        let planes = planes_for_paths([
            "ui/crates/site/src/lib.rs",
            "schemas/contracts/v1/plugin-module-v1.json",
            "README.md",
        ]);
        assert_eq!(
            planes,
            BTreeSet::from([Plane::Ui, Plane::Schemas, Plane::Docs])
        );
    }

    #[test]
    fn audit_workspace_members_accepts_ui_to_platform_dependency() {
        let root = unique_temp_dir("allow");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app", "platform/sdk"]
"#,
        );
        write_file(
            &root.join("ui/app/Cargo.toml"),
            r#"
[package]
name = "ui-app"
version = "0.1.0"
edition = "2021"

[dependencies]
sdk = { path = "../../platform/sdk" }
"#,
        );
        write_file(
            &root.join("platform/sdk/Cargo.toml"),
            r#"
[package]
name = "sdk"
version = "0.1.0"
edition = "2021"
"#,
        );

        let audits = audit_workspace_members(&root).expect("audit workspace");
        let ui_audit = audits
            .iter()
            .find(|audit| audit.member_path == "ui/app")
            .expect("ui audit");
        assert_eq!(ui_audit.dependencies.len(), 1);
        assert_eq!(ui_audit.dependencies[0].plane, Plane::Platform);
    }

    #[test]
    fn audit_workspace_members_detects_ui_to_services_dependency() {
        let root = unique_temp_dir("deny");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app", "services/backend"]
"#,
        );
        write_file(
            &root.join("ui/app/Cargo.toml"),
            r#"
[package]
name = "ui-app"
version = "0.1.0"
edition = "2021"

[dependencies]
backend = { path = "../../services/backend" }
"#,
        );
        write_file(
            &root.join("services/backend/Cargo.toml"),
            r#"
[package]
name = "backend"
version = "0.1.0"
edition = "2021"
"#,
        );

        let audits = audit_workspace_members(&root).expect("audit workspace");
        let ui_audit = audits
            .iter()
            .find(|audit| audit.member_path == "ui/app")
            .expect("ui audit");
        assert_eq!(ui_audit.dependencies[0].plane, Plane::Services);
    }
}
