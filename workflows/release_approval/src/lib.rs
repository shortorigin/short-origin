use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "release_approval".to_owned(),
        touched_domains: vec!["engineering_platform".to_owned(), "security".to_owned()],
        target_services: vec![
            "engineering-service".to_owned(),
            "security-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
