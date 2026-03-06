use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "control_testing".to_owned(),
        touched_domains: vec!["audit_assurance".to_owned(), "compliance".to_owned()],
        target_services: vec!["audit-service".to_owned(), "compliance-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
