use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "access_review".to_owned(),
        touched_domains: vec!["security".to_owned(), "infrastructure_it".to_owned()],
        target_services: vec!["security-service".to_owned(), "identity-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
