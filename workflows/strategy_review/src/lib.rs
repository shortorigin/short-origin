use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "strategy_review".to_owned(),
        touched_domains: vec!["strategy_governance".to_owned()],
        target_services: vec!["governance-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
