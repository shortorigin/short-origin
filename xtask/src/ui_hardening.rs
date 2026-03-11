use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use base64::Engine as _;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256, Sha384};

const BUILD_ROOT: &str = "build/wasm-hardening";
const CARGO_TARGET_DIR_NAME: &str = "cargo-target";
const BUILD_A_NAME: &str = "build-a";
const BUILD_B_NAME: &str = "build-b";
const REPORT_NAME: &str = "remediation-report.md";
const SITE_INDEX: &str = "ui/crates/site/index.html";
const SITE_TRUNK: &str = "ui/crates/site/Trunk.toml";
const SITE_SW: &str = "ui/crates/site/sw.js";
const SETUP_BUILD_ENVIRONMENT_ACTION: &str = ".github/actions/setup-build-environment/action.yml";
const BROWSER_SCRIPT: &str = "ui/e2e/wasm_sri_smoke.cjs";

const APPROVED_CRATE_VERSIONS: &[(&str, &str)] = &[
    ("wasm-bindgen", "0.2.114"),
    ("wasm-bindgen-futures", "0.4.64"),
    ("web-sys", "0.3.91"),
    ("js-sys", "0.3.91"),
    ("gloo-net", "0.6.0"),
];

const APPROVED_WEB_SYS_FEATURES: &[&str] = &[
    "Blob",
    "BroadcastChannel",
    "Document",
    "Element",
    "Event",
    "File",
    "FileList",
    "FileReader",
    "HtmlElement",
    "HtmlInputElement",
    "KeyboardEvent",
    "Location",
    "MessageEvent",
    "MouseEvent",
    "Navigator",
    "Notification",
    "PointerEvent",
    "ProgressEvent",
    "ServiceWorkerContainer",
    "Storage",
    "StorageManager",
    "Url",
    "Window",
];

pub fn run(args: Vec<String>) -> Result<(), String> {
    let workspace_root = super::workspace_root()?;
    let output_path = parse_output_path(&workspace_root, &args);
    let base_dir = workspace_root.join(BUILD_ROOT);
    fs::create_dir_all(&base_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", base_dir.display()))?;

    let report = verify(&workspace_root, &base_dir)?;
    fs::write(&output_path, &report).map_err(|error| {
        format!(
            "failed to write report `{}`: {error}",
            output_path.display()
        )
    })?;
    println!("{report}");
    Ok(())
}

fn parse_output_path(workspace_root: &Path, args: &[String]) -> PathBuf {
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--output" => {
                if let Some(value) = args.get(index + 1) {
                    return absolutize(workspace_root, value);
                }
            }
            value if value.starts_with("--output=") => {
                let suffix = value.trim_start_matches("--output=");
                return absolutize(workspace_root, suffix);
            }
            _ => {}
        }
        index += 1;
    }

    workspace_root.join(BUILD_ROOT).join(REPORT_NAME)
}

fn absolutize(workspace_root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn verify(workspace_root: &Path, base_dir: &Path) -> Result<String, String> {
    let logs_dir = base_dir.join("logs");
    let cargo_target_dir = base_dir.join(CARGO_TARGET_DIR_NAME);
    reset_dir(&logs_dir)?;
    reset_dir(&cargo_target_dir)?;
    let build_a_dir = base_dir.join(BUILD_A_NAME);
    let build_b_dir = base_dir.join(BUILD_B_NAME);
    reset_dir(&build_a_dir)?;
    reset_dir(&build_b_dir)?;

    let tool_versions = collect_tool_versions(workspace_root, &logs_dir)?;
    let config_audit = collect_config_audit(workspace_root)?;
    let dependency_audit = collect_dependency_audit(workspace_root, &logs_dir)?;

    clean_workspace(workspace_root, &logs_dir, "clean-1", &cargo_target_dir)?;
    build_release(
        workspace_root,
        &logs_dir,
        "build-a",
        &build_a_dir,
        &cargo_target_dir,
    )?;
    let build_a = inspect_build(workspace_root, &build_a_dir, &logs_dir, "build-a")?;

    clean_workspace(workspace_root, &logs_dir, "clean-2", &cargo_target_dir)?;
    build_release(
        workspace_root,
        &logs_dir,
        "build-b",
        &build_b_dir,
        &cargo_target_dir,
    )?;
    let build_b = inspect_build(workspace_root, &build_b_dir, &logs_dir, "build-b")?;

    let reproducibility = compare_builds(&build_a, &build_b);
    let browser_validation = run_browser_validation(workspace_root, &build_b_dir, &logs_dir)?;

    Ok(render_report(
        workspace_root,
        &tool_versions,
        &config_audit,
        &dependency_audit,
        &build_a,
        &build_b,
        &reproducibility,
        &browser_validation,
    ))
}

fn collect_tool_versions(workspace_root: &Path, logs_dir: &Path) -> Result<ToolVersions, String> {
    let rustc = run_logged_command(
        logs_dir,
        "tool-rustc",
        workspace_root,
        command("rustc", ["--version"]),
    )?;
    let cargo = run_logged_command(
        logs_dir,
        "tool-cargo",
        workspace_root,
        command("cargo", ["--version"]),
    )?;
    let trunk = run_logged_command(
        logs_dir,
        "tool-trunk",
        workspace_root,
        command("trunk", ["--version"]),
    )?;
    let rustup_targets = run_logged_command(
        logs_dir,
        "tool-rustup-targets",
        workspace_root,
        command("rustup", ["target", "list", "--installed"]),
    )?;
    let wasm_bindgen = run_optional_command(
        logs_dir,
        "tool-wasm-bindgen",
        workspace_root,
        "wasm-bindgen",
        ["--version"],
    )?;
    let wasm_opt = run_optional_command(
        logs_dir,
        "tool-wasm-opt",
        workspace_root,
        "wasm-opt",
        ["--version"],
    )?;

    Ok(ToolVersions {
        rustc: rustc.stdout_trimmed(),
        cargo: cargo.stdout_trimmed(),
        trunk: trunk.stdout_trimmed(),
        rustup_targets: rustup_targets.stdout_lines(),
        wasm_bindgen_cli: wasm_bindgen,
        wasm_opt,
    })
}

fn collect_config_audit(workspace_root: &Path) -> Result<ConfigAudit, String> {
    let trunk_config_text = fs::read_to_string(workspace_root.join(SITE_TRUNK))
        .map_err(|error| format!("failed to read `{SITE_TRUNK}`: {error}"))?;
    let trunk_config: TrunkToml = toml::from_str(&trunk_config_text)
        .map_err(|error| format!("failed to parse `{SITE_TRUNK}`: {error}"))?;
    let rust_toolchain = fs::read_to_string(workspace_root.join("rust-toolchain.toml"))
        .map_err(|error| format!("failed to read `rust-toolchain.toml`: {error}"))?;
    let service_worker = fs::read_to_string(workspace_root.join(SITE_SW))
        .map_err(|error| format!("failed to read `{SITE_SW}`: {error}"))?;
    let setup_build_environment_action = fs::read_to_string(
        workspace_root.join(SETUP_BUILD_ENVIRONMENT_ACTION),
    )
    .map_err(|error| format!("failed to read `{SETUP_BUILD_ENVIRONMENT_ACTION}`: {error}"))?;

    let workflows = [
        ".github/workflows/ci.yml",
        ".github/workflows/delivery-dev.yml",
        ".github/workflows/governance.yml",
        ".github/workflows/promote-release.yml",
        ".github/workflows/release-candidate.yml",
        ".github/workflows/security.yml",
    ]
    .iter()
    .map(|path| {
        let contents = fs::read_to_string(workspace_root.join(path))
            .map_err(|error| format!("failed to read `{path}`: {error}"))?;
        Ok(((*path).to_string(), contents))
    })
    .collect::<Result<Vec<_>, String>>()?;

    Ok(ConfigAudit {
        trunk_config,
        rust_toolchain,
        service_worker,
        setup_build_environment_action,
        workflows,
    })
}

fn collect_dependency_audit(
    workspace_root: &Path,
    logs_dir: &Path,
) -> Result<DependencyAudit, String> {
    let cargo_lock = fs::read_to_string(workspace_root.join("Cargo.lock"))
        .map_err(|error| format!("failed to read `Cargo.lock`: {error}"))?;
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let lock_packages = parse_lock_packages(&cargo_lock)?;
    for package in lock_packages {
        if APPROVED_CRATE_VERSIONS
            .iter()
            .any(|(name, _version)| *name == package.name)
        {
            versions
                .entry(package.name)
                .or_default()
                .insert(package.version);
        }
    }

    let feature_tree = run_logged_command(
        logs_dir,
        "cargo-tree-site-features",
        workspace_root,
        command(
            "cargo",
            [
                "tree",
                "-p",
                "site",
                "-e",
                "features",
                "--target",
                "wasm32-unknown-unknown",
            ],
        ),
    )?;
    let _wasm_bindgen_inverse = run_logged_command(
        logs_dir,
        "cargo-tree-wasm-bindgen",
        workspace_root,
        command("cargo", ["tree", "-i", "wasm-bindgen", "--workspace"]),
    )?;

    let declared_web_sys_features = collect_declared_web_sys_features(workspace_root)?;
    let unified_web_sys_features = extract_web_sys_features(&feature_tree.stdout);

    Ok(DependencyAudit {
        versions,
        declared_web_sys_features,
        unified_web_sys_features,
    })
}

fn clean_workspace(
    workspace_root: &Path,
    logs_dir: &Path,
    label: &str,
    cargo_target_dir: &Path,
) -> Result<(), String> {
    let build_root = workspace_root.join(BUILD_ROOT);
    let _ignored = fs::remove_dir_all(workspace_root.join("ui/crates/site/dist"));
    let _ignored = fs::remove_dir_all(workspace_root.join("target/trunk-site-dist"));
    let _ignored = fs::remove_dir_all(workspace_root.join("target/trunk-ci-dist"));
    let _ignored = fs::remove_dir_all(workspace_root.join("target/trunk-tauri-dist"));
    let _ignored = fs::remove_dir_all(workspace_root.join("target/trunk-tauri-dev"));
    let _ignored = fs::remove_dir_all(workspace_root.join("ui/crates/site/.trunk"));
    fs::create_dir_all(&build_root)
        .map_err(|error| format!("failed to create `{}`: {error}", build_root.display()))?;
    let mut command = command("cargo", ["clean"]);
    command.env("CARGO_TARGET_DIR", cargo_target_dir);
    run_logged_command(logs_dir, label, workspace_root, command)?;
    Ok(())
}

fn build_release(
    workspace_root: &Path,
    logs_dir: &Path,
    label: &str,
    dist_dir: &Path,
    cargo_target_dir: &Path,
) -> Result<(), String> {
    reset_dir(dist_dir)?;
    let dist_string = dist_dir.display().to_string();
    let mut command = command(
        "cargo",
        [
            "xtask",
            "ui",
            "build",
            "--release",
            "--dist",
            dist_string.as_str(),
        ],
    );
    command.env("CARGO_TARGET_DIR", cargo_target_dir);
    run_logged_command(logs_dir, label, workspace_root, command)?;
    Ok(())
}

fn inspect_build(
    workspace_root: &Path,
    dist_dir: &Path,
    logs_dir: &Path,
    label: &str,
) -> Result<BuildInspection, String> {
    let files = collect_artifacts(dist_dir)?;
    let html_path = dist_dir.join("index.html");
    let html = fs::read_to_string(&html_path)
        .map_err(|error| format!("failed to read `{}`: {error}", html_path.display()))?;
    let sri_assets = parse_sri_assets(&html);
    let sri_results = verify_sri_assets(dist_dir, &sri_assets)?;
    run_logged_command(
        logs_dir,
        &format!("{label}-tree"),
        workspace_root,
        command(
            "find",
            [dist_dir.display().to_string().as_str(), "-type", "f"],
        ),
    )?;

    Ok(BuildInspection {
        dist_dir: dist_dir.to_path_buf(),
        files,
        html,
        sri_results,
    })
}

fn compare_builds(build_a: &BuildInspection, build_b: &BuildInspection) -> ReproducibilityAudit {
    let mut differences = Vec::new();

    if build_a.files.len() != build_b.files.len() {
        differences.push(format!(
            "file count differs: {} vs {}",
            build_a.files.len(),
            build_b.files.len()
        ));
    }

    let files_a = build_a
        .files
        .iter()
        .map(|artifact| (&artifact.relative_path, artifact))
        .collect::<BTreeMap<_, _>>();
    let files_b = build_b
        .files
        .iter()
        .map(|artifact| (&artifact.relative_path, artifact))
        .collect::<BTreeMap<_, _>>();

    let all_paths = files_a
        .keys()
        .chain(files_b.keys())
        .copied()
        .collect::<BTreeSet<_>>();

    for path in all_paths {
        match (files_a.get(path), files_b.get(path)) {
            (Some(left), Some(right)) => {
                if left.sha256 != right.sha256 || left.byte_len != right.byte_len {
                    differences.push(format!(
                        "{path}: sha256 {} != {} or byte_len {} != {}",
                        left.sha256, right.sha256, left.byte_len, right.byte_len
                    ));
                }
            }
            (Some(_left), None) => differences.push(format!("{path}: missing from build B")),
            (None, Some(_right)) => differences.push(format!("{path}: missing from build A")),
            (None, None) => {}
        }
    }

    if build_a.html != build_b.html {
        differences.push("generated HTML differs between clean builds".to_string());
    }

    ReproducibilityAudit {
        identical: differences.is_empty(),
        differences,
    }
}

fn run_browser_validation(
    workspace_root: &Path,
    dist_dir: &Path,
    logs_dir: &Path,
) -> Result<BrowserValidation, String> {
    let node_check =
        run_optional_command(logs_dir, "tool-node", workspace_root, "node", ["--version"])?;
    let Some(node_version) = node_check else {
        return Ok(BrowserValidation {
            node_version: None,
            available: false,
            reason: Some("node is not installed".to_string()),
            browsers: Vec::new(),
        });
    };

    let python_program = if command_available("python3") {
        "python3"
    } else if command_available("python") {
        "python"
    } else {
        return Ok(BrowserValidation {
            node_version: Some(node_version),
            available: false,
            reason: Some("python is not installed".to_string()),
            browsers: Vec::new(),
        });
    };

    let script_path = workspace_root.join(BROWSER_SCRIPT);
    if !script_path.exists() {
        return Ok(BrowserValidation {
            node_version: Some(node_version),
            available: false,
            reason: Some(format!(
                "browser smoke script `{}` is missing",
                script_path.display()
            )),
            browsers: Vec::new(),
        });
    }

    let port = reserve_port()?;
    let mut server = Command::new(python_program)
        .current_dir(dist_dir)
        .args([
            "-m",
            "http.server",
            &port.to_string(),
            "--bind",
            "127.0.0.1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start static server: {error}"))?;
    let browser_output_path = logs_dir.join("browser-validation.json");
    let url = format!("http://127.0.0.1:{port}/index.html");
    wait_for_http_server(port)?;
    let browser_result = Command::new("node")
        .current_dir(workspace_root)
        .arg(script_path)
        .arg(&url)
        .arg(&browser_output_path)
        .output()
        .map_err(|error| format!("failed to run browser validation: {error}"));

    let kill_result = server.kill();
    let _wait_result = server.wait();
    if let Err(error) = kill_result
        && error.kind() != io::ErrorKind::InvalidInput
    {
        return Err(format!("failed to stop static server: {error}"));
    }

    browser_result?;
    let raw = fs::read_to_string(&browser_output_path).map_err(|error| {
        format!(
            "failed to read browser validation output `{}`: {error}",
            browser_output_path.display()
        )
    })?;
    let parsed: BrowserScriptOutput = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse browser validation output: {error}"))?;

    Ok(BrowserValidation {
        node_version: Some(node_version),
        available: parsed.available,
        reason: parsed.reason,
        browsers: parsed.browsers,
    })
}

fn render_report(
    workspace_root: &Path,
    tool_versions: &ToolVersions,
    config_audit: &ConfigAudit,
    dependency_audit: &DependencyAudit,
    build_a: &BuildInspection,
    build_b: &BuildInspection,
    reproducibility: &ReproducibilityAudit,
    browser_validation: &BrowserValidation,
) -> String {
    let findings = build_findings(
        tool_versions,
        config_audit,
        dependency_audit,
        build_a,
        reproducibility,
        browser_validation,
    );
    let actions = build_actions(&findings);
    let checklist = build_checklist(
        tool_versions,
        config_audit,
        dependency_audit,
        build_a,
        reproducibility,
        browser_validation,
    );
    let completion = completion_status(&findings, browser_validation, reproducibility);

    format!(
        "\
1. EXECUTIVE SUMMARY
- pipeline status: {pipeline_status}
- overall risk level: {risk_level}
- determinism and integrity posture: {posture}

2. ENVIRONMENT BASELINE
- repository root: {repo_root}
- rust toolchain: {rustc}
- cargo: {cargo}
- trunk: {trunk}
- wasm-bindgen CLI: {wasm_bindgen_cli}
- wasm-bindgen crate: {wasm_bindgen_crate}
- wasm-opt: {wasm_opt}
- installed Rust targets: {targets}
- canonical build commands:
  - development: `cargo ui-dev`
  - browser production: `cargo xtask ui build --release --dist {build_root}/release-dist`
  - Tauri wrapper: `cargo xtask ui build --release --features desktop-tauri --dist target/trunk-tauri-dist`
  - hardening verification: `cargo xtask ui-hardening`
- relevant configuration files discovered:
  - `Cargo.toml`
  - `Cargo.lock`
  - `.cargo/config.toml`
  - `rust-toolchain.toml`
  - `{site_trunk}`
  - `{site_index}`
  - `ui/crates/desktop_tauri/tauri.conf.json`
  - `.github/workflows/ci.yml`
  - `.github/workflows/release-candidate.yml`
  - `.github/workflows/promote-release.yml`

3. PIPELINE MODEL
- source-to-browser flow:
  1. Rust `site_app` compiles for `wasm32-unknown-unknown`.
  2. Trunk runs `wasm-bindgen` as part of `trunk build`.
  3. wasm-bindgen emits JS glue, the `.wasm`, and snippet modules.
  4. Trunk copies declared static assets from `{site_index}` and rewrites HTML.
  5. Trunk injects hashed asset filenames and SRI-bearing preload tags into the final `index.html`.
  6. The browser bootstraps from generated HTML, modulepreload JS, WASM preload, and runtime fetches.
- deployable artifact boundary: final Trunk output directory containing generated `index.html`, generated JS/WASM/snippets, copied assets, and any service worker/manifest shipped with the app.

4. FINDINGS
{findings_section}

5. REMEDIATION ACTIONS
{actions_section}

6. DETERMINISTIC REMEDIATION CHECKLIST
{checklist_section}

7. SRI VALIDATION RESULTS
{sri_section}

8. REPRODUCIBILITY VALIDATION
- build A output directory: `{build_a_dir}`
- build B output directory: `{build_b_dir}`
- build A file list and digests:
{build_a_files}
- build B file list and digests:
{build_b_files}
- byte-identical result: {reproducible}
{reproducibility_details}

9. BROWSER COMPATIBILITY VALIDATION
{browser_section}

10. CANONICAL HARDENED BUILD PROCEDURE
1. Ensure the pinned Rust toolchain and `wasm32-unknown-unknown` target from `rust-toolchain.toml` are installed.
2. Run `cargo xtask ui-hardening`.
3. The verifier will execute `cargo clean`, remove prior Trunk output, build twice with `cargo xtask ui build --release`, compare the full deployable artifact graph, independently validate SRI digests, and run browser smoke checks when Node and Playwright browsers are available.
4. Review the generated remediation report at `{build_root}/{report_name}`.
5. Use the verified production artifact directory under `{build_root}/{build_b_name}` as the canonical release artifact for browser deployment review.
6. If any checklist item is `FAIL`, do not deploy until the report shows all mandatory items as `PASS` or the remaining blocker is explicitly accepted.

11. CI/CD ENFORCEMENT RECOMMENDATIONS
- fail if `rust-toolchain.toml` changes without corresponding workflow updates.
- fail if `Cargo.lock` drifts or multiple versions of `wasm-bindgen`, `web-sys`, `js-sys`, or `wasm-bindgen-futures` appear in the browser build graph.
- fail if `ui/crates/site/Trunk.toml` does not keep SRI enabled for production output.
- fail if `cargo xtask ui-hardening` reports missing integrity attributes, digest mismatches, or non-identical clean builds.
- fail if browser smoke validation reports integrity errors, WASM/module loading failures, or stale service-worker-served assets.

12. COMPLETION STATUS
{completion}
",
        pipeline_status = if completion.is_complete { "HARDENED" } else { "INCOMPLETE" },
        risk_level = if completion.is_complete { "LOW" } else { "MEDIUM" },
        posture = if completion.is_complete {
            "production Trunk builds are explicit, SRI is independently checked, and the deployable artifact graph is compared across clean rebuilds"
        } else {
            "hardening controls are implemented, but at least one required validation remains blocked or failing"
        },
        repo_root = workspace_root.display(),
        rustc = tool_versions.rustc,
        cargo = tool_versions.cargo,
        trunk = tool_versions.trunk,
        wasm_bindgen_cli = tool_versions
            .wasm_bindgen_cli
            .as_deref()
            .unwrap_or("NOT INSTALLED"),
        wasm_bindgen_crate = dependency_audit
            .versions
            .get("wasm-bindgen")
            .map_or_else(|| "NOT FOUND".to_string(), join_set),
        wasm_opt = tool_versions.wasm_opt.as_deref().unwrap_or("NOT INSTALLED"),
        targets = tool_versions.rustup_targets.join(", "),
        build_root = BUILD_ROOT,
        site_trunk = SITE_TRUNK,
        site_index = SITE_INDEX,
        findings_section = render_findings(&findings),
        actions_section = render_actions(&actions),
        checklist_section = render_checklist(&checklist),
        sri_section = render_sri_results(&build_a.sri_results),
        build_a_dir = build_a.dist_dir.display(),
        build_b_dir = build_b.dist_dir.display(),
        build_a_files = render_artifacts(&build_a.files),
        build_b_files = render_artifacts(&build_b.files),
        reproducible = if reproducibility.identical { "PASS" } else { "FAIL" },
        reproducibility_details = render_reproducibility(reproducibility),
        browser_section = render_browser_section(browser_validation),
        report_name = REPORT_NAME,
        build_b_name = BUILD_B_NAME,
        completion = completion.message,
    )
}

fn build_findings(
    tool_versions: &ToolVersions,
    config_audit: &ConfigAudit,
    dependency_audit: &DependencyAudit,
    build_a: &BuildInspection,
    reproducibility: &ReproducibilityAudit,
    browser_validation: &BrowserValidation,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if !config_audit.workflows.iter().all(|(_path, contents)| {
        contents.contains("dtolnay/rust-toolchain@1.91.1")
            || (contents.contains("uses: ./.github/actions/setup-build-environment")
                && config_audit
                    .setup_build_environment_action
                    .contains("dtolnay/rust-toolchain@1.91.1"))
    }) {
        findings.push(Finding::fail(
            "CI still uses a floating Rust toolchain",
            "Expected all Rust-installing workflows to pin `dtolnay/rust-toolchain@1.91.1`.",
            "floating toolchain selection can alter browser artifacts across environments",
        ));
    }

    if config_audit.trunk_config.build.no_sri.unwrap_or(true) {
        findings.push(Finding::fail(
            "Trunk production SRI is not explicitly enabled",
            "Expected `no_sri = false` in `ui/crates/site/Trunk.toml`.",
            "generated HTML would not enforce SRI at the HTML boundary",
        ));
    }

    if !service_worker_hardened(&config_audit.service_worker) {
        findings.push(Finding::fail(
            "Service worker can still serve stale release assets",
            "Expected the service worker to avoid cache-first interception for hashed JS/WASM and to delete older caches on activation.",
            "stale cached assets can break integrity and release graph consistency",
        ));
    }

    let missing_versions = APPROVED_CRATE_VERSIONS
        .iter()
        .filter(|(crate_name, expected)| {
            dependency_audit
                .versions
                .get(*crate_name)
                .is_none_or(|versions| versions.len() != 1 || !versions.contains(*expected))
        })
        .map(|(crate_name, expected)| format!("{crate_name}={expected}"))
        .collect::<Vec<_>>();
    if !missing_versions.is_empty() {
        findings.push(Finding::fail(
            "Critical browser-facing dependency versions drifted from the approved release set",
            &format!("Unexpected versions for: {}", missing_versions.join(", ")),
            "output-shaping dependencies can change generated JS/WASM and SRI digests",
        ));
    }

    let direct_features = dependency_audit
        .declared_web_sys_features
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let approved_features = APPROVED_WEB_SYS_FEATURES
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    if direct_features != approved_features {
        findings.push(Finding::fail(
            "Direct workspace `web-sys` features drifted from the approved browser release set",
            &format!(
                "expected {:?}, observed {:?}",
                APPROVED_WEB_SYS_FEATURES, direct_features
            ),
            "`web-sys` feature drift changes generated bindings and can alter deployable browser artifacts",
        ));
    }

    if build_a.sri_results.iter().any(|result| !result.matches) {
        findings.push(Finding::fail(
            "Generated HTML contains incorrect SRI metadata",
            "At least one independently computed asset digest did not match the emitted HTML integrity attribute.",
            "the browser can reject assets or silently break integrity guarantees",
        ));
    }

    if reproducibility.identical {
        findings.push(Finding::pass(
            "Two clean production builds are byte-identical",
            "The full deployable artifact graph matched across build A and build B.",
            "the artifact boundary is currently deterministic under the validated environment",
        ));
    } else {
        findings.push(Finding::fail(
            "Two clean production builds are not byte-identical",
            &reproducibility.differences.join("; "),
            "release outputs are not reproducible at the deployable artifact boundary",
        ));
    }

    if !browser_validation.available {
        findings.push(Finding::fail(
            "Automated browser validation did not run",
            browser_validation
                .reason
                .as_deref()
                .unwrap_or("browser validation was unavailable"),
            "runtime integrity and WASM loading behavior remain unverified in this environment",
        ));
    } else if browser_validation
        .browsers
        .iter()
        .any(|browser| browser.available && !browser.success)
    {
        findings.push(Finding::fail(
            "Browser smoke validation reported runtime failures",
            "At least one available browser reported console, request, integrity, or WASM/module errors.",
            "the hardened artifact set does not load cleanly in all validated browsers",
        ));
    } else {
        findings.push(Finding::pass(
            "Available browsers loaded the hardened build without integrity failures",
            "Automated smoke validation completed without request failures, integrity errors, or WASM/module loading errors.",
            "runtime validation matches the hardened build-state expectations",
        ));
    }

    if tool_versions.wasm_bindgen_cli.is_none() {
        findings.push(Finding::pass(
            "Direct wasm-bindgen CLI usage is absent",
            "No standalone `wasm-bindgen` executable was found; the build relies on Trunk-integrated wasm-bindgen.",
            "there is no extra unpinned CLI layer beyond Trunk and the locked crate graph",
        ));
    }

    if tool_versions.wasm_opt.is_none() {
        findings.push(Finding::pass(
            "wasm-opt is not in use",
            "No standalone `wasm-opt` executable was found.",
            "the current pipeline avoids an extra post-processing source of output drift",
        ));
    }

    findings
}

fn build_actions(findings: &[Finding]) -> Vec<Action> {
    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| Action {
            index: index + 1,
            title: finding.title.clone(),
            status: finding.status.clone(),
            defect: finding.evidence.clone(),
            root_cause: finding.impact.clone(),
            fix: if finding.status == "FAIL" {
                format!(
                    "Remediate `{}` until the verification procedure reports `PASS`.",
                    finding.title
                )
            } else {
                format!(
                    "Keep `{}` under the canonical verification procedure.",
                    finding.title
                )
            },
            verification: "Run `cargo xtask ui-hardening` and review the matching report section."
                .to_string(),
            expected: if finding.status == "FAIL" {
                "Report item changes to `PASS` with no residual blocker.".to_string()
            } else {
                "Item remains `PASS` in the generated report.".to_string()
            },
        })
        .collect()
}

fn build_checklist(
    tool_versions: &ToolVersions,
    config_audit: &ConfigAudit,
    dependency_audit: &DependencyAudit,
    build_a: &BuildInspection,
    reproducibility: &ReproducibilityAudit,
    browser_validation: &BrowserValidation,
) -> Vec<ChecklistItem> {
    vec![
        checklist_item(
            "toolchain pinning",
            path_exists(&config_audit.rust_toolchain, "channel = \"1.91.1\""),
            "rust-toolchain.toml pins the browser build toolchain",
        ),
        checklist_item(
            "dependency version consistency",
            APPROVED_CRATE_VERSIONS
                .iter()
                .all(|(crate_name, expected)| {
                    dependency_audit
                        .versions
                        .get(*crate_name)
                        .is_some_and(|versions| versions.len() == 1 && versions.contains(*expected))
                }),
            "critical browser-facing dependencies match the approved release set",
        ),
        checklist_item(
            "Cargo feature normalization",
            dependency_audit
                .declared_web_sys_features
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                == APPROVED_WEB_SYS_FEATURES
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect::<BTreeSet<_>>(),
            "workspace-declared `web-sys` features match the approved browser set",
        ),
        checklist_item(
            "`web-sys` feature audit",
            !dependency_audit.unified_web_sys_features.is_empty(),
            "the verifier captured the unified `web-sys` feature graph for the browser target",
        ),
        checklist_item(
            "canonical Trunk configuration",
            config_audit.trunk_config.build.target.as_deref() == Some("index.html")
                && config_audit.trunk_config.build.no_sri == Some(false),
            "Trunk config exists and explicitly controls production behavior",
        ),
        checklist_item(
            "production-only SRI enforcement policy",
            config_audit.trunk_config.build.no_sri == Some(false),
            "production Trunk output keeps SRI enabled",
        ),
        checklist_item(
            "wasm-bindgen output mode control",
            tool_versions.wasm_bindgen_cli.is_none(),
            "the build uses Trunk-integrated wasm-bindgen without an extra standalone CLI layer",
        ),
        checklist_item(
            "full asset graph capture",
            build_a.files.iter().any(|artifact| {
                Path::new(&artifact.relative_path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
            }) && build_a.files.iter().any(|artifact| {
                Path::new(&artifact.relative_path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
            }) && build_a
                .files
                .iter()
                .any(|artifact| artifact.relative_path.contains("snippets/")),
            "the verifier captured JS, WASM, snippet, and copied assets",
        ),
        checklist_item(
            "clean reproducible release builds",
            reproducibility.identical,
            "two clean release builds produced the same deployable outputs",
        ),
        checklist_item(
            "byte-level build comparison",
            reproducibility.identical,
            "build A and build B match at file-count, file-name, byte, and digest level",
        ),
        checklist_item(
            "independent SRI digest verification",
            build_a.sri_results.iter().all(|result| result.matches),
            "independent hashing matches all emitted integrity attributes",
        ),
        checklist_item(
            "HTML integrity attribute verification",
            !build_a.sri_results.is_empty(),
            "generated HTML contains integrity-bearing asset references",
        ),
        checklist_item(
            "browser runtime validation",
            browser_validation.available
                && browser_validation
                    .browsers
                    .iter()
                    .filter(|browser| browser.available)
                    .all(|browser| browser.success),
            "available browsers complete the smoke validation without integrity or WASM loading errors",
        ),
        checklist_item(
            "cache and service worker invalidation policy",
            service_worker_hardened(&config_audit.service_worker),
            "service worker avoids stale hashed asset reuse and deletes old caches",
        ),
        checklist_item(
            "CI/CD enforcement steps",
            config_audit.workflows.iter().all(|(_path, contents)| {
                contents.contains("cargo xtask ui-hardening")
                    || !contents.contains("xtask ui build")
            }),
            "release-oriented workflows reference the hardened verification path where browser artifacts are involved",
        ),
    ]
}

fn render_findings(findings: &[Finding]) -> String {
    findings
        .iter()
        .enumerate()
        .map(|(index, finding)| {
            format!(
                "{idx}. [{status}] {title}\n   - evidence: {evidence}\n   - impact: {impact}",
                idx = index + 1,
                status = finding.status,
                title = finding.title,
                evidence = finding.evidence,
                impact = finding.impact,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_actions(actions: &[Action]) -> String {
    actions
        .iter()
        .map(|action| {
            format!(
                "{idx}. [{status}] {title}\n   - precise defect description: {defect}\n   - root cause: {root_cause}\n   - corrective action: {fix}\n   - verification command or procedure: {verification}\n   - final expected condition: {expected}",
                idx = action.index,
                status = action.status,
                title = action.title,
                defect = action.defect,
                root_cause = action.root_cause,
                fix = action.fix,
                verification = action.verification,
                expected = action.expected,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_checklist(items: &[ChecklistItem]) -> String {
    items
        .iter()
        .map(|item| format!("- [{}] {}: {}", item.status, item.title, item.details))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_sri_results(results: &[SRIResult]) -> String {
    if results.is_empty() {
        return "- no SRI-bearing assets were discovered in generated HTML".to_string();
    }

    results
        .iter()
        .map(|result| {
            format!(
                "- [{}] tag={tag} path={path} integrity={integrity} crossorigin={crossorigin} digest_match={match_state}",
                if result.matches { "PASS" } else { "FAIL" },
                tag = result.tag_type,
                path = result.asset_path,
                integrity = result.integrity,
                crossorigin = result.crossorigin.as_deref().unwrap_or("NONE"),
                match_state = if result.matches { "true" } else { "false" },
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_artifacts(artifacts: &[ArtifactDigest]) -> String {
    artifacts
        .iter()
        .map(|artifact| {
            format!(
                "  - {} | bytes={} | sha256={}",
                artifact.relative_path, artifact.byte_len, artifact.sha256
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_reproducibility(reproducibility: &ReproducibilityAudit) -> String {
    if reproducibility.identical {
        "- divergence analysis: none; builds are byte-identical".to_string()
    } else {
        format!(
            "- divergence analysis:\n{}",
            reproducibility
                .differences
                .iter()
                .map(|entry| format!("  - {entry}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

fn render_browser_section(browser_validation: &BrowserValidation) -> String {
    if !browser_validation.available {
        return format!(
            "- status: FAIL\n- reason: {}\n- node: {}",
            browser_validation
                .reason
                .as_deref()
                .unwrap_or("browser validation unavailable"),
            browser_validation
                .node_version
                .as_deref()
                .unwrap_or("NOT INSTALLED"),
        );
    }

    let browsers = browser_validation
        .browsers
        .iter()
        .map(|browser| {
            format!(
                "- {}: available={} success={} console_errors={} page_errors={} request_failures={} integrity_failures={} wasm_failures={} service_worker_controller={}",
                browser.name,
                browser.available,
                browser.success,
                browser.console.iter().filter(|entry| entry.r#type == "error").count(),
                browser.page_errors.len(),
                browser.request_failures.len(),
                browser.integrity_failures.len(),
                browser.wasm_failures.len(),
                browser.service_worker_controller,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "- status: {}\n- node: {}\n{}",
        if browser_validation
            .browsers
            .iter()
            .filter(|browser| browser.available)
            .all(|browser| browser.success)
        {
            "PASS"
        } else {
            "FAIL"
        },
        browser_validation
            .node_version
            .as_deref()
            .unwrap_or("UNKNOWN"),
        browsers
    )
}

fn completion_status(
    findings: &[Finding],
    browser_validation: &BrowserValidation,
    reproducibility: &ReproducibilityAudit,
) -> CompletionStatus {
    let blocking = findings
        .iter()
        .filter(|finding| finding.status == "FAIL")
        .map(|finding| finding.title.clone())
        .collect::<Vec<_>>();
    let is_complete = blocking.is_empty()
        && reproducibility.identical
        && browser_validation.available
        && browser_validation
            .browsers
            .iter()
            .filter(|browser| browser.available)
            .all(|browser| browser.success);

    let message = if is_complete {
        "The pipeline is hardened, reproducible, and compatible with the validated Trunk-driven deployment flow.".to_string()
    } else {
        format!(
            "The pipeline is not fully hardened. Remaining blockers: {}.",
            blocking.join(", ")
        )
    };

    CompletionStatus {
        is_complete,
        message,
    }
}

fn checklist_item(title: &str, pass: bool, details: &str) -> ChecklistItem {
    ChecklistItem {
        status: if pass { "PASS" } else { "FAIL" }.to_string(),
        title: title.to_string(),
        details: details.to_string(),
    }
}

fn collect_artifacts(dist_dir: &Path) -> Result<Vec<ArtifactDigest>, String> {
    let mut files = Vec::new();
    collect_artifacts_recursive(dist_dir, dist_dir, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn collect_artifacts_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<ArtifactDigest>,
) -> Result<(), String> {
    let entries = fs::read_dir(current)
        .map_err(|error| format!("failed to read `{}`: {error}", current.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_artifacts_recursive(root, &path, files)?;
        } else if path.is_file() {
            let bytes = fs::read(&path)
                .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sha256 = format!("{:x}", hasher.finalize());
            let relative_path = path
                .strip_prefix(root)
                .map_err(|error| {
                    format!(
                        "failed to strip prefix `{}` from `{}`: {error}",
                        root.display(),
                        path.display()
                    )
                })?
                .to_string_lossy()
                .replace('\\', "/");
            files.push(ArtifactDigest {
                relative_path,
                byte_len: bytes.len(),
                sha256,
            });
        }
    }
    Ok(())
}

fn parse_sri_assets(html: &str) -> Vec<SRIAsset> {
    let regex = Regex::new(
        r#"<(?P<tag>script|link)\b[^>]*(?:href|src)="(?P<path>[^"]+)"[^>]*integrity="(?P<integrity>[^"]+)"[^>]*(?:crossorigin="(?P<crossorigin>[^"]+)")?[^>]*>"#,
    )
    .expect("valid regex");
    regex
        .captures_iter(html)
        .map(|captures| SRIAsset {
            tag_type: captures["tag"].to_string(),
            asset_path: captures["path"].to_string(),
            integrity: captures["integrity"].to_string(),
            crossorigin: captures
                .name("crossorigin")
                .map(|value| value.as_str().to_string()),
        })
        .collect()
}

fn verify_sri_assets(dist_dir: &Path, assets: &[SRIAsset]) -> Result<Vec<SRIResult>, String> {
    assets
        .iter()
        .map(|asset| {
            let (algorithm, expected_digest) = asset
                .integrity
                .split_once('-')
                .ok_or_else(|| format!("invalid integrity value `{}`", asset.integrity))?;
            let relative_path = asset.asset_path.trim_start_matches('/');
            let file_path = dist_dir.join(relative_path);
            let bytes = fs::read(&file_path).map_err(|error| {
                format!(
                    "failed to read integrity target `{}`: {error}",
                    file_path.display()
                )
            })?;
            let actual_digest = match algorithm {
                "sha384" => base64::engine::general_purpose::STANDARD.encode(Sha384::digest(bytes)),
                "sha256" => base64::engine::general_purpose::STANDARD.encode(Sha256::digest(bytes)),
                other => {
                    return Err(format!("unsupported SRI algorithm `{other}`"));
                }
            };

            Ok(SRIResult {
                tag_type: asset.tag_type.clone(),
                asset_path: asset.asset_path.clone(),
                integrity: asset.integrity.clone(),
                crossorigin: asset.crossorigin.clone(),
                matches: actual_digest == expected_digest,
            })
        })
        .collect()
}

fn collect_declared_web_sys_features(workspace_root: &Path) -> Result<Vec<String>, String> {
    let files = [
        "ui/crates/site/Cargo.toml",
        "ui/crates/platform_host_web/Cargo.toml",
        "ui/crates/desktop_runtime/Cargo.toml",
        "ui/crates/apps/terminal/Cargo.toml",
    ];
    let mut features = BTreeSet::new();
    for path in files {
        let contents = fs::read_to_string(workspace_root.join(path))
            .map_err(|error| format!("failed to read `{path}`: {error}"))?;
        let value: toml::Value = toml::from_str(&contents)
            .map_err(|error| format!("failed to parse `{path}`: {error}"))?;
        if let Some(entries) = value
            .get("dependencies")
            .and_then(|dependencies| dependencies.get("web-sys"))
            .and_then(toml::Value::as_table)
            .and_then(|table| table.get("features"))
            .and_then(toml::Value::as_array)
        {
            for entry in entries {
                if let Some(feature) = entry.as_str() {
                    features.insert(feature.to_string());
                }
            }
        }
    }
    Ok(features.into_iter().collect())
}

fn extract_web_sys_features(feature_tree: &str) -> Vec<String> {
    let regex = Regex::new(r#"web-sys feature "([^"]+)""#).expect("valid regex");
    let mut features = BTreeSet::new();
    for captures in regex.captures_iter(feature_tree) {
        features.insert(captures[1].to_string());
    }
    features.into_iter().collect()
}

fn parse_lock_packages(lock_text: &str) -> Result<Vec<LockPackage>, String> {
    let value: toml::Value = toml::from_str(lock_text)
        .map_err(|error| format!("failed to parse Cargo.lock as TOML: {error}"))?;
    let packages = value
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "Cargo.lock is missing `package` entries".to_string())?;

    let mut parsed = Vec::new();
    for package in packages {
        let name = package
            .get("name")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "Cargo.lock package is missing `name`".to_string())?;
        let version = package
            .get("version")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| "Cargo.lock package is missing `version`".to_string())?;
        parsed.push(LockPackage {
            name: name.to_string(),
            version: version.to_string(),
        });
    }
    Ok(parsed)
}

fn service_worker_hardened(service_worker: &str) -> bool {
    service_worker.contains("event.request.mode !== \"navigate\"")
        && service_worker.contains("cacheName.startsWith(CACHE_PREFIX)")
        && service_worker.contains("caches.delete(cacheName)")
        && !service_worker.contains("cached || fetch(event.request)")
}

fn reset_dir(path: &Path) -> Result<(), String> {
    let _ignored = fs::remove_dir_all(path);
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create `{}`: {error}", path.display()))
}

fn run_optional_command<const N: usize>(
    logs_dir: &Path,
    label: &str,
    cwd: &Path,
    program: &str,
    args: [&str; N],
) -> Result<Option<String>, String> {
    if !command_available(program) {
        return Ok(None);
    }
    let result = run_logged_command(logs_dir, label, cwd, command(program, args))?;
    Ok(Some(result.stdout_trimmed()))
}

fn run_logged_command(
    logs_dir: &Path,
    label: &str,
    cwd: &Path,
    mut command: Command,
) -> Result<CommandOutput, String> {
    command.current_dir(cwd);
    let output = command
        .output()
        .map_err(|error| format!("failed to run `{}`: {error}", display_command(&command)))?;
    let stdout_path = logs_dir.join(format!("{label}.stdout.log"));
    let stderr_path = logs_dir.join(format!("{label}.stderr.log"));
    fs::write(&stdout_path, &output.stdout)
        .map_err(|error| format!("failed to write `{}`: {error}", stdout_path.display()))?;
    fs::write(&stderr_path, &output.stderr)
        .map_err(|error| format!("failed to write `{}`: {error}", stderr_path.display()))?;
    if !output.status.success() {
        return Err(format!(
            "`{}` failed with status {}. See `{}` and `{}`",
            display_command(&command),
            output.status,
            stdout_path.display(),
            stderr_path.display(),
        ));
    }
    Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

fn command<const N: usize>(program: &str, args: [&str; N]) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

fn command_available(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn reserve_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("failed to reserve ephemeral port: {error}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| format!("failed to read ephemeral port: {error}"))
}

fn wait_for_http_server(port: u16) -> Result<(), String> {
    for _attempt in 0..30 {
        if http_server_ready(port) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(250));
    }

    Err(format!(
        "static file server on port {port} did not become ready"
    ))
}

fn http_server_ready(port: u16) -> bool {
    let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) else {
        return false;
    };
    let request = b"GET /index.html HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }

    response.starts_with("HTTP/1.0 200") || response.starts_with("HTTP/1.1 200")
}

fn display_command(command: &Command) -> String {
    let args = command
        .get_args()
        .map(OsStr::to_string_lossy)
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        command.get_program().to_string_lossy().into_owned()
    } else {
        format!("{} {args}", command.get_program().to_string_lossy())
    }
}

fn join_set(values: &BTreeSet<String>) -> String {
    values.iter().cloned().collect::<Vec<_>>().join(", ")
}

fn path_exists(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}

#[derive(Clone)]
struct ToolVersions {
    rustc: String,
    cargo: String,
    trunk: String,
    rustup_targets: Vec<String>,
    wasm_bindgen_cli: Option<String>,
    wasm_opt: Option<String>,
}

struct ConfigAudit {
    trunk_config: TrunkToml,
    rust_toolchain: String,
    service_worker: String,
    setup_build_environment_action: String,
    workflows: Vec<(String, String)>,
}

struct DependencyAudit {
    versions: BTreeMap<String, BTreeSet<String>>,
    declared_web_sys_features: Vec<String>,
    unified_web_sys_features: Vec<String>,
}

struct BuildInspection {
    dist_dir: PathBuf,
    files: Vec<ArtifactDigest>,
    html: String,
    sri_results: Vec<SRIResult>,
}

struct ReproducibilityAudit {
    identical: bool,
    differences: Vec<String>,
}

struct BrowserValidation {
    node_version: Option<String>,
    available: bool,
    reason: Option<String>,
    browsers: Vec<BrowserSmokeResult>,
}

struct ArtifactDigest {
    relative_path: String,
    byte_len: usize,
    sha256: String,
}

struct SRIAsset {
    tag_type: String,
    asset_path: String,
    integrity: String,
    crossorigin: Option<String>,
}

struct SRIResult {
    tag_type: String,
    asset_path: String,
    integrity: String,
    crossorigin: Option<String>,
    matches: bool,
}

struct LockPackage {
    name: String,
    version: String,
}

struct Finding {
    status: String,
    title: String,
    evidence: String,
    impact: String,
}

impl Finding {
    fn fail(title: &str, evidence: &str, impact: &str) -> Self {
        Self {
            status: "FAIL".to_string(),
            title: title.to_string(),
            evidence: evidence.to_string(),
            impact: impact.to_string(),
        }
    }

    fn pass(title: &str, evidence: &str, impact: &str) -> Self {
        Self {
            status: "PASS".to_string(),
            title: title.to_string(),
            evidence: evidence.to_string(),
            impact: impact.to_string(),
        }
    }
}

struct Action {
    index: usize,
    title: String,
    status: String,
    defect: String,
    root_cause: String,
    fix: String,
    verification: String,
    expected: String,
}

struct ChecklistItem {
    status: String,
    title: String,
    details: String,
}

struct CompletionStatus {
    is_complete: bool,
    message: String,
}

struct CommandOutput {
    stdout: String,
}

impl CommandOutput {
    fn stdout_trimmed(&self) -> String {
        self.stdout.trim().to_string()
    }

    fn stdout_lines(&self) -> Vec<String> {
        self.stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }
}

#[derive(Deserialize)]
struct TrunkToml {
    build: TrunkBuild,
}

#[derive(Deserialize)]
struct TrunkBuild {
    target: Option<String>,
    no_sri: Option<bool>,
}

#[derive(Deserialize)]
struct BrowserScriptOutput {
    available: bool,
    reason: Option<String>,
    #[serde(default)]
    browsers: Vec<BrowserSmokeResult>,
}

#[derive(Clone, Deserialize)]
struct BrowserSmokeResult {
    name: String,
    available: bool,
    success: bool,
    #[serde(default)]
    console: Vec<BrowserConsoleEntry>,
    #[serde(default)]
    page_errors: Vec<String>,
    #[serde(default)]
    request_failures: Vec<Value>,
    #[serde(default)]
    integrity_failures: Vec<String>,
    #[serde(default)]
    wasm_failures: Vec<String>,
    #[serde(default)]
    service_worker_controller: bool,
}

#[derive(Clone, Deserialize)]
struct BrowserConsoleEntry {
    #[serde(rename = "type")]
    r#type: String,
}
