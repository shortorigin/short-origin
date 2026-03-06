use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "vendor_onboarding".to_owned(),
        touched_domains: vec!["procurement_vendor".to_owned(), "security".to_owned()],
        target_services: vec![
            "procurement-service".to_owned(),
            "security-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
