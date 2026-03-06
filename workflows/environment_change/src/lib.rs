use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "environment_change".to_owned(),
        touched_domains: vec!["infrastructure_it".to_owned(), "security".to_owned()],
        target_services: vec![
            "infrastructure-service".to_owned(),
            "security-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
