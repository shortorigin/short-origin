mod architecture;
mod common;
mod delivery;
mod github;
mod plugin;
mod ui_hardening;

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use common::{absolutize, run_command, workspace_root};

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

const UI_PREVIEW_SMOKE_PORT: u16 = 8095;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("architecture") => architecture::run(args.collect()),
        Some("github") => github::run(args.collect()),
        Some("delivery") => delivery::run(args.collect()),
        Some("plugin") => plugin::run(args.collect()),
        Some("wasmcloud") => run_wasmcloud(args.collect()),
        Some("ui-hardening") => ui_hardening::run(args.collect()),
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

fn run_wasmcloud(args: Vec<String>) -> Result<(), String> {
    let (subcommand, passthrough) = args
        .split_first()
        .ok_or_else(|| "expected `wasmcloud <doctor|up|down|status|manifest>`".to_string())?;
    let workspace_root = workspace_root()?;

    match subcommand.as_str() {
        "doctor" => run_wasmcloud_doctor(&workspace_root),
        "up" => run_wash(&workspace_root, "up", passthrough, true),
        "down" => run_wash(&workspace_root, "down", passthrough, false),
        "status" => run_wash(
            &workspace_root,
            "get",
            &prepend_get_hosts(passthrough),
            false,
        ),
        "manifest" => run_wasmcloud_manifest(&workspace_root, passthrough),
        other => Err(format!("unsupported wasmcloud subcommand `{other}`")),
    }
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
            verify_ui_browser_manifest_hygiene(&workspace_root)?;
            verify_ui_shell_style_hygiene(&workspace_root)?;
            run_ui_preview_smoke(&workspace_root)?;
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
            verify_ui_shell_style_hygiene(&workspace_root)?;
        }
        other => return Err(format!("unknown verification profile `{other}`")),
    }

    Ok(())
}

fn run_wasmcloud_doctor(workspace_root: &Path) -> Result<(), String> {
    ensure_command_available("rustc", &["--version"])?;
    ensure_command_available("cargo", &["--version"])?;
    ensure_command_available("rustup", &["target", "list", "--installed"])?;
    ensure_command_available("wash", &["--version"])?;
    ensure_command_available("docker", &["--version"])?;
    ensure_any_rust_target_installed(&[
        "wasm32-wasip1",
        "wasm32-wasip2",
        "wasm32-unknown-unknown",
    ])?;

    cargo(workspace_root, &["xtask", "components", "build"])?;
    cargo(
        workspace_root,
        &["test", "-p", "wasmcloud-smoke-tests", "--all-targets"],
    )?;

    Ok(())
}

fn run_wasmcloud_manifest(workspace_root: &Path, args: &[String]) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.current_dir(workspace_root);
    command.args(["xtask", "delivery", "render-manifest"]);
    command.args(args);
    run_command(&mut command)
}

fn prepend_get_hosts(args: &[String]) -> Vec<String> {
    let mut command_args = Vec::with_capacity(args.len() + 1);
    command_args.push("hosts".to_string());
    command_args.extend(args.iter().cloned());
    command_args
}

fn run_wash(
    workspace_root: &Path,
    subcommand: &str,
    passthrough: &[String],
    include_default_lattice: bool,
) -> Result<(), String> {
    ensure_command_available("wash", &["--version"])?;

    let mut command = Command::new("wash");
    command.current_dir(workspace_root);
    command.arg(subcommand);
    if include_default_lattice && !args_include_lattice(passthrough) {
        command.arg("--lattice");
        command.arg("institutional-lattice");
    }
    command.args(passthrough);
    run_command(&mut command)
}

fn args_include_lattice(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--lattice" || arg.starts_with("--lattice=") || arg == "-x")
}

fn ensure_command_available(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|error| format!("required command `{program}` is unavailable: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "required command `{program}` failed its readiness check"
        ))
    }
}

fn ensure_any_rust_target_installed(targets: &[&str]) -> Result<(), String> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|error| format!("failed to inspect installed Rust targets: {error}"))?;
    if !output.status.success() {
        return Err("`rustup target list --installed` failed".to_string());
    }

    let installed = String::from_utf8_lossy(&output.stdout);
    if targets
        .iter()
        .any(|target| installed.lines().any(|line| line.trim() == *target))
    {
        Ok(())
    } else {
        Err(format!(
            "no supported Rust wasm target is installed; install one of: {}",
            targets.join(", ")
        ))
    }
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

fn run_ui_preview_smoke(workspace_root: &Path) -> Result<(), String> {
    cargo(
        workspace_root,
        &[
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
            "ui/crates/site/Cargo.toml",
            "--bin",
            "site_app",
        ],
    )?;

    run_ui(vec![
        "build".to_string(),
        "--dist".to_string(),
        "target/trunk-preview-dist".to_string(),
    ])?;

    let site_dir = workspace_root.join("ui/crates/site");
    let mut command = Command::new("trunk");
    command.current_dir(&site_dir);
    command.arg("serve");
    command.arg("index.html");
    command.arg("--no-autoreload");
    command.arg("--port");
    command.arg(UI_PREVIEW_SMOKE_PORT.to_string());
    sanitize_trunk_environment(&mut command);

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start preview smoke server: {error}"))?;

    let smoke_result =
        wait_for_http_ready(&mut child, UI_PREVIEW_SMOKE_PORT, Duration::from_secs(30));
    terminate_child(&mut child)?;
    smoke_result
}

fn wait_for_http_ready(child: &mut Child, port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll preview smoke server: {error}"))?
        {
            return Err(format!(
                "preview smoke server exited before responding with status {status}"
            ));
        }

        match probe_http_root(port) {
            Ok(()) => return Ok(()),
            Err(_) => thread::sleep(Duration::from_millis(250)),
        }
    }

    Err(format!(
        "preview smoke server did not respond on http://127.0.0.1:{port}/ within {}s",
        timeout.as_secs()
    ))
}

fn probe_http_root(port: u16) -> Result<(), String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|error| format!("failed to connect to preview smoke server: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("failed to configure preview smoke read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("failed to configure preview smoke write timeout: {error}"))?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .map_err(|error| format!("failed to send preview smoke request: {error}"))?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|error| format!("failed to read preview smoke response: {error}"))?;
    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200") {
        Ok(())
    } else {
        Err(format!(
            "preview smoke server returned unexpected response: {}",
            response.lines().next().unwrap_or("<empty>")
        ))
    }
}

fn terminate_child(child: &mut Child) -> Result<(), String> {
    match child.try_wait() {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            child
                .kill()
                .map_err(|error| format!("failed to stop preview smoke server: {error}"))?;
            child
                .wait()
                .map_err(|error| format!("failed to reap preview smoke server: {error}"))?;
            Ok(())
        }
        Err(error) => Err(format!(
            "failed to poll preview smoke server during shutdown: {error}"
        )),
    }
}

fn verify_ui_browser_manifest_hygiene(workspace_root: &Path) -> Result<(), String> {
    let site_manifest = std::fs::read_to_string(workspace_root.join("ui/crates/site/Cargo.toml"))
        .map_err(|error| format!("failed to read site manifest: {error}"))?;
    let e2e_package = workspace_root.join("ui/e2e/package.json");
    let e2e_lockfile = workspace_root.join("ui/e2e/package-lock.json");
    if !site_manifest.contains("js-sys") {
        return Err(
            "site manifest must declare a direct `js-sys` dependency for reflective browser APIs"
                .to_string(),
        );
    }
    if !e2e_package.exists() || !e2e_lockfile.exists() {
        return Err(
            "ui/e2e must declare committed package.json and package-lock.json for clean-runner verification"
                .to_string(),
        );
    }

    let persistence_source = std::fs::read_to_string(
        workspace_root.join("ui/crates/platform_host_web/src/persistence.rs"),
    )
    .map_err(|error| format!("failed to read platform_host_web persistence source: {error}"))?;
    if persistence_source.contains(".storage()") {
        return Err(
            "platform_host_web persistence probe must not depend on typed `Navigator.storage()` bindings"
                .to_string(),
        );
    }
    if !persistence_source
        .contains("Reflect::get(window.navigator().as_ref(), &\"storage\".into())")
    {
        return Err(
            "platform_host_web persistence probe must use reflective `navigator.storage` access"
                .to_string(),
        );
    }

    Ok(())
}

fn verify_ui_shell_style_hygiene(workspace_root: &Path) -> Result<(), String> {
    let patterns = [
        "#[0-9A-Fa-f]{3,8}",
        "rgba\\(",
        "box-shadow",
        "backdrop-filter",
        "font-size",
        "border-radius",
    ];
    let targets = [
        "ui/crates/system_ui/src/origin_primitives",
        "ui/crates/system_ui/src/origin_components",
        "ui/crates/desktop_runtime/src/components",
        "ui/crates/site/src",
    ];
    let existing_targets: Vec<_> = targets
        .iter()
        .filter(|target| workspace_root.join(target).exists())
        .copied()
        .collect();

    if existing_targets.is_empty() {
        return Ok(());
    }

    for pattern in patterns {
        let output = Command::new("rg")
            .current_dir(workspace_root)
            .arg("-n")
            .arg(pattern)
            .args(&existing_targets)
            .arg("--glob")
            .arg("!ui/crates/site/src/generated/**")
            .output()
            .map_err(|error| format!("failed to run rg for UI style hygiene: {error}"))?;
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(format!(
                "UI style hygiene scan failed for pattern `{pattern}`"
            ));
        }
        if output.status.code() == Some(0) {
            return Err(format!(
                "forbidden hardcoded UI styling matched `{pattern}`:\n{}",
                String::from_utf8_lossy(&output.stdout)
            ));
        }
    }

    Ok(())
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
    if env::var_os("NO_COLOR").is_some() {
        command.env_remove("NO_COLOR");
    }
    for key in env::vars_os().map(|(key, _)| key) {
        let key = key.to_string_lossy();
        if key.starts_with("CARGO_") && key != "CARGO_HOME" {
            command.env_remove(key.as_ref());
        }
    }
}

fn help() -> String {
    "\
usage: cargo xtask <command> ...

Commands:
  architecture   Architecture boundary and dependency auditing
  github        GitHub governance sync, PR validation, and process auditing
  plugin        Governed plugin manifest validation
  verify        Workspace verification profiles
  delivery      Delivery manifest and component rendering
  ui-hardening  Deterministic UI/browser hardening verification
  ui            Compatibility shim for Trunk browser workflows
  tauri         Compatibility shim for Tauri wrapper workflows
  components    Compatibility shim for component build verification
  wasmcloud     Local wasmCloud operator helpers
"
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        args_include_lattice, drop_no_open_arg, normalize_dist_arg, prepend_get_hosts,
        probe_http_root, sanitize_trunk_environment, verify_ui_browser_manifest_hygiene,
        verify_ui_shell_style_hygiene,
    };
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::Path;
    use std::process::Command;
    use std::thread;

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock drift")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create temp dir");
        base
    }

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
    fn sanitize_trunk_environment_removes_no_color() {
        let original = std::env::var_os("NO_COLOR");
        std::env::set_var("NO_COLOR", "1");

        let mut command = Command::new("trunk");
        sanitize_trunk_environment(&mut command);

        let no_color = command
            .get_envs()
            .find(|(key, _)| *key == "NO_COLOR")
            .and_then(|(_, value)| value);
        assert_eq!(no_color, None);

        if let Some(value) = original {
            std::env::set_var("NO_COLOR", value);
        } else {
            std::env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn prepend_get_hosts_adds_hosts_subcommand() {
        let args =
            prepend_get_hosts(&["--lattice".to_string(), "institutional-lattice".to_string()]);
        assert_eq!(args, ["hosts", "--lattice", "institutional-lattice"]);
    }

    #[test]
    fn args_include_lattice_detects_short_and_long_flags() {
        assert!(args_include_lattice(&["--lattice".to_string()]));
        assert!(args_include_lattice(&["--lattice=dev".to_string()]));
        assert!(args_include_lattice(&["-x".to_string()]));
        assert!(!args_include_lattice(&["--detached".to_string()]));
    }

    #[test]
    fn probe_http_root_accepts_http_200() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = [0u8; 128];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write response");
        });

        probe_http_root(port).expect("probe should succeed");
        server.join().expect("join server");
    }

    #[test]
    fn verify_ui_browser_manifest_hygiene_accepts_expected_files() {
        let root = unique_temp_dir("ui-browser-hygiene-pass");
        let site_dir = root.join("ui/crates/site");
        let host_dir = root.join("ui/crates/platform_host_web/src");
        let e2e_dir = root.join("ui/e2e");
        fs::create_dir_all(&site_dir).expect("create site dir");
        fs::create_dir_all(&host_dir).expect("create host dir");
        fs::create_dir_all(&e2e_dir).expect("create e2e dir");
        fs::write(site_dir.join("Cargo.toml"), "js-sys = \"0.3\"\n").expect("write site manifest");
        fs::write(e2e_dir.join("package.json"), "{}\n").expect("write e2e package");
        fs::write(e2e_dir.join("package-lock.json"), "{}\n").expect("write e2e lockfile");
        fs::write(
            host_dir.join("persistence.rs"),
            "let _ = js_sys::Reflect::get(window.navigator().as_ref(), &\"storage\".into());\n",
        )
        .expect("write persistence source");

        verify_ui_browser_manifest_hygiene(&root).expect("hygiene should pass");
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn verify_ui_browser_manifest_hygiene_rejects_typed_storage_binding() {
        let root = unique_temp_dir("ui-browser-hygiene-fail");
        let site_dir = root.join("ui/crates/site");
        let host_dir = root.join("ui/crates/platform_host_web/src");
        let e2e_dir = root.join("ui/e2e");
        fs::create_dir_all(&site_dir).expect("create site dir");
        fs::create_dir_all(&host_dir).expect("create host dir");
        fs::create_dir_all(&e2e_dir).expect("create e2e dir");
        fs::write(site_dir.join("Cargo.toml"), "js-sys = \"0.3\"\n").expect("write site manifest");
        fs::write(e2e_dir.join("package.json"), "{}\n").expect("write e2e package");
        fs::write(e2e_dir.join("package-lock.json"), "{}\n").expect("write e2e lockfile");
        fs::write(
            host_dir.join("persistence.rs"),
            "window.navigator().storage();\n",
        )
        .expect("write persistence source");

        let error = verify_ui_browser_manifest_hygiene(&root).expect_err("hygiene should fail");
        assert!(error.contains("typed `Navigator.storage()` bindings"));
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn verify_ui_shell_style_hygiene_accepts_tokenized_shell_sources() {
        let root = unique_temp_dir("ui-style-hygiene-pass");
        let primitives = root.join("ui/crates/system_ui/src/origin_primitives");
        let runtime = root.join("ui/crates/desktop_runtime/src/components");
        let site = root.join("ui/crates/site/src");
        fs::create_dir_all(&primitives).expect("create primitives dir");
        fs::create_dir_all(&runtime).expect("create runtime dir");
        fs::create_dir_all(site.join("generated")).expect("create generated dir");
        fs::write(
            primitives.join("shell.rs"),
            "const STYLE: &str = \"var(--origin-semantic-surface-taskbar-background)\";\n",
        )
        .expect("write primitives source");
        fs::write(
            runtime.join("taskbar.rs"),
            "const OK: &str = \"taskbar\";\n",
        )
        .expect("write runtime source");
        fs::write(
            site.join("generated").join("tailwind.css"),
            "box-shadow: legacy;\n",
        )
        .expect("write generated source");

        verify_ui_shell_style_hygiene(&root).expect("style hygiene should pass");
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }
}
