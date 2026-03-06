pub use orchestrator::WorkflowEngine;

use contracts::WorkflowBoundaryV1;

#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "engine".to_owned(),
        touched_domains: vec!["strategy_governance".to_owned()],
        target_services: vec!["policy-service".to_owned(), "approval-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: true,
    }
}
