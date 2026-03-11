use crate::architecture::{planes_for_paths, Plane};
use crate::common::workspace_root;
use jsonschema::validator_for;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_yaml_ng::Value as YamlValue;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
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
    execution_artifacts: ExecutionArtifactsConfig,
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
struct ExecutionArtifactsConfig {
    root: String,
    task_contract_filename: String,
    exec_plan_filename: String,
    require_for_multi_plane: bool,
    require_for_risk_classes: Vec<String>,
    recommend_for_risk_classes: Vec<String>,
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

#[derive(Debug, Serialize)]
struct ProcessAuditReport {
    documented: DocumentedProcess,
    automation: Vec<WorkflowAudit>,
    drift_matrix: Vec<DriftRow>,
    defects: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DocumentedProcess {
    source_files: Vec<String>,
    branch_model: String,
    required_checks: Vec<String>,
    release_flow: Vec<String>,
    module_invariants: Vec<String>,
    required_pr_sections: Vec<String>,
    required_issue_fields: Vec<String>,
    automatic_dev_promotion: bool,
    architecture_validation_documented: bool,
    plugin_validation_documented: bool,
}

#[derive(Debug, Serialize)]
struct WorkflowAudit {
    file: String,
    workflow_name: String,
    triggers: Vec<String>,
    jobs: Vec<JobAudit>,
    environment_targets: Vec<String>,
    shared_setup_steps: Vec<String>,
    reusable_logic_candidates: Vec<String>,
}

#[derive(Debug)]
struct ExecutionArtifactsValidationInput {
    issue_id: u64,
    branch: String,
    risk_class: String,
    changed_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TaskContractDocument {
    issue_id: u64,
    issue_url: String,
    branch: String,
    exec_plan_required: bool,
    exec_plan_path: String,
}

#[derive(Debug, Deserialize)]
struct AdrFrontMatter {
    adr_id: String,
    title: String,
    status: String,
    date_adopted: String,
    owners: Vec<String>,
    architectural_planes: Vec<String>,
    impacted_domains: Vec<String>,
    source_report: Option<String>,
    roadmap_phases: Vec<String>,
    supersedes: Vec<String>,
    superseded_by: Vec<String>,
    review_cadence: String,
    tags: Vec<String>,
}

#[derive(Debug)]
struct ParsedAdrDocument {
    front_matter: AdrFrontMatter,
    body: String,
}

#[derive(Debug, Serialize)]
struct JobAudit {
    job_id: String,
    job_name: String,
    condition: Option<String>,
}

#[derive(Debug, Serialize)]
struct DriftRow {
    expectation: String,
    documented_source: String,
    automation_source: String,
    status: String,
    details: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyncTarget {
    Org,
    Repo,
}

const REQUIRED_ADR_BODY_SECTIONS: [&str; 8] = [
    "Context",
    "Decision",
    "Constraints",
    "Alternatives Considered",
    "Rationale",
    "Implementation Implications",
    "Acceptance Checks",
    "Traceability",
];

const FIRST_PRINCIPLES_ADR_MAPPINGS: [(&str, &str, &str); 13] = [
    (
        "ADR-000",
        "ADR-0005",
        "docs/adr/0005-domain-bounded-platform-topology.md",
    ),
    (
        "ADR-001",
        "ADR-0006",
        "docs/adr/0006-canonical-event-substrate.md",
    ),
    (
        "ADR-002",
        "ADR-0007",
        "docs/adr/0007-consistency-semantics-by-criticality.md",
    ),
    (
        "ADR-003",
        "ADR-0008",
        "docs/adr/0008-domain-owned-data-products-with-contracts.md",
    ),
    (
        "ADR-004",
        "ADR-0009",
        "docs/adr/0009-lakehouse-open-format-analytical-foundation.md",
    ),
    (
        "ADR-005",
        "ADR-0010",
        "docs/adr/0010-durable-workflow-execution-plane.md",
    ),
    (
        "ADR-006",
        "ADR-0011",
        "docs/adr/0011-ai-as-planner-over-typed-tools.md",
    ),
    (
        "ADR-007",
        "ADR-0012",
        "docs/adr/0012-layered-intelligence-behind-stable-retrieval.md",
    ),
    (
        "ADR-008",
        "ADR-0013",
        "docs/adr/0013-zero-trust-ai-governance-control-plane.md",
    ),
    (
        "ADR-009",
        "ADR-0014",
        "docs/adr/0014-observability-slos-and-error-budgets.md",
    ),
    (
        "ADR-010",
        "ADR-0015",
        "docs/adr/0015-gitops-and-policy-as-code-control-artifacts.md",
    ),
    (
        "ADR-011",
        "ADR-0016",
        "docs/adr/0016-finops-unit-economics-and-autonomy-budgets.md",
    ),
    (
        "ADR-012",
        "ADR-0017",
        "docs/adr/0017-replaceable-compute-and-governed-state.md",
    ),
];

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
        Some((command, rest)) if command == "validate-execution-artifacts" => {
            validate_execution_artifacts(rest)
        }
        Some((command, rest)) if command == "audit-process" => audit_process(rest),
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

fn validate_execution_artifacts(args: &[String]) -> Result<(), String> {
    let mut config_path = PathBuf::from(".github/governance.toml");
    let mut issue_id: Option<u64> = None;
    let mut branch: Option<String> = None;
    let mut risk_class: Option<String> = None;
    let mut changed_files = Vec::new();
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
            "--issue-id" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --issue-id".to_owned());
                };
                issue_id = Some(
                    value
                        .parse::<u64>()
                        .map_err(|error| format!("invalid --issue-id `{value}`: {error}"))?,
                );
                index += 2;
            }
            "--branch" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --branch".to_owned());
                };
                branch = Some(value.clone());
                index += 2;
            }
            "--risk-class" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --risk-class".to_owned());
                };
                risk_class = Some(value.clone());
                index += 2;
            }
            "--changed-file" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing value for --changed-file".to_owned());
                };
                changed_files.push(value.clone());
                index += 2;
            }
            other => {
                return Err(format!(
                    "unknown validate-execution-artifacts argument `{other}`"
                ))
            }
        }
    }

    let risk_class = risk_class.ok_or_else(|| "missing --risk-class".to_owned())?;
    let risk_class = normalized_risk_class(&risk_class).ok_or_else(|| {
        "invalid --risk-class; expected one of `low`, `medium`, or `high`".to_owned()
    })?;
    let input = ExecutionArtifactsValidationInput {
        issue_id: issue_id.ok_or_else(|| "missing --issue-id".to_owned())?,
        branch: branch.ok_or_else(|| "missing --branch".to_owned())?,
        risk_class,
        changed_files,
    };
    let config = load_config(&config_path)?;
    validate_execution_artifacts_in_workspace(&workspace_root()?, &config, &input)?;
    println!(
        "validated execution artifacts for branch `{}` and issue `#{}`",
        input.branch, input.issue_id
    );
    Ok(())
}

fn audit_process(args: &[String]) -> Result<(), String> {
    let mut output_dir = workspace_root()?.join("target/process-audit");
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--output-dir" => {
                let Some(path) = args.get(index + 1) else {
                    return Err("missing value for --output-dir".to_owned());
                };
                output_dir = PathBuf::from(path);
                index += 2;
            }
            other => return Err(format!("unknown audit-process argument `{other}`")),
        }
    }

    let workspace_root = workspace_root()?;
    let report = build_process_audit(&workspace_root)?;
    fs::create_dir_all(&output_dir).map_err(|error| {
        format!(
            "failed to create audit output directory `{}`: {error}",
            output_dir.display()
        )
    })?;

    let json_path = output_dir.join("process-audit.json");
    let markdown_path = output_dir.join("process-audit.md");
    let matrix_path = output_dir.join("drift-matrix.md");
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("failed to serialize process audit JSON: {error}"))?;
    fs::write(&json_path, format!("{json}\n"))
        .map_err(|error| format!("failed to write `{}`: {error}", json_path.display()))?;
    fs::write(&markdown_path, render_process_audit_markdown(&report)).map_err(|error| {
        format!(
            "failed to write process audit markdown `{}`: {error}",
            markdown_path.display()
        )
    })?;
    fs::write(
        &matrix_path,
        render_drift_matrix_markdown(&report.drift_matrix),
    )
    .map_err(|error| {
        format!(
            "failed to write drift matrix `{}`: {error}",
            matrix_path.display()
        )
    })?;

    if report.defects.is_empty() {
        println!(
            "process audit passed; artifacts written to `{}`",
            output_dir.display()
        );
        Ok(())
    } else {
        Err(format!(
            "process audit found {} defect(s): {}",
            report.defects.len(),
            report.defects.join("; ")
        ))
    }
}

fn build_process_audit(workspace_root: &Path) -> Result<ProcessAuditReport, String> {
    let documented = load_documented_process(workspace_root)?;
    let workflows = load_workflow_audits(workspace_root)?;
    let defects = collect_audit_defects(&documented, &workflows)?;
    let drift_matrix = build_drift_matrix(&documented, &workflows, &defects);
    Ok(ProcessAuditReport {
        documented,
        automation: workflows,
        drift_matrix,
        defects,
    })
}

fn load_documented_process(workspace_root: &Path) -> Result<DocumentedProcess, String> {
    let source_files = vec![
        "README.md".to_string(),
        "CONTRIBUTING.md".to_string(),
        "DEVELOPMENT_MODEL.md".to_string(),
        "ARCHITECTURE.md".to_string(),
        "docs/README.md".to_string(),
        "docs/architecture/first-principles-systems-architecture-report.md".to_string(),
        "docs/architecture/first-principles-implementation-roadmap.md".to_string(),
        "docs/adr/README.md".to_string(),
        "docs/architecture/layer-boundaries.md".to_string(),
        "docs/architecture/plugin-application-model.md".to_string(),
        "docs/architecture/runtime-composition.md".to_string(),
        "docs/process/execution-artifacts.md".to_string(),
        "docs/process/github-governance-rollout.md".to_string(),
        "docs/process/github-workflow-migration.md".to_string(),
        "docs/process/platform-regression-guardrails.md".to_string(),
        "plans/README.md".to_string(),
        ".github/AGENTS.md".to_string(),
        "schemas/AGENTS.md".to_string(),
        "xtask/AGENTS.md".to_string(),
        ".github/PULL_REQUEST_TEMPLATE.md".to_string(),
        ".github/governance.toml".to_string(),
    ];
    let read = |path: &str| {
        fs::read_to_string(workspace_root.join(path))
            .map_err(|error| format!("failed to read `{path}`: {error}"))
    };
    let readme = read("README.md")?;
    let contributing = read("CONTRIBUTING.md")?;
    let development = read("DEVELOPMENT_MODEL.md")?;
    let architecture = read("ARCHITECTURE.md")?;
    let workflow_migration = read("docs/process/github-workflow-migration.md")?;
    let governance = load_config(&workspace_root.join(".github/governance.toml"))?;
    let documents_reference = |document: &str, command: &str| {
        document.contains(command) || document.contains("cargo verify-repo")
    };

    let branch_model = if development.contains("main` is the only long-lived branch")
        && contributing.contains("No direct commits to `main`")
    {
        "issue-driven trunk-based pull-request flow on main".to_owned()
    } else {
        "undetermined".to_owned()
    };

    let required_checks = governance
        .repository_defaults
        .required_status_checks
        .clone();
    let release_flow = vec![
        "Delivery Dev auto-promotes dev from merges to main".to_owned(),
        "Release Candidate is manual and deploys stage".to_owned(),
        "Promote Release is manual and deploys production".to_owned(),
    ];
    let module_invariants = architecture
        .lines()
        .filter(|line| line.starts_with("- "))
        .map(|line| line.trim_start_matches("- ").trim().to_owned())
        .collect::<Vec<_>>();
    let required_pr_sections = vec![
        "Summary".to_string(),
        "Linked Issue".to_string(),
        "Execution Artifacts".to_string(),
        "ADR References".to_string(),
        "Impacted Domains".to_string(),
        "Layers Touched".to_string(),
        "Contracts Changed".to_string(),
        "Tests Added or Updated".to_string(),
        "Refreshed from Main".to_string(),
        "Risk Class".to_string(),
        "Affected Consistency Class".to_string(),
        "Affected Risk Tier".to_string(),
        "Architecture Delta".to_string(),
        "Workflow Checklist".to_string(),
        "Technical Changes".to_string(),
        "Testing Strategy".to_string(),
        "Rollback Path".to_string(),
        "Validation Artifacts".to_string(),
        "Deployment Impact".to_string(),
    ];
    let required_issue_fields = vec![
        "primary_architectural_plane".to_string(),
        "architectural_references".to_string(),
        "owning_subsystem".to_string(),
        "integration_boundaries".to_string(),
        "impacted_domains".to_string(),
        "affected_consistency_class".to_string(),
        "affected_risk_tier".to_string(),
        "scope_in".to_string(),
        "scope_out".to_string(),
        "acceptance_criteria".to_string(),
        "validation_requirements".to_string(),
        "validation_artifacts".to_string(),
        "rollback_considerations".to_string(),
        "rollback_path".to_string(),
    ];

    Ok(DocumentedProcess {
        source_files,
        branch_model,
        required_checks,
        release_flow,
        module_invariants,
        required_pr_sections,
        required_issue_fields,
        automatic_dev_promotion: readme.contains("auto-promote the `dev` environment")
            || development.contains("auto-deploys `dev`"),
        architecture_validation_documented: documents_reference(
            &readme,
            "cargo xtask architecture audit-boundaries",
        ) && documents_reference(
            &contributing,
            "cargo xtask architecture audit-boundaries",
        ) && documents_reference(
            &development,
            "cargo xtask architecture audit-boundaries",
        ) && documents_reference(
            &workflow_migration,
            "cargo xtask architecture audit-boundaries",
        ),
        plugin_validation_documented: documents_reference(
            &readme,
            "cargo xtask plugin validate-manifests",
        ) && documents_reference(
            &contributing,
            "cargo xtask plugin validate-manifests",
        ) && documents_reference(
            &development,
            "cargo xtask plugin validate-manifests",
        ) && documents_reference(
            &workflow_migration,
            "cargo xtask plugin validate-manifests",
        ),
    })
}

fn load_workflow_audits(workspace_root: &Path) -> Result<Vec<WorkflowAudit>, String> {
    let workflow_files = [
        ".github/workflows/ci.yml",
        ".github/workflows/governance.yml",
        ".github/workflows/security.yml",
        ".github/workflows/delivery-dev.yml",
        ".github/workflows/release-candidate.yml",
        ".github/workflows/promote-release.yml",
    ];

    workflow_files
        .iter()
        .map(|path| parse_workflow_audit(workspace_root, path))
        .collect()
}

fn parse_workflow_audit(workspace_root: &Path, path: &str) -> Result<WorkflowAudit, String> {
    let raw = fs::read_to_string(workspace_root.join(path))
        .map_err(|error| format!("failed to read workflow `{path}`: {error}"))?;
    let parsed: YamlValue = serde_yaml_ng::from_str(&raw)
        .map_err(|error| format!("failed to parse workflow `{path}`: {error}"))?;
    let workflow_name = yaml_string(&parsed, "name")
        .ok_or_else(|| format!("workflow `{path}` is missing top-level `name`"))?;
    let triggers = extract_trigger_names(
        parsed
            .get("on")
            .ok_or_else(|| format!("workflow `{path}` is missing top-level `on`"))?,
    );
    let jobs = extract_jobs(
        parsed
            .get("jobs")
            .ok_or_else(|| format!("workflow `{path}` is missing top-level `jobs`"))?,
    )?;
    let environment_targets = extract_environment_targets(&parsed);
    let shared_setup_steps = collect_shared_setup_steps(&raw);
    let reusable_logic_candidates = collect_reusable_candidates(&raw);

    Ok(WorkflowAudit {
        file: path.to_owned(),
        workflow_name,
        triggers,
        jobs,
        environment_targets,
        shared_setup_steps,
        reusable_logic_candidates,
    })
}

fn yaml_string(value: &YamlValue, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_owned)
}

fn extract_trigger_names(value: &YamlValue) -> Vec<String> {
    match value {
        YamlValue::String(single) => vec![single.clone()],
        YamlValue::Sequence(items) => items
            .iter()
            .filter_map(YamlValue::as_str)
            .map(str::to_owned)
            .collect(),
        YamlValue::Mapping(mapping) => mapping
            .keys()
            .filter_map(YamlValue::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn extract_jobs(value: &YamlValue) -> Result<Vec<JobAudit>, String> {
    let Some(mapping) = value.as_mapping() else {
        return Err("workflow `jobs` entry must be a mapping".to_owned());
    };
    let mut jobs = Vec::new();
    for (job_id, body) in mapping {
        let Some(job_id) = job_id.as_str() else {
            continue;
        };
        let job_name = body
            .get("name")
            .and_then(YamlValue::as_str)
            .unwrap_or(job_id)
            .to_owned();
        let condition = body
            .get("if")
            .and_then(YamlValue::as_str)
            .map(str::to_owned);
        jobs.push(JobAudit {
            job_id: job_id.to_owned(),
            job_name,
            condition,
        });
    }
    Ok(jobs)
}

fn extract_environment_targets(value: &YamlValue) -> Vec<String> {
    let mut targets = BTreeSet::new();
    if let Some(jobs) = value.get("jobs").and_then(YamlValue::as_mapping) {
        for body in jobs.values() {
            match body.get("environment") {
                Some(YamlValue::String(name)) => {
                    targets.insert(name.clone());
                }
                Some(YamlValue::Mapping(mapping)) => {
                    if let Some(name) = mapping.get("name").and_then(YamlValue::as_str) {
                        targets.insert(name.to_owned());
                    }
                }
                _ => {}
            }
        }
    }
    targets.into_iter().collect()
}

fn collect_shared_setup_steps(raw: &str) -> Vec<String> {
    let mut steps = BTreeSet::new();
    for candidate in [
        "actions/checkout@v4",
        "actions/setup-node@v4",
        "dtolnay/rust-toolchain@",
        "Swatinem/rust-cache@v2",
        "pulumi/setup-pulumi@",
        "aws-actions/configure-aws-credentials@",
        "oras-project/setup-oras@",
    ] {
        if raw.contains(candidate) {
            steps.insert(candidate.to_owned());
        }
    }
    steps.into_iter().collect()
}

fn collect_reusable_candidates(raw: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if raw.contains("Setup Node") && raw.contains("Install Rust toolchain") {
        candidates.push("shared rust/node bootstrap".to_owned());
    }
    if raw.contains("Validate required configuration")
        && raw.contains("Login to GHCR")
        && raw.contains("Pulumi login")
    {
        candidates.push("shared delivery environment bootstrap".to_owned());
    }
    candidates
}

fn collect_audit_defects(
    documented: &DocumentedProcess,
    workflows: &[WorkflowAudit],
) -> Result<Vec<String>, String> {
    let mut defects = Vec::new();
    let workflow_map = workflows
        .iter()
        .map(|workflow| (workflow.workflow_name.as_str(), workflow))
        .collect::<BTreeMap<_, _>>();

    let governance = workflow_map
        .get("Governance")
        .ok_or_else(|| "missing Governance workflow".to_owned())?;
    let ci = workflow_map
        .get("CI")
        .ok_or_else(|| "missing CI workflow".to_owned())?;
    let security = workflow_map
        .get("Security")
        .ok_or_else(|| "missing Security workflow".to_owned())?;
    let delivery = workflow_map
        .get("Delivery Dev")
        .ok_or_else(|| "missing Delivery Dev workflow".to_owned())?;

    let expected_checks = documented.required_checks.iter().collect::<BTreeSet<_>>();
    let actual_checks = workflows
        .iter()
        .flat_map(|workflow| {
            workflow
                .jobs
                .iter()
                .map(move |job| format!("{} / {}", workflow.workflow_name, job.job_name))
        })
        .collect::<BTreeSet<_>>();
    for check in expected_checks {
        if !actual_checks.contains(check) {
            defects.push(format!(
                "documented required check `{check}` is not emitted by any workflow job"
            ));
        }
    }

    for workflow in [governance, ci, security] {
        if !workflow
            .triggers
            .iter()
            .any(|trigger| trigger == "pull_request")
        {
            defects.push(format!(
                "workflow `{}` must trigger on pull_request to enforce documented PR checks",
                workflow.workflow_name
            ));
        }
    }

    for workflow in workflows {
        let has_pull_request_trigger = workflow
            .triggers
            .iter()
            .any(|trigger| trigger == "pull_request");
        for job in &workflow.jobs {
            if let Some(condition) = &job.condition {
                if condition.contains("pull_request") && !has_pull_request_trigger {
                    defects.push(format!(
                        "workflow `{}` job `{}` has PR-only condition `{condition}` without a pull_request trigger",
                        workflow.workflow_name, job.job_name
                    ));
                }
            }
        }
    }

    if documented.automatic_dev_promotion
        && !(delivery.triggers.iter().any(|trigger| trigger == "push")
            && workflow_targets_main(delivery))
    {
        defects.push(
            "documentation requires automatic dev promotion from merges to main, but Delivery Dev is not push-to-main automated"
                .to_owned(),
        );
    }

    if !documented.architecture_validation_documented {
        defects.push(
            "contributor docs must reference the architecture audit directly or through `cargo verify-repo`"
                .to_owned(),
        );
    }
    if !documented.plugin_validation_documented {
        defects.push(
            "contributor docs must reference plugin validation directly or through `cargo verify-repo`"
                .to_owned(),
        );
    }

    defects.extend(audit_issue_templates(&documented.required_issue_fields)?);
    defects.extend(audit_pr_template(&documented.required_pr_sections)?);
    defects.extend(audit_nested_agent_guides()?);
    defects.extend(audit_docs_indexes()?);
    defects.extend(audit_governance_workflow_for_architecture_step()?);
    defects.extend(audit_adr_corpus(&workspace_root()?)?);

    Ok(defects)
}

fn audit_issue_templates(required_fields: &[String]) -> Result<Vec<String>, String> {
    let templates = [
        ".github/ISSUE_TEMPLATE/feature.yml",
        ".github/ISSUE_TEMPLATE/bug.yml",
        ".github/ISSUE_TEMPLATE/docs.yml",
        ".github/ISSUE_TEMPLATE/infra.yml",
        ".github/ISSUE_TEMPLATE/refactor.yml",
        ".github/ISSUE_TEMPLATE/research.yml",
    ];
    let workspace_root = workspace_root()?;
    let mut defects = Vec::new();

    for template in templates {
        let raw = fs::read_to_string(workspace_root.join(template))
            .map_err(|error| format!("failed to read `{template}`: {error}"))?;
        let yaml: YamlValue = serde_yaml_ng::from_str(&raw)
            .map_err(|error| format!("failed to parse `{template}`: {error}"))?;
        let Some(body) = yaml.get("body").and_then(YamlValue::as_sequence) else {
            defects.push(format!(
                "issue template `{template}` is missing a body sequence"
            ));
            continue;
        };
        let ids = body
            .iter()
            .filter_map(|entry| entry.get("id").and_then(YamlValue::as_str))
            .collect::<BTreeSet<_>>();
        for field in required_fields {
            if !ids.contains(field.as_str()) {
                defects.push(format!(
                    "issue template `{template}` is missing required field `{field}`"
                ));
            }
        }
    }

    Ok(defects)
}

fn audit_pr_template(required_sections: &[String]) -> Result<Vec<String>, String> {
    let workspace_root = workspace_root()?;
    let template_path = ".github/PULL_REQUEST_TEMPLATE.md";
    let raw = fs::read_to_string(workspace_root.join(template_path))
        .map_err(|error| format!("failed to read `{template_path}`: {error}"))?;
    let mut defects = Vec::new();
    for section in required_sections {
        let heading = format!("## {section}");
        if !raw.contains(&heading) {
            defects.push(format!(
                "pull request template is missing required section `{section}`"
            ));
        }
    }
    Ok(defects)
}

fn audit_nested_agent_guides() -> Result<Vec<String>, String> {
    let workspace_root = workspace_root()?;
    let root_agents = fs::read_to_string(workspace_root.join("AGENTS.md"))
        .map_err(|error| format!("failed to read `AGENTS.md`: {error}"))?;
    let mut defects = Vec::new();
    for path in [".github/AGENTS.md", "schemas/AGENTS.md", "xtask/AGENTS.md"] {
        if !workspace_root.join(path).is_file() {
            defects.push(format!("required nested agent guide `{path}` is missing"));
        }
        if !root_agents.contains(path) {
            defects.push(format!(
                "top-level `AGENTS.md` must reference nested guide `{path}`"
            ));
        }
    }
    Ok(defects)
}

fn audit_docs_indexes() -> Result<Vec<String>, String> {
    let workspace_root = workspace_root()?;
    let root_readme = fs::read_to_string(workspace_root.join("README.md"))
        .map_err(|error| format!("failed to read `README.md`: {error}"))?;
    let docs_index_path = workspace_root.join("docs/README.md");
    if !docs_index_path.is_file() {
        return Ok(vec![
            "required docs index `docs/README.md` is missing".to_owned()
        ]);
    }

    let docs_index = fs::read_to_string(&docs_index_path)
        .map_err(|error| format!("failed to read `{}`: {error}", docs_index_path.display()))?;
    let mut defects = Vec::new();
    if !root_readme.contains("docs/README.md") {
        defects.push("root `README.md` must reference `docs/README.md`".to_owned());
    }
    for required in [
        "architecture/layer-boundaries.md",
        "process/execution-artifacts.md",
        "process/github-governance-rollout.md",
        "adr/README.md",
    ] {
        if !docs_index.contains(required) {
            defects.push(format!(
                "`docs/README.md` must reference `{required}` for discoverability"
            ));
        }
    }
    Ok(defects)
}

fn audit_governance_workflow_for_architecture_step() -> Result<Vec<String>, String> {
    let workspace_root = workspace_root()?;
    let path = ".github/workflows/governance.yml";
    let raw = fs::read_to_string(workspace_root.join(path))
        .map_err(|error| format!("failed to read `{path}`: {error}"))?;
    let mut defects = Vec::new();
    if !raw.contains("cargo xtask architecture audit-boundaries") {
        defects.push(
            "governance workflow must run `cargo xtask architecture audit-boundaries`".to_owned(),
        );
    }
    if !raw.contains("cargo xtask plugin validate-manifests") {
        defects.push(
            "governance workflow must run `cargo xtask plugin validate-manifests`".to_owned(),
        );
    }
    if !raw.contains("cargo xtask github validate-pr") {
        defects.push(
            "governance workflow must run `cargo xtask github validate-pr` for PR traceability enforcement"
                .to_owned(),
        );
    }
    Ok(defects)
}

fn audit_adr_corpus(workspace_root: &Path) -> Result<Vec<String>, String> {
    let root_regex = Regex::new(r"^(?P<number>\d{4})-.*\.md$")
        .map_err(|error| format!("failed to build root ADR filename regex: {error}"))?;
    let ui_regex = Regex::new(r".*\.md$")
        .map_err(|error| format!("failed to build UI ADR filename regex: {error}"))?;
    let allowed_statuses = ["Adopted", "Superseded", "Deprecated"];
    let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
        .map_err(|error| format!("failed to build ADR date regex: {error}"))?;
    let mut defects = Vec::new();
    let mut adr_ids = BTreeMap::new();
    let mut parsed_root_docs = BTreeMap::new();

    let root_docs = collect_adr_paths(workspace_root.join("docs/adr"), &root_regex)?;
    let ui_docs = collect_adr_paths(workspace_root.join("ui/docs/adr"), &ui_regex)?;

    let mut root_numbers = Vec::new();
    for path in root_docs {
        let relative = relative_path(workspace_root, &path);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("failed to read file name for `{}`", path.display()))?;
        let captures = root_regex
            .captures(file_name)
            .ok_or_else(|| format!("root ADR path `{relative}` does not match numbering rule"))?;
        let number = captures["number"]
            .parse::<u32>()
            .map_err(|error| format!("failed to parse ADR number from `{relative}`: {error}"))?;
        let parsed = parse_adr_document(&path)?;
        defects.extend(validate_adr_document(
            workspace_root,
            &relative,
            &parsed,
            &allowed_statuses,
            &date_regex,
        ));
        let expected_adr_id = format!("ADR-{number:04}");
        if parsed.front_matter.adr_id != expected_adr_id {
            defects.push(format!(
                "ADR `{relative}` must declare adr_id `{expected_adr_id}`, found `{}`",
                parsed.front_matter.adr_id
            ));
        }
        if let Some(previous) = adr_ids.insert(parsed.front_matter.adr_id.clone(), relative.clone())
        {
            defects.push(format!(
                "duplicate ADR id `{}` found in `{previous}` and `{relative}`",
                parsed.front_matter.adr_id
            ));
        }
        root_numbers.push(number);
        parsed_root_docs.insert(relative, parsed);
    }

    root_numbers.sort_unstable();
    if let Some(first) = root_numbers.first().copied() {
        for (offset, number) in root_numbers.iter().enumerate() {
            let expected = first
                + u32::try_from(offset)
                    .map_err(|_| "ADR corpus size exceeded supported u32 range".to_owned())?;
            if *number != expected {
                defects.push(format!(
                    "root ADR corpus must be contiguous; expected `{:04}` but found `{:04}`",
                    expected, number
                ));
                break;
            }
        }
        if first != 1 {
            defects.push(format!(
                "root ADR corpus must start at `0001`, found `{:04}`",
                first
            ));
        }
    } else {
        defects.push("root ADR corpus is empty".to_owned());
    }

    for path in ui_docs {
        let relative = relative_path(workspace_root, &path);
        let parsed = parse_adr_document(&path)?;
        defects.extend(validate_adr_document(
            workspace_root,
            &relative,
            &parsed,
            &allowed_statuses,
            &date_regex,
        ));
        if !parsed.front_matter.adr_id.starts_with("UI-ADR-") {
            defects.push(format!(
                "UI ADR `{relative}` must use a `UI-ADR-` namespaced adr_id"
            ));
        }
        if let Some(previous) = adr_ids.insert(parsed.front_matter.adr_id.clone(), relative.clone())
        {
            defects.push(format!(
                "duplicate ADR id `{}` found in `{previous}` and `{relative}`",
                parsed.front_matter.adr_id
            ));
        }
    }

    defects.extend(audit_first_principles_mapping(
        workspace_root,
        &parsed_root_docs,
    )?);

    Ok(defects)
}

fn collect_adr_paths(dir: PathBuf, name_regex: &Regex) -> Result<Vec<PathBuf>, String> {
    let mut paths = fs::read_dir(&dir)
        .map_err(|error| format!("failed to read ADR directory `{}`: {error}", dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name_regex.is_match(name))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn parse_adr_document(path: &Path) -> Result<ParsedAdrDocument, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read ADR `{}`: {error}", path.display()))?;
    let (front_matter_raw, body) = split_front_matter(&raw).map_err(|error| {
        format!(
            "failed to parse front matter for ADR `{}`: {error}",
            path.display()
        )
    })?;
    let front_matter: AdrFrontMatter =
        serde_yaml_ng::from_str(&front_matter_raw).map_err(|error| {
            format!(
                "failed to parse ADR front matter `{}`: {error}",
                path.display()
            )
        })?;
    Ok(ParsedAdrDocument { front_matter, body })
}

fn split_front_matter(raw: &str) -> Result<(String, String), String> {
    let normalized = raw.replace("\r\n", "\n");
    let Some(remainder) = normalized.strip_prefix("---\n") else {
        return Err("missing opening `---` front matter delimiter".to_owned());
    };
    let Some((front_matter, body)) = remainder.split_once("\n---\n") else {
        return Err("missing closing `---` front matter delimiter".to_owned());
    };
    Ok((front_matter.to_owned(), body.trim_start().to_owned()))
}

fn validate_adr_document(
    workspace_root: &Path,
    relative: &str,
    parsed: &ParsedAdrDocument,
    allowed_statuses: &[&str],
    date_regex: &Regex,
) -> Vec<String> {
    let mut defects = Vec::new();
    let front_matter = &parsed.front_matter;

    if front_matter.adr_id.trim().is_empty() {
        defects.push(format!("ADR `{relative}` is missing a non-empty `adr_id`"));
    }
    if front_matter.title.trim().is_empty() {
        defects.push(format!("ADR `{relative}` is missing a non-empty `title`"));
    }
    if !allowed_statuses.contains(&front_matter.status.as_str()) {
        defects.push(format!(
            "ADR `{relative}` has unsupported status `{}`",
            front_matter.status
        ));
    }
    if !date_regex.is_match(front_matter.date_adopted.trim()) {
        defects.push(format!(
            "ADR `{relative}` must use YYYY-MM-DD `date_adopted`, found `{}`",
            front_matter.date_adopted
        ));
    }
    if front_matter.owners.is_empty() {
        defects.push(format!("ADR `{relative}` must declare at least one owner"));
    }
    if front_matter.architectural_planes.is_empty() {
        defects.push(format!(
            "ADR `{relative}` must declare at least one architectural plane"
        ));
    }
    if front_matter.impacted_domains.is_empty() {
        defects.push(format!(
            "ADR `{relative}` must declare at least one impacted domain"
        ));
    }
    if front_matter.review_cadence.trim().is_empty() {
        defects.push(format!("ADR `{relative}` must declare `review_cadence`"));
    }
    if front_matter.tags.is_empty() {
        defects.push(format!("ADR `{relative}` must declare at least one tag"));
    }
    if let Some(source_report) = &front_matter.source_report {
        if !workspace_root.join(source_report).exists() {
            defects.push(format!(
                "ADR `{relative}` references missing source_report `{source_report}`"
            ));
        }
    }
    let _ = (&front_matter.supersedes, &front_matter.superseded_by);

    let mut last_position = 0usize;
    for section in REQUIRED_ADR_BODY_SECTIONS {
        let heading = format!("## {section}");
        let Some(position) = parsed.body.find(&heading) else {
            defects.push(format!(
                "ADR `{relative}` is missing required section `{section}`"
            ));
            continue;
        };
        if position < last_position {
            defects.push(format!(
                "ADR `{relative}` must keep required sections in the documented order"
            ));
            break;
        }
        last_position = position;
        match markdown_section(&parsed.body, section) {
            Some(contents) if !contents.trim().is_empty() => {}
            _ => defects.push(format!(
                "ADR `{relative}` must include non-empty contents for section `{section}`"
            )),
        }
    }

    defects
}

fn audit_first_principles_mapping(
    workspace_root: &Path,
    parsed_root_docs: &BTreeMap<String, ParsedAdrDocument>,
) -> Result<Vec<String>, String> {
    let read = |path: &str| {
        fs::read_to_string(workspace_root.join(path))
            .map_err(|error| format!("failed to read `{path}`: {error}"))
    };
    let report = read("docs/architecture/first-principles-systems-architecture-report.md")?;
    let roadmap = read("docs/architecture/first-principles-implementation-roadmap.md")?;
    let readme = read("docs/adr/README.md")?;
    let mut defects = Vec::new();

    for phase in 0..=9 {
        let phase_heading = format!("Phase {phase}");
        if !roadmap.contains(&phase_heading) {
            defects.push(format!(
                "first-principles roadmap must include `{phase_heading}`"
            ));
        }
    }

    if !report.contains("ADR-0005") || !report.contains("Phase 0") || !report.contains("Phase 9") {
        defects.push(
            "first-principles baseline report must reference the ratified ADR pack and roadmap phases"
                .to_owned(),
        );
    }

    for (blueprint_adr, repo_adr, path) in FIRST_PRINCIPLES_ADR_MAPPINGS {
        let Some(parsed) = parsed_root_docs.get(path) else {
            defects.push(format!(
                "first-principles mapping references missing ADR document `{path}`"
            ));
            continue;
        };
        if parsed.front_matter.adr_id != repo_adr {
            defects.push(format!(
                "ADR mapping for blueprint `{blueprint_adr}` expected repo id `{repo_adr}`, found `{}`",
                parsed.front_matter.adr_id
            ));
        }
        let traceability = markdown_section(&parsed.body, "Traceability").unwrap_or_default();
        if !traceability.contains(blueprint_adr) {
            defects.push(format!(
                "ADR `{repo_adr}` must reference blueprint `{blueprint_adr}` in Traceability"
            ));
        }
        if !readme.contains(blueprint_adr) || !readme.contains(repo_adr) || !readme.contains(path) {
            defects.push(format!(
                "ADR README must map blueprint `{blueprint_adr}` to `{repo_adr}` and `{path}`"
            ));
        }
        if !report.contains(repo_adr) {
            defects.push(format!(
                "first-principles baseline report must reference ratified ADR `{repo_adr}`"
            ));
        }
        for phase in &parsed.front_matter.roadmap_phases {
            if !phase.is_empty() && !roadmap.contains(phase) {
                defects.push(format!(
                    "ADR `{repo_adr}` references roadmap phase `{phase}` that is missing from the roadmap document"
                ));
            }
        }
    }

    Ok(defects)
}

fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn workflow_targets_main(workflow: &WorkflowAudit) -> bool {
    let path = workflow.file.as_str();
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    raw.contains("branches:") && raw.contains("- main")
}

fn build_drift_matrix(
    documented: &DocumentedProcess,
    workflows: &[WorkflowAudit],
    defects: &[String],
) -> Vec<DriftRow> {
    let actual_checks = workflows
        .iter()
        .flat_map(|workflow| {
            workflow
                .jobs
                .iter()
                .map(move |job| format!("{} / {}", workflow.workflow_name, job.job_name))
        })
        .collect::<BTreeSet<_>>();

    let mut rows = documented
        .required_checks
        .iter()
        .map(|check| DriftRow {
            expectation: format!("required check `{check}` is automated"),
            documented_source: ".github/governance.toml + contributor docs".to_owned(),
            automation_source: ".github/workflows/*".to_owned(),
            status: if actual_checks.contains(check) {
                "pass".to_owned()
            } else {
                "fail".to_owned()
            },
            details: if actual_checks.contains(check) {
                format!("found `{check}`")
            } else {
                format!("missing `{check}`")
            },
        })
        .collect::<Vec<_>>();

    rows.push(DriftRow {
        expectation: "Delivery Dev auto-promotes merges to main".to_owned(),
        documented_source: "README.md + DEVELOPMENT_MODEL.md".to_owned(),
        automation_source: ".github/workflows/delivery-dev.yml".to_owned(),
        status: if defects
            .iter()
            .any(|defect| defect.contains("automatic dev promotion"))
        {
            "fail".to_owned()
        } else {
            "pass".to_owned()
        },
        details: "must trigger on push to main; manual-only dispatch is drift".to_owned(),
    });

    rows.push(DriftRow {
        expectation: "Governance validates architecture boundaries".to_owned(),
        documented_source: "README.md + CONTRIBUTING.md + docs/process/*".to_owned(),
        automation_source: ".github/workflows/governance.yml".to_owned(),
        status: if defects
            .iter()
            .any(|defect| defect.contains("architecture audit-boundaries"))
        {
            "fail".to_owned()
        } else {
            "pass".to_owned()
        },
        details: "governance workflow should run `cargo xtask architecture audit-boundaries`"
            .to_owned(),
    });

    rows.push(DriftRow {
        expectation: "Governance validates governed plugin manifests".to_owned(),
        documented_source: "README.md + CONTRIBUTING.md + docs/process/*".to_owned(),
        automation_source: ".github/workflows/governance.yml".to_owned(),
        status: if defects
            .iter()
            .any(|defect| defect.contains("plugin validate-manifests"))
        {
            "fail".to_owned()
        } else {
            "pass".to_owned()
        },
        details: "governance workflow should run `cargo xtask plugin validate-manifests`"
            .to_owned(),
    });

    rows
}

fn render_process_audit_markdown(report: &ProcessAuditReport) -> String {
    let mut defects = String::new();
    if report.defects.is_empty() {
        defects.push_str("- none\n");
    } else {
        for defect in &report.defects {
            let _ = writeln!(defects, "- {defect}");
        }
    }

    let mut workflows = String::new();
    for workflow in &report.automation {
        let jobs = workflow
            .jobs
            .iter()
            .map(|job| format!("{} ({})", job.job_name, job.job_id))
            .collect::<Vec<_>>()
            .join(", ");
        let environments = if workflow.environment_targets.is_empty() {
            "none".to_owned()
        } else {
            workflow.environment_targets.join(", ")
        };
        let shared_setup = if workflow.shared_setup_steps.is_empty() {
            "none".to_owned()
        } else {
            workflow.shared_setup_steps.join(", ")
        };
        let reusable_logic = if workflow.reusable_logic_candidates.is_empty() {
            "none".to_owned()
        } else {
            workflow.reusable_logic_candidates.join(", ")
        };
        let _ = writeln!(
            workflows,
            "### {}\n- file: `{}`\n- triggers: {}\n- jobs: {}\n- environments: {}\n- shared setup: {}\n- reusable logic candidates: {}\n",
            workflow.workflow_name,
            workflow.file,
            workflow.triggers.join(", "),
            jobs,
            environments,
            shared_setup,
            reusable_logic
        );
    }

    format!(
        "# Process Flow Audit Report\n\n## Documented Source of Truth\n- source files: {}\n- branch model: {}\n- required checks: {}\n- required PR sections: {}\n- required issue fields: {}\n- architecture validation documented: {}\n- plugin validation documented: {}\n- automatic dev promotion: {}\n\n## Defects\n{}\
\n## Workflow Baseline\n{}\
\n## Drift Matrix\n{}",
        report.documented.source_files.join(", "),
        report.documented.branch_model,
        report.documented.required_checks.join(", "),
        report.documented.required_pr_sections.join(", "),
        report.documented.required_issue_fields.join(", "),
        if report.documented.architecture_validation_documented {
            "yes"
        } else {
            "no"
        },
        if report.documented.plugin_validation_documented {
            "yes"
        } else {
            "no"
        },
        if report.documented.automatic_dev_promotion {
            "yes"
        } else {
            "no"
        },
        defects,
        workflows,
        render_drift_matrix_markdown(&report.drift_matrix)
    )
}

fn render_drift_matrix_markdown(rows: &[DriftRow]) -> String {
    let mut out =
        "| Expectation | Documented Source | Automation Source | Status | Details |\n| --- | --- | --- | --- | --- |\n"
            .to_owned();
    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            row.expectation, row.documented_source, row.automation_source, row.status, row.details
        );
    }
    out
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
            "- execution artifacts live under `{}`; required for multi-plane work and risk classes [{}], recommended for risk classes [{}]",
            config.execution_artifacts.root,
            config.execution_artifacts.require_for_risk_classes.join(", "),
            config.execution_artifacts.recommend_for_risk_classes.join(", ")
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
                    "name": "Origin branch naming",
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
    let base_sha = value
        .pointer("/pull_request/base/sha")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let head_sha = value
        .pointer("/pull_request/head/sha")
        .and_then(Value::as_str)
        .map(str::to_owned);

    Ok(PullRequestEvent {
        title,
        body,
        branch,
        repository,
        base_sha,
        head_sha,
        changed_files: Vec::new(),
    })
}

#[derive(Debug)]
struct PullRequestEvent {
    title: String,
    body: String,
    branch: String,
    repository: String,
    base_sha: Option<String>,
    head_sha: Option<String>,
    changed_files: Vec<String>,
}

fn validate_pr_event(config: &GovernanceConfig, event: &PullRequestEvent) -> Result<(), String> {
    let branch_regex = Regex::new(&config.repository_defaults.branch_name_pattern)
        .map_err(|error| format!("invalid branch_name_pattern regex in config: {error}"))?;
    let title_regex = Regex::new(&config.repository_defaults.pr_title_pattern)
        .map_err(|error| format!("invalid pr_title_pattern regex in config: {error}"))?;

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
    for section in [
        "Summary",
        "Linked Issue",
        "Execution Artifacts",
        "ADR References",
        "Impacted Domains",
        "Layers Touched",
        "Contracts Changed",
        "Tests Added or Updated",
        "Refreshed from Main",
        "Risk Class",
        "Affected Consistency Class",
        "Affected Risk Tier",
        "Architecture Delta",
        "Workflow Checklist",
        "Technical Changes",
        "Testing Strategy",
        "Rollback Path",
        "Validation Artifacts",
        "Deployment Impact",
    ] {
        match markdown_section(&event.body, section) {
            Some(contents) if !contents.trim().is_empty() => {}
            _ => failures.push(format!(
                "PR body must include a non-empty `{section}` section"
            )),
        }
    }

    let linked_issue_section = markdown_section(&event.body, "Linked Issue").unwrap_or_default();
    let linked_issue_ids = extract_closing_issue_ids(&linked_issue_section, &event.repository)?;
    if linked_issue_ids.is_empty() {
        failures.push(
            "Linked Issue section must include exactly one same-repository closing directive such as `Closes #123`"
                .to_owned(),
        );
    } else if linked_issue_ids.len() > 1 {
        failures.push(
            "Linked Issue section must close exactly one same-repository issue for the branch"
                .to_owned(),
        );
    }
    if let Some(branch_issue_id) = issue_id_from_branch(&event.branch) {
        if linked_issue_ids.len() == 1 && !linked_issue_ids.contains(&branch_issue_id) {
            failures.push(format!(
                "branch issue id `#{branch_issue_id}` must match the issue closed in the Linked Issue section"
            ));
        }
    }

    let changed_files = if event.changed_files.is_empty() {
        match (&event.base_sha, &event.head_sha) {
            (Some(base), Some(head)) => changed_files_between(base, head)?,
            _ => Vec::new(),
        }
    } else {
        event.changed_files.clone()
    };
    let planes = planes_for_paths(changed_files.iter().map(String::as_str));
    let non_trivial_planes = planes
        .iter()
        .filter(|plane| !matches!(plane, Plane::Docs | Plane::Github | Plane::WorkItems))
        .copied()
        .collect::<BTreeSet<_>>();
    if non_trivial_planes.len() > 1 {
        let architecture_delta = markdown_section(&event.body, "Architecture Delta")
            .unwrap_or_default()
            .to_lowercase();
        if architecture_delta.contains("single-plane")
            || architecture_delta.contains("single plane")
            || architecture_delta.contains("n/a")
            || architecture_delta.contains("none")
        {
            failures.push(
                "multi-plane PRs must provide a substantive `Architecture Delta` section"
                    .to_owned(),
            );
        }
    }

    let risk_class_section = markdown_section(&event.body, "Risk Class").unwrap_or_default();
    let Some(risk_class) = normalized_risk_class(&risk_class_section) else {
        failures.push(
            "Risk Class section must contain exactly one of `low`, `medium`, or `high`".to_owned(),
        );
        return Err(failures.join("; "));
    };

    if let Some(issue_id) = issue_id_from_branch(&event.branch) {
        let input = ExecutionArtifactsValidationInput {
            issue_id,
            branch: event.branch.clone(),
            risk_class,
            changed_files: changed_files.clone(),
        };
        if let Err(error) =
            validate_execution_artifacts_in_workspace(&workspace_root()?, config, &input)
        {
            failures.push(error);
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn issue_id_from_branch(branch: &str) -> Option<u64> {
    let (_, rest) = branch.split_once('/')?;
    let (issue_id, _) = rest.split_once('-')?;
    issue_id.parse().ok()
}

fn extract_closing_issue_ids(body: &str, repository: &str) -> Result<BTreeSet<u64>, String> {
    let regex = Regex::new(&format!(
        "(?im)\\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\\b\\s+(?:https://github\\.com/{repo}/issues/(?P<full>[0-9]+)|#(?P<short>[0-9]+))",
        repo = regex::escape(repository)
    ))
    .map_err(|error| format!("failed to build issue extraction regex: {error}"))?;

    Ok(regex
        .captures_iter(body)
        .filter_map(|captures| {
            captures
                .name("full")
                .or_else(|| captures.name("short"))
                .and_then(|capture| capture.as_str().parse::<u64>().ok())
        })
        .collect())
}

fn normalized_risk_class(section: &str) -> Option<String> {
    let regex = Regex::new(r"(?i)\b(low|medium|high)\b").ok()?;
    let matches = regex
        .captures_iter(section)
        .filter_map(|captures| {
            captures
                .get(1)
                .map(|capture| capture.as_str().to_ascii_lowercase())
        })
        .collect::<BTreeSet<_>>();
    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

fn validate_execution_artifacts_in_workspace(
    workspace_root: &Path,
    config: &GovernanceConfig,
    input: &ExecutionArtifactsValidationInput,
) -> Result<(), String> {
    let require_artifacts = execution_artifacts_required(
        &config.execution_artifacts,
        &input.risk_class,
        &input.changed_files,
    );
    let artifact_root = workspace_root.join(&config.execution_artifacts.root);
    let matching_dirs = matching_execution_artifact_dirs(&artifact_root, input.issue_id)?;

    if !require_artifacts && matching_dirs.is_empty() {
        return Ok(());
    }

    if matching_dirs.len() != 1 {
        let requirement = if require_artifacts {
            "required"
        } else {
            "present"
        };
        return Err(format!(
            "execution artifacts are {requirement} for issue `#{}`; expected exactly one directory under `{}` matching `{}-*`, found {}",
            input.issue_id,
            config.execution_artifacts.root,
            input.issue_id,
            matching_dirs.len()
        ));
    }

    let artifact_dir = &matching_dirs[0];
    let task_contract_path = artifact_dir.join(&config.execution_artifacts.task_contract_filename);
    let exec_plan_path = artifact_dir.join(&config.execution_artifacts.exec_plan_filename);
    if !task_contract_path.is_file() {
        return Err(format!(
            "execution artifact `{}` is missing",
            relative_path(workspace_root, &task_contract_path)
        ));
    }
    if !exec_plan_path.is_file() {
        return Err(format!(
            "execution artifact `{}` is missing",
            relative_path(workspace_root, &exec_plan_path)
        ));
    }

    validate_task_contract_file(
        workspace_root,
        config,
        &task_contract_path,
        &exec_plan_path,
        input,
        require_artifacts,
    )?;
    validate_exec_plan_file(&exec_plan_path)?;
    Ok(())
}

fn execution_artifacts_required(
    config: &ExecutionArtifactsConfig,
    risk_class: &str,
    changed_files: &[String],
) -> bool {
    let non_trivial_planes = planes_for_paths(changed_files.iter().map(String::as_str))
        .into_iter()
        .filter(|plane| !matches!(plane, Plane::Docs | Plane::Github | Plane::WorkItems))
        .collect::<BTreeSet<_>>();
    (config.require_for_multi_plane && non_trivial_planes.len() > 1)
        || config
            .require_for_risk_classes
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(risk_class))
}

fn matching_execution_artifact_dirs(root: &Path, issue_id: u64) -> Result<Vec<PathBuf>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let prefix = format!("{issue_id}-");
    let mut matches = Vec::new();
    for entry in fs::read_dir(root).map_err(|error| {
        format!(
            "failed to read execution artifact root `{}`: {error}",
            root.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read execution artifact entry in `{}`: {error}",
                root.display()
            )
        })?;
        if entry
            .file_type()
            .map_err(|error| format!("failed to inspect `{}`: {error}", entry.path().display()))?
            .is_dir()
        {
            let name = entry.file_name();
            if name.to_string_lossy().starts_with(&prefix) {
                matches.push(entry.path());
            }
        }
    }

    matches.sort();
    Ok(matches)
}

fn validate_task_contract_file(
    workspace_root: &Path,
    config: &GovernanceConfig,
    task_contract_path: &Path,
    exec_plan_path: &Path,
    input: &ExecutionArtifactsValidationInput,
    require_artifacts: bool,
) -> Result<(), String> {
    let schema_path = workspace_root.join("schemas/contracts/v1/task-contract-v1.json");
    let schema_raw = fs::read_to_string(&schema_path)
        .map_err(|error| format!("failed to read `{}`: {error}", schema_path.display()))?;
    let schema_json: Value = serde_json::from_str(&schema_raw)
        .map_err(|error| format!("failed to parse `{}`: {error}", schema_path.display()))?;
    let validator =
        validator_for(&schema_json).map_err(|error| format!("schema error: {error}"))?;

    let raw = fs::read_to_string(task_contract_path)
        .map_err(|error| format!("failed to read `{}`: {error}", task_contract_path.display()))?;
    let json_value: Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "failed to parse `{}`: {error}",
            task_contract_path.display()
        )
    })?;
    let schema_errors = validator.iter_errors(&json_value).collect::<Vec<_>>();
    if !schema_errors.is_empty() {
        let details = schema_errors
            .into_iter()
            .map(|error| format!("{}: {}", error.instance_path(), error))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!(
            "task contract `{}` failed schema validation: {details}",
            relative_path(workspace_root, task_contract_path)
        ));
    }

    let contract: TaskContractDocument = serde_json::from_value(json_value).map_err(|error| {
        format!(
            "failed to deserialize task contract `{}`: {error}",
            relative_path(workspace_root, task_contract_path)
        )
    })?;
    if contract.issue_id != input.issue_id {
        return Err(format!(
            "task contract `{}` must declare issue id `#{}`, found `#{}`",
            relative_path(workspace_root, task_contract_path),
            input.issue_id,
            contract.issue_id
        ));
    }
    if contract.branch != input.branch {
        return Err(format!(
            "task contract `{}` must declare branch `{}`, found `{}`",
            relative_path(workspace_root, task_contract_path),
            input.branch,
            contract.branch
        ));
    }
    let expected_issue_url = format!(
        "https://github.com/{}/issues/{}",
        default_repository(config),
        input.issue_id
    );
    if contract.issue_url != expected_issue_url {
        return Err(format!(
            "task contract `{}` must declare issue URL `{expected_issue_url}`, found `{}`",
            relative_path(workspace_root, task_contract_path),
            contract.issue_url
        ));
    }
    let expected_exec_plan_path = relative_path(workspace_root, exec_plan_path);
    if contract.exec_plan_path != expected_exec_plan_path {
        return Err(format!(
            "task contract `{}` must point `exec_plan_path` at `{expected_exec_plan_path}`, found `{}`",
            relative_path(workspace_root, task_contract_path),
            contract.exec_plan_path
        ));
    }
    if require_artifacts && !contract.exec_plan_required {
        return Err(format!(
            "task contract `{}` must set `exec_plan_required` to true when execution artifacts are required",
            relative_path(workspace_root, task_contract_path)
        ));
    }

    Ok(())
}

fn validate_exec_plan_file(path: &Path) -> Result<(), String> {
    const REQUIRED_HEADINGS: [&str; 7] = [
        "Summary",
        "Task Contract",
        "Scope Boundaries",
        "Implementation Slices",
        "Validation Plan",
        "Rollout and Rollback",
        "Open Questions",
    ];

    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let headings = raw
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .collect::<Vec<_>>();
    if headings != REQUIRED_HEADINGS {
        return Err(format!(
            "execution plan `{}` must use exact headings {:?}, found {:?}",
            path.display(),
            REQUIRED_HEADINGS,
            headings
        ));
    }

    for heading in REQUIRED_HEADINGS {
        match markdown_section(&raw, heading) {
            Some(contents) if !contents.trim().is_empty() => {}
            _ => {
                return Err(format!(
                    "execution plan `{}` must include non-empty `{heading}` content",
                    path.display()
                ))
            }
        }
    }

    Ok(())
}

fn markdown_section(body: &str, heading: &str) -> Option<String> {
    let heading_prefix = format!("## {heading}");
    let mut capture = false;
    let mut lines = Vec::new();
    for line in body.lines() {
        if line.trim() == heading_prefix {
            capture = true;
            continue;
        }
        if capture && line.starts_with("## ") {
            break;
        }
        if capture {
            lines.push(line);
        }
    }

    if capture {
        Some(lines.join("\n").trim().to_string())
    } else {
        None
    }
}

fn changed_files_between(base: &str, head: &str) -> Result<Vec<String>, String> {
    let workspace_root = workspace_root()?;
    let output = Command::new("git")
        .current_dir(workspace_root)
        .args(["diff", "--name-only", &format!("{base}..{head}")])
        .output()
        .map_err(|error| format!("failed to run `git diff --name-only {base}..{head}`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git diff --name-only {base}..{head}` exited with status {}",
            output.status
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn default_repository(config: &GovernanceConfig) -> String {
    if let Some(repository) = config.repositories.first() {
        repository_full_name(config, repository)
    } else {
        format!("{}/origin", config.organization.login)
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
    "\
usage: cargo xtask github <subcommand> ...

Subcommands:
  sync           Sync repository governance settings from .github/governance.toml
  validate-execution-artifacts  Validate plan/task-contract artifacts for an issue branch
  validate-pr    Validate pull request title, branch, and issue linkage
  audit-process  Audit contributor docs, governance config, and workflow enforcement
"
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{
        audit_adr_corpus, branch_ruleset_payload, extract_trigger_names, load_config,
        main_ruleset_payload, render_drift_matrix_markdown, split_front_matter,
        validate_exec_plan_file, validate_execution_artifacts_in_workspace, validate_pr_event,
        DriftRow, ExecutionArtifactsValidationInput, PullRequestEvent,
    };
    use serde_yaml_ng::Value as YamlValue;
    use std::fs;
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
        assert_eq!(config.execution_artifacts.root, "plans");
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
            body: "## Summary\nAdd provider\n\n## Linked Issue\nCloses #142\n\n## Execution Artifacts\n- Not required for this single-plane low-risk change.\n\n## ADR References\n- ADR-0015\n\n## Impacted Domains\n- platform\n\n## Layers Touched\n- platform\n\n## Contracts Changed\n- None.\n\n## Tests Added or Updated\n- cargo test\n\n## Refreshed from Main\n- yes\n\n## Risk Class\n- low\n\n## Affected Consistency Class\n- Class B\n\n## Affected Risk Tier\n- low\n\n## Architecture Delta\n- Single-plane platform change.\n\n## Workflow Checklist\n- [x] refreshed\n\n## Technical Changes\n- added provider\n\n## Testing Strategy\n- cargo test\n\n## Rollback Path\n- revert the provider integration commit\n\n## Validation Artifacts\n- cargo test\n\n## Deployment Impact\n- none\n".to_owned(),
            branch: "feature/142-surrealdb-provider".to_owned(),
            repository: "shortorigin/origin".to_owned(),
            base_sha: None,
            head_sha: None,
            changed_files: vec!["platform/sdk/sdk-rs/src/lib.rs".to_owned()],
        };

        validate_pr_event(&config, &event)
            .expect("valid PR data should pass governance validation");
    }

    #[test]
    fn validate_pr_rejects_missing_issue_reference() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let event = PullRequestEvent {
            title: "feat(db): add provider".to_owned(),
            body: "## Summary\nAdd provider\n\n## Linked Issue\nTBD\n\n## Execution Artifacts\n- Not required for this single-plane low-risk change.\n\n## ADR References\n- ADR-0015\n\n## Impacted Domains\n- platform\n\n## Layers Touched\n- platform\n\n## Contracts Changed\n- None.\n\n## Tests Added or Updated\n- cargo test\n\n## Refreshed from Main\n- yes\n\n## Risk Class\n- low\n\n## Affected Consistency Class\n- Class B\n\n## Affected Risk Tier\n- low\n\n## Architecture Delta\n- Single-plane platform change.\n\n## Workflow Checklist\n- [x] refreshed\n\n## Technical Changes\n- added provider\n\n## Testing Strategy\n- cargo test\n\n## Rollback Path\n- revert the provider integration commit\n\n## Validation Artifacts\n- cargo test\n\n## Deployment Impact\n- none\n".to_owned(),
            branch: "feature/142-surrealdb-provider".to_owned(),
            repository: "shortorigin/origin".to_owned(),
            base_sha: None,
            head_sha: None,
            changed_files: vec!["platform/sdk/sdk-rs/src/lib.rs".to_owned()],
        };

        let error = validate_pr_event(&config, &event)
            .expect_err("missing issue reference should fail governance validation");
        assert!(error.contains("Linked Issue section"));
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

    #[test]
    fn extract_trigger_names_reads_mapping_keys() {
        let yaml: YamlValue = serde_yaml_ng::from_str(
            "pull_request:\npush:\n  branches:\n    - main\nworkflow_dispatch:\n",
        )
        .expect("yaml");
        let triggers = extract_trigger_names(&yaml);
        assert!(triggers.contains(&"pull_request".to_string()));
        assert!(triggers.contains(&"push".to_string()));
        assert!(triggers.contains(&"workflow_dispatch".to_string()));
    }

    #[test]
    fn drift_matrix_markdown_renders_expected_columns() {
        let markdown = render_drift_matrix_markdown(&[DriftRow {
            expectation: "required check".to_string(),
            documented_source: "docs".to_string(),
            automation_source: "workflow".to_string(),
            status: "pass".to_string(),
            details: "found".to_string(),
        }]);
        assert!(markdown.contains(
            "| Expectation | Documented Source | Automation Source | Status | Details |"
        ));
        assert!(markdown.contains("| required check | docs | workflow | pass | found |"));
    }

    #[test]
    fn validate_pr_rejects_multi_plane_delta_placeholder() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let event = PullRequestEvent {
            title: "refactor(platform): align shell boundary".to_owned(),
            body: "## Summary\nAlign boundaries\n\n## Linked Issue\nCloses #89\n\n## Execution Artifacts\n- Required for this high-risk multi-plane change.\n\n## ADR References\n- ADR-0004\n- ADR-0015\n\n## Impacted Domains\n- platform-shell\n\n## Layers Touched\n- platform\n- ui\n\n## Contracts Changed\n- plugin manifest\n\n## Tests Added or Updated\n- cargo test\n\n## Refreshed from Main\n- yes\n\n## Risk Class\n- high\n\n## Affected Consistency Class\n- Class B\n\n## Affected Risk Tier\n- high\n\n## Architecture Delta\n- Single-plane change.\n\n## Workflow Checklist\n- [x] refreshed\n\n## Technical Changes\n- aligned layers\n\n## Testing Strategy\n- cargo test\n\n## Rollback Path\n- revert the shell boundary refactor\n\n## Validation Artifacts\n- cargo test\n\n## Deployment Impact\n- none\n".to_owned(),
            branch: "refactor/89-platform-boundary".to_owned(),
            repository: "shortorigin/origin".to_owned(),
            base_sha: None,
            head_sha: None,
            changed_files: vec![
                "platform/sdk/sdk-rs/src/lib.rs".to_owned(),
                "ui/crates/site/src/lib.rs".to_owned(),
            ],
        };

        let error = validate_pr_event(&config, &event)
            .expect_err("placeholder architecture delta should fail for multi-plane change");
        assert!(error.contains("Architecture Delta"));
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "xtask-github-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock drift")
                .as_nanos()
        ));
        fs::create_dir_all(&base).expect("create temp dir");
        base
    }

    fn write_file(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }

    fn make_temp_workspace() -> PathBuf {
        let workspace = unique_temp_dir("workspace");
        let schema = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace root")
                .join("schemas/contracts/v1/task-contract-v1.json"),
        )
        .expect("read task contract schema");
        write_file(
            workspace.join("schemas/contracts/v1/task-contract-v1.json"),
            &schema,
        );
        fs::create_dir_all(workspace.join("plans")).expect("create plans root");
        workspace
    }

    fn write_plan_bundle(
        workspace: &std::path::Path,
        dir_name: &str,
        task_contract: &str,
        exec_plan: &str,
    ) {
        write_file(
            workspace.join(format!("plans/{dir_name}/task-contract.json")),
            task_contract,
        );
        write_file(
            workspace.join(format!("plans/{dir_name}/EXEC_PLAN.md")),
            exec_plan,
        );
    }

    fn valid_exec_plan() -> &'static str {
        "# Execution Plan\n\n## Summary\n- Summary.\n\n## Task Contract\n- Contract.\n\n## Scope Boundaries\n- Boundaries.\n\n## Implementation Slices\n- Slice.\n\n## Validation Plan\n- Validate.\n\n## Rollout and Rollback\n- Roll back.\n\n## Open Questions\n- None.\n"
    }

    fn valid_task_contract(issue_id: u64, dir_name: &str, branch: &str) -> String {
        format!(
            "{{\n  \"issue_id\": {issue_id},\n  \"issue_url\": \"https://github.com/shortorigin/origin/issues/{issue_id}\",\n  \"branch\": \"{branch}\",\n  \"primary_architectural_plane\": \"cross-layer\",\n  \"owning_subsystem\": \"xtask governance\",\n  \"architectural_references\": [\"docs/adr/0015-gitops-and-policy-as-code-control-artifacts.md\"],\n  \"allowed_touchpoints\": [\"xtask/\", \"plans/\"],\n  \"non_goals\": [\"no runtime redesign\"],\n  \"scope_in\": [\"validate execution artifacts\"],\n  \"scope_out\": [\"unrelated work\"],\n  \"target_paths\": [\"xtask/\", \"plans/\"],\n  \"acceptance_criteria\": [\"artifacts validate\"],\n  \"validation_commands\": [\"cargo xtask verify profile repo\"],\n  \"validation_artifacts\": [\"passing xtask output\"],\n  \"rollback_path\": \"revert the workflow changes\",\n  \"exec_plan_required\": true,\n  \"exec_plan_path\": \"plans/{dir_name}/EXEC_PLAN.md\"\n}}\n"
        )
    }

    #[test]
    fn validate_pr_rejects_branch_issue_id_mismatch() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let event = PullRequestEvent {
            title: "fix(governance): tighten traceability".to_owned(),
            body: "## Summary\nTighten traceability\n\n## Linked Issue\nCloses #118\n\n## Execution Artifacts\n- Not required for this single-plane low-risk change.\n\n## ADR References\n- ADR-0015\n\n## Impacted Domains\n- governance\n\n## Layers Touched\n- .github\n\n## Contracts Changed\n- None.\n\n## Tests Added or Updated\n- cargo test\n\n## Refreshed from Main\n- yes\n\n## Risk Class\n- low\n\n## Affected Consistency Class\n- Not Applicable\n\n## Affected Risk Tier\n- low\n\n## Architecture Delta\n- Single-plane governance change.\n\n## Workflow Checklist\n- [x] refreshed\n\n## Technical Changes\n- tightened checks\n\n## Testing Strategy\n- cargo test\n\n## Rollback Path\n- revert\n\n## Validation Artifacts\n- cargo test\n\n## Deployment Impact\n- none\n".to_owned(),
            branch: "fix/117-traceability-check".to_owned(),
            repository: "shortorigin/origin".to_owned(),
            base_sha: None,
            head_sha: None,
            changed_files: vec![".github/PULL_REQUEST_TEMPLATE.md".to_owned()],
        };

        let error = validate_pr_event(&config, &event)
            .expect_err("mismatched issue id should fail validation");
        assert!(error.contains("branch issue id"));
    }

    #[test]
    fn validate_execution_artifacts_rejects_missing_required_bundle() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let workspace = make_temp_workspace();
        let input = ExecutionArtifactsValidationInput {
            issue_id: 117,
            branch: "infra/117-execution-discipline-traceability".to_owned(),
            risk_class: "high".to_owned(),
            changed_files: vec![
                "xtask/src/github.rs".to_owned(),
                "schemas/contracts/v1/task-contract-v1.json".to_owned(),
            ],
        };

        let error = validate_execution_artifacts_in_workspace(&workspace, &config, &input)
            .expect_err("missing required artifacts should fail");
        assert!(error.contains("expected exactly one directory"));
    }

    #[test]
    fn validate_execution_artifacts_accepts_optional_single_plane_bundle() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let workspace = make_temp_workspace();
        write_plan_bundle(
            &workspace,
            "142-surrealdb-provider",
            &valid_task_contract(
                142,
                "142-surrealdb-provider",
                "feature/142-surrealdb-provider",
            ),
            valid_exec_plan(),
        );
        let input = ExecutionArtifactsValidationInput {
            issue_id: 142,
            branch: "feature/142-surrealdb-provider".to_owned(),
            risk_class: "medium".to_owned(),
            changed_files: vec!["platform/sdk/sdk-rs/src/lib.rs".to_owned()],
        };

        validate_execution_artifacts_in_workspace(&workspace, &config, &input)
            .expect("optional single-plane bundle should validate");
    }

    #[test]
    fn validate_execution_artifacts_rejects_mismatched_exec_plan_path() {
        let config = load_config(&config_path()).expect("governance config should parse");
        let workspace = make_temp_workspace();
        let broken_task_contract = valid_task_contract(
            117,
            "117-execution-discipline-traceability",
            "infra/117-execution-discipline-traceability",
        )
        .replace(
            "\"plans/117-execution-discipline-traceability/EXEC_PLAN.md\"",
            "\"plans/117-execution-discipline-traceability/WRONG.md\"",
        );
        write_plan_bundle(
            &workspace,
            "117-execution-discipline-traceability",
            &broken_task_contract,
            valid_exec_plan(),
        );
        let input = ExecutionArtifactsValidationInput {
            issue_id: 117,
            branch: "infra/117-execution-discipline-traceability".to_owned(),
            risk_class: "high".to_owned(),
            changed_files: vec![
                "xtask/src/github.rs".to_owned(),
                "schemas/contracts/v1/task-contract-v1.json".to_owned(),
            ],
        };

        let error = validate_execution_artifacts_in_workspace(&workspace, &config, &input)
            .expect_err("mismatched exec plan path should fail");
        assert!(error.contains("exec_plan_path"));
    }

    #[test]
    fn validate_exec_plan_rejects_invalid_heading_set() {
        let workspace = unique_temp_dir("exec-plan");
        let path = workspace.join("EXEC_PLAN.md");
        write_file(
            path.clone(),
            "# Execution Plan\n\n## Summary\n- Summary.\n\n## Task Contract\n- Contract.\n\n## Wrong Heading\n- Wrong.\n",
        );

        let error = validate_exec_plan_file(&path).expect_err("invalid heading set should fail");
        assert!(error.contains("exact headings"));
    }

    #[test]
    fn split_front_matter_reads_yaml_and_body() {
        let raw = "---\nadr_id: ADR-0001\nstatus: Adopted\n---\n# Title\n\n## Context\ntext\n";
        let (front_matter, body) = split_front_matter(raw).expect("front matter should parse");
        assert!(front_matter.contains("adr_id: ADR-0001"));
        assert!(body.starts_with("# Title"));
    }

    #[test]
    fn adr_corpus_audit_passes_for_workspace() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask crate should be nested under the workspace root")
            .to_path_buf();
        let defects = audit_adr_corpus(&workspace_root).expect("ADR corpus audit should run");
        assert!(defects.is_empty(), "ADR corpus defects: {defects:?}");
    }
}
