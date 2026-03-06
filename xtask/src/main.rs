mod delivery;
mod github;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("github") => github::run(args.collect()),
        Some("delivery") => delivery::run(args.collect()),
        Some("ui") => run_ui(args.collect()),
        Some("tauri") => run_tauri(args.collect()),
        Some("components") => run_components(args.collect()),
        Some("verify") => run_verify(args.collect()),
        Some(command) => Err(format!("unknown xtask command `{command}`")),
        None => Err(help()),
    }
}

fn run_ui(args: Vec<String>) -> Result<(), String> {
    let (subcommand, passthrough) = args
        .split_first()
        .ok_or_else(|| "expected `ui <dev|build>`".to_string())?;
    let trunk_subcommand = match subcommand.as_str() {
        "dev" => "serve",
        "build" => "build",
        other => return Err(format!("unsupported ui subcommand `{other}`")),
    };
    let workspace_root = workspace_root()?;
    let site_dir = workspace_root.join("ui/crates/site");
    let index = site_dir.join("index.html");
    let mut command = Command::new("trunk");
    command.current_dir(&site_dir);
    command.env_remove("NO_COLOR");
    command.arg(trunk_subcommand);
    command.arg(index);

    let mut passthrough = passthrough.to_vec();
    normalize_dist_arg(&workspace_root, &mut passthrough);
    drop_no_open_arg(&mut passthrough);
    command.args(passthrough);

    run_command(&mut command)
}

fn run_tauri(args: Vec<String>) -> Result<(), String> {
    let (subcommand, passthrough) = args
        .split_first()
        .ok_or_else(|| "expected `tauri <dev|build>`".to_string())?;
    let workspace_root = workspace_root()?;
    let tauri_dir = workspace_root.join("ui/crates/desktop_tauri");
    let mut command = Command::new("cargo");
    command.current_dir(&tauri_dir);
    command.arg("tauri");
    command.arg(subcommand);
    command.args(passthrough);
    run_command(&mut command)
}

fn run_components(args: Vec<String>) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let subcommand = args.first().map_or("build", String::as_str);
    if subcommand != "build" {
        return Err(format!("unsupported components subcommand `{subcommand}`"));
    }

    let mut command = Command::new("cargo");
    command.current_dir(workspace_root);
    command.args([
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
    ]);
    run_command(&mut command)
}

fn run_verify(args: Vec<String>) -> Result<(), String> {
    let profile = match args.as_slice() {
        [profile] => profile.as_str(),
        [first, profile] if first == "profile" => profile.as_str(),
        _ => return Err("expected `verify profile <fast|ui|ci|full>`".to_string()),
    };

    let workspace_root = workspace_root()?;
    match profile {
        "fast" => {
            cargo(&workspace_root, &["fmt", "--all", "--check"])?;
            cargo(&workspace_root, &["check", "--workspace", "--all-targets"])?;
        }
        "ui" => {
            cargo(
                &workspace_root,
                &[
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
                ],
            )?;
            cargo(
                &workspace_root,
                &["check", "-p", "desktop_tauri", "--all-features"],
            )?;
        }
        "ci" => {
            cargo(&workspace_root, &["fmt", "--all", "--check"])?;
            cargo(
                &workspace_root,
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            cargo(&workspace_root, &["test", "--workspace", "--all-targets"])?;
            run_components_build(&workspace_root)?;
            cargo(
                &workspace_root,
                &[
                    "test",
                    "-p",
                    "wasmcloud-bindings",
                    "-p",
                    "wasmcloud-smoke-tests",
                    "-p",
                    "surrealdb-access",
                ],
            )?;
            run_ui_build(
                &workspace_root,
                &[
                    "--features",
                    "desktop-tauri",
                    "--dist",
                    "target/trunk-ci-dist",
                ],
            )?;
            cargo(
                &workspace_root,
                &["check", "-p", "desktop_tauri", "--all-features"],
            )?;
            validate_nomad_posture(&workspace_root)?;
        }
        "full" => {
            cargo(&workspace_root, &["fmt", "--all", "--check"])?;
            cargo(
                &workspace_root,
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            cargo(&workspace_root, &["test", "--workspace", "--all-targets"])?;
            cargo(
                &workspace_root,
                &[
                    "test",
                    "-p",
                    "wasmcloud-bindings",
                    "-p",
                    "wasmcloud-smoke-tests",
                    "-p",
                    "surrealdb-access",
                ],
            )?;
        }
        other => return Err(format!("unknown verification profile `{other}`")),
    }

    Ok(())
}

fn run_components_build(workspace_root: &Path) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.current_dir(workspace_root);
    command.args([
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
    ]);
    run_command(&mut command)
}

fn run_ui_build(workspace_root: &Path, passthrough: &[&str]) -> Result<(), String> {
    let site_dir = workspace_root.join("ui/crates/site");
    let index = site_dir.join("index.html");
    let mut command = Command::new("trunk");
    command.current_dir(&site_dir);
    command.env_remove("NO_COLOR");
    command.arg("build");
    command.arg(index);

    let mut passthrough = passthrough
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    normalize_dist_arg(workspace_root, &mut passthrough);
    drop_no_open_arg(&mut passthrough);
    command.args(passthrough);

    run_command(&mut command)
}

fn validate_nomad_posture(workspace_root: &Path) -> Result<(), String> {
    let jobs_dir = workspace_root.join("infrastructure/nomad/jobs");
    let raw_exec_pattern = regex::Regex::new(r#"driver\s*=\s*"raw_exec""#)
        .map_err(|error| format!("failed to compile Nomad posture regex: {error}"))?;

    for path in collect_files(&jobs_dir)? {
        let contents = std::fs::read_to_string(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        if raw_exec_pattern.is_match(&contents) {
            return Err(format!(
                "raw_exec deployments are not allowed for workload services: `{}`",
                path.display()
            ));
        }
    }

    Ok(())
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(path) = pending.pop() {
        if !path.exists() {
            continue;
        }

        let metadata = std::fs::metadata(&path).map_err(|error| {
            format!("failed to read metadata for `{}`: {error}", path.display())
        })?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).map_err(|error| {
                format!("failed to read directory `{}`: {error}", path.display())
            })? {
                let entry = entry.map_err(|error| {
                    format!("failed to read entry in `{}`: {error}", path.display())
                })?;
                pending.push(entry.path());
            }
        } else if metadata.is_file() {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn cargo(workspace_root: &Path, args: &[&str]) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.current_dir(workspace_root);
    command.args(args);
    run_command(&mut command)
}

fn normalize_dist_arg(workspace_root: &Path, args: &mut [String]) {
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

fn drop_no_open_arg(args: &mut Vec<String>) {
    args.retain(|arg| arg != "--no-open");
}

fn absolutize(workspace_root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        workspace_root.join(path)
    }
}

fn run_command(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|error| format!("failed to start `{}`: {error}", display_command(command)))?;
    ensure_success(command, status)
}

fn ensure_success(command: &Command, status: ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`{}` exited with status {status}",
            display_command(command)
        ))
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

fn workspace_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest dir is missing a workspace root".to_string())
}

fn help() -> String {
    "usage: cargo xtask <delivery|github|ui|tauri|components|verify> ...".to_string()
}
