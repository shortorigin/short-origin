mod delivery;
mod github;
mod infra;
pub mod security_book;
mod ui_e2e;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use clap::{Args, Parser, Subcommand, ValueEnum};
use codegen::{embedded_contract_schemas, embedded_event_schemas, embedded_surrealdb_schemas};
use infra::InfraStack;
use serde::Serialize;
use ui_e2e::UiE2eScene;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPlan {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
    pub env: BTreeMap<String, String>,
}

impl CommandPlan {
    #[must_use]
    pub fn new(program: impl Into<String>, current_dir: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: current_dir.into(),
            env: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn push_arg(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
    }

    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.env.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }

    fn to_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.current_dir(&self.current_dir);
        command.args(&self.args);
        command.envs(self.env.iter());
        command
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum VerifyProfile {
    Fast,
    Ui,
    Full,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CacheMode {
    Auto,
    On,
    Off,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheSource {
    ExistingWrapper,
    AutoDetectedSccache,
    DisabledByMode,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheStatus {
    pub requested_mode: CacheMode,
    pub source: CacheSource,
    pub wrapper: Option<String>,
}

impl CacheMode {
    fn parse(raw: Option<&str>) -> Result<Self, String> {
        match raw.unwrap_or("auto") {
            "auto" => Ok(Self::Auto),
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            other => Err(format!(
                "invalid SHORT_ORIGIN_SCCACHE value `{other}`; expected one of auto, on, off"
            )),
        }
    }

    #[must_use]
    fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

impl CacheSource {
    #[must_use]
    fn label(&self) -> &'static str {
        match self {
            Self::ExistingWrapper => "existing-rustc-wrapper",
            Self::AutoDetectedSccache => "auto-detected-sccache",
            Self::DisabledByMode => "disabled",
            Self::Unavailable => "unavailable",
        }
    }
}

impl CacheStatus {
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.wrapper.is_some()
    }

    pub fn apply_to(&self, plan: &mut CommandPlan) {
        if let Some(wrapper) = &self.wrapper {
            plan.set_env("RUSTC_WRAPPER", wrapper);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, ValueEnum)]
enum DoctorDomain {
    Core,
    Ui,
    Docs,
    Infra,
    Security,
    All,
}

impl DoctorDomain {
    #[must_use]
    fn label(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Ui => "ui",
            Self::Docs => "docs",
            Self::Infra => "infra",
            Self::Security => "security",
            Self::All => "all",
        }
    }

    #[must_use]
    fn matches(self, domain: TaskDomain) -> bool {
        matches!(self, Self::All) || domain == self.into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

impl From<DoctorDomain> for TaskDomain {
    fn from(value: DoctorDomain) -> Self {
        match value {
            DoctorDomain::Core | DoctorDomain::All => Self::Core,
            DoctorDomain::Ui => Self::Ui,
            DoctorDomain::Docs => Self::Docs,
            DoctorDomain::Infra => Self::Infra,
            DoctorDomain::Security => Self::Security,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TaskDomain {
    Core,
    Ui,
    Docs,
    Infra,
    Security,
}

impl TaskDomain {
    #[must_use]
    fn label(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Ui => "ui",
            Self::Docs => "docs",
            Self::Infra => "infra",
            Self::Security => "security",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum ToolKind {
    Rustfmt,
    Clippy,
    Sccache,
    Trunk,
    CargoTauri,
    Mdbook,
    CargoNextest,
    Node,
    Npm,
    Pulumi,
    CargoAudit,
    CargoFuzz,
}

impl ToolKind {
    #[must_use]
    fn label(self) -> &'static str {
        match self {
            Self::Rustfmt => "rustfmt",
            Self::Clippy => "clippy",
            Self::Sccache => "sccache",
            Self::Trunk => "trunk",
            Self::CargoTauri => "cargo tauri",
            Self::Mdbook => "mdbook",
            Self::CargoNextest => "cargo-nextest",
            Self::Node => "node",
            Self::Npm => "npm",
            Self::Pulumi => "pulumi",
            Self::CargoAudit => "cargo-audit",
            Self::CargoFuzz => "cargo-fuzz",
        }
    }

    #[must_use]
    fn guidance(self) -> &'static str {
        match self {
            Self::Rustfmt => "Install the rustfmt component with `rustup component add rustfmt`.",
            Self::Clippy => "Install the clippy component with `rustup component add clippy`.",
            Self::Sccache => "Install sccache with `cargo install sccache --locked`.",
            Self::Trunk => "Install Trunk with `cargo install trunk --locked`.",
            Self::CargoTauri => "Install the Tauri CLI with `cargo install tauri-cli --locked`.",
            Self::Mdbook => "Install mdBook with `cargo install mdbook --locked`.",
            Self::CargoNextest => {
                "Install cargo-nextest with `cargo install cargo-nextest --locked`."
            }
            Self::Node | Self::Npm => "Install Node.js 20+ so both `node` and `npm` are on PATH.",
            Self::Pulumi => "Install the Pulumi CLI and ensure `pulumi` is on PATH.",
            Self::CargoAudit => "Install cargo-audit with `cargo install cargo-audit --locked`.",
            Self::CargoFuzz => "Install cargo-fuzz with `cargo install cargo-fuzz --locked`.",
        }
    }

    fn check(self) -> Result<(), String> {
        match self {
            Self::Rustfmt => {
                ensure_command_available("cargo", &["fmt", "--version"], self.guidance())
            }
            Self::Clippy => {
                ensure_command_available("cargo", &["clippy", "--version"], self.guidance())
            }
            Self::Sccache => ensure_command_available("sccache", &["--version"], self.guidance()),
            Self::Trunk => ensure_command_available("trunk", &["--version"], self.guidance()),
            Self::CargoTauri => {
                ensure_command_available("cargo", &["tauri", "--version"], self.guidance())
            }
            Self::Mdbook => ensure_command_available("mdbook", &["--version"], self.guidance()),
            Self::CargoNextest => {
                ensure_command_available("cargo", &["nextest", "--version"], self.guidance())
            }
            Self::Node => ensure_command_available("node", &["--version"], self.guidance()),
            Self::Npm => ensure_command_available("npm", &["--version"], self.guidance()),
            Self::Pulumi => ensure_command_available("pulumi", &["version"], self.guidance()),
            Self::CargoAudit => {
                ensure_command_available("cargo", &["audit", "--version"], self.guidance())
            }
            Self::CargoFuzz => {
                ensure_command_available("cargo", &["fuzz", "--version"], self.guidance())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TaskListItem {
    id: &'static str,
    description: &'static str,
    domains: Vec<&'static str>,
    prerequisites: Vec<&'static str>,
    dependencies: Vec<&'static str>,
    ci_included: bool,
    listed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TaskListOutput {
    tasks: Vec<TaskListItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DoctorEntryStatus {
    tool: &'static str,
    required_by: Vec<&'static str>,
    optional: bool,
    status: &'static str,
    guidance: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DoctorOutput {
    domain: &'static str,
    missing_required: bool,
    entries: Vec<DoctorEntryStatus>,
    notes: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum TaskAction {
    NoOp,
    VerifyFast,
    VerifyFull,
    UiCompileChecks,
    UiValidationBuild,
    UiE2eAll,
    ComponentsBuild,
    DocsSecurityBookTest,
    InfraVerify,
    SecurityAudit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaskSpec {
    id: &'static str,
    description: &'static str,
    domains: Vec<TaskDomain>,
    prerequisites: Vec<ToolKind>,
    dependencies: Vec<&'static str>,
    ci_included: bool,
    listed: bool,
    action: TaskAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DoctorEntry {
    tool: ToolKind,
    required_by: BTreeSet<&'static str>,
    optional: bool,
}

pub fn resolve_cache_status(
    existing_wrapper: Option<String>,
    mode: CacheMode,
    detected_sccache: Option<PathBuf>,
) -> Result<CacheStatus, String> {
    if let Some(wrapper) = existing_wrapper {
        return Ok(CacheStatus {
            requested_mode: mode,
            source: CacheSource::ExistingWrapper,
            wrapper: Some(wrapper),
        });
    }

    match (mode, detected_sccache) {
        (CacheMode::Off, _) => Ok(CacheStatus {
            requested_mode: mode,
            source: CacheSource::DisabledByMode,
            wrapper: None,
        }),
        (CacheMode::Auto | CacheMode::On, Some(path)) => Ok(CacheStatus {
            requested_mode: mode,
            source: CacheSource::AutoDetectedSccache,
            wrapper: Some(path.display().to_string()),
        }),
        (CacheMode::Auto, None) => Ok(CacheStatus {
            requested_mode: mode,
            source: CacheSource::Unavailable,
            wrapper: None,
        }),
        (CacheMode::On, None) => {
            Err("SHORT_ORIGIN_SCCACHE=on but `sccache` was not found on PATH".to_string())
        }
    }
}

pub fn cache_status_from_env() -> Result<CacheStatus, String> {
    let requested_mode = CacheMode::parse(env::var("SHORT_ORIGIN_SCCACHE").ok().as_deref())?;
    let existing_wrapper = env::var("RUSTC_WRAPPER").ok();
    let detected_sccache = find_in_path("sccache");
    resolve_cache_status(existing_wrapper, requested_mode, detected_sccache)
}

pub fn run() -> Result<(), String> {
    run_from(env::args_os())
}

pub fn run_from<I, T>(args: I) -> Result<(), String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let rendered = error.to_string();
            if error.use_stderr() {
                return Err(rendered);
            }
            print!("{rendered}");
            return Ok(());
        }
    };
    dispatch(cli)
}

pub fn workspace_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest dir is missing a workspace root".to_string())
}

pub fn ensure_command_available(
    program: &str,
    args: &[&str],
    guidance: &str,
) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args);
    let output = command.output().map_err(|error| {
        format!(
            "failed to start `{}`: {error}. {guidance}",
            display_command(&command)
        )
    })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "`{}` is unavailable. {guidance}",
            display_command(&command)
        ))
    }
}

pub fn run_plan(plan: &CommandPlan) -> Result<(), String> {
    let status = plan
        .to_command()
        .status()
        .map_err(|error| format!("failed to start `{}`: {error}", plan.display()))?;
    ensure_success(plan, status)
}

pub fn run_compile_plan(mut plan: CommandPlan) -> Result<(), String> {
    let cache = cache_status_from_env()?;
    cache.apply_to(&mut plan);
    run_plan(&plan)
}

fn ensure_success(plan: &CommandPlan, status: ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("`{}` exited with status {status}", plan.display()))
    }
}

fn display_command(command: &Command) -> String {
    let program = command.get_program().to_string_lossy();
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        program.into_owned()
    } else {
        format!("{program} {args}")
    }
}

#[must_use]
pub fn absolutize(workspace_root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

pub fn normalize_dist_arg(workspace_root: &Path, args: &mut [String]) {
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == "--dist" {
            if let Some(value) = args.get_mut(index + 1) {
                *value = absolutize(workspace_root, value).display().to_string();
            }
            index += 2;
            continue;
        }

        if let Some(value) = args[index].strip_prefix("--dist=") {
            args[index] = format!("--dist={}", absolutize(workspace_root, value).display());
        }
        index += 1;
    }
}

pub fn drop_no_open_arg(args: &mut Vec<String>) {
    args.retain(|arg| arg != "--no-open");
}

pub fn export_schemas(output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut written = export_schema_group(
        output_dir,
        "contracts",
        embedded_contract_schemas().map_err(|error| error.to_string())?,
    )?;
    written.extend(export_schema_group(
        output_dir,
        "events",
        embedded_event_schemas().map_err(|error| error.to_string())?,
    )?);
    written.extend(export_schema_group(
        output_dir,
        "surrealdb",
        embedded_surrealdb_schemas().map_err(|error| error.to_string())?,
    )?);
    Ok(written)
}

fn export_schema_group(
    output_dir: &Path,
    group: &str,
    schemas: Vec<codegen::EmbeddedSchema>,
) -> Result<Vec<PathBuf>, String> {
    let group_dir = output_dir.join(group);
    fs::create_dir_all(&group_dir).map_err(|error| {
        format!(
            "failed to create schema output directory `{}`: {error}",
            group_dir.display()
        )
    })?;

    schemas
        .into_iter()
        .map(|schema| {
            let path = group_dir.join(format!("{}.json", schema.name));
            let json = serde_json::to_string_pretty(&schema.document).map_err(|error| {
                format!("failed to serialize schema `{}`: {error}", schema.name)
            })?;
            fs::write(&path, format!("{json}\n"))
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            Ok(path)
        })
        .collect()
}

pub fn component_check_args() -> Vec<&'static str> {
    vec![
        "check",
        "-p",
        "wasmcloud-bindings",
        "-p",
        "lattice-config",
        "-p",
        "finance-service",
        "-p",
        "treasury-disbursement",
        "-p",
        "wasmcloud-smoke-tests",
    ]
}

fn run_components_build() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    run_compile_plan(CommandPlan::new("cargo", workspace_root).args(component_check_args()))
}

fn run_workspace_verify(profile: VerifyProfile) -> Result<(), String> {
    match profile {
        VerifyProfile::Fast => run_task_action(TaskAction::VerifyFast),
        VerifyProfile::Ui => run_named_task("ui-verify"),
        VerifyProfile::Full => run_task_action(TaskAction::VerifyFull),
    }
}

fn run_task_action(action: TaskAction) -> Result<(), String> {
    match action {
        TaskAction::NoOp => Ok(()),
        TaskAction::VerifyFast => {
            let workspace_root = workspace_root()?;
            run_plan(
                &CommandPlan::new("cargo", &workspace_root).args(["fmt", "--all", "--check"]),
            )?;
            run_compile_plan(CommandPlan::new("cargo", workspace_root).args([
                "check",
                "--workspace",
                "--all-targets",
            ]))
        }
        TaskAction::VerifyFull => {
            let workspace_root = workspace_root()?;
            run_plan(
                &CommandPlan::new("cargo", &workspace_root).args(["fmt", "--all", "--check"]),
            )?;
            run_compile_plan(CommandPlan::new("cargo", &workspace_root).args([
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ]))?;
            run_compile_plan(CommandPlan::new("cargo", &workspace_root).args([
                "test",
                "--workspace",
                "--all-targets",
            ]))?;
            run_compile_plan(CommandPlan::new("cargo", workspace_root).args([
                "test",
                "-p",
                "wasmcloud-bindings",
                "-p",
                "wasmcloud-smoke-tests",
                "-p",
                "surrealdb-access",
            ]))
        }
        TaskAction::UiCompileChecks => run_ui_compile_checks(),
        TaskAction::UiValidationBuild => run_ui_validation_build(),
        TaskAction::UiE2eAll => run_ui_e2e(None),
        TaskAction::ComponentsBuild => run_components_build(),
        TaskAction::DocsSecurityBookTest => security_book::test_book(),
        TaskAction::InfraVerify => infra::verify(),
        TaskAction::SecurityAudit => run_security_audit(),
    }
}

fn run_ui_compile_checks() -> Result<(), String> {
    let workspace_root = workspace_root()?;
    run_compile_plan(CommandPlan::new("cargo", &workspace_root).args([
        "check",
        "-p",
        "desktop_app_contract",
        "-p",
        "desktop_app_control_center",
        "-p",
        "desktop_runtime",
        "-p",
        "site",
        "--all-features",
    ]))?;
    run_compile_plan(CommandPlan::new("cargo", workspace_root).args([
        "check",
        "-p",
        "desktop_tauri",
        "--all-features",
    ]))
}

fn ui_web_plan(subcommand: &str, passthrough: &[String]) -> Result<CommandPlan, String> {
    let trunk_subcommand = match subcommand {
        "dev" => "serve",
        "build" => "build",
        other => return Err(format!("unsupported ui web subcommand `{other}`")),
    };
    let workspace_root = workspace_root()?;
    let site_dir = workspace_root.join("ui/crates/site");
    let index = site_dir.join("index.html");
    let mut passthrough = passthrough.to_vec();
    normalize_dist_arg(&workspace_root, &mut passthrough);
    drop_no_open_arg(&mut passthrough);

    Ok(CommandPlan::new("trunk", site_dir)
        .arg(trunk_subcommand)
        .arg(index.display().to_string())
        .args(passthrough))
}

fn run_ui_web(subcommand: &str, passthrough: &[String]) -> Result<(), String> {
    ensure_tools_available(&[ToolKind::Trunk])?;
    run_compile_plan(ui_web_plan(subcommand, passthrough)?)
}

fn run_ui_validation_build() -> Result<(), String> {
    run_ui_web(
        "build",
        &[
            "--features".to_string(),
            "desktop-tauri".to_string(),
            "--dist".to_string(),
            "target/trunk-ui-dist".to_string(),
        ],
    )
}

fn run_ui_tauri(subcommand: &str, passthrough: &[String]) -> Result<(), String> {
    ensure_tools_available(&[ToolKind::CargoTauri])?;
    let workspace_root = workspace_root()?;
    let tauri_dir = workspace_root.join("ui/crates/desktop_tauri");
    run_compile_plan(
        CommandPlan::new("cargo", tauri_dir)
            .arg("tauri")
            .arg(subcommand)
            .args(passthrough.to_vec()),
    )
}

fn run_ui_e2e(scene: Option<UiE2eScene>) -> Result<(), String> {
    ensure_tools_available(&[ToolKind::Trunk, ToolKind::Node, ToolKind::Npm])?;
    ui_e2e::run(scene)
}

fn run_security_audit() -> Result<(), String> {
    ensure_tools_available(&[ToolKind::CargoAudit])?;
    let workspace_root = workspace_root()?;
    run_compile_plan(CommandPlan::new("cargo", workspace_root).arg("audit"))
}

fn run_schema_check() -> Result<(), String> {
    let contracts = embedded_contract_schemas().map_err(|error| error.to_string())?;
    let events = embedded_event_schemas().map_err(|error| error.to_string())?;
    let surrealdb = embedded_surrealdb_schemas().map_err(|error| error.to_string())?;
    println!(
        "validated {} contract schemas, {} event schemas, and {} SurrealDB schemas",
        contracts.len(),
        events.len(),
        surrealdb.len()
    );
    Ok(())
}

fn run_schema_export(output_dir: &Path) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let output_dir = if output_dir.is_absolute() {
        output_dir.to_path_buf()
    } else {
        workspace_root.join(output_dir)
    };
    let written = export_schemas(&output_dir)?;
    println!(
        "exported {} schema artifacts under {}",
        written.len(),
        output_dir.display()
    );
    Ok(())
}

fn run_cache_status() -> Result<(), String> {
    let status = cache_status_from_env()?;
    println!("requested_mode: {}", status.requested_mode.label());
    println!("source: {}", status.source.label());
    println!("active: {}", if status.is_active() { "yes" } else { "no" });
    println!("wrapper: {}", status.wrapper.as_deref().unwrap_or("<none>"));
    Ok(())
}

fn legacy_warning(legacy: &str, canonical: &str) {
    eprintln!("warning: `{legacy}` is deprecated; use `{canonical}` instead");
}

fn ensure_tools_available(tools: &[ToolKind]) -> Result<(), String> {
    for tool in tools {
        tool.check()?;
    }
    Ok(())
}

fn registered_tasks() -> Vec<TaskSpec> {
    vec![
        TaskSpec {
            id: "verify-fast",
            description: "Run the fast workspace format and check profile.",
            domains: vec![TaskDomain::Core],
            prerequisites: vec![ToolKind::Rustfmt],
            dependencies: vec![],
            ci_included: false,
            listed: true,
            action: TaskAction::VerifyFast,
        },
        TaskSpec {
            id: "verify-full",
            description: "Run the canonical full Rust workspace validation profile.",
            domains: vec![TaskDomain::Core],
            prerequisites: vec![ToolKind::Rustfmt, ToolKind::Clippy],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::VerifyFull,
        },
        TaskSpec {
            id: "ui-compile-checks",
            description: "Compile the tracked browser and desktop UI crates.",
            domains: vec![TaskDomain::Ui],
            prerequisites: vec![],
            dependencies: vec![],
            ci_included: false,
            listed: false,
            action: TaskAction::UiCompileChecks,
        },
        TaskSpec {
            id: "ui-web-build",
            description: "Build the browser preview bundle with desktop-tauri parity features.",
            domains: vec![TaskDomain::Ui],
            prerequisites: vec![ToolKind::Trunk],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::UiValidationBuild,
        },
        TaskSpec {
            id: "ui-e2e",
            description: "Run deterministic browser UI scenes through Playwright.",
            domains: vec![TaskDomain::Ui],
            prerequisites: vec![ToolKind::Trunk, ToolKind::Node, ToolKind::Npm],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::UiE2eAll,
        },
        TaskSpec {
            id: "ui-verify",
            description: "Run compile, build, and browser E2E validation for the UI surface.",
            domains: vec![TaskDomain::Ui],
            prerequisites: vec![ToolKind::Trunk, ToolKind::Node, ToolKind::Npm],
            dependencies: vec!["ui-compile-checks", "ui-web-build", "ui-e2e"],
            ci_included: true,
            listed: true,
            action: TaskAction::NoOp,
        },
        TaskSpec {
            id: "components-build",
            description: "Compile the tracked wasmCloud component set.",
            domains: vec![TaskDomain::Core],
            prerequisites: vec![],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::ComponentsBuild,
        },
        TaskSpec {
            id: "docs-security-book-test",
            description: "Run the mdBook validation and teaching-crate test path.",
            domains: vec![TaskDomain::Docs],
            prerequisites: vec![ToolKind::Mdbook, ToolKind::CargoNextest],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::DocsSecurityBookTest,
        },
        TaskSpec {
            id: "infra-verify",
            description: "Run Pulumi workspace install, tests, and TypeScript linting.",
            domains: vec![TaskDomain::Infra],
            prerequisites: vec![ToolKind::Node, ToolKind::Npm],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::InfraVerify,
        },
        TaskSpec {
            id: "security-audit",
            description: "Run cargo-audit against the workspace lockfile.",
            domains: vec![TaskDomain::Security],
            prerequisites: vec![ToolKind::CargoAudit],
            dependencies: vec![],
            ci_included: true,
            listed: true,
            action: TaskAction::SecurityAudit,
        },
    ]
}

#[cfg(test)]
fn find_task(task_id: &str) -> Option<TaskSpec> {
    registered_tasks()
        .into_iter()
        .find(|task| task.id == task_id)
}

fn run_named_task(task_id: &str) -> Result<(), String> {
    let registry = registered_tasks()
        .into_iter()
        .map(|task| (task.id, task))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    run_named_task_inner(task_id, &registry, &mut visited)
}

fn run_named_task_inner(
    task_id: &str,
    registry: &BTreeMap<&'static str, TaskSpec>,
    visited: &mut BTreeSet<&'static str>,
) -> Result<(), String> {
    let task = registry
        .get(task_id)
        .ok_or_else(|| format!("unknown task `{task_id}`"))?;
    if !visited.insert(task.id) {
        return Ok(());
    }

    for dependency in &task.dependencies {
        run_named_task_inner(dependency, registry, visited)?;
    }

    ensure_tools_available(&task.prerequisites)?;
    if !matches!(task.action, TaskAction::NoOp) {
        println!("==> {}", task.id);
    }
    run_task_action(task.action)
}

fn task_list_output() -> TaskListOutput {
    TaskListOutput {
        tasks: registered_tasks()
            .into_iter()
            .filter(|task| task.listed)
            .map(|task| TaskListItem {
                id: task.id,
                description: task.description,
                domains: task.domains.into_iter().map(TaskDomain::label).collect(),
                prerequisites: task
                    .prerequisites
                    .into_iter()
                    .map(ToolKind::label)
                    .collect(),
                dependencies: task.dependencies,
                ci_included: task.ci_included,
                listed: task.listed,
            })
            .collect(),
    }
}

fn run_tasks_list(format: OutputFormat) -> Result<(), String> {
    if matches!(format, OutputFormat::Json) {
        let output = task_list_output();
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|error| format!("failed to serialize task list: {error}"))?
        );
        return Ok(());
    }

    for task in registered_tasks().into_iter().filter(|task| task.listed) {
        let domains = task
            .domains
            .iter()
            .map(|domain| domain.label())
            .collect::<Vec<_>>()
            .join(",");
        let deps = if task.dependencies.is_empty() {
            "-".to_string()
        } else {
            task.dependencies.join(",")
        };
        let prereqs = if task.prerequisites.is_empty() {
            "-".to_string()
        } else {
            task.prerequisites
                .iter()
                .map(|tool| tool.label())
                .collect::<Vec<_>>()
                .join(",")
        };
        println!(
            "{:<22} ci={} domains={} deps={} tools={} :: {}",
            task.id,
            if task.ci_included { "yes" } else { "no" },
            domains,
            deps,
            prereqs,
            task.description
        );
    }
    Ok(())
}

fn optional_tools_for_domain(domain: DoctorDomain) -> Vec<ToolKind> {
    match domain {
        DoctorDomain::Core => vec![ToolKind::Sccache],
        DoctorDomain::Ui => vec![ToolKind::CargoTauri],
        DoctorDomain::Docs => vec![],
        DoctorDomain::Infra => vec![ToolKind::Pulumi],
        DoctorDomain::Security => vec![ToolKind::CargoFuzz],
        DoctorDomain::All => vec![
            ToolKind::Sccache,
            ToolKind::CargoTauri,
            ToolKind::Pulumi,
            ToolKind::CargoFuzz,
        ],
    }
}

fn doctor_entries(domain: DoctorDomain) -> Vec<DoctorEntry> {
    let mut entries = BTreeMap::<ToolKind, DoctorEntry>::new();

    for task in registered_tasks().into_iter().filter(|task| {
        task.domains
            .iter()
            .copied()
            .any(|task_domain| domain.matches(task_domain))
    }) {
        for tool in task.prerequisites {
            entries
                .entry(tool)
                .or_insert_with(|| DoctorEntry {
                    tool,
                    required_by: BTreeSet::new(),
                    optional: false,
                })
                .required_by
                .insert(task.id);
        }
    }

    for tool in optional_tools_for_domain(domain) {
        entries.entry(tool).or_insert(DoctorEntry {
            tool,
            required_by: BTreeSet::new(),
            optional: true,
        });
    }

    entries.into_values().collect()
}

fn doctor_notes(domain: DoctorDomain) -> Vec<String> {
    if matches!(domain, DoctorDomain::Ui | DoctorDomain::All) {
        vec![
            "note: `cargo xtask ui e2e` bootstraps Playwright package dependencies and Chromium automatically.".to_string(),
            "note: Linux desktop UI system packages are provisioned in CI; local host packages vary by OS.".to_string(),
        ]
    } else {
        Vec::new()
    }
}

fn doctor_output(domain: DoctorDomain) -> DoctorOutput {
    let mut missing_required = false;
    let entries = doctor_entries(domain)
        .into_iter()
        .map(|entry| {
            let status = if entry.tool.check().is_ok() {
                "ok"
            } else if entry.optional {
                "warn"
            } else {
                missing_required = true;
                "missing"
            };
            DoctorEntryStatus {
                tool: entry.tool.label(),
                required_by: entry.required_by.into_iter().collect(),
                optional: entry.optional,
                status,
                guidance: entry.tool.guidance(),
            }
        })
        .collect();

    DoctorOutput {
        domain: domain.label(),
        missing_required,
        entries,
        notes: doctor_notes(domain),
    }
}

fn run_doctor(domain: DoctorDomain, ci: bool, format: OutputFormat) -> Result<(), String> {
    let output = doctor_output(domain);

    if matches!(format, OutputFormat::Json) {
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .map_err(|error| format!("failed to serialize doctor output: {error}"))?
        );
    } else {
        println!("DX doctor domain={}", output.domain);
        for entry in &output.entries {
            let required_by = if entry.required_by.is_empty() {
                String::new()
            } else {
                format!(" required-by={}", entry.required_by.join(","))
            };

            match entry.status {
                "ok" => println!("[ok] {}{}", entry.tool, required_by),
                "warn" | "missing" => println!(
                    "[{}] {}{} :: {}",
                    entry.status, entry.tool, required_by, entry.guidance
                ),
                _ => {}
            }
        }
        for note in &output.notes {
            println!("{note}");
        }
    }

    if ci && output.missing_required {
        return Err(format!(
            "doctor failed for domain {} because required tools are missing",
            domain.label()
        ));
    }
    Ok(())
}

fn dispatch(cli: Cli) -> Result<(), String> {
    match cli.command {
        CommandGroup::Tasks(args) => match args.command {
            TasksCommand::List(args) => run_tasks_list(args.format),
        },
        CommandGroup::Run(args) => run_named_task(&args.task),
        CommandGroup::Doctor(args) => run_doctor(args.domain, args.ci, args.format),
        CommandGroup::Workspace(args) => match args.command {
            WorkspaceCommand::Verify(args) => run_workspace_verify(args.profile),
        },
        CommandGroup::Components(args) => match args.command {
            ComponentsCommand::Build => run_components_build(),
        },
        CommandGroup::Docs(args) => match args.command {
            DocsCommand::SecurityBook(args) => match args.command {
                SecurityBookCommand::Build(args) => security_book::build_book(&args.args),
                SecurityBookCommand::Preview(args) => security_book::preview_book(&args.args),
                SecurityBookCommand::Test => security_book::test_book(),
            },
        },
        CommandGroup::Infra(args) => match args.command {
            InfraCommand::Verify => {
                ensure_tools_available(&[ToolKind::Node, ToolKind::Npm])?;
                infra::verify()
            }
            InfraCommand::Preview(args) => {
                ensure_tools_available(&[ToolKind::Node, ToolKind::Npm, ToolKind::Pulumi])?;
                infra::preview(args.stack)
            }
        },
        CommandGroup::Artifacts(args) => match args.command {
            ArtifactsCommand::Delivery(args) => match args.command {
                DeliveryCommand::Components(args) => delivery::run(vec![
                    "render-components".to_string(),
                    "--environment".to_string(),
                    args.environment,
                    "--tag".to_string(),
                    args.tag,
                    "--output-dir".to_string(),
                    args.output_dir.display().to_string(),
                    "--registry".to_string(),
                    args.registry,
                ]),
                DeliveryCommand::Manifest(args) => delivery::run(vec![
                    "render-manifest".to_string(),
                    "--environment".to_string(),
                    args.environment,
                    "--finance-ref".to_string(),
                    args.finance_ref,
                    "--finance-digest".to_string(),
                    args.finance_digest,
                    "--treasury-ref".to_string(),
                    args.treasury_ref,
                    "--treasury-digest".to_string(),
                    args.treasury_digest,
                    "--output".to_string(),
                    args.output.display().to_string(),
                ]),
            },
            ArtifactsCommand::Schemas(args) => match args.command {
                SchemasCommand::Check => run_schema_check(),
                SchemasCommand::Export(args) => run_schema_export(&args.output_dir),
            },
        },
        CommandGroup::Ui(args) => match args.command {
            UiCommand::Web(args) => match args.command {
                UiWebCommand::Dev(args) => run_ui_web("dev", &args.args),
                UiWebCommand::Build(args) => run_ui_web("build", &args.args),
            },
            UiCommand::Tauri(args) => match args.command {
                UiTauriCommand::Dev(args) => run_ui_tauri("dev", &args.args),
                UiTauriCommand::Build(args) => run_ui_tauri("build", &args.args),
            },
            UiCommand::E2e(args) => {
                if args.scene.is_some() && args.all_scenes {
                    return Err("choose either --scene <scene> or --all-scenes".to_string());
                }
                run_ui_e2e(if args.all_scenes { None } else { args.scene })
            }
            UiCommand::Dev(args) => {
                legacy_warning("cargo xtask ui dev", "cargo xtask ui web dev");
                run_ui_web("dev", &args.args)
            }
            UiCommand::Build(args) => {
                legacy_warning("cargo xtask ui build", "cargo xtask ui web build");
                run_ui_web("build", &args.args)
            }
        },
        CommandGroup::Github(args) => match args.command {
            GithubCommand::Sync(args) => {
                let mut forwarded = vec!["sync".to_string(), args.target.to_string()];
                forwarded.extend(["--config".to_string(), args.config.display().to_string()]);
                if let Some(repository) = args.repository {
                    forwarded.extend(["--repository".to_string(), repository]);
                }
                if args.apply == args.dry_run {
                    return Err("choose exactly one of --dry-run or --apply".to_string());
                }
                forwarded.push(if args.apply {
                    "--apply".to_string()
                } else {
                    "--dry-run".to_string()
                });
                github::run(forwarded)
            }
            GithubCommand::ValidatePr(args) => github::run(vec![
                "validate-pr".to_string(),
                "--event-path".to_string(),
                args.event_path.display().to_string(),
                "--config".to_string(),
                args.config.display().to_string(),
            ]),
        },
        CommandGroup::Cache(args) => match args.command {
            CacheCommand::Status => run_cache_status(),
        },
        CommandGroup::LegacySecurityBook(args) => match args.command {
            LegacySecurityBookCommand::Build(args) => {
                legacy_warning(
                    "cargo xtask security-book build",
                    "cargo xtask docs security-book build",
                );
                security_book::build_book(&args.args)
            }
            LegacySecurityBookCommand::Serve(args) => {
                legacy_warning(
                    "cargo xtask security-book serve",
                    "cargo xtask docs security-book preview",
                );
                security_book::preview_book(&args.args)
            }
            LegacySecurityBookCommand::Test => {
                legacy_warning(
                    "cargo xtask security-book test",
                    "cargo xtask docs security-book test",
                );
                security_book::test_book()
            }
        },
        CommandGroup::LegacyDelivery(args) => match args.command {
            LegacyDeliveryCommand::RenderComponents(args) => {
                legacy_warning(
                    "cargo xtask delivery render-components",
                    "cargo xtask artifacts delivery components",
                );
                delivery::run(vec![
                    "render-components".to_string(),
                    "--environment".to_string(),
                    args.environment,
                    "--tag".to_string(),
                    args.tag,
                    "--output-dir".to_string(),
                    args.output_dir.display().to_string(),
                    "--registry".to_string(),
                    args.registry,
                ])
            }
            LegacyDeliveryCommand::RenderManifest(args) => {
                legacy_warning(
                    "cargo xtask delivery render-manifest",
                    "cargo xtask artifacts delivery manifest",
                );
                delivery::run(vec![
                    "render-manifest".to_string(),
                    "--environment".to_string(),
                    args.environment,
                    "--finance-ref".to_string(),
                    args.finance_ref,
                    "--finance-digest".to_string(),
                    args.finance_digest,
                    "--treasury-ref".to_string(),
                    args.treasury_ref,
                    "--treasury-digest".to_string(),
                    args.treasury_digest,
                    "--output".to_string(),
                    args.output.display().to_string(),
                ])
            }
        },
        CommandGroup::LegacyVerify(args) => match args.command {
            LegacyVerifyCommand::Profile(args) => {
                legacy_warning(
                    "cargo xtask verify profile <fast|ui|full>",
                    "cargo xtask workspace verify --profile <fast|ui|full>",
                );
                run_workspace_verify(args.profile)
            }
        },
        CommandGroup::LegacyTauri(args) => match args.command {
            LegacyTauriCommand::Dev(args) => {
                legacy_warning("cargo xtask tauri dev", "cargo xtask ui tauri dev");
                run_ui_tauri("dev", &args.args)
            }
            LegacyTauriCommand::Build(args) => {
                legacy_warning("cargo xtask tauri build", "cargo xtask ui tauri build");
                run_ui_tauri("build", &args.args)
            }
        },
    }
}

fn find_in_path(program: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Short Origin root task orchestration",
    arg_required_else_help = true,
    subcommand_required = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: CommandGroup,
}

#[derive(Debug, Subcommand)]
enum CommandGroup {
    /// Discover registered repository tasks.
    Tasks(TasksArgs),
    /// Execute a named registered repository task.
    Run(RunArgs),
    /// Diagnose local toolchain and workflow prerequisites.
    Doctor(DoctorArgs),
    /// Run workspace verification profiles.
    Workspace(WorkspaceArgs),
    /// Compile the tracked wasmCloud component package set.
    Components(ComponentsArgs),
    /// Build, preview, and validate project documentation.
    Docs(DocsArgs),
    /// Run Pulumi workspace verification and preview flows.
    Infra(InfraArgs),
    /// Render delivery and schema artifacts.
    Artifacts(ArtifactsArgs),
    /// Run browser and Tauri UI tasks.
    Ui(UiArgs),
    /// Apply or validate GitHub governance workflows.
    Github(GithubArgs),
    /// Show compilation cache configuration and status.
    Cache(CacheArgs),
    #[command(name = "security-book", hide = true)]
    LegacySecurityBook(LegacySecurityBookArgs),
    #[command(name = "delivery", hide = true)]
    LegacyDelivery(LegacyDeliveryArgs),
    #[command(name = "verify", hide = true)]
    LegacyVerify(LegacyVerifyArgs),
    #[command(name = "tauri", hide = true)]
    LegacyTauri(LegacyTauriArgs),
}

#[derive(Debug, Args)]
struct TasksArgs {
    #[command(subcommand)]
    command: TasksCommand,
}

#[derive(Debug, Subcommand)]
enum TasksCommand {
    /// List registered tasks, domains, prerequisites, and CI participation.
    List(ListArgs),
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct RunArgs {
    /// Registered task identifier.
    task: String,
}

#[derive(Debug, Args)]
struct DoctorArgs {
    #[arg(long, value_enum, default_value = "all")]
    domain: DoctorDomain,
    #[arg(long)]
    ci: bool,
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,
}

#[derive(Debug, Args)]
struct WorkspaceArgs {
    #[command(subcommand)]
    command: WorkspaceCommand,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommand {
    /// Run a named verification profile.
    Verify(VerifyArgs),
}

#[derive(Debug, Args)]
struct VerifyArgs {
    #[arg(long, value_enum)]
    profile: VerifyProfile,
}

#[derive(Debug, Args)]
struct ComponentsArgs {
    #[command(subcommand)]
    command: ComponentsCommand,
}

#[derive(Debug, Subcommand)]
enum ComponentsCommand {
    /// Compile the repository's tracked deployable component set.
    Build,
}

#[derive(Debug, Args)]
struct DocsArgs {
    #[command(subcommand)]
    command: DocsCommand,
}

#[derive(Debug, Subcommand)]
enum DocsCommand {
    /// Build, preview, or validate the security mdBook.
    SecurityBook(SecurityBookArgs),
}

#[derive(Debug, Args)]
struct SecurityBookArgs {
    #[command(subcommand)]
    command: SecurityBookCommand,
}

#[derive(Debug, Subcommand)]
enum SecurityBookCommand {
    /// Build the security mdBook.
    Build(PassthroughArgs),
    /// Start a local mdBook preview server.
    Preview(PassthroughArgs),
    /// Run the full security-book validation path.
    Test,
}

#[derive(Debug, Args)]
struct InfraArgs {
    #[command(subcommand)]
    command: InfraCommand,
}

#[derive(Debug, Subcommand)]
enum InfraCommand {
    /// Install infra dependencies, run tests, and lint the Pulumi workspace.
    Verify,
    /// Preview a Pulumi environment stack through the canonical workspace wrapper.
    Preview(InfraPreviewArgs),
}

#[derive(Debug, Args)]
struct InfraPreviewArgs {
    #[arg(long, value_enum)]
    stack: InfraStack,
}

#[derive(Debug, Args)]
struct ArtifactsArgs {
    #[command(subcommand)]
    command: ArtifactsCommand,
}

#[derive(Debug, Subcommand)]
enum ArtifactsCommand {
    /// Render delivery component descriptors and manifests.
    Delivery(DeliveryArgs),
    /// Validate or export embedded schema artifacts.
    Schemas(SchemasArgs),
}

#[derive(Debug, Args)]
struct DeliveryArgs {
    #[command(subcommand)]
    command: DeliveryCommand,
}

#[derive(Debug, Subcommand)]
enum DeliveryCommand {
    /// Render component-descriptor JSON files.
    Components(RenderComponentsArgs),
    /// Render a digest-pinned lattice manifest.
    Manifest(RenderManifestArgs),
}

#[derive(Debug, Args)]
struct RenderComponentsArgs {
    #[arg(long)]
    environment: String,
    #[arg(long)]
    tag: String,
    #[arg(long)]
    output_dir: PathBuf,
    #[arg(long, default_value = "ghcr.io/shortorigin")]
    registry: String,
}

#[derive(Debug, Args)]
struct RenderManifestArgs {
    #[arg(long)]
    environment: String,
    #[arg(long)]
    finance_ref: String,
    #[arg(long)]
    finance_digest: String,
    #[arg(long)]
    treasury_ref: String,
    #[arg(long)]
    treasury_digest: String,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Debug, Args)]
struct SchemasArgs {
    #[command(subcommand)]
    command: SchemasCommand,
}

#[derive(Debug, Subcommand)]
enum SchemasCommand {
    /// Validate embedded contract, event, and SurrealDB schemas.
    Check,
    /// Export embedded schemas to a filesystem directory.
    Export(SchemaExportArgs),
}

#[derive(Debug, Args)]
struct SchemaExportArgs {
    #[arg(long, default_value = "target/generated/schemas")]
    output_dir: PathBuf,
}

#[derive(Debug, Args)]
struct UiArgs {
    #[command(subcommand)]
    command: UiCommand,
}

#[derive(Debug, Subcommand)]
enum UiCommand {
    /// Run browser/WASM preview tasks with Trunk.
    Web(UiWebArgs),
    /// Run Tauri desktop tasks.
    Tauri(UiTauriArgs),
    /// Run deterministic browser scenes through Playwright.
    E2e(UiE2eArgs),
    #[command(hide = true)]
    Dev(PassthroughArgs),
    #[command(hide = true)]
    Build(PassthroughArgs),
}

#[derive(Debug, Args)]
struct UiWebArgs {
    #[command(subcommand)]
    command: UiWebCommand,
}

#[derive(Debug, Subcommand)]
enum UiWebCommand {
    /// Start the browser preview server.
    Dev(PassthroughArgs),
    /// Produce a browser preview build.
    Build(PassthroughArgs),
}

#[derive(Debug, Args)]
struct UiTauriArgs {
    #[command(subcommand)]
    command: UiTauriCommand,
}

#[derive(Debug, Subcommand)]
enum UiTauriCommand {
    /// Start the Tauri desktop app in dev mode.
    Dev(PassthroughArgs),
    /// Build the Tauri desktop app.
    Build(PassthroughArgs),
}

#[derive(Debug, Args)]
struct UiE2eArgs {
    #[arg(long, value_enum)]
    scene: Option<UiE2eScene>,
    #[arg(long)]
    all_scenes: bool,
}

#[derive(Debug, Args)]
struct GithubArgs {
    #[command(subcommand)]
    command: GithubCommand,
}

#[derive(Debug, Subcommand)]
enum GithubCommand {
    /// Sync organization or repository governance settings.
    Sync(GithubSyncArgs),
    /// Validate pull request governance requirements.
    ValidatePr(GithubValidatePrArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum GithubSyncTarget {
    Org,
    Repo,
}

impl std::fmt::Display for GithubSyncTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Org => formatter.write_str("org"),
            Self::Repo => formatter.write_str("repo"),
        }
    }
}

#[derive(Debug, Args)]
struct GithubSyncArgs {
    #[arg(value_enum)]
    target: GithubSyncTarget,
    #[arg(long, default_value = ".github/governance.toml")]
    config: PathBuf,
    #[arg(long)]
    repository: Option<String>,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct GithubValidatePrArgs {
    #[arg(long, default_value = ".github/governance.toml")]
    config: PathBuf,
    #[arg(long)]
    event_path: PathBuf,
}

#[derive(Debug, Args)]
struct CacheArgs {
    #[command(subcommand)]
    command: CacheCommand,
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    /// Print the active RUSTC_WRAPPER / sccache status.
    Status,
}

#[derive(Debug, Clone, Args)]
struct PassthroughArgs {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(Debug, Args)]
struct LegacySecurityBookArgs {
    #[command(subcommand)]
    command: LegacySecurityBookCommand,
}

#[derive(Debug, Subcommand)]
enum LegacySecurityBookCommand {
    Build(PassthroughArgs),
    Serve(PassthroughArgs),
    Test,
}

#[derive(Debug, Args)]
struct LegacyDeliveryArgs {
    #[command(subcommand)]
    command: LegacyDeliveryCommand,
}

#[derive(Debug, Subcommand)]
enum LegacyDeliveryCommand {
    #[command(name = "render-components")]
    RenderComponents(RenderComponentsArgs),
    #[command(name = "render-manifest")]
    RenderManifest(RenderManifestArgs),
}

#[derive(Debug, Args)]
struct LegacyVerifyArgs {
    #[command(subcommand)]
    command: LegacyVerifyCommand,
}

#[derive(Debug, Subcommand)]
enum LegacyVerifyCommand {
    Profile(LegacyVerifyProfileArgs),
}

#[derive(Debug, Args)]
struct LegacyTauriArgs {
    #[command(subcommand)]
    command: LegacyTauriCommand,
}

#[derive(Debug, Subcommand)]
enum LegacyTauriCommand {
    Dev(PassthroughArgs),
    Build(PassthroughArgs),
}

#[derive(Debug, Args)]
struct LegacyVerifyProfileArgs {
    #[arg(value_enum)]
    profile: VerifyProfile,
}

#[cfg(test)]
mod tests {
    use super::{
        doctor_entries, doctor_output, find_task, infra, resolve_cache_status, task_list_output,
        ui_e2e, CacheMode, CacheSource, Cli, CommandPlan, DoctorDomain, TaskAction, ToolKind,
        VerifyProfile,
    };
    use clap::{Parser, ValueEnum};
    use std::path::{Path, PathBuf};

    #[test]
    fn cli_parses_canonical_workspace_verify_command() {
        let cli = Cli::try_parse_from(["xtask", "workspace", "verify", "--profile", "full"])
            .expect("parse canonical workspace verify");
        let debug = format!("{cli:?}");
        assert!(debug.contains("Workspace"));
        assert!(debug.contains("Full"));
    }

    #[test]
    fn cli_parses_run_task_command() {
        let cli = Cli::try_parse_from(["xtask", "run", "ui-verify"]).expect("parse run command");
        let debug = format!("{cli:?}");
        assert!(debug.contains("Run"));
        assert!(debug.contains("ui-verify"));
    }

    #[test]
    fn cli_parses_legacy_security_book_command() {
        let cli = Cli::try_parse_from(["xtask", "security-book", "serve", "--hostname", "0.0.0.0"])
            .expect("parse legacy security-book command");
        let debug = format!("{cli:?}");
        assert!(debug.contains("LegacySecurityBook"));
        assert!(debug.contains("Serve"));
    }

    #[test]
    fn resolve_cache_prefers_existing_wrapper() {
        let status = resolve_cache_status(
            Some("/custom/wrapper".to_string()),
            CacheMode::Off,
            Some(PathBuf::from("/usr/bin/sccache")),
        )
        .expect("resolve cache");
        assert_eq!(status.source, CacheSource::ExistingWrapper);
        assert_eq!(status.wrapper.as_deref(), Some("/custom/wrapper"));
    }

    #[test]
    fn resolve_cache_uses_detected_sccache_when_auto_enabled() {
        let status = resolve_cache_status(
            None,
            CacheMode::Auto,
            Some(PathBuf::from("/usr/bin/sccache")),
        )
        .expect("resolve cache");
        assert_eq!(status.source, CacheSource::AutoDetectedSccache);
        assert_eq!(status.wrapper.as_deref(), Some("/usr/bin/sccache"));
    }

    #[test]
    fn resolve_cache_errors_when_required_but_missing() {
        let error = resolve_cache_status(None, CacheMode::On, None).expect_err("missing sccache");
        assert!(error.contains("SHORT_ORIGIN_SCCACHE=on"));
    }

    #[test]
    fn component_build_args_use_hyphenated_package_name() {
        let args = super::component_check_args();
        assert!(args.contains(&"treasury-disbursement"));
        assert!(!args.contains(&"treasury_disbursement"));
    }

    #[test]
    fn command_plan_display_renders_program_and_args() {
        let plan = CommandPlan::new("cargo", "/tmp").args(["xtask", "cache", "status"]);
        assert_eq!(plan.display(), "cargo xtask cache status");
    }

    #[test]
    fn verify_profile_value_enum_matches_expected_names() {
        let full = VerifyProfile::from_str("full", true).expect("full");
        assert_eq!(full, VerifyProfile::Full);
    }

    #[test]
    fn ui_verify_task_is_dependency_driven() {
        let task = find_task("ui-verify").expect("ui task");
        assert_eq!(task.action, TaskAction::NoOp);
        assert_eq!(
            task.dependencies,
            vec!["ui-compile-checks", "ui-web-build", "ui-e2e"]
        );
    }

    #[test]
    fn doctor_collects_ui_prerequisites_and_optional_tauri() {
        let entries = doctor_entries(DoctorDomain::Ui);
        let labels = entries.iter().map(|entry| entry.tool).collect::<Vec<_>>();
        assert!(labels.contains(&ToolKind::Trunk));
        assert!(labels.contains(&ToolKind::Node));
        assert!(labels.contains(&ToolKind::Npm));
        assert!(labels.contains(&ToolKind::CargoTauri));
    }

    #[test]
    fn cli_parses_json_output_flags() {
        let tasks = Cli::try_parse_from(["xtask", "tasks", "list", "--format", "json"])
            .expect("parse tasks json");
        assert!(format!("{tasks:?}").contains("Json"));

        let doctor = Cli::try_parse_from(["xtask", "doctor", "--domain", "ui", "--format", "json"])
            .expect("parse doctor json");
        assert!(format!("{doctor:?}").contains("Json"));
    }

    #[test]
    fn task_list_output_contains_public_task_metadata() {
        let output = task_list_output();
        let verify = output
            .tasks
            .into_iter()
            .find(|task| task.id == "verify-full")
            .expect("verify-full");
        assert!(verify.ci_included);
        assert!(verify.prerequisites.contains(&"rustfmt"));
        assert!(verify.prerequisites.contains(&"clippy"));
    }

    #[test]
    fn doctor_output_marks_missing_required_tools_for_security_domain() {
        let output = doctor_output(DoctorDomain::Security);
        assert_eq!(output.domain, "security");
        assert!(output
            .entries
            .iter()
            .any(|entry| entry.tool == "cargo-audit" && matches!(entry.status, "ok" | "missing")));
    }

    #[test]
    fn infra_preview_plan_uses_requested_stack() {
        let plan = infra::preview_plan(Path::new("/repo"), infra::InfraStack::Prod);
        assert_eq!(plan.display(), "npm --workspace live run preview:prod");
    }

    #[test]
    fn ui_e2e_plan_forwards_scene_environment() {
        let plan = ui_e2e::playwright_test_plan(
            Path::new("/repo"),
            Some(ui_e2e::UiE2eScene::ShellDefault),
        );
        assert_eq!(
            plan.env.get("SHORT_ORIGIN_E2E_SCENE").map(String::as_str),
            Some("shell-default")
        );
    }

    #[test]
    fn doctor_collects_core_required_tools_and_optional_sccache() {
        let entries = doctor_entries(DoctorDomain::Core);
        let labels = entries.iter().map(|entry| entry.tool).collect::<Vec<_>>();
        assert!(labels.contains(&ToolKind::Rustfmt));
        assert!(labels.contains(&ToolKind::Clippy));
        assert!(labels.contains(&ToolKind::Sccache));
    }

    #[test]
    fn doctor_collects_infra_preview_optional_pulumi() {
        let entries = doctor_entries(DoctorDomain::Infra);
        let labels = entries.iter().map(|entry| entry.tool).collect::<Vec<_>>();
        assert!(labels.contains(&ToolKind::Node));
        assert!(labels.contains(&ToolKind::Npm));
        assert!(labels.contains(&ToolKind::Pulumi));
    }
}
