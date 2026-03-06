//! Shared models for the DX console.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Supported DX domains surfaced by `cargo xtask doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DxDomain {
    /// Core Rust workspace validation and build flows.
    Core,
    /// Browser/Tauri UI flows.
    Ui,
    /// Documentation and mdBook flows.
    Docs,
    /// Infrastructure verification flows.
    Infra,
    /// Security-specific tooling flows.
    Security,
}

impl DxDomain {
    /// Returns the machine-readable label for this domain.
    pub fn label(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Ui => "ui",
            Self::Docs => "docs",
            Self::Infra => "infra",
            Self::Security => "security",
        }
    }

    /// Returns the tracked domains shown in the setup journey.
    pub fn tracked() -> [Self; 5] {
        [
            Self::Core,
            Self::Ui,
            Self::Docs,
            Self::Infra,
            Self::Security,
        ]
    }
}

/// Repository-local registered task metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaskInfo {
    /// Stable task identifier.
    pub id: String,
    /// Human-readable summary.
    pub description: String,
    /// Task domains.
    pub domains: Vec<String>,
    /// Required tools.
    pub prerequisites: Vec<String>,
    /// Task dependencies.
    pub dependencies: Vec<String>,
    /// Whether CI currently includes the task.
    pub ci_included: bool,
    /// Whether the task is listed in normal discovery output.
    pub listed: bool,
}

/// Task catalog snapshot exposed by `xtask`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TaskCatalogSnapshot {
    /// Registered tasks.
    pub tasks: Vec<TaskInfo>,
}

/// Tool health state reported by `cargo xtask doctor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    /// Tool is available.
    #[default]
    Ok,
    /// Tool is optional and unavailable.
    Warn,
    /// Tool is required and unavailable.
    Missing,
}

impl CheckStatus {
    /// Returns `true` when this status blocks the journey.
    pub fn blocks(self) -> bool {
        matches!(self, Self::Missing)
    }
}

/// One tool entry in the doctor output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DoctorEntry {
    /// Tool label.
    pub tool: String,
    /// Tasks that require this tool.
    pub required_by: Vec<String>,
    /// Whether this tool is optional for the selected domain.
    pub optional: bool,
    /// Current availability.
    pub status: CheckStatus,
    /// Remediation guidance.
    pub guidance: String,
}

/// Structured doctor snapshot for a single domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorSnapshot {
    /// Selected domain.
    pub domain: DxDomain,
    /// Whether required tools are missing.
    pub missing_required: bool,
    /// Tool entries.
    pub entries: Vec<DoctorEntry>,
    /// Additional notes.
    pub notes: Vec<String>,
}

impl DoctorSnapshot {
    /// Returns the number of missing required tools.
    pub fn missing_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.status, CheckStatus::Missing))
            .count()
    }
}

/// Local repository status and branch context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RepoSnapshot {
    /// Current local branch.
    pub branch: String,
    /// Upstream branch if one is configured.
    pub upstream_branch: Option<String>,
    /// Current HEAD commit SHA.
    pub head_sha: Option<String>,
    /// Number of staged files.
    pub staged_files: usize,
    /// Number of unstaged files.
    pub unstaged_files: usize,
    /// Number of untracked files.
    pub untracked_files: usize,
    /// Number of commits ahead of upstream.
    pub ahead: u32,
    /// Number of commits behind upstream.
    pub behind: u32,
    /// Changed paths across staged, unstaged, and untracked files.
    pub changed_paths: Vec<String>,
    /// Inferred issue id from the branch naming convention.
    pub inferred_issue_id: Option<u64>,
}

impl RepoSnapshot {
    /// Returns the total number of local file changes.
    pub fn dirty_files(&self) -> usize {
        self.staged_files + self.unstaged_files + self.untracked_files
    }

    /// Returns the best branch name to use for GitHub PR lookup.
    pub fn pr_head_branch(&self) -> String {
        let candidate = self
            .upstream_branch
            .as_deref()
            .unwrap_or(&self.branch)
            .trim_start_matches("origin/");
        candidate.to_string()
    }
}

/// GitHub auth posture for live data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GithubAuthStatus {
    /// `gh` is authenticated.
    Authenticated,
    /// `gh` is installed but not authenticated or unavailable.
    Unavailable,
}

/// Linked issue state for the active branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IssueSummary {
    /// Issue number.
    pub number: u64,
    /// Title.
    pub title: String,
    /// State such as `OPEN`.
    pub state: String,
    /// Label names.
    pub labels: Vec<String>,
    /// Milestone title if present.
    pub milestone: Option<String>,
    /// Project status field value if present.
    pub project_status: Option<String>,
    /// Canonical web URL.
    pub url: String,
}

/// Status check rollup entry for the active PR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatusCheckSummary {
    /// Check name.
    pub name: String,
    /// Check state.
    pub status: String,
    /// Final conclusion when present.
    pub conclusion: Option<String>,
}

/// Pull request summary for the active branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PullRequestSummary {
    /// Pull request number.
    pub number: u64,
    /// Title.
    pub title: String,
    /// State such as `OPEN`.
    pub state: String,
    /// Review decision if available.
    pub review_decision: Option<String>,
    /// Head branch.
    pub head_ref_name: String,
    /// Base branch.
    pub base_ref_name: String,
    /// Status check rollup.
    pub status_checks: Vec<StatusCheckSummary>,
    /// Canonical web URL.
    pub url: String,
}

/// Combined work state for issue/PR/project context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkSnapshot {
    /// GitHub auth posture.
    pub auth_status: GithubAuthStatus,
    /// Repository identity if GitHub lookup succeeds.
    pub repository: Option<String>,
    /// Linked issue for the active branch.
    pub issue: Option<IssueSummary>,
    /// Open PR for the active branch.
    pub pull_request: Option<PullRequestSummary>,
    /// Non-fatal warnings encountered during lookup.
    pub warnings: Vec<String>,
}

impl Default for WorkSnapshot {
    fn default() -> Self {
        Self {
            auth_status: GithubAuthStatus::Unavailable,
            repository: None,
            issue: None,
            pull_request: None,
            warnings: Vec::new(),
        }
    }
}

/// One registered GitHub workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowSummary {
    /// Workflow id.
    pub id: u64,
    /// Workflow name.
    pub name: String,
    /// Current state.
    pub state: String,
}

/// One recent workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowRunSummary {
    /// Run id.
    pub id: u64,
    /// Workflow display name.
    pub workflow_name: String,
    /// Display title.
    pub display_title: String,
    /// Run status.
    pub status: String,
    /// Conclusion if available.
    pub conclusion: Option<String>,
    /// Head branch.
    pub head_branch: String,
    /// Trigger event.
    pub event: String,
    /// Creation time as ISO-8601 text.
    pub created_at: String,
    /// Run URL if available.
    pub url: Option<String>,
}

/// Workflow registry and recent run state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowRunSnapshot {
    /// Registered workflows.
    pub workflows: Vec<WorkflowSummary>,
    /// Recent runs.
    pub runs: Vec<WorkflowRunSummary>,
}

impl WorkflowRunSnapshot {
    /// Returns the first failed or unstable run when present.
    pub fn first_failure(&self) -> Option<&WorkflowRunSummary> {
        self.runs.iter().find(|run| {
            matches!(
                run.conclusion.as_deref(),
                Some("failure" | "cancelled" | "timed_out" | "startup_failure")
            )
        })
    }

    /// Returns whether the named workflow exists.
    pub fn has_workflow(&self, workflow_name: &str) -> bool {
        self.workflows
            .iter()
            .any(|workflow| workflow.name == workflow_name)
    }
}

/// Risk classification for a user-triggered action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ActionRisk {
    /// Safe read-only or local validation.
    #[default]
    Low,
    /// State-changing repo or GitHub action.
    Medium,
    /// High-impact release or promotion action.
    High,
}

/// Confirmation policy required before execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ConfirmationRequirement {
    /// No confirmation required.
    #[default]
    None,
    /// Simple yes/no confirmation.
    Confirm,
    /// Exact typed confirmation is required.
    TypedMatch {
        /// Required text.
        expected: String,
    },
}

/// Shell command invocation used by provider-backed actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ShellCommand {
    /// Program to execute.
    pub program: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Working directory.
    pub cwd: Option<String>,
    /// Additional environment variables.
    pub env: Vec<(String, String)>,
}

impl ShellCommand {
    /// Renders a human-readable preview string.
    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

/// Executable action surfaced by the DX console.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionPlan {
    /// Stable action id.
    pub id: String,
    /// Short action title.
    pub title: String,
    /// User-facing summary.
    pub description: String,
    /// Exact command preview.
    pub command_preview: String,
    /// Executable command.
    pub command: ShellCommand,
    /// Risk posture.
    pub risk: ActionRisk,
    /// Confirmation requirement.
    pub confirmation: ConfirmationRequirement,
    /// Whether to refresh snapshots after completion.
    pub refresh_after: bool,
}

/// Result of a completed action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ActionOutcome {
    /// Whether the action completed successfully.
    pub success: bool,
    /// Exit code when command-backed.
    pub exit_code: Option<i32>,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// One-line summary for activity logs.
    pub summary: String,
}

/// Request payload for the release-candidate workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReleaseCandidateRequest {
    /// Target prerelease tag.
    pub version: String,
    /// Full 40-character commit SHA to promote.
    pub target_sha: String,
}

/// Request payload for the promote-release workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PromoteReleaseRequest {
    /// Existing prerelease tag.
    pub rc_tag: String,
    /// Final release tag.
    pub release_tag: String,
}

/// Preflight state for one release action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReleasePreflight {
    /// Workflow being prepared.
    pub workflow_name: String,
    /// Whether the action is ready to dispatch.
    pub ready: bool,
    /// Blocking validation errors.
    pub blockers: Vec<String>,
    /// Non-blocking notes.
    pub warnings: Vec<String>,
    /// Dispatch action when ready.
    pub action: Option<ActionPlan>,
}

/// Mutable release wizard state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReleaseWizardState {
    /// Release-candidate form values.
    pub rc_request: ReleaseCandidateRequest,
    /// Promote-release form values.
    pub promote_request: PromoteReleaseRequest,
    /// Release-candidate preflight result.
    pub rc_preflight: ReleasePreflight,
    /// Promote-release preflight result.
    pub promote_preflight: ReleasePreflight,
    /// Last dispatch result if one exists.
    pub last_dispatch: Option<ActionOutcome>,
}

/// High-level journey categories shown in the DX console.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum JourneyKind {
    /// Environment setup and prerequisites.
    #[default]
    Setup,
    /// Day-to-day issue, branch, and PR progress.
    Develop,
    /// Validation and CI readiness.
    Verify,
    /// Release and promotion readiness.
    Release,
}

impl JourneyKind {
    /// Returns a display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Setup => "Setup",
            Self::Develop => "Develop",
            Self::Verify => "Verify",
            Self::Release => "Release",
        }
    }
}

/// Step state inside a journey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum JourneyStepStatus {
    /// Hidden until predecessor state is resolved.
    Locked,
    /// Ready to be acted on.
    #[default]
    Ready,
    /// Actively in flight.
    Running,
    /// Blocked by a missing requirement.
    Blocked,
    /// Completed.
    Complete,
}

/// One journey step surfaced in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct JourneyStep {
    /// Short title.
    pub title: String,
    /// Progress state.
    pub status: JourneyStepStatus,
    /// Supporting detail.
    pub detail: String,
    /// Suggested action when relevant.
    pub action: Option<ActionPlan>,
}

/// One guided developer journey.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct JourneyState {
    /// Journey kind.
    pub kind: JourneyKind,
    /// Aggregate completion score from 0 to 100.
    pub score: u8,
    /// Steps within the journey.
    pub steps: Vec<JourneyStep>,
    /// Preferred next action for this journey.
    pub next_action: Option<ActionPlan>,
}

/// Aggregated data model consumed by the Ratatui frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DashboardState {
    /// Repository-local state.
    pub repo: RepoSnapshot,
    /// Doctor snapshots across tracked domains.
    pub doctor: Vec<DoctorSnapshot>,
    /// Registered task catalog.
    pub tasks: TaskCatalogSnapshot,
    /// GitHub work state.
    pub work: WorkSnapshot,
    /// Workflow registry and runs.
    pub workflows: WorkflowRunSnapshot,
    /// Guided journeys.
    pub journeys: Vec<JourneyState>,
    /// Recommended task subset for the current changes.
    pub recommended_tasks: Vec<TaskInfo>,
    /// Overall next-best action.
    pub recommended_action: Option<ActionPlan>,
    /// Release console state.
    pub release: ReleaseWizardState,
    /// Non-fatal refresh alerts.
    pub alerts: Vec<String>,
}

/// Infers the current issue id from a branch naming convention.
pub fn infer_issue_id_from_branch(branch: &str) -> Option<u64> {
    let pattern = Regex::new(r"^(?:codex/)?(?:feature|fix|infra|docs|refactor|research)/([0-9]+)-")
        .expect("issue inference regex");
    pattern
        .captures(branch.trim())
        .and_then(|captures| captures.get(1))
        .and_then(|capture| capture.as_str().parse::<u64>().ok())
}

/// Validates a release-candidate tag in the form `vX.Y.Z-rc.N`.
pub fn is_valid_rc_tag(value: &str) -> bool {
    Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+$")
        .expect("rc tag regex")
        .is_match(value)
}

/// Validates a final release tag in the form `vX.Y.Z`.
pub fn is_valid_release_tag(value: &str) -> bool {
    Regex::new(r"^v[0-9]+\.[0-9]+\.[0-9]+$")
        .expect("release tag regex")
        .is_match(value)
}

/// Validates a full 40-character lowercase commit SHA.
pub fn is_valid_commit_sha(value: &str) -> bool {
    Regex::new(r"^[0-9a-f]{40}$")
        .expect("sha regex")
        .is_match(value)
}

/// Returns whether the typed confirmation matches the expected input exactly.
pub fn matches_confirmation(requirement: &ConfirmationRequirement, typed: &str) -> bool {
    match requirement {
        ConfirmationRequirement::None | ConfirmationRequirement::Confirm => true,
        ConfirmationRequirement::TypedMatch { expected } => typed == expected,
    }
}

/// Maps changed paths to the most relevant tracked task ids.
pub fn recommended_task_ids(paths: &[String]) -> Vec<&'static str> {
    let mut ids = Vec::new();
    let has_prefix = |prefixes: &[&str]| {
        paths
            .iter()
            .any(|path| prefixes.iter().any(|prefix| path.starts_with(prefix)))
    };

    if has_prefix(&["ui/"]) {
        ids.push("ui-verify");
    }
    if has_prefix(&["infrastructure/pulumi/"]) {
        ids.push("infra-verify");
    }
    if has_prefix(&[
        "docs/security-rust/",
        "testing/security-labs/",
        "shared/secure-patterns/",
        "shared/exploit-mitigation/",
        "shared/runtime-security/",
        "shared/security-instrumentation/",
    ]) {
        ids.push("docs-security-book-test");
    }
    if has_prefix(&[
        "platform/wasmcloud/",
        "services/",
        "workflows/",
        "schemas/",
        "shared/surrealdb-access/",
    ]) {
        ids.push("components-build");
    }
    if !paths.is_empty() {
        ids.push("verify-full");
    }

    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod tests {
    use super::{
        infer_issue_id_from_branch, is_valid_commit_sha, is_valid_rc_tag, is_valid_release_tag,
        matches_confirmation, recommended_task_ids, ConfirmationRequirement,
    };

    #[test]
    fn infers_issue_id_from_standard_branch() {
        assert_eq!(
            infer_issue_id_from_branch("feature/13-runtime-security-delivery-foundations"),
            Some(13)
        );
        assert_eq!(
            infer_issue_id_from_branch("codex/feature/13-runtime-security-delivery-foundations"),
            Some(13)
        );
        assert_eq!(infer_issue_id_from_branch("main"), None);
    }

    #[test]
    fn validates_release_inputs() {
        assert!(is_valid_rc_tag("v1.2.3-rc.4"));
        assert!(!is_valid_rc_tag("1.2.3-rc.4"));
        assert!(is_valid_release_tag("v1.2.3"));
        assert!(!is_valid_release_tag("v1.2"));
        assert!(is_valid_commit_sha(
            "0123456789abcdef0123456789abcdef01234567"
        ));
        assert!(!is_valid_commit_sha("abc123"));
    }

    #[test]
    fn typed_confirmation_must_match_exactly() {
        let requirement = ConfirmationRequirement::TypedMatch {
            expected: "v1.2.3".to_string(),
        };
        assert!(matches_confirmation(&requirement, "v1.2.3"));
        assert!(!matches_confirmation(&requirement, " v1.2.3 "));
    }

    #[test]
    fn recommends_tasks_from_changed_paths() {
        let ids = recommended_task_ids(&[
            "ui/crates/site/src/web_app.rs".to_string(),
            "infrastructure/pulumi/live/src/index.ts".to_string(),
        ]);
        assert!(ids.contains(&"ui-verify"));
        assert!(ids.contains(&"infra-verify"));
        assert!(ids.contains(&"verify-full"));
    }
}
