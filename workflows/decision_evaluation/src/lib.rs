use contracts::{DecisionContextV1, RecommendationStatusV1, WorkflowBoundaryV1};
use error_model::InstitutionalResult;
use governance_service::{DecisionEvaluationReport, GovernanceService};

/// Read-only workflow report produced by `decision_evaluation`.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionEvaluationWorkflowReport {
    /// Structured evaluation report returned by governance-service.
    pub evaluation: DecisionEvaluationReport,
    /// Explicit workflow-level status.
    pub status: RecommendationStatusV1,
}

/// Returns the workflow boundary for quantitative decision evaluation.
#[must_use]
pub fn workflow_boundary() -> WorkflowBoundaryV1 {
    WorkflowBoundaryV1 {
        workflow_name: "decision_evaluation".to_owned(),
        touched_domains: vec!["strategy_governance".to_owned()],
        target_services: vec!["governance-service".to_owned()],
        emits_evidence: true,
        mutation_path_only: false,
    }
}

/// Executes the read-only decision-evaluation workflow.
pub fn execute(
    governance_service: &mut GovernanceService,
    context: DecisionContextV1,
) -> InstitutionalResult<DecisionEvaluationWorkflowReport> {
    let evaluation = governance_service.evaluate_decision(context)?;
    Ok(DecisionEvaluationWorkflowReport {
        status: evaluation.recommendation.status,
        evaluation,
    })
}
