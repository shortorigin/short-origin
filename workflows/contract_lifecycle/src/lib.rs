use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "contract_lifecycle".to_owned(),
        touched_domains: vec!["legal".to_owned(), "sales_revenue".to_owned()],
        target_services: vec!["legal-service".to_owned(), "revenue-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
