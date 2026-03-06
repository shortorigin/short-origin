use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "payroll".to_owned(),
        touched_domains: vec!["finance_treasury".to_owned(), "hr_talent".to_owned()],
        target_services: vec!["finance-service".to_owned(), "hr-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
