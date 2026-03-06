use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "disaster_recovery_test".to_owned(),
        touched_domains: vec![
            "resilience_continuity".to_owned(),
            "infrastructure_it".to_owned(),
        ],
        target_services: vec![
            "resilience-service".to_owned(),
            "infrastructure-service".to_owned(),
        ],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
