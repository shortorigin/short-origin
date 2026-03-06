use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "incident_response".to_owned(),
        touched_domains: vec![
            "operations".to_owned(),
            "security".to_owned(),
            "resilience_continuity".to_owned(),
        ],
        target_services: vec![
            "operations-service".to_owned(),
            "security-service".to_owned(),
            "resilience-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
