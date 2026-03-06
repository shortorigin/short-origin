use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const GH_ACCEPT_HEADER: &str = "Accept: application/vnd.github+json";
const GH_API_VERSION_HEADER: &str = "X-GitHub-Api-Version: 2022-11-28";

#[derive(Debug, Deserialize)]
struct GovernanceConfig {
    version: u32,
    organization: OrganizationConfig,
    repository_defaults: RepositoryDefaults,
    project: ProjectConfig,
    labels: Vec<LabelConfig>,
    milestones: Vec<MilestoneConfig>,
    repositories: Vec<RepositoryConfig>,
}

#[derive(Debug, Deserialize)]
struct OrganizationConfig {
    login: String,
    governance_repository: String,
    governance_repository_description: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryDefaults {
    default_branch: String,
    branch_name_pattern: String,
    pr_title_pattern: String,
    required_status_checks: Vec<String>,
    required_approving_review_count: u8,
    dismiss_stale_reviews_on_push: bool,
    require_code_owner_review: bool,
    required_review_thread_resolution: bool,
    allow_auto_merge: bool,
    allow_squash_merge: bool,
    allow_merge_commit: bool,
    allow_rebase_merge: bool,
    delete_branch_on_merge: bool,
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    title: String,
    short_description: String,
    status_field_name: String,
    status_options: Vec<String>,
    repository_views: Vec<String>,
    milestone_views: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LabelConfig {
    name: String,
    color: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct MilestoneConfig {
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryConfig {
    name: String,
    link_to_project: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyncTarget {
    Org,
    Repo,
}

struct SyncArgs {
    target: SyncTarget,
    config_path: PathBuf,
    repository: Option<String>,
    apply: bool,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    match args.split_first() {
        Some((command, rest)) if command == "sync" => sync(rest),
        Some((command, rest)) if command == "validate-pr" => validate_pr(rest),
        Some((command, _)) => Err(format!("unknown github xtask command `{command}`")),
        None => Err(help()),
    }
}

fn sync(args: &[String]) -> Result<(), String> {
    let parsed = parse_sync_args(args)?;
    let config = load_config(&parsed.config_path)?;
    if parsed.apply {
        ensure_gh_is_ready()?;
    }

    match parsed.target {
        SyncTarget::Org => {
            let plan = render_org_plan(&config);
            println!("{plan}");
            if parsed.apply {
                apply_org_sync(&config)?;
            }
        }
        SyncTarget::Repo => {
            let repository = parsed
                .repository
                .unwrap_or_else(|| default_repository(&config));
            let plan = render_repo_plan(&config, &repository);
            println!("{plan}");
            if parsed.apply {
                apply_repo_sync(&config, &repository)?;
            }
        }
    }

    Ok(())
}

fn validate_pr(args: &[String]) -> Result<(), String> {
    let mut config_path = PathBuf::from(".github/governance.toml");
    let mut event_path: Option<PathBuf> = None;
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                let Some(path) = args.get(index + 1) else {
                    return Err("missing value for --config".to_owned());
                };
                config_path = PathBuf::from(path);
                index += 2;
            }
            "--event-path" => {
                let Some(path) = args.get(index + 1) else {
                    return Err("missing value for --event-path".to_owned());
                };
                event_path = Some(PathBuf::from(path));
                index += 2;
            }
            other => return Err(format!("unknown validate-pr argument `{other}`")),
        }
    }

    let event_path = event_path.ok_or_else(|| "missing --event-path".to_owned())?;
    let config = load_config(&config_path)?;
    let event = load_pr_event(&event_path)?;
    validate_pr_event(&config, &event)?;
    println!(
        "validated PR governance for branch `{}` with title `{}`",
        event.branch, event.title
    );
    Ok(())
}

fn parse_sync_args(args: &[String]) -> Result<SyncArgs, String> {
    let Some((target, rest)) = args.split_first() else {
        return Err(help());
    };
    let target = match target.as_str() {
        "org" => SyncTarget::Org,
        "repo" => SyncTarget::Repo,
        other => return Err(format!("unknown sync target `{other}`")),
    };

    let mut config_path = PathBuf::from(".github/governance.toml");
    let mut repository = None;
    let mut apply = false;
    let mut dry_run = false;
    let mut index = 0usize;

    while index < rest.len() {
        match rest[index].as_str() {
            "--config" => {
                let Some(path) = rest.get(index + 1) else {
                    return Err("missing value for --config".to_owned());
                };
                config_path = PathBuf::from(path);
                index += 2;
            }
            "--repository" => {
                let Some(repo) = rest.get(index + 1) else {
                    return Err("missing value for --repository".to_owned());
                };
                repository = Some(repo.clone());
                index += 2;
            }
            "--apply" => {
                apply = true;
                index += 1;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            other => return Err(format!("unknown sync argument `{other}`")),
        }
    }

    if apply == dry_run {
        return Err("choose exactly one of --dry-run or --apply".to_owned());
    }

    Ok(SyncArgs {
        target,
        config_path,
        repository,
        apply,
    })
}

fn load_config(path: &Path) -> Result<GovernanceConfig, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read config `{}`: {error}", path.display()))?;
    let config: GovernanceConfig = toml::from_str(&raw)
        .map_err(|error| format!("failed to parse config `{}`: {error}", path.display()))?;
    if config.version != 1 {
        return Err(format!(
            "unsupported governance config version `{}` in `{}`",
            config.version,
            path.display()
        ));
    }
    Ok(config)
}

fn render_org_plan(config: &GovernanceConfig) -> String {
    let mut lines = vec![
        format!("GitHub org sync plan for `{}`", config.organization.login),
        format!(
            "- ensure public governance repository `{}` exists",
            governance_repository_full_name(config)
        ),
        format!(
            "- ensure project `{}` exists with status field `{}` = [{}]",
            config.project.title,
            config.project.status_field_name,
            config.project.status_options.join(", ")
        ),
        format!("- project intent: {}", config.project.short_description),
    ];

    if !config.repositories.is_empty() {
        let repositories = config
            .repositories
            .iter()
            .filter(|repository| repository.link_to_project)
            .map(|repository| repository.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "- ensure linked repositories are attached to the project: {repositories}"
        ));
    }

    lines.push(format!(
        "- manual follow-up: create saved repository views [{}]",
        config.project.repository_views.join(", ")
    ));
    lines.push(format!(
        "- manual follow-up: create saved milestone views [{}]",
        config.project.milestone_views.join(", ")
    ));
    lines.push(
        "- manual follow-up: enable built-in project workflows for auto-add and status transitions"
            .to_owned(),
    );
    lines.join("\n")
}

fn render_repo_plan(config: &GovernanceConfig, repository: &str) -> String {
    let mut lines = vec![
        format!("GitHub repo sync plan for `{repository}`"),
        format!(
            "- repository settings: default branch `{}`, auto-merge={}, squash-only={}, merge commits={}, rebase merges={}, delete head branch on merge={}",
            config.repository_defaults.default_branch,
            config.repository_defaults.allow_auto_merge,
            config.repository_defaults.allow_squash_merge,
            config.repository_defaults.allow_merge_commit,
            config.repository_defaults.allow_rebase_merge,
            config.repository_defaults.delete_branch_on_merge
        ),
        format!(
            "- labels: {}",
            config
                .labels
                .iter()
                .map(|label| label.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "- milestones: {}",
            config
                .milestones
                .iter()
                .map(|milestone| milestone.title.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "- ruleset `main-protection` requires checks [{}], code owner review={}, and conventional squash commits",
            config.repository_defaults.required_status_checks.join(", "),
            config.repository_defaults.require_code_owner_review
        ),
        format!(
            "- ruleset `branch-naming` enforces source branch regex `{}`",
            config.repository_defaults.branch_name_pattern
        ),
        format!(
            "- governance workflow validates PR title regex `{}` and linked issue references",
            config.repository_defaults.pr_title_pattern
        ),
    ];

    if let Ok(payload) = serde_json::to_string_pretty(&main_ruleset_payload(config)) {
        lines.push("- generated main ruleset payload:".to_owned());
        lines.push(payload);
    }
    if let Ok(payload) = serde_json::to_string_pretty(&branch_ruleset_payload(config)) {
        lines.push("- generated branch-naming ruleset payload:".to_owned());
        lines.push(payload);
    }

    lines.join("\n")
}

fn apply_org_sync(config: &GovernanceConfig) -> Result<(), String> {
    ensure_governance_repository(config)?;
    let project_number = ensure_project(config)?;
    ensure_project_status_field(config, project_number)?;
    for repository in config
        .repositories
        .iter()
        .filter(|repository| repository.link_to_project)
    {
        ensure_project_link(config, project_number, repository)?;
    }

    println!(
        "manual follow-up required: create the saved repository/milestone views and project workflows described in the dry-run output"
    );
    Ok(())
}

fn apply_repo_sync(config: &GovernanceConfig, repository: &str) -> Result<(), String> {
    let (owner, repo) = split_repository(repository)?;
    sync_repository_settings(config, owner, repo)?;
    sync_labels(config, owner, repo)?;
    sync_milestones(config, owner, repo)?;
    sync_rulesets(config, owner, repo)?;
    Ok(())
}

fn ensure_governance_repository(config: &GovernanceConfig) -> Result<(), String> {
    let repository = governance_repository_full_name(config);
    if gh_repo_exists(&repository)? {
        return Ok(());
    }

    run_gh(&[
        "repo".to_owned(),
        "create".to_owned(),
        repository,
        "--public".to_owned(),
        "--description".to_owned(),
        config
            .organization
            .governance_repository_description
            .clone(),
    ])?;
    Ok(())
}

fn ensure_project(config: &GovernanceConfig) -> Result<u64, String> {
    if let Some(number) = find_project_number(config, &config.project.title)? {
        return Ok(number);
    }

    let output = run_gh(&[
        "project".to_owned(),
        "create".to_owned(),
        "--owner".to_owned(),
        config.organization.login.clone(),
        "--title".to_owned(),
        config.project.title.clone(),
        "--format".to_owned(),
        "json".to_owned(),
    ])?;
    let value: Value = serde_json::from_str(&output)
        .map_err(|error| format!("failed to parse project create output: {error}"))?;
    value
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| "project create output did not include a project number".to_owned())
}

fn ensure_project_status_field(
    config: &GovernanceConfig,
    project_number: u64,
) -> Result<(), String> {
    let output = run_gh(&[
        "project".to_owned(),
        "field-list".to_owned(),
        project_number.to_string(),
        "--owner".to_owned(),
        config.organization.login.clone(),
        "--format".to_owned(),
        "json".to_owned(),
    ])?;
    let fields: Vec<Value> = serde_json::from_str(&output)
        .map_err(|error| format!("failed to parse project field list output: {error}"))?;
    if fields.iter().any(|field| {
        field
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == config.project.status_field_name)
    }) {
        return Ok(());
    }

    run_gh(&[
        "project".to_owned(),
        "field-create".to_owned(),
        project_number.to_string(),
        "--owner".to_owned(),
        config.organization.login.clone(),
        "--name".to_owned(),
        config.project.status_field_name.clone(),
        "--data-type".to_owned(),
        "SINGLE_SELECT".to_owned(),
        "--single-select-options".to_owned(),
        config.project.status_options.join(","),
    ])?;
    Ok(())
}

fn ensure_project_link(
    config: &GovernanceConfig,
    project_number: u64,
    repository: &RepositoryConfig,
) -> Result<(), String> {
    let repo_argument = repository_full_name(config, repository);
    let error = run_gh(&[
        "project".to_owned(),
        "link".to_owned(),
        project_number.to_string(),
        "--owner".to_owned(),
        config.organization.login.clone(),
        "--repo".to_owned(),
        repo_argument,
    ])
    .err();

    if let Some(message) = error {
        if message.contains("already linked") {
            return Ok(());
        }
        return Err(message);
    }

    Ok(())
}

fn sync_repository_settings(
    config: &GovernanceConfig,
    owner: &str,
    repo: &str,
) -> Result<(), String> {
    let body = json!({
        "default_branch": config.repository_defaults.default_branch,
        "allow_auto_merge": config.repository_defaults.allow_auto_merge,
        "allow_squash_merge": config.repository_defaults.allow_squash_merge,
        "allow_merge_commit": config.repository_defaults.allow_merge_commit,
        "allow_rebase_merge": config.repository_defaults.allow_rebase_merge,
        "delete_branch_on_merge": config.repository_defaults.delete_branch_on_merge,
    });
    gh_api_json("PATCH", &format!("repos/{owner}/{repo}"), Some(body))?;
    Ok(())
}

fn sync_labels(config: &GovernanceConfig, owner: &str, repo: &str) -> Result<(), String> {
    let response = gh_api_json(
        "GET",
        &format!("repos/{owner}/{repo}/labels?per_page=100"),
        None,
    )?;
    let labels = response
        .as_array()
        .ok_or_else(|| "label list API response was not an array".to_owned())?;

    for label in &config.labels {
        let existing = labels.iter().find(|candidate| {
            candidate
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == label.name)
        });

        let body = json!({
            "name": label.name,
            "color": label.color,
            "description": label.description,
        });

        if existing.is_some() {
            gh_api_json(
                "PATCH",
                &format!(
                    "repos/{owner}/{repo}/labels/{}",
                    percent_encode(&label.name)
                ),
                Some(body),
            )?;
        } else {
            gh_api_json("POST", &format!("repos/{owner}/{repo}/labels"), Some(body))?;
        }
    }

    Ok(())
}

fn sync_milestones(config: &GovernanceConfig, owner: &str, repo: &str) -> Result<(), String> {
    let response = gh_api_json(
        "GET",
        &format!("repos/{owner}/{repo}/milestones?state=all&per_page=100"),
        None,
    )?;
    let milestones = response
        .as_array()
        .ok_or_else(|| "milestone list API response was not an array".to_owned())?;

    for milestone in &config.milestones {
        let existing = milestones.iter().find(|candidate| {
            candidate
                .get("title")
                .and_then(Value::as_str)
                .is_some_and(|title| title == milestone.title)
        });

        let body = json!({
            "title": milestone.title,
            "description": milestone.description,
            "state": "open",
        });

        if let Some(existing) = existing {
            let number = existing
                .get("number")
                .and_then(Value::as_u64)
                .ok_or_else(|| "existing milestone response was missing a number".to_owned())?;
            gh_api_json(
                "PATCH",
                &format!("repos/{owner}/{repo}/milestones/{number}"),
                Some(body),
            )?;
        } else {
            gh_api_json(
                "POST",
                &format!("repos/{owner}/{repo}/milestones"),
                Some(body),
            )?;
        }
    }

    Ok(())
}

fn sync_rulesets(config: &GovernanceConfig, owner: &str, repo: &str) -> Result<(), String> {
    let response = gh_api_json("GET", &format!("repos/{owner}/{repo}/rulesets"), None)?;
    let rulesets = response
        .as_array()
        .ok_or_else(|| "ruleset list API response was not an array".to_owned())?;

    upsert_ruleset(config, owner, repo, rulesets, main_ruleset_payload(config))?;
    upsert_ruleset(
        config,
        owner,
        repo,
        rulesets,
        branch_ruleset_payload(config),
    )?;
    Ok(())
}

fn upsert_ruleset(
    config: &GovernanceConfig,
    owner: &str,
    repo: &str,
    existing_rulesets: &[Value],
    payload: Value,
) -> Result<(), String> {
    let name = payload
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "ruleset payload was missing a name".to_owned())?;
    let repository = repository_full_name(
        config,
        &RepositoryConfig {
            name: repo.to_owned(),
            link_to_project: false,
        },
    );

    if let Some(existing) = existing_rulesets.iter().find(|candidate| {
        candidate
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|candidate_name| candidate_name == name)
    }) {
        let ruleset_id = existing
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("ruleset `{name}` in `{repository}` is missing an id"))?;
        gh_api_json(
            "PUT",
            &format!("repos/{owner}/{repo}/rulesets/{ruleset_id}"),
            Some(payload),
        )?;
    } else {
        gh_api_json(
            "POST",
            &format!("repos/{owner}/{repo}/rulesets"),
            Some(payload),
        )?;
    }

    Ok(())
}

fn main_ruleset_payload(config: &GovernanceConfig) -> Value {
    json!({
        "name": "main-protection",
        "target": "branch",
        "enforcement": "active",
        "conditions": {
            "ref_name": {
                "include": [format!("refs/heads/{}", config.repository_defaults.default_branch)],
                "exclude": []
            }
        },
        "rules": [
            { "type": "deletion" },
            { "type": "non_fast_forward" },
            {
                "type": "pull_request",
                "parameters": {
                    "dismiss_stale_reviews_on_push": config.repository_defaults.dismiss_stale_reviews_on_push,
                    "require_code_owner_review": config.repository_defaults.require_code_owner_review,
                    "require_last_push_approval": false,
                    "required_approving_review_count": config.repository_defaults.required_approving_review_count,
                    "required_review_thread_resolution": config.repository_defaults.required_review_thread_resolution
                }
            },
            {
                "type": "required_status_checks",
                "parameters": {
                    "do_not_enforce_on_create": false,
                    "strict_required_status_checks_policy": true,
                    "required_status_checks": config
                        .repository_defaults
                        .required_status_checks
                        .iter()
                        .map(|context| json!({ "context": context }))
                        .collect::<Vec<_>>()
                }
            },
            {
                "type": "commit_message_pattern",
                "parameters": {
                    "name": "Conventional squash commit",
                    "negate": false,
                    "operator": "regex",
                    "pattern": commit_message_pattern(&config.repository_defaults.pr_title_pattern)
                }
            }
        ]
    })
}

fn branch_ruleset_payload(config: &GovernanceConfig) -> Value {
    json!({
        "name": "branch-naming",
        "target": "branch",
        "enforcement": "active",
        "conditions": {
            "ref_name": {
                "include": ["~ALL"],
                "exclude": []
            }
        },
        "rules": [
            {
                "type": "branch_name_pattern",
                "parameters": {
                    "name": "Short Origin branch naming",
                    "negate": false,
                    "operator": "regex",
                    "pattern": ruleset_branch_pattern(&config.repository_defaults.branch_name_pattern)
                }
            }
        ]
    })
}

fn load_pr_event(path: &Path) -> Result<PullRequestEvent, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read event file `{}`: {error}", path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse event file `{}`: {error}", path.display()))?;

    let title = value
        .pointer("/pull_request/title")
        .and_then(Value::as_str)
        .ok_or_else(|| "event payload is missing pull_request.title".to_owned())?
        .to_owned();
    let body = value
        .pointer("/pull_request/body")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let branch = value
        .pointer("/pull_request/head/ref")
        .and_then(Value::as_str)
        .ok_or_else(|| "event payload is missing pull_request.head.ref".to_owned())?
        .to_owned();
    let repository = value
        .pointer("/repository/full_name")
        .and_then(Value::as_str)
        .ok_or_else(|| "event payload is missing repository.full_name".to_owned())?
        .to_owned();

    Ok(PullRequestEvent {
        title,
        body,
        branch,
        repository,
    })
}

#[derive(Debug)]
struct PullRequestEvent {
    title: String,
    body: String,
    branch: String,
    repository: String,
}

fn validate_pr_event(config: &GovernanceConfig, event: &PullRequestEvent) -> Result<(), String> {
    let branch_regex = Regex::new(&config.repository_defaults.branch_name_pattern)
        .map_err(|error| format!("invalid branch_name_pattern regex in config: {error}"))?;
    let title_regex = Regex::new(&config.repository_defaults.pr_title_pattern)
        .map_err(|error| format!("invalid pr_title_pattern regex in config: {error}"))?;
    let same_repo_issue_regex = Regex::new(&format!(
        "(?m)(#[0-9]+\\b|https://github\\.com/{}/issues/[0-9]+\\b)",
        regex::escape(&event.repository)
    ))
    .map_err(|error| format!("failed to build same-repo issue regex: {error}"))?;

    let mut failures = Vec::new();
    if !branch_regex.is_match(&event.branch) {
        failures.push(format!(
            "branch `{}` does not match `{}`",
            event.branch, config.repository_defaults.branch_name_pattern
        ));
    }
    if !title_regex.is_match(&event.title) {
        failures.push(format!(
            "PR title `{}` does not match `{}`",
            event.title, config.repository_defaults.pr_title_pattern
        ));
    }
    if !same_repo_issue_regex.is_match(&event.body) {
        failures.push(
            "PR body must reference a same-repository issue using `#123` or a full issue URL"
                .to_owned(),
        );
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn default_repository(config: &GovernanceConfig) -> String {
    if let Some(repository) = config.repositories.first() {
        repository_full_name(config, repository)
    } else {
        format!("{}/short-origin", config.organization.login)
    }
}

fn governance_repository_full_name(config: &GovernanceConfig) -> String {
    format!(
        "{}/{}",
        config.organization.login, config.organization.governance_repository
    )
}

fn repository_full_name(config: &GovernanceConfig, repository: &RepositoryConfig) -> String {
    format!("{}/{}", config.organization.login, repository.name)
}

fn split_repository(repository: &str) -> Result<(&str, &str), String> {
    repository
        .split_once('/')
        .ok_or_else(|| format!("repository `{repository}` must be in OWNER/REPO format"))
}

fn commit_message_pattern(pr_title_pattern: &str) -> String {
    let trimmed = pr_title_pattern
        .strip_suffix('$')
        .unwrap_or(pr_title_pattern);
    format!("{trimmed}\\n?$")
}

fn ruleset_branch_pattern(branch_pattern: &str) -> String {
    let without_prefix = branch_pattern.strip_prefix('^').unwrap_or(branch_pattern);
    let trimmed = without_prefix.strip_suffix('$').unwrap_or(without_prefix);
    format!("^(?:main|{trimmed})$")
}

fn percent_encode(input: &str) -> String {
    input
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{:02X}", byte)
            }
        })
        .collect::<String>()
}

fn ensure_gh_is_ready() -> Result<(), String> {
    run_gh(&["auth".to_owned(), "status".to_owned()]).map(|_| ())
}

fn gh_repo_exists(repository: &str) -> Result<bool, String> {
    match run_gh(&[
        "repo".to_owned(),
        "view".to_owned(),
        repository.to_owned(),
        "--json".to_owned(),
        "name".to_owned(),
    ]) {
        Ok(_) => Ok(true),
        Err(message) if message.contains("Could not resolve to a Repository") => Ok(false),
        Err(message) if message.contains("HTTP 404") => Ok(false),
        Err(message) => Err(message),
    }
}

fn find_project_number(config: &GovernanceConfig, title: &str) -> Result<Option<u64>, String> {
    let output = run_gh(&[
        "project".to_owned(),
        "list".to_owned(),
        "--owner".to_owned(),
        config.organization.login.clone(),
        "--format".to_owned(),
        "json".to_owned(),
    ])?;
    let projects: Vec<Value> = serde_json::from_str(&output)
        .map_err(|error| format!("failed to parse project list output: {error}"))?;
    Ok(projects.iter().find_map(|project| {
        let project_title = project.get("title").and_then(Value::as_str)?;
        if project_title != title {
            return None;
        }
        project.get("number").and_then(Value::as_u64)
    }))
}

fn gh_api_json(method: &str, path: &str, body: Option<Value>) -> Result<Value, String> {
    let mut args = vec![
        "api".to_owned(),
        "-H".to_owned(),
        GH_ACCEPT_HEADER.to_owned(),
        "-H".to_owned(),
        GH_API_VERSION_HEADER.to_owned(),
        "--method".to_owned(),
        method.to_owned(),
        path.to_owned(),
    ];

    let payload = body
        .map(|body| serde_json::to_string(&body))
        .transpose()
        .map_err(|error| format!("failed to serialize GitHub API payload for `{path}`: {error}"))?;

    if payload.is_some() {
        args.push("--input".to_owned());
        args.push("-".to_owned());
    }

    let output = run_gh_with_input(&args, payload.as_deref())?;
    if output.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&output)
        .map_err(|error| format!("failed to parse GitHub API output for `{path}`: {error}"))
}

fn run_gh(args: &[String]) -> Result<String, String> {
    run_gh_with_input(args, None)
}

fn run_gh_with_input(args: &[String], input: Option<&str>) -> Result<String, String> {
    let mut command = Command::new("gh");
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start `gh {}`: {error}", args.join(" ")))?;

    if let Some(input) = input {
        let Some(mut stdin) = child.stdin.take() else {
            return Err(format!(
                "`gh {}` did not expose stdin for JSON payload",
                args.join(" ")
            ));
        };
        stdin.write_all(input.as_bytes()).map_err(|error| {
            format!(
                "failed to write JSON payload to `gh {}`: {error}",
                args.join(" ")
            )
        })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for `gh {}`: {error}", args.join(" ")))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Err(format!("`gh {}` failed: {detail}", args.join(" ")))
    }
}

fn help() -> String {
    "usage: cargo xtask github <sync|validate-pr> ...".to_owned()
}

#[cfg(test)]
mod tests {
    use super::{
        branch_ruleset_payload, load_config, main_ruleset_payload, validate_pr_event,
        PullRequestEvent,
    };
    use std::path::PathBuf;

    fn config_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask crate should be nested under the workspace root")
            .join(".github/governance.toml")
    }

    #[test]
    fn governance_config_parses() {
        let config = load_config(&config_path()).expect("governance config should parse");
        assert_eq!(config.organization.login, "shortorigin");
        assert_eq!(config.project.title, "Engineering Flow");
        assert!(config.repository_defaults.allow_auto_merge);
        assert!(config.repository_defaults.require_code_owner_review);
        assert_eq!(
            config.repository_defaults.required_status_checks,
            vec![
                "Governance / validate",
                "CI / pr-gate",
                "Security / security-gate",
            ]
        );
        assert!(config
            .labels
            .iter()
            .any(|label| label.name == "type:feature"));
    }

    #[test]
    fn validate_pr_accepts_compliant_event() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let event = PullRequestEvent {
            title: "feat(db): add provider".to_owned(),
            body: "## Linked Issue\n#142".to_owned(),
            branch: "feature/142-surrealdb-provider".to_owned(),
            repository: "shortorigin/short-origin".to_owned(),
        };

        validate_pr_event(&config, &event)
            .expect("valid PR data should pass governance validation");
    }

    #[test]
    fn validate_pr_rejects_missing_issue_reference() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let event = PullRequestEvent {
            title: "feat(db): add provider".to_owned(),
            body: "## Linked Issue\nTBD".to_owned(),
            branch: "feature/142-surrealdb-provider".to_owned(),
            repository: "shortorigin/short-origin".to_owned(),
        };

        let error = validate_pr_event(&config, &event)
            .expect_err("missing issue reference should fail governance validation");
        assert!(error.contains("same-repository issue"));
    }

    #[test]
    fn ruleset_payloads_include_expected_rules() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let main_ruleset = main_ruleset_payload(&config);
        let branch_ruleset = branch_ruleset_payload(&config);

        let main_rules = main_ruleset["rules"]
            .as_array()
            .expect("main ruleset should expose a rules array");
        let branch_rules = branch_ruleset["rules"]
            .as_array()
            .expect("branch ruleset should expose a rules array");

        assert!(main_rules.iter().any(|rule| rule["type"] == "pull_request"));
        assert!(main_rules
            .iter()
            .any(|rule| rule["type"] == "required_status_checks"));
        assert!(main_rules.iter().any(|rule| {
            rule["type"] == "pull_request"
                && rule["parameters"]["require_code_owner_review"] == serde_json::Value::Bool(true)
        }));
        assert!(branch_rules
            .iter()
            .any(|rule| rule["type"] == "branch_name_pattern"));
    }
}
