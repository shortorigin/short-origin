use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "compliance_attestation".to_owned(),
        touched_domains: vec!["compliance".to_owned(), "audit_assurance".to_owned()],
        target_services: vec!["compliance-service".to_owned(), "audit-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
