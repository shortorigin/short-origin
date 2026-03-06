mod delivery;
mod github;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

const UI_PACKAGES: &[&str] = &[
    "desktop_app_contract",
    "desktop_runtime",
    "desktop_tauri",
    "platform_host",
    "platform_host_web",
    "site",
    "system_ui",
    "system_shell",
    "system_shell_contract",
    "shrs_core_headless",
    "desktop_app_control_center",
    "desktop_app_settings",
    "desktop_app_terminal",
];

const CORE_EXCLUDED_PACKAGES: &[&str] = &[
    "desktop_app_contract",
    "desktop_runtime",
    "desktop_tauri",
    "platform_host",
    "platform_host_web",
    "site",
    "system_ui",
    "system_shell",
    "system_shell_contract",
    "shrs_core_headless",
    "desktop_app_control_center",
    "desktop_app_settings",
    "desktop_app_terminal",
    "wasmcloud-smoke-tests",
];

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
    command.arg(trunk_subcommand);
    command.arg(index);
    sanitize_trunk_environment(&mut command);

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
        _ => return Err("expected `verify profile <core|fast|ui|ui-ci|full>`".to_string()),
    };

    let workspace_root = workspace_root()?;
    match profile {
        "core" | "fast" => {
            cargo(&workspace_root, &["fmt", "--all", "--check"])?;
            cargo(
                &workspace_root,
                &workspace_command_with_excludes(
                    &["clippy", "--workspace", "--all-targets", "--all-features"],
                    CORE_EXCLUDED_PACKAGES,
                    &["--", "-D", "warnings"],
                ),
            )?;
            cargo(
                &workspace_root,
                &workspace_command_with_excludes(
                    &["test", "--workspace", "--all-targets"],
                    CORE_EXCLUDED_PACKAGES,
                    &[],
                ),
            )?;
        }
        "ui" | "ui-ci" => {
            cargo(
                &workspace_root,
                &package_command_with_packages(
                    &["clippy", "--all-targets", "--all-features"],
                    UI_PACKAGES,
                    &["--", "-D", "warnings"],
                ),
            )?;
            cargo(
                &workspace_root,
                &package_command_with_packages(&["test", "--all-targets"], UI_PACKAGES, &[]),
            )?;
            run_ui(vec![
                "build".to_string(),
                "--features".to_string(),
                "desktop-tauri".to_string(),
                "--dist".to_string(),
                "target/trunk-ci-dist".to_string(),
            ])?;
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
        }
        other => return Err(format!("unknown verification profile `{other}`")),
    }

    Ok(())
}

fn workspace_command_with_excludes(
    prefix: &[&'static str],
    excluded_packages: &[&'static str],
    suffix: &[&'static str],
) -> Vec<&'static str> {
    let mut args = Vec::with_capacity(prefix.len() + excluded_packages.len() * 2 + suffix.len());
    args.extend_from_slice(prefix);
    for package in excluded_packages {
        args.push("--exclude");
        args.push(package);
    }
    args.extend_from_slice(suffix);
    args
}

fn package_command_with_packages(
    prefix: &[&'static str],
    packages: &[&'static str],
    suffix: &[&'static str],
) -> Vec<&'static str> {
    let mut args = Vec::with_capacity(prefix.len() + packages.len() * 2 + suffix.len());
    args.extend_from_slice(prefix);
    for package in packages {
        args.push("-p");
        args.push(package);
    }
    args.extend_from_slice(suffix);
    args
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

fn sanitize_trunk_environment(command: &mut Command) {
    if matches!(env::var("NO_COLOR").as_deref(), Ok("1")) {
        command.env("NO_COLOR", "true");
    }
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

#[cfg(test)]
mod tests {
    use super::{drop_no_open_arg, normalize_dist_arg, sanitize_trunk_environment};
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn normalize_dist_arg_absolutizes_split_flag_value() {
        let workspace_root = Path::new("/workspace");
        let mut args = vec!["--dist".to_string(), "target/ui-dist".to_string()];
        normalize_dist_arg(workspace_root, &mut args);
        assert_eq!(args, ["--dist", "/workspace/target/ui-dist"]);
    }

    #[test]
    fn normalize_dist_arg_absolutizes_inline_value() {
        let workspace_root = Path::new("/workspace");
        let mut args = vec!["--dist=target/ui-dist".to_string()];
        normalize_dist_arg(workspace_root, &mut args);
        assert_eq!(args, ["--dist=/workspace/target/ui-dist"]);
    }

    #[test]
    fn drop_no_open_arg_removes_flag() {
        let mut args = vec![
            "--port".to_string(),
            "1420".to_string(),
            "--no-open".to_string(),
        ];
        drop_no_open_arg(&mut args);
        assert_eq!(args, ["--port", "1420"]);
    }

    #[test]
    fn sanitize_trunk_environment_normalizes_no_color_value() {
        let original = std::env::var_os("NO_COLOR");
        std::env::set_var("NO_COLOR", "1");

        let mut command = Command::new("trunk");
        sanitize_trunk_environment(&mut command);

        let no_color = command
            .get_envs()
            .find(|(key, _)| *key == "NO_COLOR")
            .and_then(|(_, value)| value);
        assert_eq!(no_color, Some("true".as_ref()));

        if let Some(value) = original {
            std::env::set_var("NO_COLOR", value);
        } else {
            std::env::remove_var("NO_COLOR");
        }
    }
}
