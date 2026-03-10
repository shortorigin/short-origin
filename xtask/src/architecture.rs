use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use crate::common::workspace_root;
use regex::Regex;
use serde::Deserialize;

const FOUNDATIONAL_WORKSPACE_DEPENDENCIES: &[&str] = &[
    "futures",
    "quick-xml",
    "reqwest",
    "serde",
    "serde_json",
    "tempfile",
    "thiserror",
    "tokio",
    "tracing",
];

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
    requested_path: String,
    target_path: Option<String>,
    plane: Option<Plane>,
    within_workspace: bool,
    resolution_error: Option<String>,
}

#[derive(Debug)]
struct MemberAudit {
    member_path: String,
    plane: Plane,
    dependency_declarations: Vec<ManifestDependencyDeclaration>,
    dependencies: Vec<ManifestDependency>,
    documented_workspace_exceptions: BTreeMap<String, String>,
}

#[derive(Debug)]
struct ManifestDependencyDeclaration {
    section: String,
    name: String,
    uses_workspace_inheritance: bool,
    explicit_version: Option<String>,
}

#[derive(Debug)]
struct RootWorkspaceManifest {
    members: BTreeSet<String>,
    workspace_dependencies: BTreeSet<String>,
}

#[derive(Debug)]
struct MetadataWorkspace {
    packages: BTreeMap<String, MetadataPackageAudit>,
}

#[derive(Debug)]
struct MetadataPackageAudit {
    id: String,
    name: String,
    member_path: String,
    plane: Plane,
    crate_names: BTreeSet<String>,
    direct_dependencies: Vec<MetadataDependencyAudit>,
}

#[derive(Debug)]
struct MetadataDependencyAudit {
    package_id: String,
    crate_name: String,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    resolve: Option<CargoResolve>,
    workspace_root: String,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    manifest_path: String,
    targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    kind: Vec<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoResolveNode>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveNode {
    id: String,
    deps: Vec<CargoResolveDep>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveDep {
    name: String,
    pkg: String,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command] if command == "audit-boundaries" => audit_boundaries(),
        _ => Err(help()),
    }
}

fn audit_boundaries() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let root_manifest = read_root_workspace_manifest(&workspace_root)?;
    let audits = audit_workspace_members(&workspace_root)?;
    let metadata = load_workspace_metadata(&workspace_root)?;
    let defects = collect_boundary_defects(&workspace_root, &root_manifest, &audits, &metadata)?;

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
    let root_manifest = read_root_workspace_manifest(workspace_root)?;

    root_manifest
        .members
        .iter()
        .map(String::as_str)
        .map(|member| audit_member(workspace_root, member))
        .collect()
}

fn read_root_workspace_manifest(workspace_root: &Path) -> Result<RootWorkspaceManifest, String> {
    let root_manifest = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .map_err(|error| format!("failed to read workspace Cargo.toml: {error}"))?;
    let root_value: toml::Value = toml::from_str(&root_manifest)
        .map_err(|error| format!("failed to parse workspace Cargo.toml: {error}"))?;
    let workspace = root_value
        .get("workspace")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "workspace table is missing from root Cargo.toml".to_string())?;

    let members = workspace
        .get("members")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "workspace members are missing from root Cargo.toml".to_string())?
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let workspace_dependencies = workspace
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|dependencies| dependencies.keys().cloned().collect())
        .unwrap_or_default();

    Ok(RootWorkspaceManifest {
        members,
        workspace_dependencies,
    })
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
        dependency_declarations: collect_manifest_declarations(&manifest),
        dependencies: collect_manifest_dependencies(workspace_root, manifest_dir, &manifest)?,
        documented_workspace_exceptions: collect_documented_workspace_exceptions(&manifest),
    })
}

fn collect_manifest_declarations(manifest: &toml::Value) -> Vec<ManifestDependencyDeclaration> {
    let mut declarations = Vec::new();

    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = manifest.get(section).and_then(toml::Value::as_table) {
            declarations.extend(parse_dependency_declaration_table(section, table));
        }
    }

    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for (target_name, value) in targets {
            if let Some(target_table) = value.as_table() {
                for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(table) = target_table.get(section).and_then(toml::Value::as_table) {
                        let scoped = format!("target.{target_name}.{section}");
                        declarations.extend(parse_dependency_declaration_table(&scoped, table));
                    }
                }
            }
        }
    }

    declarations
}

fn parse_dependency_declaration_table(
    section: &str,
    table: &toml::Table,
) -> Vec<ManifestDependencyDeclaration> {
    let mut declarations = Vec::new();

    for (name, value) in table {
        let (uses_workspace_inheritance, explicit_version) = match value {
            toml::Value::Table(inner) => (
                inner
                    .get("workspace")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
                inner
                    .get("version")
                    .and_then(toml::Value::as_str)
                    .map(str::to_owned),
            ),
            toml::Value::String(version) => (false, Some(version.clone())),
            _ => (false, None),
        };

        declarations.push(ManifestDependencyDeclaration {
            section: section.to_string(),
            name: name.clone(),
            uses_workspace_inheritance,
            explicit_version,
        });
    }

    declarations
}

fn collect_documented_workspace_exceptions(manifest: &toml::Value) -> BTreeMap<String, String> {
    manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("metadata"))
        .and_then(toml::Value::as_table)
        .and_then(|metadata| metadata.get("origin"))
        .and_then(toml::Value::as_table)
        .and_then(|origin| origin.get("workspace-dependency-exceptions"))
        .and_then(toml::Value::as_table)
        .map(|exceptions| {
            exceptions
                .iter()
                .filter_map(|(name, value)| {
                    value
                        .as_str()
                        .map(str::trim)
                        .filter(|reason| !reason.is_empty())
                        .map(|reason| (name.clone(), reason.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
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
        match fs::canonicalize(&candidate) {
            Ok(resolved) if resolved.starts_with(&canonical_workspace_root) => {
                let relative = resolved
                    .strip_prefix(&canonical_workspace_root)
                    .map_err(|error| format!("failed to strip workspace prefix: {error}"))?
                    .to_string_lossy()
                    .replace('\\', "/");
                dependencies.push(ManifestDependency {
                    section: section.to_string(),
                    name: name.clone(),
                    requested_path: path_value.to_string(),
                    target_path: Some(relative.clone()),
                    plane: Some(classify_member_path(&relative)),
                    within_workspace: true,
                    resolution_error: None,
                });
            }
            Ok(_) => {
                dependencies.push(ManifestDependency {
                    section: section.to_string(),
                    name: name.clone(),
                    requested_path: path_value.to_string(),
                    target_path: None,
                    plane: None,
                    within_workspace: false,
                    resolution_error: None,
                });
            }
            Err(error) => {
                dependencies.push(ManifestDependency {
                    section: section.to_string(),
                    name: name.clone(),
                    requested_path: path_value.to_string(),
                    target_path: None,
                    plane: None,
                    within_workspace: true,
                    resolution_error: Some(format!(
                        "failed to resolve dependency path `{}` from `{}`: {error}",
                        candidate.display(),
                        manifest_dir.display()
                    )),
                });
            }
        }
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
    root_manifest: &RootWorkspaceManifest,
    audits: &[MemberAudit],
    metadata: &MetadataWorkspace,
) -> Result<Vec<String>, String> {
    let mut defects = scan_for_manifest_governance_defects(root_manifest, audits);

    for audit in audits {
        let allowed = allowed_planes(audit.plane);
        for dependency in &audit.dependencies {
            if let (Some(target_path), Some(plane)) = (&dependency.target_path, dependency.plane) {
                if !allowed.contains(&plane) {
                    defects.push(format!(
                        "member `{}` in plane `{}` has disallowed {} dependency `{}` -> `{}` ({})",
                        audit.member_path,
                        audit.plane.as_str(),
                        dependency.section,
                        dependency.name,
                        target_path,
                        plane.as_str()
                    ));
                }
            }
        }
    }

    defects.extend(scan_for_invalid_workspace_path_dependencies(
        root_manifest,
        audits,
    ));
    defects.extend(scan_for_direct_surreal_usage(workspace_root)?);
    defects.extend(scan_for_transitive_workspace_violations(metadata));
    defects.extend(scan_for_workspace_import_violations(
        workspace_root,
        metadata,
    )?);
    Ok(defects)
}

fn scan_for_manifest_governance_defects(
    root_manifest: &RootWorkspaceManifest,
    audits: &[MemberAudit],
) -> Vec<String> {
    let foundational = FOUNDATIONAL_WORKSPACE_DEPENDENCIES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut defects = Vec::new();

    for audit in audits {
        for declaration in &audit.dependency_declarations {
            if declaration.uses_workspace_inheritance
                && !root_manifest
                    .workspace_dependencies
                    .contains(&declaration.name)
            {
                defects.push(format!(
                    "member `{}` uses workspace dependency `{}` in `{}` but root `[workspace.dependencies]` does not define it",
                    audit.member_path, declaration.name, declaration.section
                ));
            }
            if foundational.contains(declaration.name.as_str())
                && declaration.explicit_version.is_some()
                && !audit
                    .documented_workspace_exceptions
                    .contains_key(&declaration.name)
            {
                defects.push(format!(
                    "member `{}` pins foundational dependency `{}` in `{}` instead of workspace inheritance",
                    audit.member_path, declaration.name, declaration.section
                ));
            }
        }
    }

    defects
}

fn scan_for_invalid_workspace_path_dependencies(
    root_manifest: &RootWorkspaceManifest,
    audits: &[MemberAudit],
) -> Vec<String> {
    let mut defects = Vec::new();

    for audit in audits {
        for dependency in &audit.dependencies {
            if let Some(error) = &dependency.resolution_error {
                defects.push(format!(
                    "member `{}` has unresolved {} path dependency `{}` -> `{}`: {error}",
                    audit.member_path,
                    dependency.section,
                    dependency.name,
                    dependency.requested_path
                ));
                continue;
            }
            if !dependency.within_workspace {
                continue;
            }
            let Some(target_path) = &dependency.target_path else {
                continue;
            };
            let member_path = target_path
                .strip_suffix("/Cargo.toml")
                .unwrap_or(target_path);
            if root_manifest.members.contains(member_path) {
                continue;
            }
            defects.push(format!(
                "member `{}` has {} path dependency `{}` -> `{}` that is not listed in workspace members",
                audit.member_path, dependency.section, dependency.name, member_path
            ));
        }
    }

    defects
}

fn load_workspace_metadata(workspace_root: &Path) -> Result<MetadataWorkspace, String> {
    let output = Command::new("cargo")
        .current_dir(workspace_root)
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|error| format!("failed to run `cargo metadata`: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("`cargo metadata` failed: {stderr}"));
    }

    let raw = String::from_utf8(output.stdout)
        .map_err(|error| format!("`cargo metadata` output was not valid UTF-8: {error}"))?;
    let metadata: CargoMetadata = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse `cargo metadata` output: {error}"))?;

    build_metadata_workspace(&metadata)
}

fn build_metadata_workspace(metadata: &CargoMetadata) -> Result<MetadataWorkspace, String> {
    let workspace_root = Path::new(&metadata.workspace_root);
    let mut package_index = BTreeMap::new();

    for package in &metadata.packages {
        let manifest_path = Path::new(&package.manifest_path);
        if !manifest_path.starts_with(workspace_root) {
            continue;
        }
        let manifest_dir = manifest_path.parent().ok_or_else(|| {
            format!(
                "manifest path `{}` returned by cargo metadata has no parent",
                package.manifest_path
            )
        })?;
        let relative = manifest_dir
            .strip_prefix(workspace_root)
            .map_err(|error| {
                format!(
                    "failed to strip workspace root from `{}`: {error}",
                    manifest_dir.display()
                )
            })?
            .to_string_lossy()
            .replace('\\', "/");

        let crate_names = package
            .targets
            .iter()
            .filter(|target| {
                target.kind.iter().any(|kind| {
                    matches!(
                        kind.as_str(),
                        "lib" | "rlib" | "cdylib" | "dylib" | "proc-macro"
                    )
                })
            })
            .map(|target| target.name.clone())
            .collect::<BTreeSet<_>>();

        package_index.insert(
            package.id.clone(),
            MetadataPackageAudit {
                id: package.id.clone(),
                name: package.name.clone(),
                member_path: relative.clone(),
                plane: classify_member_path(&relative),
                crate_names,
                direct_dependencies: Vec::new(),
            },
        );
    }

    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| "`cargo metadata` did not return a dependency graph".to_string())?;

    for node in &resolve.nodes {
        if !package_index.contains_key(&node.id) {
            continue;
        }

        let direct_dependencies = node
            .deps
            .iter()
            .filter_map(|dep| {
                package_index.get(&dep.pkg)?;
                Some(MetadataDependencyAudit {
                    package_id: dep.pkg.clone(),
                    crate_name: dep.name.clone(),
                })
            })
            .collect::<Vec<_>>();

        if let Some(package) = package_index.get_mut(&node.id) {
            package.direct_dependencies = direct_dependencies;
        }
    }

    Ok(MetadataWorkspace {
        packages: package_index,
    })
}

fn scan_for_transitive_workspace_violations(metadata: &MetadataWorkspace) -> Vec<String> {
    let mut defects = Vec::new();

    for package in metadata.packages.values() {
        let allowed = allowed_transitive_planes(package.plane);
        let mut queue = VecDeque::new();
        let mut seen = BTreeSet::from([package.id.clone()]);

        for dependency in &package.direct_dependencies {
            queue.push_back((dependency.package_id.clone(), vec![package.name.clone()]));
        }

        while let Some((package_id, mut path)) = queue.pop_front() {
            if !seen.insert(package_id.clone()) {
                continue;
            }

            let Some(target) = metadata.packages.get(&package_id) else {
                continue;
            };
            path.push(target.name.clone());

            if !allowed.contains(&target.plane) {
                defects.push(format!(
                    "member `{}` in plane `{}` reaches disallowed transitive plane `{}` via `{}`",
                    package.member_path,
                    package.plane.as_str(),
                    target.plane.as_str(),
                    path.join(" -> ")
                ));
                continue;
            }

            for dependency in &target.direct_dependencies {
                queue.push_back((dependency.package_id.clone(), path.clone()));
            }
        }
    }

    defects
}

fn scan_for_workspace_import_violations(
    workspace_root: &Path,
    metadata: &MetadataWorkspace,
) -> Result<Vec<String>, String> {
    let import_regex =
        Regex::new(r"(?m)^\s*(?:pub\s+use|use|extern\s+crate)\s+(?:::)?([A-Za-z_][A-Za-z0-9_]*)")
            .map_err(|error| format!("failed to compile workspace import regex: {error}"))?;

    let crate_to_package = metadata
        .packages
        .values()
        .flat_map(|package| {
            package
                .crate_names
                .iter()
                .cloned()
                .map(move |crate_name| (crate_name, package))
        })
        .collect::<BTreeMap<_, _>>();

    let mut defects = Vec::new();

    for package in metadata.packages.values() {
        let mut files = Vec::new();
        collect_rust_files(&workspace_root.join(&package.member_path), &mut files)?;
        let direct_crates = package
            .direct_dependencies
            .iter()
            .map(|dependency| dependency.crate_name.clone())
            .collect::<BTreeSet<_>>();
        let allowed = allowed_planes(package.plane);

        for path in files {
            let raw = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
            for captures in import_regex.captures_iter(&raw) {
                let Some(crate_name) = captures.get(1).map(|capture| capture.as_str()) else {
                    continue;
                };
                if matches!(crate_name, "crate" | "self" | "super") {
                    continue;
                }

                let Some(target) = crate_to_package.get(crate_name) else {
                    continue;
                };
                if target.id == package.id {
                    continue;
                }

                let relative = path
                    .strip_prefix(workspace_root)
                    .map_err(|error| {
                        format!(
                            "failed to strip workspace root from `{}`: {error}",
                            path.display()
                        )
                    })?
                    .to_string_lossy()
                    .replace('\\', "/");

                if !allowed.contains(&target.plane) {
                    defects.push(format!(
                        "source import in `{relative}` references workspace crate `{crate_name}` from disallowed plane `{}`",
                        target.plane.as_str()
                    ));
                    continue;
                }

                if !direct_crates.contains(crate_name) {
                    defects.push(format!(
                        "source import in `{relative}` references workspace crate `{crate_name}` without a direct dependency declaration"
                    ));
                }
            }
        }
    }

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

fn allowed_transitive_planes(plane: Plane) -> &'static [Plane] {
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
        || path.starts_with("plans/")
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
        audit_workspace_members, build_metadata_workspace, classify_member_path,
        classify_repo_path, planes_for_paths, read_root_workspace_manifest,
        scan_for_invalid_workspace_path_dependencies, scan_for_manifest_governance_defects,
        scan_for_transitive_workspace_violations, scan_for_workspace_import_violations,
        CargoMetadata, Plane,
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

    fn fixture_package(
        id: &str,
        name: &str,
        manifest_path: &str,
        lib_name: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": name,
            "manifest_path": manifest_path,
            "targets": [
                {
                    "kind": ["lib"],
                    "name": lib_name
                }
            ]
        })
    }

    fn fixture_metadata(
        root: &Path,
        packages: Vec<serde_json::Value>,
        nodes: Vec<serde_json::Value>,
    ) -> CargoMetadata {
        serde_json::from_value(serde_json::json!({
            "packages": packages,
            "resolve": { "nodes": nodes },
            "workspace_root": root.display().to_string()
        }))
        .expect("fixture metadata")
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
            classify_repo_path("plans/117-example/EXEC_PLAN.md"),
            Plane::Docs
        );
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
        assert_eq!(ui_audit.dependencies[0].plane, Some(Plane::Platform));
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
        assert_eq!(ui_audit.dependencies[0].plane, Some(Plane::Services));
    }

    #[test]
    fn manifest_governance_detects_missing_workspace_dependency_definition() {
        let root = unique_temp_dir("missing-workspace-dep");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app"]

[workspace.dependencies]
serde = { version = "1.0.228", features = ["derive"] }
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
tokio.workspace = true
"#,
        );

        let root_manifest = read_root_workspace_manifest(&root).expect("root manifest");
        let audits = audit_workspace_members(&root).expect("audit workspace");
        let defects = scan_for_manifest_governance_defects(&root_manifest, &audits);
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("root `[workspace.dependencies]` does not define")),
            "expected missing workspace dependency defect, got {defects:?}"
        );
    }

    #[test]
    fn manifest_governance_detects_non_member_path_dependency() {
        let root = unique_temp_dir("non-member-path");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app"]
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
helper = { path = "../../shared/helper" }
"#,
        );
        write_file(
            &root.join("shared/helper/Cargo.toml"),
            r#"
[package]
name = "helper"
version = "0.1.0"
edition = "2021"
"#,
        );

        let root_manifest = read_root_workspace_manifest(&root).expect("root manifest");
        let audits = audit_workspace_members(&root).expect("audit workspace");
        let defects = scan_for_invalid_workspace_path_dependencies(&root_manifest, &audits);
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("not listed in workspace members")),
            "expected non-member path dependency defect, got {defects:?}"
        );
    }

    #[test]
    fn manifest_governance_detects_undocumented_foundational_pin() {
        let root = unique_temp_dir("foundational-pin");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app"]

[workspace.dependencies]
serde = { version = "1.0.228", features = ["derive"] }
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
serde = { version = "1", features = ["derive"] }
"#,
        );

        let root_manifest = read_root_workspace_manifest(&root).expect("root manifest");
        let audits = audit_workspace_members(&root).expect("audit workspace");
        let defects = scan_for_manifest_governance_defects(&root_manifest, &audits);
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("pins foundational dependency `serde`")),
            "expected foundational pin defect, got {defects:?}"
        );
    }

    #[test]
    fn manifest_governance_allows_documented_foundational_exception() {
        let root = unique_temp_dir("documented-exception");
        write_file(
            &root.join("Cargo.toml"),
            r#"
[workspace]
members = ["ui/app"]

[workspace.dependencies]
serde = { version = "1.0.228", features = ["derive"] }
"#,
        );
        write_file(
            &root.join("ui/app/Cargo.toml"),
            r#"
[package]
name = "ui-app"
version = "0.1.0"
edition = "2021"

[package.metadata.origin.workspace-dependency-exceptions]
serde = "Pinned intentionally for a temporary compatibility window."

[dependencies]
serde = { version = "1", features = ["derive"] }
"#,
        );

        let root_manifest = read_root_workspace_manifest(&root).expect("root manifest");
        let audits = audit_workspace_members(&root).expect("audit workspace");
        let defects = scan_for_manifest_governance_defects(&root_manifest, &audits);
        assert!(
            defects
                .iter()
                .all(|defect| !defect.contains("pins foundational dependency `serde`")),
            "expected documented exception to suppress foundational pin defect, got {defects:?}"
        );
    }

    #[test]
    fn transitive_audit_rejects_ui_reaching_enterprise_plane() {
        let root = unique_temp_dir("transitive");
        let metadata = fixture_metadata(
            &root,
            vec![
                fixture_package(
                    "ui",
                    "site",
                    &root.join("ui/site/Cargo.toml").display().to_string(),
                    "site",
                ),
                fixture_package(
                    "platform",
                    "sdk-rs",
                    &root.join("platform/sdk/Cargo.toml").display().to_string(),
                    "sdk_rs",
                ),
                fixture_package(
                    "shared",
                    "identity",
                    &root
                        .join("shared/identity/Cargo.toml")
                        .display()
                        .to_string(),
                    "identity",
                ),
                fixture_package(
                    "enterprise",
                    "ontology-model",
                    &root
                        .join("enterprise/model/Cargo.toml")
                        .display()
                        .to_string(),
                    "ontology_model",
                ),
            ],
            vec![
                serde_json::json!({
                    "id": "ui",
                    "deps": [{ "name": "sdk_rs", "pkg": "platform" }]
                }),
                serde_json::json!({
                    "id": "platform",
                    "deps": [{ "name": "identity", "pkg": "shared" }]
                }),
                serde_json::json!({
                    "id": "shared",
                    "deps": [{ "name": "ontology_model", "pkg": "enterprise" }]
                }),
                serde_json::json!({
                    "id": "enterprise",
                    "deps": []
                }),
            ],
        );

        let workspace = build_metadata_workspace(&metadata).expect("workspace");
        let defects = scan_for_transitive_workspace_violations(&workspace);
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("disallowed transitive plane `enterprise`")),
            "expected enterprise transitive defect, got {defects:?}"
        );
    }

    #[test]
    fn source_import_audit_rejects_ui_import_of_service_crate() {
        let root = unique_temp_dir("imports");
        write_file(
            &root.join("ui/site/src/lib.rs"),
            "use finance_service::FinanceService;\n",
        );
        write_file(
            &root.join("services/finance/src/lib.rs"),
            "pub struct FinanceService;\n",
        );

        let metadata = fixture_metadata(
            &root,
            vec![
                fixture_package(
                    "ui",
                    "site",
                    &root.join("ui/site/Cargo.toml").display().to_string(),
                    "site",
                ),
                fixture_package(
                    "services",
                    "finance-service",
                    &root
                        .join("services/finance/Cargo.toml")
                        .display()
                        .to_string(),
                    "finance_service",
                ),
            ],
            vec![
                serde_json::json!({ "id": "ui", "deps": [] }),
                serde_json::json!({ "id": "services", "deps": [] }),
            ],
        );

        let workspace = build_metadata_workspace(&metadata).expect("workspace");
        let defects = scan_for_workspace_import_violations(&root, &workspace).expect("scan");
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("disallowed plane `services`")),
            "expected import defect, got {defects:?}"
        );
    }

    #[test]
    fn source_import_audit_rejects_workspace_crate_without_direct_dependency() {
        let root = unique_temp_dir("undeclared");
        write_file(
            &root.join("platform/sdk/src/lib.rs"),
            "use identity::ActorRef;\n",
        );
        write_file(
            &root.join("shared/identity/src/lib.rs"),
            "pub struct ActorRef;\n",
        );

        let metadata = fixture_metadata(
            &root,
            vec![
                fixture_package(
                    "platform",
                    "sdk-rs",
                    &root.join("platform/sdk/Cargo.toml").display().to_string(),
                    "sdk_rs",
                ),
                fixture_package(
                    "shared",
                    "identity",
                    &root
                        .join("shared/identity/Cargo.toml")
                        .display()
                        .to_string(),
                    "identity",
                ),
            ],
            vec![
                serde_json::json!({ "id": "platform", "deps": [] }),
                serde_json::json!({ "id": "shared", "deps": [] }),
            ],
        );

        let workspace = build_metadata_workspace(&metadata).expect("workspace");
        let defects = scan_for_workspace_import_violations(&root, &workspace).expect("scan");
        assert!(
            defects
                .iter()
                .any(|defect| defect.contains("without a direct dependency declaration")),
            "expected undeclared direct dependency defect, got {defects:?}"
        );
    }
}
