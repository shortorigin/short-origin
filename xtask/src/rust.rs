use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;
use serde::Serialize;

use crate::common::{absolutize, display_command, run_command, workspace_root};

const DEFAULT_TARGET_DIR: &str = "target";

pub fn run(args: Vec<String>) -> Result<(), String> {
    let (subcommand, passthrough) = args.split_first().ok_or_else(help)?;

    match subcommand.as_str() {
        "audit" => run_audit(passthrough),
        "clean" => run_clean(passthrough),
        "trace" => run_trace(passthrough),
        other => Err(format!("unsupported rust subcommand `{other}`")),
    }
}

fn run_audit(args: &[String]) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let options = parse_audit_options(&workspace_root, args)?;
    let report = build_audit_report(&workspace_root, &options)?;
    let markdown = render_audit_markdown(&report);

    if let Some(output_dir) = &options.output_dir {
        write_audit_artifacts(output_dir, &report)?;
    }

    println!("{markdown}");
    Ok(())
}

fn run_clean(args: &[String]) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let request = parse_clean_request(&workspace_root, args)?;
    let plan = build_cleanup_plan(&request)?;
    let markdown = render_cleanup_plan(&plan);

    if request.apply {
        apply_cleanup_plan(&plan)?;
    }

    println!("{markdown}");
    Ok(())
}

fn run_trace(args: &[String]) -> Result<(), String> {
    let workspace_root = workspace_root()?;
    let request = parse_trace_request(args)?;
    let plan = build_trace_plan(&request);
    if request.dry_run {
        println!("{}", render_trace_plan(&plan));
        return Ok(());
    }

    let mut command = Command::new(&plan.program);
    command.current_dir(&workspace_root);
    command.args(&plan.args);
    for (key, value) in &plan.env {
        command.env(key, value);
    }
    run_command(&mut command)
}

fn parse_audit_options(workspace_root: &Path, args: &[String]) -> Result<RustAuditOptions, String> {
    let mut output_dir = None;
    let mut include_timings = false;
    let mut target_dir = workspace_root.join(DEFAULT_TARGET_DIR);
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--output-dir" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --output-dir".to_string());
                };
                output_dir = Some(absolutize(workspace_root, value));
                index += 2;
            }
            "--target-dir" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --target-dir".to_string());
                };
                target_dir = absolutize(workspace_root, value);
                index += 2;
            }
            "--timings" => {
                include_timings = true;
                index += 1;
            }
            other => return Err(format!("unknown rust audit argument `{other}`")),
        }
    }

    Ok(RustAuditOptions {
        target_dir,
        output_dir,
        include_timings,
    })
}

fn parse_clean_request(workspace_root: &Path, args: &[String]) -> Result<RustCleanRequest, String> {
    let (scope, rest) = args.split_first().ok_or_else(|| {
        "expected `rust clean <docs|release|incremental|timings|target|all>`".to_string()
    })?;
    let mut apply = false;
    let mut target_dir = workspace_root.join(DEFAULT_TARGET_DIR);
    let mut trailing = Vec::new();
    let mut index = 0usize;
    while index < rest.len() {
        match rest[index].as_str() {
            "--apply" => {
                apply = true;
                index += 1;
            }
            "--target-dir" => {
                let Some(value) = rest.get(index + 1) else {
                    return Err("missing value for --target-dir".to_string());
                };
                target_dir = absolutize(workspace_root, value);
                index += 2;
            }
            other => {
                trailing.push(other.to_string());
                index += 1;
            }
        }
    }

    let scope = match scope.as_str() {
        "docs" => {
            ensure_no_trailing(&trailing, "rust clean docs")?;
            CleanupScope::Docs
        }
        "release" => {
            ensure_no_trailing(&trailing, "rust clean release")?;
            CleanupScope::Release
        }
        "incremental" => {
            ensure_no_trailing(&trailing, "rust clean incremental")?;
            CleanupScope::Incremental
        }
        "timings" => {
            ensure_no_trailing(&trailing, "rust clean timings")?;
            CleanupScope::Timings
        }
        "all" => {
            ensure_no_trailing(&trailing, "rust clean all")?;
            CleanupScope::All
        }
        "target" => {
            let Some(triple) = trailing.first() else {
                return Err("expected `rust clean target <triple>`".to_string());
            };
            if trailing.len() != 1 {
                return Err("expected only one target triple for `rust clean target`".to_string());
            }
            CleanupScope::TargetTriple(triple.clone())
        }
        other => return Err(format!("unsupported rust clean scope `{other}`")),
    };

    Ok(RustCleanRequest {
        scope,
        target_dir,
        apply,
    })
}

fn ensure_no_trailing(args: &[String], command: &str) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("unexpected trailing arguments for `{command}`"))
    }
}

fn parse_trace_request(args: &[String]) -> Result<RustTraceRequest, String> {
    let (preset, rest) = args
        .split_first()
        .ok_or_else(|| "expected `rust trace <site|desktop|cargo>`".to_string())?;
    let preset = match preset.as_str() {
        "site" => TracePreset::Site,
        "desktop" => TracePreset::Desktop,
        "cargo" => TracePreset::Cargo,
        other => return Err(format!("unsupported rust trace preset `{other}`")),
    };

    let mut dry_run = false;
    let mut tokio_console = false;
    let mut cargo_log = None;
    let mut passthrough = Vec::new();
    let mut parsing_passthrough = false;
    let mut index = 0usize;
    while index < rest.len() {
        let value = &rest[index];
        if parsing_passthrough {
            passthrough.push(value.clone());
            index += 1;
            continue;
        }

        match value.as_str() {
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            "--tokio-console" => {
                tokio_console = true;
                index += 1;
            }
            "--cargo-log" => {
                let Some(filter) = rest.get(index + 1) else {
                    return Err("missing value for --cargo-log".to_string());
                };
                cargo_log = Some(filter.clone());
                index += 2;
            }
            "--" => {
                parsing_passthrough = true;
                index += 1;
            }
            other => return Err(format!("unknown rust trace argument `{other}`")),
        }
    }

    if tokio_console && preset != TracePreset::Desktop {
        return Err("`--tokio-console` is only supported with `rust trace desktop`".to_string());
    }

    Ok(RustTraceRequest {
        preset,
        dry_run,
        tokio_console,
        cargo_log,
        passthrough,
    })
}

fn build_audit_report(
    workspace_root: &Path,
    options: &RustAuditOptions,
) -> Result<RustAuditReport, String> {
    let top_level_sizes = collect_top_level_sizes(&options.target_dir)?;
    let total_bytes = total_path_bytes(&options.target_dir)?;
    let target_families = collect_target_families(&options.target_dir)?;

    let duplicates = capture_command(
        workspace_root,
        command("cargo", ["tree", "-d", "--workspace"]),
    )?;
    let features = capture_command(
        workspace_root,
        command("cargo", ["tree", "-e", "features", "--workspace"]),
    )?;

    let build_script_audit = audit_build_scripts(workspace_root)?;
    let timings = if options.include_timings {
        Some(run_cargo_timings(workspace_root, &options.target_dir)?)
    } else {
        None
    };

    Ok(RustAuditReport {
        target_dir: options.target_dir.display().to_string(),
        total_bytes,
        top_level_sizes,
        target_families,
        duplicate_summary: command_summary(&duplicates, &build_duplicate_roots(&duplicates.stdout)),
        feature_summary: command_summary(&features, &sample_non_empty_lines(&features.stdout, 20)),
        duplicate_raw: duplicates.stdout,
        feature_raw: features.stdout,
        build_script_audit,
        timings,
    })
}

fn build_cleanup_plan(request: &RustCleanRequest) -> Result<CleanupPlan, String> {
    let paths = cleanup_paths_for_scope(&request.target_dir, &request.scope)?;
    let entries = paths
        .into_iter()
        .map(|path| {
            let bytes = if path.exists() {
                total_path_bytes(&path)?
            } else {
                0
            };
            Ok(CleanupEntry {
                path: path.display().to_string(),
                bytes,
                exists: path.exists(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let reclaimable_bytes = entries.iter().map(|entry| entry.bytes).sum();

    Ok(CleanupPlan {
        scope: request.scope.clone(),
        apply: request.apply,
        target_dir: request.target_dir.display().to_string(),
        entries,
        reclaimable_bytes,
    })
}

fn build_trace_plan(request: &RustTraceRequest) -> TracePlan {
    let mut env = vec![
        ("RUST_BACKTRACE".to_string(), "1".to_string()),
        ("RUST_LIB_BACKTRACE".to_string(), "1".to_string()),
    ];

    match request.preset {
        TracePreset::Site => {
            if std::env::var_os("ORIGIN_ENVIRONMENT").is_none() {
                env.push(("ORIGIN_ENVIRONMENT".to_string(), "dev".to_string()));
            }
            TracePlan {
                preset: request.preset,
                program: "cargo".to_string(),
                args: trace_passthrough_or_default(&request.passthrough, &["xtask", "ui", "dev"]),
                env,
            }
        }
        TracePreset::Desktop => {
            if std::env::var_os("ORIGIN_ENVIRONMENT").is_none() {
                env.push(("ORIGIN_ENVIRONMENT".to_string(), "dev".to_string()));
            }
            if request.tokio_console {
                env.push(("ORIGIN_ENABLE_TOKIO_CONSOLE".to_string(), "1".to_string()));
                env.push((
                    "RUSTFLAGS".to_string(),
                    append_rustflag(std::env::var("RUSTFLAGS").ok(), "--cfg tokio_unstable"),
                ));
            }
            TracePlan {
                preset: request.preset,
                program: "cargo".to_string(),
                args: trace_passthrough_or_default(
                    &request.passthrough,
                    &["xtask", "tauri", "dev"],
                ),
                env,
            }
        }
        TracePreset::Cargo => {
            if let Some(filter) = &request.cargo_log {
                env.push(("CARGO_LOG".to_string(), filter.clone()));
            }
            TracePlan {
                preset: request.preset,
                program: "cargo".to_string(),
                args: trace_passthrough_or_default(&request.passthrough, &["check"]),
                env,
            }
        }
    }
}

fn trace_passthrough_or_default(passthrough: &[String], default: &[&str]) -> Vec<String> {
    if passthrough.is_empty() {
        default.iter().map(|value| (*value).to_string()).collect()
    } else {
        passthrough.to_vec()
    }
}

fn append_rustflag(existing: Option<String>, flag: &str) -> String {
    match existing {
        Some(existing) if !existing.contains(flag) => format!("{existing} {flag}"),
        Some(existing) => existing,
        None => flag.to_string(),
    }
}

fn cleanup_paths_for_scope(
    target_dir: &Path,
    scope: &CleanupScope,
) -> Result<Vec<PathBuf>, String> {
    let mut paths = match scope {
        CleanupScope::Docs => collect_named_directories(target_dir, "doc")?,
        CleanupScope::Release => collect_release_directories(target_dir)?,
        CleanupScope::Incremental => collect_named_directories(target_dir, "incremental")?,
        CleanupScope::Timings => vec![target_dir.join("cargo-timings")],
        CleanupScope::All => vec![target_dir.to_path_buf()],
        CleanupScope::TargetTriple(triple) => vec![target_dir.join(triple)],
    };
    dedupe_nested_paths(&mut paths);
    Ok(paths)
}

fn collect_top_level_sizes(target_dir: &Path) -> Result<Vec<PathSize>, String> {
    let mut sizes = Vec::new();
    if !target_dir.exists() {
        return Ok(sizes);
    }

    for entry in fs::read_dir(target_dir)
        .map_err(|error| format!("failed to read `{}`: {error}", target_dir.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read entry in `{}`: {error}",
                target_dir.display()
            )
        })?;
        let path = entry.path();
        sizes.push(PathSize {
            path: path.display().to_string(),
            bytes: total_path_bytes(&path)?,
        });
    }
    sizes.sort_by(|left, right| right.bytes.cmp(&left.bytes));
    Ok(sizes)
}

fn collect_target_families(target_dir: &Path) -> Result<Vec<TargetFamilyBreakdown>, String> {
    let mut families = Vec::new();
    families.push(target_family_breakdown("host", target_dir)?);

    if target_dir.exists() {
        for entry in fs::read_dir(target_dir)
            .map_err(|error| format!("failed to read `{}`: {error}", target_dir.display()))?
        {
            let entry = entry.map_err(|error| {
                format!("failed to inspect `{}`: {error}", target_dir.display())
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(OsStr::to_str) else {
                continue;
            };
            if !looks_like_target_triple(&path, name) {
                continue;
            }
            families.push(target_family_breakdown(name, &path)?);
        }
    }

    families.sort_by(|left, right| right.total_bytes.cmp(&left.total_bytes));
    Ok(families)
}

fn looks_like_target_triple(path: &Path, name: &str) -> bool {
    if matches!(name, "debug" | "release" | "doc" | "cargo-timings" | "tmp") {
        return false;
    }
    path.join("debug").exists() || path.join("release").exists() || path.join("doc").exists()
}

fn target_family_breakdown(
    label: &str,
    family_root: &Path,
) -> Result<TargetFamilyBreakdown, String> {
    let mut profiles = Vec::new();
    for profile in ["debug", "release"] {
        let path = family_root.join(profile);
        if !path.exists() {
            continue;
        }
        let deps = total_path_bytes(&path.join("deps"))?;
        let build = total_path_bytes(&path.join("build"))?;
        let incremental = total_path_bytes(&path.join("incremental"))?;
        let examples = total_path_bytes(&path.join("examples"))?;
        let total = total_path_bytes(&path)?;
        let other = total.saturating_sub(deps + build + incremental + examples);
        profiles.push(ProfileBreakdown {
            profile: profile.to_string(),
            path: path.display().to_string(),
            total_bytes: total,
            deps_bytes: deps,
            build_bytes: build,
            incremental_bytes: incremental,
            examples_bytes: examples,
            other_bytes: other,
        });
    }
    let doc_bytes = total_path_bytes(&family_root.join("doc"))?;
    let total_bytes = doc_bytes
        + profiles
            .iter()
            .map(|profile| profile.total_bytes)
            .sum::<u64>();
    Ok(TargetFamilyBreakdown {
        target: label.to_string(),
        root_path: family_root.display().to_string(),
        doc_bytes,
        total_bytes,
        profiles,
    })
}

fn collect_release_directories(target_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut directories = Vec::new();
    if !target_dir.exists() {
        return Ok(directories);
    }

    if target_dir.join("release").exists() {
        directories.push(target_dir.join("release"));
    }
    for entry in fs::read_dir(target_dir)
        .map_err(|error| format!("failed to read `{}`: {error}", target_dir.display()))?
    {
        let entry = entry
            .map_err(|error| format!("failed to inspect `{}`: {error}", target_dir.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };
        if looks_like_target_triple(&path, name) && path.join("release").exists() {
            directories.push(path.join("release"));
        }
    }
    Ok(directories)
}

fn collect_named_directories(root: &Path, wanted: &str) -> Result<Vec<PathBuf>, String> {
    let mut directories = Vec::new();
    if !root.exists() {
        return Ok(directories);
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?
        {
            let entry = entry
                .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?;
            let child = entry.path();
            if !child.is_dir() {
                continue;
            }
            if child.file_name().and_then(OsStr::to_str) == Some(wanted) {
                directories.push(child);
                continue;
            }
            stack.push(child);
        }
    }
    Ok(directories)
}

fn dedupe_nested_paths(paths: &mut Vec<PathBuf>) {
    paths.sort();
    let mut deduped = Vec::new();
    for path in paths.drain(..) {
        if deduped
            .iter()
            .any(|existing: &PathBuf| path.starts_with(existing))
        {
            continue;
        }
        deduped.push(path);
    }
    *paths = deduped;
}

fn audit_build_scripts(workspace_root: &Path) -> Result<Vec<BuildScriptAudit>, String> {
    let mut scripts = Vec::new();
    for script in workspace_build_scripts(workspace_root)? {
        let contents = fs::read_to_string(&script)
            .map_err(|error| format!("failed to read `{}`: {error}", script.display()))?;
        let relative = script
            .strip_prefix(workspace_root)
            .unwrap_or(&script)
            .display()
            .to_string();
        let has_changed = has_rerun_if_changed(&contents);
        let has_env = has_rerun_if_env_changed(&contents);
        scripts.push(BuildScriptAudit {
            path: relative,
            has_rerun_if_changed: has_changed,
            has_rerun_if_env_changed: has_env,
            status: BuildScriptStatus::from_flags(has_changed, has_env),
        });
    }
    scripts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(scripts)
}

fn workspace_build_scripts(workspace_root: &Path) -> Result<Vec<PathBuf>, String> {
    let roots = [
        "agents",
        "enterprise",
        "platform",
        "schemas",
        "services",
        "shared",
        "ui",
        "workflows",
        "xtask",
    ];
    let mut scripts = Vec::new();
    for root in roots {
        let base = workspace_root.join(root);
        if !base.exists() {
            continue;
        }
        let mut stack = vec![base];
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(&path)
                .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?
            {
                let entry = entry
                    .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?;
                let child = entry.path();
                if child.is_dir() {
                    if child.file_name().and_then(OsStr::to_str) == Some("target") {
                        continue;
                    }
                    stack.push(child);
                    continue;
                }
                if child.file_name().and_then(OsStr::to_str) == Some("build.rs") {
                    scripts.push(child);
                }
            }
        }
    }
    Ok(scripts)
}

fn run_cargo_timings(workspace_root: &Path, target_dir: &Path) -> Result<TimingsAudit, String> {
    let mut command = Command::new("cargo");
    command.current_dir(workspace_root);
    command.args(["build", "--timings"]);
    command.env("CARGO_TARGET_DIR", target_dir);

    let display = display_command(&command);
    let output = command
        .output()
        .map_err(|error| format!("failed to run `{display}`: {error}"))?;
    if !output.status.success() {
        return Err(format!("`{display}` exited with status {}", output.status));
    }

    Ok(TimingsAudit {
        command: display,
        report_dir: target_dir.join("cargo-timings").display().to_string(),
        stdout_preview: sample_non_empty_lines(&String::from_utf8_lossy(&output.stdout), 20),
        stderr_preview: sample_non_empty_lines(&String::from_utf8_lossy(&output.stderr), 20),
        stdout_raw: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr_raw: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn command(program: &str, args: impl IntoIterator<Item = &'static str>) -> Command {
    let mut command = Command::new(program);
    command.args(args);
    command
}

fn capture_command(workspace_root: &Path, mut command: Command) -> Result<CommandCapture, String> {
    command.current_dir(workspace_root);
    let display = display_command(&command);
    let output = command
        .output()
        .map_err(|error| format!("failed to run `{display}`: {error}"))?;
    if !output.status.success() {
        return Err(format!("`{display}` exited with status {}", output.status));
    }

    Ok(CommandCapture {
        command: display,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

fn command_summary(capture: &CommandCapture, sample: &[String]) -> CommandSummary {
    CommandSummary {
        command: capture.command.clone(),
        line_count: capture.stdout.lines().count(),
        sample: sample.to_vec(),
    }
}

fn build_duplicate_roots(stdout: &str) -> Vec<String> {
    let root_pattern = Regex::new(r"^[A-Za-z0-9_.-]+ v\d").expect("valid duplicate regex");
    let mut roots = BTreeSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if root_pattern.is_match(trimmed) {
            roots.insert(trimmed.to_string());
        }
    }
    if roots.is_empty() {
        sample_non_empty_lines(stdout, 20)
    } else {
        roots.into_iter().take(20).collect()
    }
}

fn sample_non_empty_lines(text: &str, limit: usize) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(limit)
        .map(str::to_string)
        .collect()
}

fn has_rerun_if_changed(contents: &str) -> bool {
    contents.contains("cargo::rerun-if-changed") || contents.contains("cargo:rerun-if-changed")
}

fn has_rerun_if_env_changed(contents: &str) -> bool {
    contents.contains("cargo::rerun-if-env-changed")
        || contents.contains("cargo:rerun-if-env-changed")
}

fn write_audit_artifacts(output_dir: &Path, report: &RustAuditReport) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;

    let json = serde_json::to_string_pretty(report)
        .map_err(|error| format!("failed to serialize rust audit report: {error}"))?;
    fs::write(output_dir.join("rust-audit.json"), format!("{json}\n"))
        .map_err(|error| format!("failed to write rust audit JSON: {error}"))?;
    fs::write(
        output_dir.join("rust-audit.md"),
        render_audit_markdown(report),
    )
    .map_err(|error| format!("failed to write rust audit markdown: {error}"))?;
    fs::write(
        output_dir.join("cargo-tree-duplicates.txt"),
        &report.duplicate_raw,
    )
    .map_err(|error| format!("failed to write duplicate tree appendix: {error}"))?;
    fs::write(
        output_dir.join("cargo-tree-features.txt"),
        &report.feature_raw,
    )
    .map_err(|error| format!("failed to write feature tree appendix: {error}"))?;
    if let Some(timings) = &report.timings {
        fs::write(
            output_dir.join("cargo-build-timings.stdout.txt"),
            &timings.stdout_raw,
        )
        .map_err(|error| format!("failed to write timings stdout appendix: {error}"))?;
        fs::write(
            output_dir.join("cargo-build-timings.stderr.txt"),
            &timings.stderr_raw,
        )
        .map_err(|error| format!("failed to write timings stderr appendix: {error}"))?;
    }
    Ok(())
}

fn apply_cleanup_plan(plan: &CleanupPlan) -> Result<(), String> {
    for entry in &plan.entries {
        if !entry.exists {
            continue;
        }
        let path = Path::new(&entry.path);
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
        } else if path.exists() {
            fs::remove_file(path)
                .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
        }
    }
    Ok(())
}

fn total_path_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if metadata.file_type().is_symlink() {
        return Ok(0);
    }

    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(next) = stack.pop() {
        let metadata = fs::symlink_metadata(&next)
            .map_err(|error| format!("failed to inspect `{}`: {error}", next.display()))?;
        if metadata.is_file() {
            total = total.saturating_add(metadata.len());
            continue;
        }
        if metadata.file_type().is_symlink() {
            continue;
        }
        for entry in fs::read_dir(&next)
            .map_err(|error| format!("failed to read `{}`: {error}", next.display()))?
        {
            let entry = entry
                .map_err(|error| format!("failed to inspect `{}`: {error}", next.display()))?;
            stack.push(entry.path());
        }
    }
    Ok(total)
}

fn render_audit_markdown(report: &RustAuditReport) -> String {
    let mut lines = vec![
        "# Rust Build Audit".to_string(),
        String::new(),
        format!("- Target directory: `{}`", report.target_dir),
        format!("- Total size: {}", human_bytes(report.total_bytes)),
        format!(
            "- Timings: {}",
            report
                .timings
                .as_ref()
                .map_or("not requested".to_string(), |timings| format!(
                    "captured in `{}`",
                    timings.report_dir
                ))
        ),
        String::new(),
        "## Top-Level Subtrees".to_string(),
    ];

    if report.top_level_sizes.is_empty() {
        lines.push("- No target directory contents found.".to_string());
    } else {
        for entry in report.top_level_sizes.iter().take(10) {
            lines.push(format!("- `{}`: {}", entry.path, human_bytes(entry.bytes)));
        }
    }

    lines.push(String::new());
    lines.push("## Target Families".to_string());
    for family in &report.target_families {
        lines.push(format!(
            "- `{}` total: {} (doc: {})",
            family.target,
            human_bytes(family.total_bytes),
            human_bytes(family.doc_bytes)
        ));
        for profile in &family.profiles {
            lines.push(format!(
                "  - {}: total {}, deps {}, incremental {}, build {}, examples {}, other {}",
                profile.profile,
                human_bytes(profile.total_bytes),
                human_bytes(profile.deps_bytes),
                human_bytes(profile.incremental_bytes),
                human_bytes(profile.build_bytes),
                human_bytes(profile.examples_bytes),
                human_bytes(profile.other_bytes)
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Duplicate Versions".to_string());
    lines.push(format!("- Command: `{}`", report.duplicate_summary.command));
    lines.push(format!(
        "- Captured lines: {}",
        report.duplicate_summary.line_count
    ));
    if report.duplicate_summary.sample.is_empty() {
        lines.push("- No duplicate-version sample lines captured.".to_string());
    } else {
        for sample in &report.duplicate_summary.sample {
            lines.push(format!("- `{sample}`"));
        }
    }

    lines.push(String::new());
    lines.push("## Feature Activation Sample".to_string());
    lines.push(format!("- Command: `{}`", report.feature_summary.command));
    lines.push(format!(
        "- Captured lines: {}",
        report.feature_summary.line_count
    ));
    for sample in &report.feature_summary.sample {
        lines.push(format!("- `{sample}`"));
    }

    lines.push(String::new());
    lines.push("## Build Script Rerun Hygiene".to_string());
    for audit in &report.build_script_audit {
        lines.push(format!("- `{}`: {}", audit.path, audit.status.as_str()));
    }

    if let Some(timings) = &report.timings {
        lines.push(String::new());
        lines.push("## Timings".to_string());
        lines.push(format!("- Command: `{}`", timings.command));
        lines.push(format!("- Output directory: `{}`", timings.report_dir));
        for line in &timings.stdout_preview {
            lines.push(format!("- stdout: `{line}`"));
        }
        for line in &timings.stderr_preview {
            lines.push(format!("- stderr: `{line}`"));
        }
    }

    lines.join("\n")
}

fn render_cleanup_plan(plan: &CleanupPlan) -> String {
    let action = if plan.apply { "Applied" } else { "Dry run" };
    let mut lines = vec![
        format!(
            "# Rust Cleanup {}",
            if plan.apply { "Applied" } else { "Preview" }
        ),
        String::new(),
        format!("- Scope: `{}`", plan.scope.as_str()),
        format!("- Target directory: `{}`", plan.target_dir),
        format!("- Mode: {action}"),
        format!(
            "- Reclaimable size: {}",
            human_bytes(plan.reclaimable_bytes)
        ),
        String::new(),
        "## Paths".to_string(),
    ];
    if plan.entries.is_empty() {
        lines.push("- Nothing matched the requested cleanup scope.".to_string());
    } else {
        for entry in &plan.entries {
            lines.push(format!(
                "- `{}`: {}{}",
                entry.path,
                human_bytes(entry.bytes),
                if entry.exists { "" } else { " (missing)" }
            ));
        }
    }
    if !plan.apply {
        lines.push(String::new());
        lines.push("Re-run with `--apply` to delete the matched paths.".to_string());
    }
    lines.join("\n")
}

fn render_trace_plan(plan: &TracePlan) -> String {
    let mut lines = vec![
        "# Rust Trace Dry Run".to_string(),
        String::new(),
        format!("- Preset: `{}`", plan.preset.as_str()),
        format!("- Command: `{}` {}", plan.program, plan.args.join(" ")),
        String::new(),
        "## Environment".to_string(),
    ];
    for (key, value) in &plan.env {
        lines.push(format!("- `{key}={value}`"));
    }
    lines.join("\n")
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [(&str, u64); 5] = [
        ("B", 1),
        ("KiB", 1024),
        ("MiB", 1024_u64.pow(2)),
        ("GiB", 1024_u64.pow(3)),
        ("TiB", 1024_u64.pow(4)),
    ];
    let (unit, unit_size) = UNITS
        .iter()
        .rev()
        .find(|(_, unit_size)| bytes >= *unit_size)
        .copied()
        .unwrap_or(("B", 1));

    if unit_size == 1 {
        return format!("{bytes} {unit}");
    }

    let whole = bytes / unit_size;
    let remainder = bytes % unit_size;
    let tenths = remainder.saturating_mul(10) / unit_size;
    format!("{whole}.{tenths} {unit}")
}

fn help() -> String {
    "\
usage: cargo xtask rust <audit|clean|trace> ...

Commands:
  audit   Read-only target usage, dependency tree, feature, and build-script hygiene audit
  clean   Targeted cleanup with dry-run by default and `--apply` for deletion
  trace   Launch browser, desktop, or cargo workflows with backtrace/tracing defaults
"
    .to_string()
}

#[derive(Debug, Clone)]
struct RustAuditOptions {
    target_dir: PathBuf,
    output_dir: Option<PathBuf>,
    include_timings: bool,
}

#[derive(Debug, Clone)]
struct RustCleanRequest {
    scope: CleanupScope,
    target_dir: PathBuf,
    apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustTraceRequest {
    preset: TracePreset,
    dry_run: bool,
    tokio_console: bool,
    cargo_log: Option<String>,
    passthrough: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RustAuditReport {
    target_dir: String,
    total_bytes: u64,
    top_level_sizes: Vec<PathSize>,
    target_families: Vec<TargetFamilyBreakdown>,
    duplicate_summary: CommandSummary,
    feature_summary: CommandSummary,
    #[serde(skip_serializing)]
    duplicate_raw: String,
    #[serde(skip_serializing)]
    feature_raw: String,
    build_script_audit: Vec<BuildScriptAudit>,
    timings: Option<TimingsAudit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PathSize {
    path: String,
    bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TargetFamilyBreakdown {
    target: String,
    root_path: String,
    doc_bytes: u64,
    total_bytes: u64,
    profiles: Vec<ProfileBreakdown>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProfileBreakdown {
    profile: String,
    path: String,
    total_bytes: u64,
    deps_bytes: u64,
    build_bytes: u64,
    incremental_bytes: u64,
    examples_bytes: u64,
    other_bytes: u64,
}

#[derive(Debug, Clone)]
struct CommandCapture {
    command: String,
    stdout: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CommandSummary {
    command: String,
    line_count: usize,
    sample: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BuildScriptAudit {
    path: String,
    has_rerun_if_changed: bool,
    has_rerun_if_env_changed: bool,
    status: BuildScriptStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum BuildScriptStatus {
    Configured,
    ChangedOnly,
    EnvOnly,
    Missing,
}

impl BuildScriptStatus {
    fn from_flags(has_changed: bool, has_env: bool) -> Self {
        match (has_changed, has_env) {
            (true, true) => Self::Configured,
            (true, false) => Self::ChangedOnly,
            (false, true) => Self::EnvOnly,
            (false, false) => Self::Missing,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Configured => "configured with rerun-if-changed and rerun-if-env-changed",
            Self::ChangedOnly => "only rerun-if-changed directives detected",
            Self::EnvOnly => "only rerun-if-env-changed directives detected",
            Self::Missing => "missing rerun-if directives",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TimingsAudit {
    command: String,
    report_dir: String,
    stdout_preview: Vec<String>,
    stderr_preview: Vec<String>,
    #[serde(skip_serializing)]
    stdout_raw: String,
    #[serde(skip_serializing)]
    stderr_raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CleanupPlan {
    scope: CleanupScope,
    apply: bool,
    target_dir: String,
    entries: Vec<CleanupEntry>,
    reclaimable_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CleanupEntry {
    path: String,
    bytes: u64,
    exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CleanupScope {
    Docs,
    Release,
    Incremental,
    Timings,
    TargetTriple(String),
    All,
}

impl CleanupScope {
    fn as_str(&self) -> &str {
        match self {
            Self::Docs => "docs",
            Self::Release => "release",
            Self::Incremental => "incremental",
            Self::Timings => "timings",
            Self::TargetTriple(triple) => triple.as_str(),
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TracePreset {
    Site,
    Desktop,
    Cargo,
}

impl TracePreset {
    fn as_str(self) -> &'static str {
        match self {
            Self::Site => "site",
            Self::Desktop => "desktop",
            Self::Cargo => "cargo",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TracePlan {
    preset: TracePreset,
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::{
        BuildScriptAudit, BuildScriptStatus, CleanupScope, CommandSummary, PathSize,
        ProfileBreakdown, RustAuditReport, RustCleanRequest, TargetFamilyBreakdown, TracePreset,
        append_rustflag, build_cleanup_plan, build_duplicate_roots, build_trace_plan,
        has_rerun_if_changed, has_rerun_if_env_changed, parse_clean_request, parse_trace_request,
        render_audit_markdown, sample_non_empty_lines,
    };
    use std::fs;
    use std::path::PathBuf;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask-rust-{label}-{}-{}",
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
    fn parse_clean_request_defaults_to_dry_run() {
        let request = parse_clean_request(
            PathBuf::from("/workspace").as_path(),
            &["incremental".to_string()],
        )
        .expect("parse clean request");
        assert_eq!(request.scope, CleanupScope::Incremental);
        assert!(!request.apply);
        assert_eq!(request.target_dir, PathBuf::from("/workspace/target"));
    }

    #[test]
    fn build_cleanup_plan_collects_incremental_dirs() {
        let root = unique_temp_dir("cleanup");
        let target = root.join("target");
        fs::create_dir_all(target.join("debug/incremental")).expect("create incremental");
        fs::write(target.join("debug/incremental/state.bin"), vec![0u8; 128]).expect("write data");

        let plan = build_cleanup_plan(&RustCleanRequest {
            scope: CleanupScope::Incremental,
            target_dir: target.clone(),
            apply: false,
        })
        .expect("build cleanup plan");

        assert_eq!(plan.entries.len(), 1);
        assert!(plan.reclaimable_bytes >= 128);
        fs::remove_dir_all(root).expect("cleanup temp dir");
    }

    #[test]
    fn parse_trace_request_supports_tokio_console_and_passthrough() {
        let request = parse_trace_request(&[
            "desktop".to_string(),
            "--tokio-console".to_string(),
            "--dry-run".to_string(),
            "--".to_string(),
            "xtask".to_string(),
            "tauri".to_string(),
            "dev".to_string(),
        ])
        .expect("parse trace request");
        assert_eq!(request.preset, TracePreset::Desktop);
        assert!(request.tokio_console);
        assert!(request.dry_run);
        assert_eq!(request.passthrough, ["xtask", "tauri", "dev"]);
    }

    #[test]
    fn build_trace_plan_sets_tokio_unstable_when_requested() {
        let plan = build_trace_plan(&super::RustTraceRequest {
            preset: TracePreset::Desktop,
            dry_run: true,
            tokio_console: true,
            cargo_log: None,
            passthrough: Vec::new(),
        });
        assert!(
            plan.env
                .iter()
                .any(|(key, value)| key == "RUSTFLAGS" && value.contains("tokio_unstable"))
        );
        assert!(
            plan.env
                .iter()
                .any(|(key, value)| key == "ORIGIN_ENABLE_TOKIO_CONSOLE" && value == "1")
        );
    }

    #[test]
    fn append_rustflag_appends_when_missing() {
        let combined = append_rustflag(Some("-Dwarnings".to_string()), "--cfg tokio_unstable");
        assert_eq!(combined, "-Dwarnings --cfg tokio_unstable");
    }

    #[test]
    fn sample_non_empty_lines_skips_blanks() {
        let lines = sample_non_empty_lines("\nalpha\n\nbeta\n", 5);
        assert_eq!(lines, ["alpha", "beta"]);
    }

    #[test]
    fn build_duplicate_roots_extracts_unique_roots() {
        let roots = build_duplicate_roots(
            "serde v1.0.0\n├── foo v0.1.0\nserde_json v1.0.0\n└── bar v0.1.0\n",
        );
        assert_eq!(roots, ["serde v1.0.0", "serde_json v1.0.0"]);
    }

    #[test]
    fn build_script_hygiene_detection_covers_both_directive_types() {
        assert!(has_rerun_if_changed(
            "println!(\"cargo::rerun-if-changed=src/\");"
        ));
        assert!(has_rerun_if_env_changed(
            "println!(\"cargo::rerun-if-env-changed=FOO\");"
        ));
        assert!(!has_rerun_if_changed("println!(\"hello\");"));
    }

    #[test]
    fn render_audit_markdown_includes_key_sections() {
        let markdown = render_audit_markdown(&RustAuditReport {
            target_dir: "target".to_string(),
            total_bytes: 1024,
            top_level_sizes: vec![PathSize {
                path: "target/debug".to_string(),
                bytes: 512,
            }],
            target_families: vec![TargetFamilyBreakdown {
                target: "host".to_string(),
                root_path: "target".to_string(),
                doc_bytes: 0,
                total_bytes: 512,
                profiles: vec![ProfileBreakdown {
                    profile: "debug".to_string(),
                    path: "target/debug".to_string(),
                    total_bytes: 512,
                    deps_bytes: 256,
                    build_bytes: 64,
                    incremental_bytes: 128,
                    examples_bytes: 0,
                    other_bytes: 64,
                }],
            }],
            duplicate_summary: CommandSummary {
                command: "cargo tree -d --workspace".to_string(),
                line_count: 2,
                sample: vec!["serde v1.0.0".to_string()],
            },
            feature_summary: CommandSummary {
                command: "cargo tree -e features --workspace".to_string(),
                line_count: 2,
                sample: vec!["desktop_tauri feature \"default\"".to_string()],
            },
            duplicate_raw: "serde v1.0.0\n".to_string(),
            feature_raw: "desktop_tauri feature \"default\"\n".to_string(),
            build_script_audit: vec![BuildScriptAudit {
                path: "ui/crates/system_ui/build.rs".to_string(),
                has_rerun_if_changed: true,
                has_rerun_if_env_changed: false,
                status: BuildScriptStatus::ChangedOnly,
            }],
            timings: None,
        });

        assert!(markdown.contains("# Rust Build Audit"));
        assert!(markdown.contains("## Build Script Rerun Hygiene"));
        assert!(markdown.contains("cargo tree -d --workspace"));
    }
}
