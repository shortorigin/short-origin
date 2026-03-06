//! Provider traits used by the DX console service.

use thiserror::Error;

use crate::model::{
    ActionOutcome, ActionPlan, DoctorSnapshot, DxDomain, PromoteReleaseRequest,
    ReleaseCandidateRequest, ReleasePreflight, RepoSnapshot, TaskCatalogSnapshot, WorkSnapshot,
    WorkflowRunSnapshot,
};

/// Error emitted by provider or action execution layers.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProviderError {
    /// A command failed to execute or returned a non-zero status.
    #[error("{command}: {message}")]
    CommandFailed {
        /// Rendered command string.
        command: String,
        /// Human-readable failure detail.
        message: String,
    },
    /// JSON or text parsing failed.
    #[error("{0}")]
    Parse(String),
}

/// Reads Git repository state.
pub trait GitProvider {
    /// Returns the current repo snapshot.
    fn repo_snapshot(&self) -> Result<RepoSnapshot, ProviderError>;
}

/// Reads GitHub issue and PR state.
pub trait GithubProvider {
    /// Returns GitHub-backed work state for the active branch.
    fn work_snapshot(
        &self,
        issue_hint: Option<u64>,
        pr_head_branch: &str,
    ) -> Result<WorkSnapshot, ProviderError>;
}

/// Reads repository-local DX surfaces from `xtask`.
pub trait DxProvider {
    /// Returns the doctor snapshot for one domain.
    fn doctor(&self, domain: DxDomain) -> Result<DoctorSnapshot, ProviderError>;
    /// Returns the registered task catalog.
    fn task_catalog(&self) -> Result<TaskCatalogSnapshot, ProviderError>;
}

/// Reads GitHub workflow state and constructs release preflights.
pub trait WorkflowProvider {
    /// Returns the workflow registry and recent runs.
    fn workflow_snapshot(&self) -> Result<WorkflowRunSnapshot, ProviderError>;
    /// Builds release-candidate preflight state.
    fn preflight_release_candidate(
        &self,
        request: &ReleaseCandidateRequest,
    ) -> Result<ReleasePreflight, ProviderError>;
    /// Builds promote-release preflight state.
    fn preflight_promote_release(
        &self,
        request: &PromoteReleaseRequest,
    ) -> Result<ReleasePreflight, ProviderError>;
}

/// Executes an action plan.
pub trait ActionExecutor {
    /// Executes the action and returns the captured outcome.
    fn execute(&self, action: &ActionPlan) -> Result<ActionOutcome, ProviderError>;
}
