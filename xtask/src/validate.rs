use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::common::{display_command, run_command, workspace_root};

const VALIDATION_CONFIG_PATH: &str = "xtask/validation.toml";
const SECURITY_EXCEPTIONS_PATH: &str = ".github/security-exceptions.toml";
const REPORT_DIR: &str = "target/validation";
const PRE_PUSH_HOOK_PATH: &str = ".githooks/pre-push";

#[derive(Debug, Deserialize)]
struct ValidationConfig {
    version: u32,
    tools: ToolConfig,
    selectors: SelectorConfig,
    freshness: FreshnessConfig,
}

#[derive(Debug, Deserialize)]
struct ToolConfig {
    node_major: u64,
    cargo_audit_version: String,
    trunk_version: String,
    nomad_image: String,
}

#[derive(Debug, Deserialize)]
struct SelectorConfig {
    shared_root_files: Vec<String>,
    shared_root_prefixes: Vec<String>,
    core_rust_prefixes: Vec<String>,
    ui_prefixes: Vec<String>,
    nomad_prefixes: Vec<String>,
    nomad_suffixes: Vec<String>,
    pulumi_prefixes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FreshnessConfig {
    required_prefixes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SecurityExceptions {
    version: u32,
    exceptions: Vec<SecurityException>,
}

#[derive(Debug, Deserialize)]
struct SecurityException {
    ids: Vec<String>,
    owner: String,
    issue: u64,
    expires: String,
    reason: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Suite {
    Governance,
    Security,
    Core,
    Ui,
    UiHardening,
    Nomad,
    Pulumi,
}

#[derive(Debug, Serialize)]
struct ValidationReport {
    mode: String,
    event: Option<String>,
    branch: String,
    base_ref: Option<String>,
    merge_base: Option<String>,
    head_ref: String,
    changed_files: Vec<String>,
    freshness_required: bool,
    freshness_ok: bool,
    selected_suites: Vec<String>,
    suite_results: Vec<SuiteResult>,
}

#[derive(Debug, Serialize)]
struct SuiteResult {
    suite: String,
    status: String,
    detail: String,
    elapsed_ms: u128,
}

#[derive(Debug)]
struct ValidationContext {
    workspace_root: PathBuf,
    config: ValidationConfig,
}

#[derive(Debug)]
struct ChangedSelection {
    branch: String,
    base_ref: String,
    merge_base: String,
    head_ref: String,
    changed_files: Vec<String>,
    suites: Vec<Suite>,
    freshness_required: bool,
    freshness_ok: bool,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    let (subcommand, rest) = args.split_first().ok_or_else(|| help().to_string())?;
    match subcommand.as_str() {
        "doctor" => run_doctor(rest),
        "bootstrap" => run_bootstrap(rest),
        "changed" => run_changed(rest),
        "suite" => run_suite_command(rest),
        "ci" => run_ci(rest),
        "install-hooks" => run_install_hooks(rest),
        other => Err(format!("unsupported validate subcommand `{other}`")),
    }
}

fn run_doctor(args: &[String]) -> Result<(), String> {
    ensure_no_trailing(args, "validate doctor")?;
    let context = load_context()?;
    let checks = doctor_checks(&context);
    let report = render_doctor_report(&checks);
    write_text_report(&context.workspace_root, "doctor.md", &report)?;
    println!("{report}");

    let failures = checks.iter().filter(|check| !check.ok).collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "validation doctor found {} blocking prerequisite issue(s); see `{REPORT_DIR}/doctor.md`",
            failures.len()
        ))
    }
}

fn run_bootstrap(args: &[String]) -> Result<(), String> {
    ensure_no_trailing(args, "validate bootstrap")?;
    let context = load_context()?;

    super::ensure_command_available("rustup", &["--version"])?;
    run_command(
        Command::new("rustup")
            .current_dir(&context.workspace_root)
            .args(["target", "add", "wasm32-unknown-unknown"]),
    )?;

    if !command_available("trunk") {
        run_command(
            Command::new("cargo")
                .current_dir(&context.workspace_root)
                .args([
                    "install",
                    "trunk",
                    "--locked",
                    "--version",
                    &context.config.tools.trunk_version,
                ]),
        )?;
    }

    if !cargo_subcommand_available(&context.workspace_root, "audit") {
        run_command(
            Command::new("cargo")
                .current_dir(&context.workspace_root)
                .args([
                    "install",
                    "cargo-audit",
                    "--locked",
                    "--version",
                    &context.config.tools.cargo_audit_version,
                ]),
        )?;
    }

    ensure_command(
        "node",
        &["--version"],
        "install Node from `.nvmrc` before bootstrap",
    )?;
    ensure_command("npm", &["--version"], "install npm before bootstrap")?;

    run_command(
        Command::new("npm")
            .current_dir(&context.workspace_root)
            .args(["ci", "--prefix", "ui/e2e"]),
    )?;
    run_command(&mut playwright_install_command(
        &context.workspace_root,
        true,
    ))?;
    run_command(
        Command::new("npm")
            .current_dir(&context.workspace_root)
            .args(["ci", "--prefix", "infrastructure/pulumi"]),
    )?;

    run_doctor(&[])?;
    Ok(())
}

fn run_changed(args: &[String]) -> Result<(), String> {
    let mut base_ref = None;
    let mut head_ref = "HEAD".to_string();
    let mut fetch_base = false;
    let mut enforce_freshness = true;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--base" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --base".to_string());
                };
                base_ref = Some(value.clone());
                index += 2;
            }
            "--head" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --head".to_string());
                };
                head_ref.clone_from(value);
                index += 2;
            }
            "--fetch-base" => {
                fetch_base = true;
                index += 1;
            }
            "--no-freshness" => {
                enforce_freshness = false;
                index += 1;
            }
            other => return Err(format!("unknown validate changed argument `{other}`")),
        }
    }

    let context = load_context()?;
    let selection = select_changed_suites(
        &context,
        base_ref.as_deref(),
        &head_ref,
        fetch_base,
        enforce_freshness,
    )?;
    let suite_results = if selection.freshness_required && !selection.freshness_ok {
        vec![freshness_failure_result(&selection.base_ref)]
    } else {
        execute_suites(&context, &selection.suites)
    };
    let report = ValidationReport {
        mode: "changed".to_string(),
        event: None,
        branch: selection.branch,
        base_ref: Some(selection.base_ref),
        merge_base: Some(selection.merge_base),
        head_ref: selection.head_ref,
        changed_files: selection.changed_files,
        freshness_required: selection.freshness_required,
        freshness_ok: selection.freshness_ok,
        selected_suites: selection
            .suites
            .iter()
            .map(|suite| suite.as_str().to_string())
            .collect(),
        suite_results,
    };
    finish_report(&context.workspace_root, "changed", &report)
}

fn run_suite_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("expected `validate suite <governance|security|core|ui|ui-hardening|nomad|pulumi|full>...`".to_string());
    }
    let context = load_context()?;
    let suites = parse_suite_args(args)?;
    let branch = current_branch(&context.workspace_root)?;
    let head_ref = resolve_revision(&context.workspace_root, "HEAD")?;
    let suite_results = execute_suites(&context, &suites);
    let report = ValidationReport {
        mode: "suite".to_string(),
        event: None,
        branch,
        base_ref: None,
        merge_base: None,
        head_ref,
        changed_files: Vec::new(),
        freshness_required: false,
        freshness_ok: true,
        selected_suites: suites
            .iter()
            .map(|suite| suite.as_str().to_string())
            .collect(),
        suite_results,
    };
    finish_report(&context.workspace_root, "suite", &report)
}

fn run_ci(args: &[String]) -> Result<(), String> {
    let mut event = None;
    let mut base_ref = None;
    let mut head_ref = "HEAD".to_string();
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--event" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --event".to_string());
                };
                event = Some(value.clone());
                index += 2;
            }
            "--base" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --base".to_string());
                };
                base_ref = Some(value.clone());
                index += 2;
            }
            "--head" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --head".to_string());
                };
                head_ref.clone_from(value);
                index += 2;
            }
            other => return Err(format!("unknown validate ci argument `{other}`")),
        }
    }

    let event = event.ok_or_else(|| {
        "missing `--event <pull_request|push|merge_group|workflow_dispatch>`".to_string()
    })?;
    let context = load_context()?;
    let branch = current_branch(&context.workspace_root)?;
    let head_sha = resolve_revision(&context.workspace_root, &head_ref)?;

    let (selection, suites) = if matches!(event.as_str(), "merge_group" | "workflow_dispatch") {
        (
            None,
            vec![
                Suite::Governance,
                Suite::Security,
                Suite::Core,
                Suite::Ui,
                Suite::UiHardening,
                Suite::Nomad,
                Suite::Pulumi,
            ],
        )
    } else {
        let selection =
            select_changed_suites(&context, base_ref.as_deref(), &head_ref, false, false)?;
        let suites = selection
            .suites
            .iter()
            .copied()
            .filter(|suite| !matches!(suite, Suite::Governance | Suite::Security))
            .collect::<Vec<_>>();
        (Some(selection), suites)
    };
    let suite_results = execute_suites(&context, &suites);
    let report = ValidationReport {
        mode: "ci".to_string(),
        event: Some(event),
        branch,
        base_ref: selection.as_ref().map(|value| value.base_ref.clone()),
        merge_base: selection.as_ref().map(|value| value.merge_base.clone()),
        head_ref: head_sha,
        changed_files: selection
            .as_ref()
            .map(|value| value.changed_files.clone())
            .unwrap_or_default(),
        freshness_required: false,
        freshness_ok: true,
        selected_suites: suites
            .iter()
            .map(|suite| suite.as_str().to_string())
            .collect(),
        suite_results,
    };
    finish_report(&context.workspace_root, "ci", &report)
}

fn run_install_hooks(args: &[String]) -> Result<(), String> {
    ensure_no_trailing(args, "validate install-hooks")?;
    let workspace_root = workspace_root()?;
    let hook_path = workspace_root.join(PRE_PUSH_HOOK_PATH);
    if !hook_path.is_file() {
        return Err(format!(
            "required hook script `{}` is missing",
            hook_path.display()
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(&hook_path)
            .map_err(|error| format!("failed to read `{}`: {error}", hook_path.display()))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook_path, permissions)
            .map_err(|error| format!("failed to chmod `{}`: {error}", hook_path.display()))?;
    }

    run_command(Command::new("git").current_dir(&workspace_root).args([
        "config",
        "core.hooksPath",
        ".githooks",
    ]))?;
    println!("installed repository hooks from `.githooks`");
    Ok(())
}

fn load_context() -> Result<ValidationContext, String> {
    let workspace_root = workspace_root()?;
    let config = load_validation_config(&workspace_root)?;
    Ok(ValidationContext {
        workspace_root,
        config,
    })
}

fn load_validation_config(workspace_root: &Path) -> Result<ValidationConfig, String> {
    let raw = fs::read_to_string(workspace_root.join(VALIDATION_CONFIG_PATH))
        .map_err(|error| format!("failed to read `{VALIDATION_CONFIG_PATH}`: {error}"))?;
    let config: ValidationConfig = toml::from_str(&raw)
        .map_err(|error| format!("failed to parse `{VALIDATION_CONFIG_PATH}`: {error}"))?;
    if config.version != 1 {
        return Err(format!(
            "`{VALIDATION_CONFIG_PATH}` must declare `version = 1`"
        ));
    }
    Ok(config)
}

fn load_security_exceptions(workspace_root: &Path) -> Result<SecurityExceptions, String> {
    let raw = fs::read_to_string(workspace_root.join(SECURITY_EXCEPTIONS_PATH))
        .map_err(|error| format!("failed to read `{SECURITY_EXCEPTIONS_PATH}`: {error}"))?;
    let exceptions: SecurityExceptions = toml::from_str(&raw)
        .map_err(|error| format!("failed to parse `{SECURITY_EXCEPTIONS_PATH}`: {error}"))?;
    if exceptions.version != 1 {
        return Err(format!(
            "`{SECURITY_EXCEPTIONS_PATH}` must declare `version = 1`"
        ));
    }
    validate_security_exceptions(&exceptions)?;
    Ok(exceptions)
}

fn validate_security_exceptions(exceptions: &SecurityExceptions) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let today = Utc::now().date_naive();
    for exception in &exceptions.exceptions {
        if exception.ids.is_empty() {
            return Err(
                "security exception entries must include at least one advisory id".to_string(),
            );
        }
        if exception.owner.trim().is_empty() {
            return Err("security exception entries must include a non-empty owner".to_string());
        }
        if exception.issue == 0 {
            return Err("security exception entries must include a non-zero issue id".to_string());
        }
        if exception.reason.trim().is_empty() {
            return Err("security exception entries must include a non-empty reason".to_string());
        }
        let expires =
            chrono::NaiveDate::parse_from_str(&exception.expires, "%Y-%m-%d").map_err(|error| {
                format!(
                    "invalid security exception expiry `{}`: {error}",
                    exception.expires
                )
            })?;
        if expires < today {
            return Err(format!(
                "security exception entry for issue `#{}' is expired on `{}`",
                exception.issue, exception.expires
            ));
        }
        for id in &exception.ids {
            if !ids.insert(id.clone()) {
                return Err(format!("duplicate security exception id `{id}`"));
            }
        }
    }
    Ok(())
}

fn parse_suite_args(args: &[String]) -> Result<Vec<Suite>, String> {
    let mut suites = Vec::new();
    for name in args {
        match name.as_str() {
            "governance" => suites.push(Suite::Governance),
            "security" => suites.push(Suite::Security),
            "core" => suites.push(Suite::Core),
            "ui" => suites.push(Suite::Ui),
            "ui-hardening" => suites.push(Suite::UiHardening),
            "nomad" => suites.push(Suite::Nomad),
            "pulumi" => suites.push(Suite::Pulumi),
            "full" => suites.extend([
                Suite::Governance,
                Suite::Security,
                Suite::Core,
                Suite::Ui,
                Suite::UiHardening,
                Suite::Nomad,
                Suite::Pulumi,
            ]),
            other => return Err(format!("unknown validation suite `{other}`")),
        }
    }
    Ok(dedup_suites(suites))
}

fn select_changed_suites(
    context: &ValidationContext,
    explicit_base_ref: Option<&str>,
    head_ref: &str,
    fetch_base: bool,
    enforce_freshness: bool,
) -> Result<ChangedSelection, String> {
    let base_ref = resolve_base_ref(&context.workspace_root, explicit_base_ref);
    if fetch_base {
        fetch_base_ref(&context.workspace_root, &base_ref)?;
    }
    let merge_base = merge_base(&context.workspace_root, &base_ref, head_ref)?;
    let head_sha = resolve_revision(&context.workspace_root, head_ref)?;
    let changed_files = diff_name_only(&context.workspace_root, &merge_base, &head_sha)?;
    let suites = suites_for_changed_files(&changed_files, &context.config);
    let freshness_required = enforce_freshness
        && changed_files
            .iter()
            .any(|path| matches_any_prefix(path, &context.config.freshness.required_prefixes));
    let freshness_ok = if freshness_required {
        is_ancestor(&context.workspace_root, &base_ref, &head_sha)?
    } else {
        true
    };
    Ok(ChangedSelection {
        branch: current_branch(&context.workspace_root)?,
        base_ref,
        merge_base,
        head_ref: head_sha,
        changed_files,
        suites,
        freshness_required,
        freshness_ok,
    })
}

fn suites_for_changed_files(changed_files: &[String], config: &ValidationConfig) -> Vec<Suite> {
    let mut suites = vec![Suite::Governance, Suite::Security];
    let run_core = changed_files.iter().any(|path| {
        matches_shared_root(path, &config.selectors)
            || matches_any_prefix(path, &config.selectors.core_rust_prefixes)
    });
    let run_ui = changed_files.iter().any(|path| {
        matches_shared_root(path, &config.selectors)
            || matches_any_prefix(path, &config.selectors.ui_prefixes)
    });
    let run_nomad = changed_files.iter().any(|path| {
        matches_any_prefix(path, &config.selectors.nomad_prefixes)
            && config
                .selectors
                .nomad_suffixes
                .iter()
                .any(|suffix| path.ends_with(suffix))
    });
    let run_pulumi = changed_files
        .iter()
        .any(|path| matches_any_prefix(path, &config.selectors.pulumi_prefixes));

    if run_core {
        suites.push(Suite::Core);
    }
    if run_ui {
        suites.push(Suite::Ui);
        suites.push(Suite::UiHardening);
    }
    if run_nomad {
        suites.push(Suite::Nomad);
    }
    if run_pulumi {
        suites.push(Suite::Pulumi);
    }

    dedup_suites(suites)
}

fn execute_suites(context: &ValidationContext, suites: &[Suite]) -> Vec<SuiteResult> {
    let mut results = Vec::new();
    for suite in suites {
        let start = Instant::now();
        let outcome = run_suite(context, *suite);
        let elapsed_ms = start.elapsed().as_millis();
        match outcome {
            Ok(detail) => results.push(SuiteResult {
                suite: suite.as_str().to_string(),
                status: "passed".to_string(),
                detail,
                elapsed_ms,
            }),
            Err(detail) => {
                results.push(SuiteResult {
                    suite: suite.as_str().to_string(),
                    status: "failed".to_string(),
                    detail: detail.clone(),
                    elapsed_ms,
                });
                break;
            }
        }
    }
    results
}

fn run_suite(context: &ValidationContext, suite: Suite) -> Result<String, String> {
    match suite {
        Suite::Governance => {
            crate::architecture::run(vec!["audit-boundaries".to_string()])?;
            crate::plugin::run(vec!["validate-manifests".to_string()])?;
            crate::github::run(vec!["audit-process".to_string()])?;
            Ok("architecture, plugin, and process audits passed".to_string())
        }
        Suite::Security => run_security_suite(context),
        Suite::Core => {
            super::cargo(&context.workspace_root, &["fmt", "--all", "--check"])?;
            super::cargo(
                &context.workspace_root,
                &super::workspace_command_with_excludes(
                    &["clippy", "--workspace", "--all-targets", "--all-features"],
                    super::CORE_EXCLUDED_PACKAGES,
                    &["--", "-D", "warnings"],
                ),
            )?;
            super::cargo(
                &context.workspace_root,
                &super::workspace_command_with_excludes(
                    &["test", "--workspace", "--all-targets"],
                    super::CORE_EXCLUDED_PACKAGES,
                    &[],
                ),
            )?;
            Ok("workspace fmt, core clippy, and core tests passed".to_string())
        }
        Suite::Ui => {
            super::cargo(
                &context.workspace_root,
                &super::package_command_with_packages(
                    &["clippy", "--all-targets", "--all-features"],
                    super::UI_PACKAGES,
                    &["--", "-D", "warnings"],
                ),
            )?;
            super::cargo(
                &context.workspace_root,
                &super::package_command_with_packages(
                    &["test", "--all-targets"],
                    super::UI_PACKAGES,
                    &[],
                ),
            )?;
            super::verify_ui_browser_manifest_hygiene(&context.workspace_root)?;
            super::verify_ui_shell_style_hygiene(&context.workspace_root)?;
            super::run_ui_preview_smoke(&context.workspace_root)?;
            super::run_ui(vec![
                "build".to_string(),
                "--features".to_string(),
                "desktop-tauri".to_string(),
                "--dist".to_string(),
                "target/trunk-ci-dist".to_string(),
            ])?;
            Ok("ui clippy, tests, hygiene, preview smoke, and build passed".to_string())
        }
        Suite::UiHardening => {
            prepare_ui_browser_tooling(context)?;
            crate::ui_hardening::run(Vec::new())?;
            Ok("ui hardening verification passed".to_string())
        }
        Suite::Nomad => run_nomad_suite(context),
        Suite::Pulumi => run_pulumi_suite(context),
    }
}

fn run_security_suite(context: &ValidationContext) -> Result<String, String> {
    let exceptions = load_security_exceptions(&context.workspace_root)?;
    if !cargo_subcommand_available(&context.workspace_root, "audit") {
        run_command(
            Command::new("cargo")
                .current_dir(&context.workspace_root)
                .args([
                    "install",
                    "cargo-audit",
                    "--locked",
                    "--version",
                    &context.config.tools.cargo_audit_version,
                ]),
        )?;
    }

    let mut command = Command::new("cargo");
    command.current_dir(&context.workspace_root);
    command.arg("audit");
    command.arg("--json");
    for exception in &exceptions.exceptions {
        for id in &exception.ids {
            command.arg("--ignore");
            command.arg(id);
        }
    }

    let output = command_output(&mut command)?;
    let remaining = if output.stdout.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str::<serde_json::Value>(&output.stdout)
            .map_err(|error| format!("failed to parse cargo audit JSON output: {error}"))?
    };
    let report = render_security_report(&exceptions, &remaining);
    write_text_report(&context.workspace_root, "security.md", &report)?;
    write_json_report(&context.workspace_root, "security.json", &remaining)?;

    if output.status.success() {
        Ok(format!(
            "cargo audit passed with {} repo-owned exception group(s)",
            exceptions.exceptions.len()
        ))
    } else {
        Err(format!(
            "cargo audit reported unapproved findings; see `{REPORT_DIR}/security.md`"
        ))
    }
}

fn run_nomad_suite(context: &ValidationContext) -> Result<String, String> {
    ensure_command(
        "docker",
        &["--version"],
        "install Docker before running the Nomad suite",
    )?;
    let job_dir = context.workspace_root.join("infrastructure/nomad/jobs");
    if job_dir.exists() {
        let output = command_output(
            Command::new("rg")
                .current_dir(&context.workspace_root)
                .args([
                    "-n",
                    "driver\\s*=\\s*\"raw_exec\"",
                    "infrastructure/nomad/jobs",
                ]),
        )?;
        if output.status.success() {
            return Err("raw_exec deployments are not allowed for workload services".to_string());
        }
        if output.status.code() != Some(1) {
            return Err("failed to scan Nomad jobs for raw_exec posture".to_string());
        }
    }

    let files = collect_nomad_job_files(&job_dir)?;
    for file in files {
        run_command(
            Command::new("docker")
                .current_dir(&context.workspace_root)
                .args([
                    "run",
                    "--rm",
                    "-v",
                    &format!("{}:/workspace", context.workspace_root.display()),
                    "-w",
                    "/workspace",
                    &context.config.tools.nomad_image,
                    "nomad",
                    "job",
                    "validate",
                    &relative_to_workspace(&context.workspace_root, &file),
                ]),
        )?;
    }
    Ok("nomad posture scan and job validation passed".to_string())
}

fn run_pulumi_suite(context: &ValidationContext) -> Result<String, String> {
    ensure_command(
        "node",
        &["--version"],
        "install Node before running the Pulumi suite",
    )?;
    ensure_command(
        "npm",
        &["--version"],
        "install npm before running the Pulumi suite",
    )?;
    run_command(
        Command::new("npm")
            .current_dir(&context.workspace_root)
            .args(["ci", "--prefix", "infrastructure/pulumi"]),
    )?;
    run_command(
        Command::new("npm")
            .current_dir(context.workspace_root.join("infrastructure/pulumi"))
            .arg("test"),
    )?;
    Ok("pulumi workspace install and tests passed".to_string())
}

fn doctor_checks(context: &ValidationContext) -> Vec<DoctorCheck> {
    let workspace_root = &context.workspace_root;
    let mut checks = Vec::new();
    checks.push(command_check(
        "rustc",
        Command::new("rustc").arg("--version"),
        "Install the pinned Rust toolchain from `rust-toolchain.toml`.",
    ));
    checks.push(command_check(
        "cargo",
        Command::new("cargo").arg("--version"),
        "Install the pinned Rust toolchain from `rust-toolchain.toml`.",
    ));
    checks.push(target_check(
        workspace_root,
        "wasm32-unknown-unknown",
        "Run `rustup target add wasm32-unknown-unknown`.",
    ));
    checks.push(command_check(
        "trunk",
        Command::new("trunk").arg("--version"),
        &format!(
            "Run `cargo install trunk --locked --version {}`.",
            context.config.tools.trunk_version
        ),
    ));
    checks.push(command_check(
        "cargo-audit",
        Command::new("cargo").arg("audit").arg("--version"),
        &format!(
            "Run `cargo install cargo-audit --locked --version {}`.",
            context.config.tools.cargo_audit_version
        ),
    ));
    checks.push(command_check(
        "node",
        Command::new("node").arg("--version"),
        "Install Node using the version declared in `.nvmrc`.",
    ));
    if let Some(node_check) = checks.last_mut()
        && node_check.ok
        && let Some(major) = node_major_version(&node_check.observed)
        && major < context.config.tools.node_major
    {
        node_check.ok = false;
        node_check.remediation = format!(
            "Install Node {} or newer using the version declared in `.nvmrc`.",
            context.config.tools.node_major
        );
        node_check.observed = format!(
            "{} (requires Node {} or newer)",
            node_check.observed, context.config.tools.node_major
        );
    }
    checks.push(command_check(
        "npm",
        Command::new("npm").arg("--version"),
        "Install npm alongside Node.",
    ));
    checks.push(path_check(
        "ui/e2e dependencies",
        workspace_root
            .join("ui/e2e/node_modules/playwright")
            .exists(),
        "Run `npm ci --prefix ui/e2e`.",
    ));
    checks.push(path_check(
        "Playwright browsers",
        playwright_cache_available(),
        "Run `npx --prefix ui/e2e playwright install chromium firefox webkit`.",
    ));
    checks.push(path_check(
        "Pulumi workspace dependencies",
        workspace_root
            .join("infrastructure/pulumi/node_modules")
            .exists(),
        "Run `npm ci --prefix infrastructure/pulumi`.",
    ));
    checks.push(command_check(
        "docker",
        Command::new("docker").arg("--version"),
        "Install Docker Desktop or another compatible Docker runtime.",
    ));
    checks
}

#[derive(Debug)]
struct DoctorCheck {
    name: String,
    ok: bool,
    observed: String,
    remediation: String,
}

fn command_check(name: &str, command: &mut Command, remediation: &str) -> DoctorCheck {
    let result = command_output(command);
    match result {
        Ok(output) if output.status.success() => DoctorCheck {
            name: name.to_string(),
            ok: true,
            observed: output.stdout.trim().to_string(),
            remediation: remediation.to_string(),
        },
        Ok(output) => DoctorCheck {
            name: name.to_string(),
            ok: false,
            observed: stderr_or_stdout(&output),
            remediation: remediation.to_string(),
        },
        Err(error) => DoctorCheck {
            name: name.to_string(),
            ok: false,
            observed: error,
            remediation: remediation.to_string(),
        },
    }
}

fn target_check(workspace_root: &Path, target: &str, remediation: &str) -> DoctorCheck {
    let output = command_output(Command::new("rustup").current_dir(workspace_root).args([
        "target",
        "list",
        "--installed",
    ]));
    match output {
        Ok(output) if output.status.success() => {
            let installed = output.stdout.lines().any(|line| line.trim() == target);
            DoctorCheck {
                name: format!("Rust target `{target}`"),
                ok: installed,
                observed: output.stdout.trim().to_string(),
                remediation: remediation.to_string(),
            }
        }
        Ok(output) => DoctorCheck {
            name: format!("Rust target `{target}`"),
            ok: false,
            observed: stderr_or_stdout(&output),
            remediation: remediation.to_string(),
        },
        Err(error) => DoctorCheck {
            name: format!("Rust target `{target}`"),
            ok: false,
            observed: error,
            remediation: remediation.to_string(),
        },
    }
}

fn path_check(name: &str, exists: bool, remediation: &str) -> DoctorCheck {
    DoctorCheck {
        name: name.to_string(),
        ok: exists,
        observed: if exists { "available" } else { "missing" }.to_string(),
        remediation: remediation.to_string(),
    }
}

fn render_doctor_report(checks: &[DoctorCheck]) -> String {
    let mut lines = vec!["# Validation Doctor".to_string(), String::new()];
    for check in checks {
        lines.push(format!(
            "- [{}] {}: {}",
            if check.ok { "PASS" } else { "FAIL" },
            check.name,
            check.observed
        ));
        if !check.ok {
            lines.push(format!("  remediation: {}", check.remediation));
        }
    }
    lines.join("\n")
}

fn render_security_report(
    exceptions: &SecurityExceptions,
    remaining: &serde_json::Value,
) -> String {
    let mut lines = vec!["# Security Validation".to_string(), String::new()];
    lines.push(format!(
        "- exception groups: {}",
        exceptions.exceptions.len()
    ));
    for exception in &exceptions.exceptions {
        lines.push(format!(
            "- issue #{issue} | expires {expires} | owner {owner} | ids {ids}",
            issue = exception.issue,
            expires = exception.expires,
            owner = exception.owner,
            ids = exception.ids.join(", "),
        ));
        lines.push(format!("  reason: {}", exception.reason));
    }
    let remaining_count = remaining
        .pointer("/vulnerabilities/count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    lines.push(String::new());
    lines.push(format!(
        "- remaining unignored vulnerability count: {remaining_count}"
    ));
    lines.join("\n")
}

fn finish_report(
    workspace_root: &Path,
    stem: &str,
    report: &ValidationReport,
) -> Result<(), String> {
    write_json_report(workspace_root, &format!("{stem}.json"), report)?;
    write_text_report(
        workspace_root,
        &format!("{stem}.md"),
        &render_validation_report(report),
    )?;
    println!("{}", render_validation_report(report));

    if report
        .suite_results
        .iter()
        .any(|result| result.status != "passed")
    {
        Err(format!("validation failed; see `{REPORT_DIR}/{stem}.md`"))
    } else {
        Ok(())
    }
}

fn freshness_failure_result(base_ref: &str) -> SuiteResult {
    SuiteResult {
        suite: "freshness".to_string(),
        status: "failed".to_string(),
        detail: format!(
            "branch is stale relative to `{base_ref}` for conflict-prone paths; refresh from the latest target branch before pushing"
        ),
        elapsed_ms: 0,
    }
}

fn render_validation_report(report: &ValidationReport) -> String {
    let mut lines = vec!["# Validation Report".to_string(), String::new()];
    lines.push(format!("- mode: {}", report.mode));
    if let Some(event) = &report.event {
        lines.push(format!("- event: {event}"));
    }
    lines.push(format!("- branch: {}", report.branch));
    if let Some(base_ref) = &report.base_ref {
        lines.push(format!("- base ref: {base_ref}"));
    }
    if let Some(merge_base) = &report.merge_base {
        lines.push(format!("- merge base: {merge_base}"));
    }
    lines.push(format!("- head ref: {}", report.head_ref));
    lines.push(format!(
        "- freshness: {}",
        if report.freshness_required {
            if report.freshness_ok {
                "required and satisfied"
            } else {
                "required and failed"
            }
        } else {
            "not required"
        }
    ));
    lines.push(format!(
        "- selected suites: {}",
        if report.selected_suites.is_empty() {
            "none".to_string()
        } else {
            report.selected_suites.join(", ")
        }
    ));
    if !report.changed_files.is_empty() {
        lines.push("- changed files:".to_string());
        lines.extend(
            report
                .changed_files
                .iter()
                .map(|path| format!("  - {path}")),
        );
    }
    lines.push(String::new());
    lines.push("## Suites".to_string());
    for result in &report.suite_results {
        lines.push(format!(
            "- [{}] {} ({} ms): {}",
            if result.status == "passed" {
                "PASS"
            } else {
                "FAIL"
            },
            result.suite,
            result.elapsed_ms,
            result.detail
        ));
    }
    lines.join("\n")
}

fn write_json_report<T: Serialize>(
    workspace_root: &Path,
    name: &str,
    value: &T,
) -> Result<(), String> {
    let report_dir = workspace_root.join(REPORT_DIR);
    fs::create_dir_all(&report_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", report_dir.display()))?;
    fs::write(
        report_dir.join(name),
        serde_json::to_vec_pretty(value)
            .map_err(|error| format!("failed to serialize validation report: {error}"))?,
    )
    .map_err(|error| {
        format!(
            "failed to write `{}`: {error}",
            report_dir.join(name).display()
        )
    })
}

fn write_text_report(workspace_root: &Path, name: &str, contents: &str) -> Result<(), String> {
    let report_dir = workspace_root.join(REPORT_DIR);
    fs::create_dir_all(&report_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", report_dir.display()))?;
    fs::write(report_dir.join(name), contents).map_err(|error| {
        format!(
            "failed to write `{}`: {error}",
            report_dir.join(name).display()
        )
    })
}

fn prepare_ui_browser_tooling(context: &ValidationContext) -> Result<(), String> {
    ensure_command(
        "node",
        &["--version"],
        "install Node before running UI hardening",
    )?;
    ensure_command(
        "npm",
        &["--version"],
        "install npm before running UI hardening",
    )?;
    run_command(
        Command::new("npm")
            .current_dir(&context.workspace_root)
            .args(["ci", "--prefix", "ui/e2e"]),
    )?;
    run_command(&mut playwright_install_command(
        &context.workspace_root,
        env::var_os("CI").is_some(),
    ))?;
    Ok(())
}

fn playwright_install_command(workspace_root: &Path, ci_mode: bool) -> Command {
    let mut command = Command::new("npx");
    command.current_dir(workspace_root);
    command.args(["--prefix", "ui/e2e", "playwright", "install"]);
    if ci_mode && cfg!(target_os = "linux") {
        command.arg("--with-deps");
    }
    command.args(["chromium", "firefox", "webkit"]);
    command
}

fn resolve_base_ref(workspace_root: &Path, explicit: Option<&str>) -> String {
    if let Some(base) = explicit {
        return base.to_string();
    }
    if let Ok(output) = command_output(Command::new("git").current_dir(workspace_root).args([
        "rev-parse",
        "--abbrev-ref",
        "--symbolic-full-name",
        "@{upstream}",
    ])) && output.status.success()
    {
        let upstream = output.stdout.trim();
        if !upstream.is_empty() {
            return upstream.to_string();
        }
    }
    "origin/main".to_string()
}

fn fetch_base_ref(workspace_root: &Path, base_ref: &str) -> Result<(), String> {
    let remote = base_ref
        .strip_prefix("refs/remotes/")
        .and_then(|value| value.split('/').next())
        .or_else(|| base_ref.split('/').next())
        .filter(|value| *value != "HEAD")
        .unwrap_or("origin");
    run_command(
        Command::new("git")
            .current_dir(workspace_root)
            .args(["fetch", remote]),
    )
}

fn current_branch(workspace_root: &Path) -> Result<String, String> {
    let output = command_output(
        Command::new("git")
            .current_dir(workspace_root)
            .args(["branch", "--show-current"]),
    )?;
    let branch = output.stdout.trim();
    if branch.is_empty() {
        Ok("detached".to_string())
    } else {
        Ok(branch.to_string())
    }
}

fn resolve_revision(workspace_root: &Path, rev: &str) -> Result<String, String> {
    let output = command_output(
        Command::new("git")
            .current_dir(workspace_root)
            .args(["rev-parse", rev]),
    )?;
    if output.status.success() {
        Ok(output.stdout.trim().to_string())
    } else {
        Err(stderr_or_stdout(&output))
    }
}

fn merge_base(workspace_root: &Path, base_ref: &str, head_ref: &str) -> Result<String, String> {
    let output = command_output(Command::new("git").current_dir(workspace_root).args([
        "merge-base",
        base_ref,
        head_ref,
    ]))?;
    if output.status.success() {
        Ok(output.stdout.trim().to_string())
    } else {
        Err(stderr_or_stdout(&output))
    }
}

fn is_ancestor(workspace_root: &Path, base_ref: &str, head_ref: &str) -> Result<bool, String> {
    let output = command_output(Command::new("git").current_dir(workspace_root).args([
        "merge-base",
        "--is-ancestor",
        base_ref,
        head_ref,
    ]))?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(stderr_or_stdout(&output)),
    }
}

fn diff_name_only(workspace_root: &Path, base: &str, head: &str) -> Result<Vec<String>, String> {
    let output = command_output(Command::new("git").current_dir(workspace_root).args([
        "diff",
        "--name-only",
        &format!("{base}..{head}"),
    ]))?;
    if !output.status.success() {
        return Err(stderr_or_stdout(&output));
    }
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn matches_shared_root(path: &str, selectors: &SelectorConfig) -> bool {
    selectors
        .shared_root_files
        .iter()
        .any(|candidate| path == candidate)
        || matches_any_prefix(path, &selectors.shared_root_prefixes)
}

fn matches_any_prefix(path: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| path.starts_with(prefix))
}

fn dedup_suites(suites: Vec<Suite>) -> Vec<Suite> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for suite in suites {
        if seen.insert(suite) {
            deduped.push(suite);
        }
    }
    deduped
}

fn command_available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn cargo_subcommand_available(workspace_root: &Path, subcommand: &str) -> bool {
    Command::new("cargo")
        .current_dir(workspace_root)
        .arg(subcommand)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn ensure_command(program: &str, args: &[&str], remediation: &str) -> Result<(), String> {
    if !command_available(program) {
        return Err(remediation.to_string());
    }
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("failed to launch `{program}`: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(remediation.to_string())
    }
}

fn command_output(command: &mut Command) -> Result<CapturedOutput, String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to start `{}`: {error}", display_command(command)))?;
    Ok(CapturedOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

struct CapturedOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn stderr_or_stdout(output: &CapturedOutput) -> String {
    if !output.stderr.trim().is_empty() {
        output.stderr.trim().to_string()
    } else if !output.stdout.trim().is_empty() {
        output.stdout.trim().to_string()
    } else {
        "command failed without output".to_string()
    }
}

fn collect_nomad_job_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_nomad_job_files_recursive(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_nomad_job_files_recursive(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry = entry
            .map_err(|error| format!("failed to read entry in `{}`: {error}", root.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_nomad_job_files_recursive(&path, files)?;
            continue;
        }
        if path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.ends_with(".nomad.hcl"))
        {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_to_workspace(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn playwright_cache_available() -> bool {
    let mut candidates = Vec::new();
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        candidates.push(home.join("Library/Caches/ms-playwright"));
        candidates.push(home.join(".cache/ms-playwright"));
    }
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        candidates.push(PathBuf::from(local_app_data).join("ms-playwright"));
    }
    candidates.into_iter().any(|path| path.exists())
}

fn node_major_version(raw: &str) -> Option<u64> {
    let trimmed = raw.trim().trim_start_matches('v');
    trimmed.split('.').next()?.parse().ok()
}

fn ensure_no_trailing(args: &[String], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("unexpected trailing arguments for `{command}`"))
    }
}

impl Suite {
    fn as_str(self) -> &'static str {
        match self {
            Suite::Governance => "governance",
            Suite::Security => "security",
            Suite::Core => "core",
            Suite::Ui => "ui",
            Suite::UiHardening => "ui-hardening",
            Suite::Nomad => "nomad",
            Suite::Pulumi => "pulumi",
        }
    }
}

fn help() -> &'static str {
    "\
usage: cargo xtask validate <command> ...

Commands:
  doctor         Validate local prerequisites and report remediation commands
  bootstrap      Install or prepare native-first local validation dependencies
  changed        Run repo-owned validation suites selected from changed files
  suite          Run one or more explicit validation suites
  ci             Run repo-owned CI suite selection and execution for GitHub workflows
  install-hooks  Install blocking repository pre-push hooks
"
}

#[cfg(test)]
mod tests {
    use super::{
        SecurityException, SecurityExceptions, Suite, ValidationContext, dedup_suites,
        load_validation_config, resolve_base_ref, select_changed_suites, suites_for_changed_files,
        validate_security_exceptions,
    };
    use chrono::{Duration, Utc};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn suites_for_ui_only_changes_include_ui_hardening() {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let config = load_validation_config(&workspace_root).expect("load validation config");
        let suites =
            suites_for_changed_files(&["ui/crates/site/src/web_app.rs".to_string()], &config);
        assert_eq!(
            suites,
            vec![
                Suite::Governance,
                Suite::Security,
                Suite::Ui,
                Suite::UiHardening
            ]
        );
    }

    #[test]
    fn suites_for_service_changes_include_core_only() {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let config = load_validation_config(&workspace_root).expect("load validation config");
        let suites = suites_for_changed_files(
            &["services/finance-service/src/lib.rs".to_string()],
            &config,
        );
        assert_eq!(
            suites,
            vec![Suite::Governance, Suite::Security, Suite::Core]
        );
    }

    #[test]
    fn suites_for_shared_root_changes_include_core_and_ui() {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let config = load_validation_config(&workspace_root).expect("load validation config");
        let suites = suites_for_changed_files(&["Cargo.toml".to_string()], &config);
        assert_eq!(
            suites,
            vec![
                Suite::Governance,
                Suite::Security,
                Suite::Core,
                Suite::Ui,
                Suite::UiHardening
            ]
        );
    }

    #[test]
    fn suites_for_nomad_and_pulumi_changes_include_infra_suites() {
        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let config = load_validation_config(&workspace_root).expect("load validation config");
        let suites = suites_for_changed_files(
            &[
                "infrastructure/nomad/jobs/app.nomad.hcl".to_string(),
                "infrastructure/pulumi/package.json".to_string(),
            ],
            &config,
        );
        assert_eq!(
            suites,
            vec![
                Suite::Governance,
                Suite::Security,
                Suite::Nomad,
                Suite::Pulumi
            ]
        );
    }

    #[test]
    fn dedup_suites_preserves_order() {
        assert_eq!(
            dedup_suites(vec![Suite::Governance, Suite::Security, Suite::Governance]),
            vec![Suite::Governance, Suite::Security]
        );
    }

    #[test]
    fn security_exceptions_require_non_empty_owner() {
        let error = validate_security_exceptions(&SecurityExceptions {
            version: 1,
            exceptions: vec![SecurityException {
                ids: vec!["RUSTSEC-2099-0001".to_string()],
                owner: String::new(),
                issue: 139,
                expires: future_expiry(),
                reason: "tracked".to_string(),
            }],
        })
        .expect_err("missing owner should fail");
        assert!(error.contains("non-empty owner"));
    }

    #[test]
    fn security_exceptions_reject_duplicate_ids() {
        let error = validate_security_exceptions(&SecurityExceptions {
            version: 1,
            exceptions: vec![
                SecurityException {
                    ids: vec!["RUSTSEC-2099-0001".to_string()],
                    owner: "@owner".to_string(),
                    issue: 139,
                    expires: future_expiry(),
                    reason: "tracked".to_string(),
                },
                SecurityException {
                    ids: vec!["RUSTSEC-2099-0001".to_string()],
                    owner: "@owner".to_string(),
                    issue: 140,
                    expires: future_expiry(),
                    reason: "tracked".to_string(),
                },
            ],
        })
        .expect_err("duplicate ids should fail");
        assert!(error.contains("duplicate security exception id"));
    }

    #[test]
    fn security_exceptions_require_issue_link() {
        let error = validate_security_exceptions(&SecurityExceptions {
            version: 1,
            exceptions: vec![SecurityException {
                ids: vec!["RUSTSEC-2099-0001".to_string()],
                owner: "@owner".to_string(),
                issue: 0,
                expires: future_expiry(),
                reason: "tracked".to_string(),
            }],
        })
        .expect_err("missing issue should fail");
        assert!(error.contains("non-zero issue id"));
    }

    #[test]
    fn security_exceptions_require_valid_expiry() {
        let error = validate_security_exceptions(&SecurityExceptions {
            version: 1,
            exceptions: vec![SecurityException {
                ids: vec!["RUSTSEC-2099-0001".to_string()],
                owner: "@owner".to_string(),
                issue: 139,
                expires: String::new(),
                reason: "tracked".to_string(),
            }],
        })
        .expect_err("missing expiry should fail");
        assert!(error.contains("invalid security exception expiry"));
    }

    #[test]
    fn security_exceptions_reject_expired_entries() {
        let error = validate_security_exceptions(&SecurityExceptions {
            version: 1,
            exceptions: vec![SecurityException {
                ids: vec!["RUSTSEC-2099-0001".to_string()],
                owner: "@owner".to_string(),
                issue: 139,
                expires: expired_expiry(),
                reason: "tracked".to_string(),
            }],
        })
        .expect_err("expired exception should fail");
        assert!(error.contains("expired"));
    }

    #[test]
    fn resolve_base_ref_uses_upstream_branch_when_available() {
        let (_tempdir, repo) = init_git_repo();
        commit_file(&repo, "README.md", "base\n", "base");
        run_git(&repo, &["push", "-u", "origin", "main"]);

        run_git(&repo, &["checkout", "-b", "feature/139-parent"]);
        commit_file(&repo, "parent.txt", "parent\n", "parent");
        run_git(&repo, &["push", "-u", "origin", "feature/139-parent"]);

        run_git(&repo, &["checkout", "-b", "feature/139-child"]);
        run_git(
            &repo,
            &[
                "branch",
                "--set-upstream-to",
                "origin/feature/139-parent",
                "feature/139-child",
            ],
        );

        let base = resolve_base_ref(&repo, None);
        assert_eq!(base, "origin/feature/139-parent");
    }

    #[test]
    fn resolve_base_ref_falls_back_to_origin_main_without_upstream() {
        let (_tempdir, repo) = init_git_repo();
        commit_file(&repo, "README.md", "base\n", "base");
        run_git(&repo, &["push", "-u", "origin", "main"]);
        run_git(&repo, &["checkout", "-b", "feature/139-local-only"]);

        let base = resolve_base_ref(&repo, None);
        assert_eq!(base, "origin/main");
    }

    #[test]
    fn stale_hotspot_branches_require_refresh() {
        let (_tempdir, repo) = init_git_repo();
        commit_file(&repo, "README.md", "base\n", "base");
        run_git(&repo, &["push", "-u", "origin", "main"]);

        run_git(&repo, &["checkout", "-b", "feature/139-ui-change"]);
        commit_file(
            &repo,
            "ui/crates/system_ui/src/lib.rs",
            "feature change\n",
            "ui change",
        );

        run_git(&repo, &["checkout", "main"]);
        commit_file(&repo, "shared/stability.txt", "main drift\n", "main drift");
        run_git(&repo, &["push", "origin", "main"]);
        run_git(&repo, &["checkout", "feature/139-ui-change"]);
        run_git(&repo, &["fetch", "origin"]);

        let workspace_root = crate::common::workspace_root().expect("workspace root");
        let config = load_validation_config(&workspace_root).expect("load validation config");
        let context = ValidationContext {
            workspace_root: repo.clone(),
            config,
        };
        let selection = select_changed_suites(&context, Some("origin/main"), "HEAD", false, true)
            .expect("select changed suites");
        assert!(selection.freshness_required);
        assert!(!selection.freshness_ok);
        assert!(
            selection
                .changed_files
                .iter()
                .any(|path| path == "ui/crates/system_ui/src/lib.rs")
        );
        assert_eq!(
            selection
                .suites
                .iter()
                .map(|suite| suite.as_str())
                .collect::<Vec<_>>(),
            vec!["governance", "security", "ui", "ui-hardening"]
        );
    }

    fn future_expiry() -> String {
        (Utc::now().date_naive() + Duration::days(30))
            .format("%Y-%m-%d")
            .to_string()
    }

    fn expired_expiry() -> String {
        (Utc::now().date_naive() - Duration::days(1))
            .format("%Y-%m-%d")
            .to_string()
    }

    fn init_git_repo() -> (TempDir, PathBuf) {
        let tempdir = tempfile::tempdir().expect("create tempdir");
        let origin = tempdir.path().join("origin.git");
        let repo = tempdir.path().join("repo");

        run_git(
            tempdir.path(),
            &["init", "--bare", origin.to_str().expect("origin path")],
        );
        run_git(
            tempdir.path(),
            &[
                "clone",
                origin.to_str().expect("origin path"),
                repo.to_str().expect("repo path"),
            ],
        );
        run_git(&repo, &["config", "user.name", "Origin Validation Tests"]);
        run_git(
            &repo,
            &["config", "user.email", "validation-tests@example.com"],
        );
        run_git(&repo, &["checkout", "-b", "main"]);

        (tempdir, repo)
    }

    fn commit_file(repo: &Path, relative_path: &str, contents: &str, message: &str) {
        let path = repo.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directories");
        }
        fs::write(&path, contents).expect("write file");
        run_git(repo, &["add", relative_path]);
        run_git(repo, &["commit", "-m", message]);
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(repo)
            .args(args)
            .output()
            .expect("launch git");
        assert!(
            output.status.success(),
            "git {:?} failed:\nstdout:\n{}\nstderr:\n{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
