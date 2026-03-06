//! Headless orchestration service for the DX console.

use crate::model::{
    recommended_task_ids, ActionPlan, ActionRisk, ConfirmationRequirement, DashboardState,
    DoctorSnapshot, DxDomain, JourneyKind, JourneyState, JourneyStep, JourneyStepStatus,
    ReleaseCandidateRequest, ReleaseWizardState, RepoSnapshot, ShellCommand, TaskCatalogSnapshot,
    TaskInfo, WorkSnapshot, WorkflowRunSnapshot,
};
use crate::providers::{DxProvider, GitProvider, GithubProvider, WorkflowProvider};

/// Mutable input values for the release console.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReleaseInputs {
    /// Release-candidate tag.
    pub rc_tag: String,
    /// Release-candidate target SHA.
    pub target_sha: String,
    /// Existing prerelease tag to promote.
    pub promote_rc_tag: String,
    /// Final release tag.
    pub release_tag: String,
}

/// Aggregates provider-backed snapshots into one dashboard state.
pub struct DevConsoleService<G, H, D, W> {
    git: G,
    github: H,
    dx: D,
    workflows: W,
}

impl<G, H, D, W> DevConsoleService<G, H, D, W>
where
    G: GitProvider,
    H: GithubProvider,
    D: DxProvider,
    W: WorkflowProvider,
{
    /// Creates a new service from the provider set.
    pub fn new(git: G, github: H, dx: D, workflows: W) -> Self {
        Self {
            git,
            github,
            dx,
            workflows,
        }
    }

    /// Refreshes all snapshots and derives guided journeys.
    pub fn refresh(&self, release_inputs: ReleaseInputs) -> DashboardState {
        let mut alerts = Vec::new();

        let repo = match self.git.repo_snapshot() {
            Ok(repo) => repo,
            Err(error) => {
                alerts.push(format!("git snapshot unavailable: {error}"));
                RepoSnapshot::default()
            }
        };

        let doctor = DxDomain::tracked()
            .into_iter()
            .filter_map(|domain| match self.dx.doctor(domain) {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    alerts.push(format!(
                        "doctor {domain_label} unavailable: {error}",
                        domain_label = domain.label()
                    ));
                    None
                }
            })
            .collect::<Vec<_>>();

        let tasks = match self.dx.task_catalog() {
            Ok(tasks) => tasks,
            Err(error) => {
                alerts.push(format!("task catalog unavailable: {error}"));
                TaskCatalogSnapshot::default()
            }
        };

        let work = match self
            .github
            .work_snapshot(repo.inferred_issue_id, &repo.pr_head_branch())
        {
            Ok(snapshot) => snapshot,
            Err(error) => {
                alerts.push(format!("github work snapshot unavailable: {error}"));
                WorkSnapshot::default()
            }
        };

        let workflows = match self.workflows.workflow_snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                alerts.push(format!("workflow snapshot unavailable: {error}"));
                WorkflowRunSnapshot::default()
            }
        };

        let recommended_tasks = select_recommended_tasks(&repo.changed_paths, &tasks);
        let release = build_release_state(&self.workflows, &repo, release_inputs, &mut alerts);
        let journeys = build_journeys(
            &repo,
            &doctor,
            &recommended_tasks,
            &work,
            &workflows,
            &release,
        );
        let recommended_action = journeys
            .iter()
            .find_map(|journey| journey.next_action.clone());

        DashboardState {
            repo,
            doctor,
            tasks,
            work,
            workflows,
            journeys,
            recommended_tasks,
            recommended_action,
            release,
            alerts,
        }
    }
}

fn build_release_state<W: WorkflowProvider>(
    workflows: &W,
    repo: &RepoSnapshot,
    inputs: ReleaseInputs,
    alerts: &mut Vec<String>,
) -> ReleaseWizardState {
    let rc_request = ReleaseCandidateRequest {
        version: inputs.rc_tag,
        target_sha: if inputs.target_sha.is_empty() {
            repo.head_sha.clone().unwrap_or_default()
        } else {
            inputs.target_sha
        },
    };
    let promote_request = crate::model::PromoteReleaseRequest {
        rc_tag: inputs.promote_rc_tag,
        release_tag: inputs.release_tag,
    };

    let rc_preflight = workflows
        .preflight_release_candidate(&rc_request)
        .unwrap_or_else(|error| {
            alerts.push(format!("release-candidate preflight unavailable: {error}"));
            crate::model::ReleasePreflight {
                workflow_name: "Release Candidate".to_string(),
                ready: false,
                blockers: vec!["Release Candidate preflight unavailable.".to_string()],
                warnings: Vec::new(),
                action: None,
            }
        });

    let promote_preflight = workflows
        .preflight_promote_release(&promote_request)
        .unwrap_or_else(|error| {
            alerts.push(format!("promote-release preflight unavailable: {error}"));
            crate::model::ReleasePreflight {
                workflow_name: "Promote Release".to_string(),
                ready: false,
                blockers: vec!["Promote Release preflight unavailable.".to_string()],
                warnings: Vec::new(),
                action: None,
            }
        });

    ReleaseWizardState {
        rc_request,
        promote_request,
        rc_preflight,
        promote_preflight,
        last_dispatch: None,
    }
}

fn select_recommended_tasks(paths: &[String], tasks: &TaskCatalogSnapshot) -> Vec<TaskInfo> {
    let ids = recommended_task_ids(paths);
    tasks
        .tasks
        .iter()
        .filter(|task| ids.iter().any(|id| *id == task.id))
        .cloned()
        .collect()
}

fn build_journeys(
    repo: &RepoSnapshot,
    doctor: &[DoctorSnapshot],
    recommended_tasks: &[TaskInfo],
    work: &WorkSnapshot,
    workflows: &WorkflowRunSnapshot,
    release: &ReleaseWizardState,
) -> Vec<JourneyState> {
    let setup = build_setup_journey(doctor);
    let develop = build_develop_journey(repo, work);
    let verify = build_verify_journey(repo, recommended_tasks, workflows);
    let release = build_release_journey(release);
    vec![setup, develop, verify, release]
}

fn build_setup_journey(doctor: &[DoctorSnapshot]) -> JourneyState {
    let mut steps = doctor
        .iter()
        .map(|snapshot| JourneyStep {
            title: format!("{} toolchain", snapshot.domain.label()),
            status: if snapshot.missing_required {
                JourneyStepStatus::Blocked
            } else {
                JourneyStepStatus::Complete
            },
            detail: if snapshot.missing_required {
                format!("{} required tool(s) missing.", snapshot.missing_count())
            } else {
                "Ready for this domain.".to_string()
            },
            action: Some(simple_command_action(
                format!("doctor-{}", snapshot.domain.label()),
                format!("Recheck {}", snapshot.domain.label()),
                format!("cargo xtask doctor --domain {}", snapshot.domain.label()),
            )),
        })
        .collect::<Vec<_>>();

    if steps.is_empty() {
        steps.push(JourneyStep {
            title: "Doctor unavailable".to_string(),
            status: JourneyStepStatus::Blocked,
            detail: "Unable to read `cargo xtask doctor` output.".to_string(),
            action: None,
        });
    }

    let score = percentage(
        steps
            .iter()
            .filter(|step| matches!(step.status, JourneyStepStatus::Complete))
            .count(),
        steps.len(),
    );
    let next_action = steps
        .iter()
        .find(|step| {
            matches!(
                step.status,
                JourneyStepStatus::Blocked | JourneyStepStatus::Ready
            )
        })
        .and_then(|step| step.action.clone());

    JourneyState {
        kind: JourneyKind::Setup,
        score,
        steps,
        next_action,
    }
}

fn build_develop_journey(repo: &RepoSnapshot, work: &WorkSnapshot) -> JourneyState {
    let issue_action = repo.inferred_issue_id.map(|issue| {
        gh_action(
            "issue-view",
            "Open issue",
            vec!["issue", "view", &issue.to_string()],
        )
    });
    let pr_action = work.pull_request.as_ref().map(|pr| {
        gh_action(
            "pr-view",
            "Open PR",
            vec!["pr", "view", &pr.number.to_string()],
        )
    });
    let branch_clean = repo.dirty_files() == 0;

    let steps = vec![
        JourneyStep {
            title: "Issue linked from branch".to_string(),
            status: if work.issue.is_some() {
                JourneyStepStatus::Complete
            } else if repo.inferred_issue_id.is_some() {
                JourneyStepStatus::Ready
            } else {
                JourneyStepStatus::Blocked
            },
            detail: work.issue.as_ref().map_or_else(
                || "Branch does not resolve to an active issue.".to_string(),
                |issue| format!("#{} {}", issue.number, issue.title),
            ),
            action: issue_action,
        },
        JourneyStep {
            title: "Pull request open".to_string(),
            status: if work.pull_request.is_some() {
                JourneyStepStatus::Complete
            } else if work.issue.is_some() {
                JourneyStepStatus::Ready
            } else {
                JourneyStepStatus::Locked
            },
            detail: work.pull_request.as_ref().map_or_else(
                || "Open a PR once branch work is ready.".to_string(),
                |pr| format!("#{} {}", pr.number, pr.title),
            ),
            action: pr_action,
        },
        JourneyStep {
            title: "Working tree stable".to_string(),
            status: if branch_clean {
                JourneyStepStatus::Complete
            } else {
                JourneyStepStatus::Running
            },
            detail: if branch_clean {
                "No local file changes pending.".to_string()
            } else {
                format!("{} local change(s) pending.", repo.dirty_files())
            },
            action: None,
        },
    ];

    let score = percentage(
        steps
            .iter()
            .filter(|step| matches!(step.status, JourneyStepStatus::Complete))
            .count(),
        steps.len(),
    );
    let next_action = steps
        .iter()
        .find(|step| {
            matches!(
                step.status,
                JourneyStepStatus::Blocked | JourneyStepStatus::Ready
            )
        })
        .and_then(|step| step.action.clone());

    JourneyState {
        kind: JourneyKind::Develop,
        score,
        steps,
        next_action,
    }
}

fn build_verify_journey(
    repo: &RepoSnapshot,
    recommended_tasks: &[TaskInfo],
    workflows: &WorkflowRunSnapshot,
) -> JourneyState {
    let verification_action = recommended_tasks.first().map(task_action).or_else(|| {
        Some(simple_command_action(
            "verify-full",
            "Run verify-full",
            "cargo xtask run verify-full",
        ))
    });
    let failing_run = workflows.first_failure().cloned();

    let steps = vec![
        JourneyStep {
            title: "Recommended local verification".to_string(),
            status: if repo.changed_paths.is_empty() {
                JourneyStepStatus::Locked
            } else {
                JourneyStepStatus::Ready
            },
            detail: if recommended_tasks.is_empty() {
                "No changed-path recommendation available; fall back to verify-full.".to_string()
            } else {
                recommended_tasks
                    .iter()
                    .map(|task| task.id.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            },
            action: verification_action,
        },
        JourneyStep {
            title: "Recent workflow signal".to_string(),
            status: if let Some(run) = &failing_run {
                if run.status == "completed" {
                    JourneyStepStatus::Blocked
                } else {
                    JourneyStepStatus::Running
                }
            } else {
                JourneyStepStatus::Complete
            },
            detail: failing_run.map_or_else(
                || "No failing recent workflow runs.".to_string(),
                |run| {
                    format!(
                        "{}: {}",
                        run.workflow_name,
                        run.conclusion.unwrap_or(run.status)
                    )
                },
            ),
            action: workflows.first_failure().map(|run| rerun_action(run.id)),
        },
    ];

    let score = percentage(
        steps
            .iter()
            .filter(|step| matches!(step.status, JourneyStepStatus::Complete))
            .count(),
        steps.len(),
    );
    let next_action = steps
        .iter()
        .find(|step| {
            matches!(
                step.status,
                JourneyStepStatus::Blocked | JourneyStepStatus::Ready
            )
        })
        .and_then(|step| step.action.clone());

    JourneyState {
        kind: JourneyKind::Verify,
        score,
        steps,
        next_action,
    }
}

fn build_release_journey(release: &ReleaseWizardState) -> JourneyState {
    let rc_status = if release.rc_preflight.ready {
        JourneyStepStatus::Ready
    } else if release.rc_preflight.blockers.is_empty() {
        JourneyStepStatus::Locked
    } else {
        JourneyStepStatus::Blocked
    };
    let promote_status = if release.promote_preflight.ready {
        JourneyStepStatus::Ready
    } else if release.promote_preflight.blockers.is_empty() {
        JourneyStepStatus::Locked
    } else {
        JourneyStepStatus::Blocked
    };
    let steps = vec![
        JourneyStep {
            title: "Release candidate preflight".to_string(),
            status: rc_status,
            detail: if release.rc_preflight.ready {
                "Ready to dispatch Release Candidate.".to_string()
            } else {
                release.rc_preflight.blockers.join(" ")
            },
            action: release.rc_preflight.action.clone(),
        },
        JourneyStep {
            title: "Promote release preflight".to_string(),
            status: promote_status,
            detail: if release.promote_preflight.ready {
                "Ready to dispatch Promote Release.".to_string()
            } else {
                release.promote_preflight.blockers.join(" ")
            },
            action: release.promote_preflight.action.clone(),
        },
    ];

    let score = percentage(
        steps
            .iter()
            .filter(|step| matches!(step.status, JourneyStepStatus::Complete))
            .count(),
        steps.len(),
    );
    let next_action = steps
        .iter()
        .find(|step| {
            matches!(
                step.status,
                JourneyStepStatus::Blocked | JourneyStepStatus::Ready
            )
        })
        .and_then(|step| step.action.clone());

    JourneyState {
        kind: JourneyKind::Release,
        score,
        steps,
        next_action,
    }
}

fn percentage(done: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    u8::try_from((done * 100) / total).unwrap_or(100)
}

fn simple_command_action(
    id: impl Into<String>,
    title: impl Into<String>,
    command: impl Into<String>,
) -> ActionPlan {
    let command_preview = command.into();
    let parts = command_preview
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let program = parts.first().cloned().unwrap_or_default();
    let args = parts.into_iter().skip(1).collect::<Vec<_>>();
    ActionPlan {
        id: id.into(),
        title: title.into(),
        description: "Run the underlying DX command.".to_string(),
        command_preview: command_preview.clone(),
        command: ShellCommand {
            program,
            args,
            cwd: None,
            env: Vec::new(),
        },
        risk: ActionRisk::Low,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: true,
    }
}

fn task_action(task: &TaskInfo) -> ActionPlan {
    ActionPlan {
        id: format!("task-{}", task.id),
        title: format!("Run {}", task.id),
        description: task.description.clone(),
        command_preview: format!("cargo xtask run {}", task.id),
        command: ShellCommand {
            program: "cargo".to_string(),
            args: vec!["xtask".to_string(), "run".to_string(), task.id.clone()],
            cwd: None,
            env: Vec::new(),
        },
        risk: ActionRisk::Low,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: true,
    }
}

fn rerun_action(run_id: u64) -> ActionPlan {
    ActionPlan {
        id: format!("rerun-{run_id}"),
        title: format!("Rerun workflow {run_id}"),
        description: "Rerun a recent GitHub Actions workflow.".to_string(),
        command_preview: format!("gh run rerun {run_id}"),
        command: ShellCommand {
            program: "gh".to_string(),
            args: vec!["run".to_string(), "rerun".to_string(), run_id.to_string()],
            cwd: None,
            env: Vec::new(),
        },
        risk: ActionRisk::Medium,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: true,
    }
}

fn gh_action(id: &str, title: &str, args: Vec<&str>) -> ActionPlan {
    ActionPlan {
        id: id.to_string(),
        title: title.to_string(),
        description: "Open GitHub context for the current work item.".to_string(),
        command_preview: format!("gh {}", args.join(" ")),
        command: ShellCommand {
            program: "gh".to_string(),
            args: args.into_iter().map(str::to_string).collect(),
            cwd: None,
            env: Vec::new(),
        },
        risk: ActionRisk::Low,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: false,
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        CheckStatus, DoctorEntry, DoctorSnapshot, DxDomain, GithubAuthStatus, IssueSummary,
        PullRequestSummary, ReleaseCandidateRequest, ReleasePreflight, RepoSnapshot,
        TaskCatalogSnapshot, TaskInfo, WorkSnapshot, WorkflowRunSnapshot, WorkflowSummary,
    };
    use crate::providers::{
        DxProvider, GitProvider, GithubProvider, ProviderError, WorkflowProvider,
    };

    use super::{DevConsoleService, ReleaseInputs};

    #[derive(Clone)]
    struct StubGit(RepoSnapshot);
    impl GitProvider for StubGit {
        fn repo_snapshot(&self) -> Result<RepoSnapshot, ProviderError> {
            Ok(self.0.clone())
        }
    }

    #[derive(Clone)]
    struct StubGithub(WorkSnapshot);
    impl GithubProvider for StubGithub {
        fn work_snapshot(
            &self,
            _issue_hint: Option<u64>,
            _pr_head_branch: &str,
        ) -> Result<WorkSnapshot, ProviderError> {
            Ok(self.0.clone())
        }
    }

    #[derive(Clone)]
    struct StubDx {
        doctor: Vec<DoctorSnapshot>,
        tasks: TaskCatalogSnapshot,
    }
    impl DxProvider for StubDx {
        fn doctor(&self, domain: DxDomain) -> Result<DoctorSnapshot, ProviderError> {
            self.doctor
                .iter()
                .find(|snapshot| snapshot.domain == domain)
                .cloned()
                .ok_or_else(|| ProviderError::Parse(format!("missing domain {}", domain.label())))
        }

        fn task_catalog(&self) -> Result<TaskCatalogSnapshot, ProviderError> {
            Ok(self.tasks.clone())
        }
    }

    #[derive(Clone)]
    struct StubWorkflow {
        snapshot: WorkflowRunSnapshot,
        rc: ReleasePreflight,
        promote: ReleasePreflight,
    }
    impl WorkflowProvider for StubWorkflow {
        fn workflow_snapshot(&self) -> Result<WorkflowRunSnapshot, ProviderError> {
            Ok(self.snapshot.clone())
        }

        fn preflight_release_candidate(
            &self,
            _request: &ReleaseCandidateRequest,
        ) -> Result<ReleasePreflight, ProviderError> {
            Ok(self.rc.clone())
        }

        fn preflight_promote_release(
            &self,
            _request: &crate::model::PromoteReleaseRequest,
        ) -> Result<ReleasePreflight, ProviderError> {
            Ok(self.promote.clone())
        }
    }

    #[test]
    fn refresh_recommends_tasks_and_release_actions() {
        let service = DevConsoleService::new(
            StubGit(RepoSnapshot {
                branch: "feature/13-runtime-security-delivery-foundations".to_string(),
                upstream_branch: Some(
                    "origin/feature/13-runtime-security-delivery-foundations".to_string(),
                ),
                head_sha: Some("0123456789abcdef0123456789abcdef01234567".to_string()),
                staged_files: 1,
                unstaged_files: 0,
                untracked_files: 0,
                ahead: 0,
                behind: 0,
                changed_paths: vec!["ui/crates/site/src/web_app.rs".to_string()],
                inferred_issue_id: Some(13),
            }),
            StubGithub(WorkSnapshot {
                auth_status: GithubAuthStatus::Authenticated,
                repository: Some("shortorigin/short-origin".to_string()),
                issue: Some(IssueSummary {
                    number: 13,
                    title: "Establish runtime security".to_string(),
                    state: "OPEN".to_string(),
                    labels: vec!["type:feature".to_string()],
                    milestone: None,
                    project_status: Some("PR Open".to_string()),
                    url: "https://example.test/issues/13".to_string(),
                }),
                pull_request: Some(PullRequestSummary {
                    number: 14,
                    title: "feat(platform): establish runtime security".to_string(),
                    state: "OPEN".to_string(),
                    review_decision: Some("REVIEW_REQUIRED".to_string()),
                    head_ref_name: "feature/13-runtime-security-delivery-foundations".to_string(),
                    base_ref_name: "main".to_string(),
                    status_checks: Vec::new(),
                    url: "https://example.test/pull/14".to_string(),
                }),
                warnings: Vec::new(),
            }),
            StubDx {
                doctor: DxDomain::tracked()
                    .into_iter()
                    .map(|domain| DoctorSnapshot {
                        domain,
                        missing_required: false,
                        entries: vec![DoctorEntry {
                            tool: "rustfmt".to_string(),
                            required_by: vec!["verify-full".to_string()],
                            optional: false,
                            status: CheckStatus::Ok,
                            guidance: "install".to_string(),
                        }],
                        notes: Vec::new(),
                    })
                    .collect(),
                tasks: TaskCatalogSnapshot {
                    tasks: vec![
                        TaskInfo {
                            id: "ui-verify".to_string(),
                            description: "UI verify".to_string(),
                            domains: vec!["ui".to_string()],
                            prerequisites: vec!["trunk".to_string()],
                            dependencies: Vec::new(),
                            ci_included: true,
                            listed: true,
                        },
                        TaskInfo {
                            id: "verify-full".to_string(),
                            description: "Full verify".to_string(),
                            domains: vec!["core".to_string()],
                            prerequisites: vec!["rustfmt".to_string()],
                            dependencies: Vec::new(),
                            ci_included: true,
                            listed: true,
                        },
                    ],
                },
            },
            StubWorkflow {
                snapshot: WorkflowRunSnapshot {
                    workflows: vec![WorkflowSummary {
                        id: 1,
                        name: "Release Candidate".to_string(),
                        state: "active".to_string(),
                    }],
                    runs: Vec::new(),
                },
                rc: ReleasePreflight {
                    workflow_name: "Release Candidate".to_string(),
                    ready: true,
                    blockers: Vec::new(),
                    warnings: Vec::new(),
                    action: Some(crate::model::ActionPlan::default()),
                },
                promote: ReleasePreflight {
                    workflow_name: "Promote Release".to_string(),
                    ready: false,
                    blockers: vec!["missing rc".to_string()],
                    warnings: Vec::new(),
                    action: None,
                },
            },
        );

        let state = service.refresh(ReleaseInputs::default());
        assert_eq!(state.recommended_tasks.len(), 2);
        assert!(state.release.rc_preflight.ready);
        assert!(state.recommended_action.is_some());
    }
}
