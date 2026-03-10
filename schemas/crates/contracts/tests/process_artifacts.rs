use chrono::{TimeZone, Utc};
use contracts::{
    ArchitectureDesignV1, ChangeBatchV1, ImplementationPlanV1, OpenQuestionV1,
    ProcessTraceabilityV1, ProfileEvidenceV1, QuestionSeverityV1, RefinementRecordV1,
    RequirementsSpecV1, ResearchSynthesisV1, TaskContractV1, ValidationReportV1,
    VerificationStatusV1, WorkItemV1,
};

fn traceability(work_item_id: &str) -> ProcessTraceabilityV1 {
    ProcessTraceabilityV1 {
        work_item_id: work_item_id.to_owned(),
        parent_work_item_id: Some("short-origin-root".to_owned()),
        iteration: 1,
        affected_paths: vec!["schemas/contracts/v1".to_owned()],
        affected_modules: vec!["schemas".to_owned()],
        policy_refs: vec!["enterprise.policies.invariants".to_owned()],
        acceptance_criteria: vec!["round-trip all process contracts".to_owned()],
        open_questions: vec![OpenQuestionV1 {
            question: "No blocking questions remain".to_owned(),
            severity: QuestionSeverityV1::Low,
        }],
        verification_status: VerificationStatusV1::Passed,
    }
}

fn profile_evidence(area: &str) -> ProfileEvidenceV1 {
    ProfileEvidenceV1 {
        area: area.to_owned(),
        notes: vec!["covered".to_owned()],
    }
}

#[test]
fn process_artifacts_round_trip_through_json() {
    let completed_at = Utc
        .with_ymd_and_hms(2026, 3, 5, 18, 0, 0)
        .single()
        .expect("valid timestamp");

    let work_item = WorkItemV1 {
        traceability: traceability("short-origin-process"),
        root_work_item_id: "short-origin-process".to_owned(),
        title: "Recursive process contracts".to_owned(),
        objective: "Define auditable process artifacts".to_owned(),
        status: "completed".to_owned(),
        current_stage: "refinement".to_owned(),
        child_work_item_ids: vec!["short-origin-process-child".to_owned()],
        decomposition_reason: Some("cross-module initiative".to_owned()),
    };
    let task_contract = TaskContractV1 {
        issue_id: 117,
        issue_url: "https://github.com/shortorigin/origin/issues/117".to_owned(),
        branch: "infra/117-execution-discipline-traceability".to_owned(),
        primary_architectural_plane: "cross-layer".to_owned(),
        owning_subsystem: ".github governance and xtask validation".to_owned(),
        architectural_references: vec![
            "docs/adr/0015-gitops-and-policy-as-code-control-artifacts.md".to_owned(),
        ],
        allowed_touchpoints: vec![".github/".to_owned(), "xtask/".to_owned()],
        non_goals: vec!["do not replace github issues".to_owned()],
        scope_in: vec!["add execution artifacts".to_owned()],
        scope_out: vec!["runtime redesign".to_owned()],
        target_paths: vec!["plans/".to_owned(), "xtask/".to_owned()],
        acceptance_criteria: vec!["artifacts validate deterministically".to_owned()],
        validation_commands: vec!["cargo xtask verify profile repo".to_owned()],
        validation_artifacts: vec!["passing xtask output".to_owned()],
        rollback_path: "revert the repository-governance changes".to_owned(),
        exec_plan_required: true,
        exec_plan_path: "plans/117-execution-discipline-traceability/EXEC_PLAN.md".to_owned(),
    };
    let research = ResearchSynthesisV1 {
        traceability: traceability("short-origin-process"),
        completed_at,
        objective: "Research delivery model".to_owned(),
        research_inputs: vec!["docs/adr/0001".to_owned()],
        source_refs: vec!["docs/process/recursive-chatgpt-engineering.md".to_owned()],
        findings: vec!["contracts must be machine-readable".to_owned()],
        constraints: vec!["workflow boundaries stay authoritative".to_owned()],
        decomposition_signals: vec!["touches schemas, agents, docs, and ci".to_owned()],
    };
    let requirements = RequirementsSpecV1 {
        traceability: traceability("short-origin-process"),
        completed_at,
        objective: "Specify process outputs".to_owned(),
        functional_requirements: vec!["emit stage artifacts".to_owned()],
        non_functional_requirements: vec!["support CI validation".to_owned()],
        out_of_scope: vec!["runtime mutation by agents".to_owned()],
        success_metrics: vec!["all changed paths covered".to_owned()],
        assumptions: vec!["artifacts are committed in repo".to_owned()],
    };
    let architecture = ArchitectureDesignV1 {
        traceability: traceability("short-origin-process"),
        completed_at,
        objective: "Map process to workspace boundaries".to_owned(),
        design_summary: vec!["schemas own contracts".to_owned()],
        public_interface_changes: vec!["add process contracts".to_owned()],
        boundary_impacts: vec!["shared validator consumes schemas".to_owned()],
        decomposition_decision: "split".to_owned(),
        child_work_item_ids: vec!["short-origin-process-child".to_owned()],
    };
    let implementation_plan = ImplementationPlanV1 {
        traceability: traceability("short-origin-process"),
        completed_at,
        objective: "Plan schema slice".to_owned(),
        change_slices: vec!["add contracts and tests".to_owned()],
        target_paths: vec!["schemas/".to_owned()],
        test_scenarios: vec!["embed new contracts".to_owned()],
        profile_evidence: vec![
            profile_evidence("schemas.compatibility_notes"),
            profile_evidence("schemas.fixture_updates"),
        ],
        rollout_notes: vec!["merge before validator rollout".to_owned()],
    };
    let change_batch = ChangeBatchV1 {
        traceability: traceability("short-origin-process-child"),
        completed_at,
        batch_id: "batch-01".to_owned(),
        summary: vec!["created process schema files".to_owned()],
        target_paths: vec!["schemas/contracts/v1".to_owned()],
        change_kinds: vec!["contract_addition".to_owned()],
        prerequisite_checks: vec!["cargo test -p codegen".to_owned()],
    };
    let validation = ValidationReportV1 {
        traceability: traceability("short-origin-process-child"),
        completed_at,
        objective: "Validate schema slice".to_owned(),
        checks_run: vec!["cargo test -p contracts".to_owned()],
        passed_checks: vec!["serde round trip".to_owned()],
        failed_checks: Vec::new(),
        findings: Vec::new(),
        changed_paths_validated: vec!["schemas/contracts/v1/work-item-v1.json".to_owned()],
        profile_evidence: vec![
            profile_evidence("schemas.compatibility_notes"),
            profile_evidence("schemas.fixture_updates"),
        ],
    };
    let refinement = RefinementRecordV1 {
        traceability: traceability("short-origin-process-child"),
        completed_at,
        decision: "close".to_owned(),
        improvements: vec!["carry pattern into next slice".to_owned()],
        residual_risks: vec!["future profiles may add new keys".to_owned()],
        next_work_item_ids: Vec::new(),
    };

    let artifacts = [
        serde_json::to_value(&task_contract).expect("serialize task contract"),
        serde_json::to_value(&work_item).expect("serialize work item"),
        serde_json::to_value(&research).expect("serialize research"),
        serde_json::to_value(&requirements).expect("serialize requirements"),
        serde_json::to_value(&architecture).expect("serialize architecture"),
        serde_json::to_value(&implementation_plan).expect("serialize plan"),
        serde_json::to_value(&change_batch).expect("serialize batch"),
        serde_json::to_value(&validation).expect("serialize validation"),
        serde_json::to_value(&refinement).expect("serialize refinement"),
    ];

    let _: TaskContractV1 =
        serde_json::from_value(artifacts[0].clone()).expect("parse task contract");
    let _: WorkItemV1 = serde_json::from_value(artifacts[1].clone()).expect("parse work item");
    let _: ResearchSynthesisV1 =
        serde_json::from_value(artifacts[2].clone()).expect("parse research");
    let _: RequirementsSpecV1 =
        serde_json::from_value(artifacts[3].clone()).expect("parse requirements");
    let _: ArchitectureDesignV1 =
        serde_json::from_value(artifacts[4].clone()).expect("parse architecture");
    let _: ImplementationPlanV1 =
        serde_json::from_value(artifacts[5].clone()).expect("parse implementation plan");
    let _: ChangeBatchV1 =
        serde_json::from_value(artifacts[6].clone()).expect("parse change batch");
    let _: ValidationReportV1 =
        serde_json::from_value(artifacts[7].clone()).expect("parse validation report");
    let _: RefinementRecordV1 =
        serde_json::from_value(artifacts[8].clone()).expect("parse refinement record");
}

#[test]
fn high_severity_questions_are_detected() {
    let traceability = ProcessTraceabilityV1 {
        open_questions: vec![OpenQuestionV1 {
            question: "Awaiting contract owner sign-off".to_owned(),
            severity: QuestionSeverityV1::High,
        }],
        ..traceability("short-origin-process")
    };

    assert!(traceability.has_high_severity_open_questions());
}
