//! Shell-backed provider implementations for the DX console.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use crate::model::{
    infer_issue_id_from_branch, is_valid_commit_sha, is_valid_rc_tag, is_valid_release_tag,
    ActionOutcome, ActionPlan, CheckStatus, DoctorEntry, DoctorSnapshot, DxDomain,
    GithubAuthStatus, IssueSummary, PullRequestSummary, ReleaseCandidateRequest, ReleasePreflight,
    RepoSnapshot, ShellCommand, StatusCheckSummary, TaskCatalogSnapshot, TaskInfo, WorkSnapshot,
    WorkflowRunSnapshot, WorkflowRunSummary, WorkflowSummary,
};
use crate::providers::{
    ActionExecutor, DxProvider, GitProvider, GithubProvider, ProviderError, WorkflowProvider,
};

/// Concrete provider set using local `git`, `cargo xtask`, and `gh`.
#[derive(Clone)]
pub struct ShellContext {
    workspace_root: PathBuf,
}

impl ShellContext {
    /// Creates a shell-backed context rooted at `workspace_root`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    fn run_command(&self, program: &str, args: &[&str]) -> Result<String, ProviderError> {
        let output = Command::new(program)
            .args(args)
            .current_dir(&self.workspace_root)
            .output()
            .map_err(|error| ProviderError::CommandFailed {
                command: render_command(program, args),
                message: error.to_string(),
            })?;

        if !output.status.success() {
            return Err(ProviderError::CommandFailed {
                command: render_command(program, args),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn run_json<T: for<'de> Deserialize<'de>>(
        &self,
        program: &str,
        args: &[&str],
    ) -> Result<T, ProviderError> {
        let stdout = self.run_command(program, args)?;
        serde_json::from_str(&stdout).map_err(|error| {
            ProviderError::Parse(format!(
                "failed to parse `{}` output: {error}",
                render_command(program, args)
            ))
        })
    }

    fn repo_name(&self) -> Result<String, ProviderError> {
        #[derive(Deserialize)]
        struct RepoView {
            #[serde(rename = "nameWithOwner")]
            name_with_owner: String,
        }

        let repo: RepoView = self.run_json("gh", &["repo", "view", "--json", "nameWithOwner"])?;
        Ok(repo.name_with_owner)
    }

    fn build_dispatch_action(
        &self,
        id: &str,
        title: &str,
        description: &str,
        expected: String,
        args: Vec<String>,
    ) -> ActionPlan {
        ActionPlan {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            command_preview: format!("gh {}", args.join(" ")),
            command: ShellCommand {
                program: "gh".to_string(),
                args,
                cwd: Some(self.workspace_root.display().to_string()),
                env: Vec::new(),
            },
            risk: crate::model::ActionRisk::High,
            confirmation: crate::model::ConfirmationRequirement::TypedMatch { expected },
            refresh_after: true,
        }
    }
}

impl GitProvider for ShellContext {
    fn repo_snapshot(&self) -> Result<RepoSnapshot, ProviderError> {
        let status = self.run_command("git", &["status", "--porcelain=2", "--branch"])?;
        let mut repo = RepoSnapshot {
            changed_paths: collect_changed_paths(self)?,
            ..RepoSnapshot::default()
        };

        for line in status.lines() {
            if let Some(branch) = line.strip_prefix("# branch.head ") {
                repo.branch = branch.to_string();
                repo.inferred_issue_id = infer_issue_id_from_branch(branch);
            } else if let Some(upstream) = line.strip_prefix("# branch.upstream ") {
                repo.upstream_branch = Some(upstream.to_string());
            } else if let Some(divergence) = line.strip_prefix("# branch.ab ") {
                let mut parts = divergence.split_whitespace();
                repo.ahead = parts
                    .next()
                    .and_then(|value| value.strip_prefix('+'))
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or_default();
                repo.behind = parts
                    .next()
                    .and_then(|value| value.strip_prefix('-'))
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or_default();
            } else if let Some(rest) = line.strip_prefix("1 ") {
                let fields = rest.split_whitespace().collect::<Vec<_>>();
                if let Some(code) = fields.first() {
                    increment_change_counts(code, &mut repo);
                }
            } else if let Some(rest) = line.strip_prefix("2 ") {
                let fields = rest.split_whitespace().collect::<Vec<_>>();
                if let Some(code) = fields.first() {
                    increment_change_counts(code, &mut repo);
                }
            } else if line.starts_with("? ") {
                repo.untracked_files += 1;
            }
        }

        repo.head_sha = self
            .run_command("git", &["rev-parse", "HEAD"])
            .ok()
            .filter(|value| !value.is_empty());

        Ok(repo)
    }
}

impl DxProvider for ShellContext {
    fn doctor(&self, domain: DxDomain) -> Result<DoctorSnapshot, ProviderError> {
        #[derive(Deserialize)]
        struct RawDoctorEntry {
            tool: String,
            required_by: Vec<String>,
            optional: bool,
            status: String,
            guidance: String,
        }

        #[derive(Deserialize)]
        struct RawDoctorSnapshot {
            #[serde(rename = "domain")]
            _domain: String,
            missing_required: bool,
            entries: Vec<RawDoctorEntry>,
            notes: Vec<String>,
        }

        let raw: RawDoctorSnapshot = self.run_json(
            "cargo",
            &[
                "xtask",
                "doctor",
                "--domain",
                domain.label(),
                "--format",
                "json",
            ],
        )?;

        Ok(DoctorSnapshot {
            domain,
            missing_required: raw.missing_required,
            entries: raw
                .entries
                .into_iter()
                .map(|entry| DoctorEntry {
                    tool: entry.tool,
                    required_by: entry.required_by,
                    optional: entry.optional,
                    status: match entry.status.as_str() {
                        "ok" => CheckStatus::Ok,
                        "warn" => CheckStatus::Warn,
                        _ => CheckStatus::Missing,
                    },
                    guidance: entry.guidance,
                })
                .collect(),
            notes: raw.notes,
        })
    }

    fn task_catalog(&self) -> Result<TaskCatalogSnapshot, ProviderError> {
        #[derive(Deserialize)]
        struct RawTaskCatalog {
            tasks: Vec<TaskInfo>,
        }

        let raw: RawTaskCatalog =
            self.run_json("cargo", &["xtask", "tasks", "list", "--format", "json"])?;
        Ok(TaskCatalogSnapshot { tasks: raw.tasks })
    }
}

impl GithubProvider for ShellContext {
    fn work_snapshot(
        &self,
        issue_hint: Option<u64>,
        pr_head_branch: &str,
    ) -> Result<WorkSnapshot, ProviderError> {
        let repository = self.repo_name().ok();
        let auth_status = if self.run_command("gh", &["auth", "status"]).is_ok() {
            GithubAuthStatus::Authenticated
        } else {
            GithubAuthStatus::Unavailable
        };
        if auth_status == GithubAuthStatus::Unavailable {
            return Ok(WorkSnapshot {
                auth_status,
                repository,
                issue: None,
                pull_request: None,
                warnings: vec!["GitHub CLI authentication is unavailable.".to_string()],
            });
        }

        let issue = issue_hint
            .map(|issue_number| self.issue_summary(issue_number))
            .transpose()?
            .flatten();
        let pull_request = self.pull_request_summary(pr_head_branch)?;

        Ok(WorkSnapshot {
            auth_status,
            repository,
            issue,
            pull_request,
            warnings: Vec::new(),
        })
    }
}

impl WorkflowProvider for ShellContext {
    fn workflow_snapshot(&self) -> Result<WorkflowRunSnapshot, ProviderError> {
        #[derive(Deserialize)]
        struct RawWorkflow {
            id: u64,
            name: String,
            state: String,
        }
        #[derive(Deserialize)]
        struct RawRun {
            #[serde(rename = "databaseId")]
            id: u64,
            #[serde(rename = "workflowName")]
            workflow_name: String,
            #[serde(rename = "displayTitle")]
            display_title: String,
            status: String,
            conclusion: Option<String>,
            #[serde(rename = "headBranch")]
            head_branch: String,
            event: String,
            #[serde(rename = "createdAt")]
            created_at: String,
            url: Option<String>,
        }

        let workflows = self
            .run_json::<Vec<RawWorkflow>>("gh", &["workflow", "list", "--json", "id,name,state"])?
            .into_iter()
            .map(|workflow| WorkflowSummary {
                id: workflow.id,
                name: workflow.name,
                state: workflow.state,
            })
            .collect::<Vec<_>>();
        let runs = self
            .run_json::<Vec<RawRun>>(
                "gh",
                &[
                    "run",
                    "list",
                    "--limit",
                    "20",
                    "--json",
                    "databaseId,workflowName,displayTitle,status,conclusion,headBranch,event,createdAt,url",
                ],
            )?
            .into_iter()
            .map(|run| WorkflowRunSummary {
                id: run.id,
                workflow_name: run.workflow_name,
                display_title: run.display_title,
                status: run.status,
                conclusion: run.conclusion,
                head_branch: run.head_branch,
                event: run.event,
                created_at: run.created_at,
                url: run.url,
            })
            .collect::<Vec<_>>();

        Ok(WorkflowRunSnapshot { workflows, runs })
    }

    fn preflight_release_candidate(
        &self,
        request: &ReleaseCandidateRequest,
    ) -> Result<ReleasePreflight, ProviderError> {
        let snapshot = self.workflow_snapshot()?;
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();

        if !snapshot.has_workflow("Release Candidate") {
            blockers.push("GitHub workflow `Release Candidate` is unavailable.".to_string());
        }
        if !is_valid_rc_tag(&request.version) {
            blockers.push("Version must match vX.Y.Z-rc.N.".to_string());
        }
        if !is_valid_commit_sha(&request.target_sha) {
            blockers.push("Target SHA must be a full 40-character commit SHA.".to_string());
        } else if self
            .run_command(
                "gh",
                &[
                    "api",
                    &format!("repos/{}/commits/{}", self.repo_name()?, request.target_sha),
                ],
            )
            .is_err()
        {
            blockers.push("Target SHA was not found in the repository.".to_string());
        }
        if self.run_command("gh", &["auth", "status"]).is_err() {
            blockers.push("GitHub CLI authentication is unavailable.".to_string());
        }
        if request.version.is_empty() || request.target_sha.is_empty() {
            warnings.push("Complete both form fields to enable dispatch.".to_string());
        }

        let ready = blockers.is_empty();
        let action = ready.then(|| {
            self.build_dispatch_action(
                "dispatch-release-candidate",
                "Dispatch Release Candidate",
                "Start the guarded release-candidate workflow.",
                request.version.clone(),
                vec![
                    "workflow".to_string(),
                    "run".to_string(),
                    "Release Candidate".to_string(),
                    "-f".to_string(),
                    format!("version={}", request.version),
                    "-f".to_string(),
                    format!("target_sha={}", request.target_sha),
                ],
            )
        });

        Ok(ReleasePreflight {
            workflow_name: "Release Candidate".to_string(),
            ready,
            blockers,
            warnings,
            action,
        })
    }

    fn preflight_promote_release(
        &self,
        request: &crate::model::PromoteReleaseRequest,
    ) -> Result<ReleasePreflight, ProviderError> {
        let snapshot = self.workflow_snapshot()?;
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();

        if !snapshot.has_workflow("Promote Release") {
            blockers.push("GitHub workflow `Promote Release` is unavailable.".to_string());
        }
        if !is_valid_rc_tag(&request.rc_tag) {
            blockers.push("Release-candidate tag must match vX.Y.Z-rc.N.".to_string());
        }
        if !is_valid_release_tag(&request.release_tag) {
            blockers.push("Release tag must match vX.Y.Z.".to_string());
        }
        if self.run_command("gh", &["auth", "status"]).is_err() {
            blockers.push("GitHub CLI authentication is unavailable.".to_string());
        }
        if !request.rc_tag.is_empty()
            && self
                .run_command(
                    "gh",
                    &[
                        "release",
                        "view",
                        &request.rc_tag,
                        "--json",
                        "tagName,isPrerelease,url",
                    ],
                )
                .is_err()
        {
            blockers.push("The referenced release-candidate tag does not exist.".to_string());
        }
        if request.rc_tag.is_empty() || request.release_tag.is_empty() {
            warnings.push("Complete both form fields to enable dispatch.".to_string());
        }

        let ready = blockers.is_empty();
        let action = ready.then(|| {
            self.build_dispatch_action(
                "dispatch-promote-release",
                "Dispatch Promote Release",
                "Promote an approved release candidate into production.",
                request.release_tag.clone(),
                vec![
                    "workflow".to_string(),
                    "run".to_string(),
                    "Promote Release".to_string(),
                    "-f".to_string(),
                    format!("rc_tag={}", request.rc_tag),
                    "-f".to_string(),
                    format!("release_tag={}", request.release_tag),
                ],
            )
        });

        Ok(ReleasePreflight {
            workflow_name: "Promote Release".to_string(),
            ready,
            blockers,
            warnings,
            action,
        })
    }
}

impl ActionExecutor for ShellContext {
    fn execute(&self, action: &ActionPlan) -> Result<ActionOutcome, ProviderError> {
        let output = Command::new(&action.command.program)
            .args(action.command.args.iter())
            .current_dir(
                action
                    .command
                    .cwd
                    .as_deref()
                    .map_or(self.workspace_root.as_path(), Path::new),
            )
            .envs(action.command.env.iter().cloned())
            .output()
            .map_err(|error| ProviderError::CommandFailed {
                command: action.command.display(),
                message: error.to_string(),
            })?;

        Ok(ActionOutcome {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            summary: if output.status.success() {
                format!("Completed {}", action.title)
            } else {
                format!("Failed {}", action.title)
            },
        })
    }
}

impl ShellContext {
    fn issue_summary(&self, issue_number: u64) -> Result<Option<IssueSummary>, ProviderError> {
        #[derive(Deserialize)]
        struct RawLabel {
            name: String,
        }
        #[derive(Deserialize)]
        struct RawMilestone {
            title: String,
        }
        #[derive(Deserialize)]
        struct RawProjectStatus {
            name: String,
        }
        #[derive(Deserialize)]
        struct RawProjectItem {
            status: Option<RawProjectStatus>,
        }
        #[derive(Deserialize)]
        struct RawIssue {
            number: u64,
            title: String,
            state: String,
            labels: Vec<RawLabel>,
            milestone: Option<RawMilestone>,
            #[serde(rename = "projectItems")]
            project_items: Vec<RawProjectItem>,
            url: String,
        }

        match self.run_json::<RawIssue>(
            "gh",
            &[
                "issue",
                "view",
                &issue_number.to_string(),
                "--json",
                "number,title,state,labels,milestone,projectItems,url",
            ],
        ) {
            Ok(issue) => Ok(Some(IssueSummary {
                number: issue.number,
                title: issue.title,
                state: issue.state,
                labels: issue.labels.into_iter().map(|label| label.name).collect(),
                milestone: issue.milestone.map(|milestone| milestone.title),
                project_status: issue
                    .project_items
                    .into_iter()
                    .find_map(|item| item.status.map(|status| status.name)),
                url: issue.url,
            })),
            Err(ProviderError::CommandFailed { message, .. }) if message.contains("not found") => {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn pull_request_summary(
        &self,
        head_branch: &str,
    ) -> Result<Option<PullRequestSummary>, ProviderError> {
        #[derive(Deserialize)]
        struct RawPr {
            number: u64,
            title: String,
            state: String,
            #[serde(rename = "reviewDecision")]
            review_decision: Option<String>,
            #[serde(rename = "headRefName")]
            head_ref_name: String,
            #[serde(rename = "baseRefName")]
            base_ref_name: String,
            #[serde(rename = "statusCheckRollup")]
            status_check_rollup: Vec<serde_json::Value>,
            url: String,
        }

        let pulls = self.run_json::<Vec<RawPr>>(
            "gh",
            &[
                "pr",
                "list",
                "--head",
                head_branch,
                "--state",
                "open",
                "--json",
                "number,title,state,reviewDecision,headRefName,baseRefName,statusCheckRollup,url",
            ],
        )?;

        Ok(pulls.into_iter().next().map(|pull| PullRequestSummary {
            number: pull.number,
            title: pull.title,
            state: pull.state,
            review_decision: pull.review_decision,
            head_ref_name: pull.head_ref_name,
            base_ref_name: pull.base_ref_name,
            status_checks: parse_status_checks(&pull.status_check_rollup),
            url: pull.url,
        }))
    }
}

fn collect_changed_paths(shell: &ShellContext) -> Result<Vec<String>, ProviderError> {
    let mut paths = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only"],
        vec!["diff", "--name-only", "--cached"],
        vec!["ls-files", "--others", "--exclude-standard"],
    ] {
        let output = shell.run_command("git", &args)?;
        for path in output.lines().filter(|line| !line.trim().is_empty()) {
            paths.insert(path.to_string());
        }
    }
    Ok(paths.into_iter().collect())
}

fn increment_change_counts(code: &str, repo: &mut RepoSnapshot) {
    let staged = code.chars().next().unwrap_or('.');
    let unstaged = code.chars().nth(1).unwrap_or('.');
    if staged != '.' {
        repo.staged_files += 1;
    }
    if unstaged != '.' {
        repo.unstaged_files += 1;
    }
}

fn render_command(program: &str, args: &[&str]) -> String {
    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {}", args.join(" "))
    }
}

fn parse_status_checks(values: &[serde_json::Value]) -> Vec<StatusCheckSummary> {
    values
        .iter()
        .map(|value| StatusCheckSummary {
            name: value
                .get("name")
                .or_else(|| value.get("context"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            status: value
                .get("status")
                .or_else(|| value.get("state"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            conclusion: value
                .get("conclusion")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::model::{
        matches_confirmation, CheckStatus, ConfirmationRequirement, TaskCatalogSnapshot,
    };

    use super::parse_status_checks;

    #[test]
    fn parses_status_check_rollup_values() {
        let checks = parse_status_checks(&[
            serde_json::json!({"name":"CI / pr-gate","status":"COMPLETED","conclusion":"SUCCESS"}),
            serde_json::json!({"context":"Governance / validate","state":"IN_PROGRESS"}),
        ]);
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].name, "CI / pr-gate");
        assert_eq!(checks[1].status, "IN_PROGRESS");
    }

    #[test]
    fn typed_confirmation_requires_exact_release_tag() {
        let requirement = ConfirmationRequirement::TypedMatch {
            expected: "v1.0.0".to_string(),
        };
        assert!(matches_confirmation(&requirement, "v1.0.0"));
        assert!(!matches_confirmation(&requirement, "v1.0.0-rc.1"));
    }

    #[test]
    fn doctor_snapshot_deserializes_machine_readable_status() {
        let snapshot: crate::model::DoctorSnapshot = crate::model::DoctorSnapshot {
            domain: crate::model::DxDomain::Core,
            missing_required: true,
            entries: vec![crate::model::DoctorEntry {
                tool: "cargo-audit".to_string(),
                required_by: vec!["security-audit".to_string()],
                optional: false,
                status: CheckStatus::Missing,
                guidance: "Install cargo-audit".to_string(),
            }],
            notes: Vec::new(),
        };
        assert!(snapshot.missing_required);
        assert_eq!(snapshot.entries[0].status, CheckStatus::Missing);
    }

    #[test]
    fn task_catalog_defaults_to_empty() {
        let snapshot = TaskCatalogSnapshot::default();
        assert!(snapshot.tasks.is_empty());
    }
}
