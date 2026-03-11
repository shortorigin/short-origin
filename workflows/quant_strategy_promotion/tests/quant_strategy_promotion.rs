use approval_service::ApprovalService;
use chrono::Utc;
use compliance_attestation::workflow_boundary as compliance_attestation_boundary;
use compliance_service::ComplianceService;
use contracts::{
    AgentActionRequestV1, ApprovalDecisionV1, Classification, ImpactTier,
    QuantStrategyPromotionRequestV1,
};
use evidence_service::EvidenceService;
use governance_service::GovernanceService;
use identity::{ActorRef, InstitutionalRole};
use market_data_service::MarketDataService;
use orchestrator::WorkflowEngine;
use policy_service::PolicyService;
use quant_research_service::QuantResearchService;
use quant_strategy_promotion::{PipelineSummary, execute};
use strategy_review::workflow_boundary as strategy_review_boundary;

fn assert_compliance_report_semantics(
    actual: &contracts::ComplianceReportV1,
    expected: &contracts::ComplianceReportV1,
) {
    assert_eq!(
        actual
            .order_audit_records
            .iter()
            .map(|record| {
                (
                    record.order.strategy_id.clone(),
                    record.order.symbol.clone(),
                    record.order.venue,
                    record.order.side,
                    record.order.quantity,
                    record.order.limit_price,
                    record.order.order_type,
                    record.order.tif,
                    record.decision_trace.clone(),
                )
            })
            .collect::<Vec<_>>(),
        expected
            .order_audit_records
            .iter()
            .map(|record| {
                (
                    record.order.strategy_id.clone(),
                    record.order.symbol.clone(),
                    record.order.venue,
                    record.order.side,
                    record.order.quantity,
                    record.order.limit_price,
                    record.order.order_type,
                    record.order.tif,
                    record.decision_trace.clone(),
                )
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual
            .limit_breach_records
            .iter()
            .map(|record| {
                (
                    record.control.clone(),
                    record.severity.clone(),
                    record.details.clone(),
                )
            })
            .collect::<Vec<_>>(),
        expected
            .limit_breach_records
            .iter()
            .map(|record| {
                (
                    record.control.clone(),
                    record.severity.clone(),
                    record.details.clone(),
                )
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual
            .best_execution_records
            .iter()
            .map(|record| {
                (
                    record.venue,
                    record.slippage_bps,
                    record.expected_price,
                    record.executed_price,
                )
            })
            .collect::<Vec<_>>(),
        expected
            .best_execution_records
            .iter()
            .map(|record| {
                (
                    record.venue,
                    record.slippage_bps,
                    record.expected_price,
                    record.executed_price,
                )
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        actual.compliance_report_fields(),
        expected.compliance_report_fields()
    );
}

trait ComplianceReportParity {
    fn compliance_report_fields(&self) -> (String, Vec<String>, Vec<String>, Vec<String>);
}

impl ComplianceReportParity for contracts::ComplianceReportV1 {
    fn compliance_report_fields(&self) -> (String, Vec<String>, Vec<String>, Vec<String>) {
        let mut controls_checked = self.daily_control_attestation.controls_checked.clone();
        controls_checked.sort();
        (
            self.daily_control_attestation.business_date.clone(),
            self.daily_control_attestation.approved_models.clone(),
            controls_checked,
            self.daily_control_attestation.exceptions.clone(),
        )
    }
}

fn build_action() -> AgentActionRequestV1 {
    AgentActionRequestV1 {
        action_id: "action::quant::promotion".into(),
        actor_ref: ActorRef("agent.quant_research".to_string()),
        objective: "Promote validated trading strategy".to_string(),
        requested_workflow: "quant_strategy_promotion".into(),
        impact_tier: ImpactTier::Tier3,
        classification: Classification::Restricted,
        required_approver_roles: vec![
            InstitutionalRole::InstitutionalCouncil,
            InstitutionalRole::ChiefComplianceOfficer,
        ],
        policy_refs: vec!["capital.markets.promotion.v1".to_string()],
    }
}

fn build_request() -> QuantStrategyPromotionRequestV1 {
    QuantStrategyPromotionRequestV1 {
        promotion_id: "promotion::trend_follower".to_string(),
        strategy_id: "trend_follower".to_string(),
        business_date: "2026-03-05".to_string(),
        seed: 20_260_305,
        configuration_ref: "config/trend_follower".to_string(),
    }
}

#[tokio::test]
async fn quant_strategy_promotion_matches_fixture_summary_and_gate() {
    let action = build_action();
    let request = build_request();
    let mut approval_service = ApprovalService::default();
    let decided_at = Utc::now();
    approval_service.record_decision(ApprovalDecisionV1 {
        action_id: action.action_id.clone(),
        approver: ActorRef("human.council".to_string()),
        approver_role: InstitutionalRole::InstitutionalCouncil,
        approved: true,
        rationale: "Promotion evidence reviewed".to_string(),
        decided_at,
    });
    approval_service.record_decision(ApprovalDecisionV1 {
        action_id: action.action_id.clone(),
        approver: ActorRef("human.cco".to_string()),
        approver_role: InstitutionalRole::ChiefComplianceOfficer,
        approved: true,
        rationale: "Compliance signoff granted".to_string(),
        decided_at,
    });

    let policy_service = PolicyService::institutional_default();
    let evidence_sink = EvidenceService::default();
    let mut engine = WorkflowEngine::new(policy_service, approval_service, evidence_sink);
    let mut governance_service = GovernanceService::default();
    let mut compliance_service = ComplianceService::default();
    let mut audit_service = EvidenceService::default();
    let mut market_data_service = MarketDataService::default();
    let mut research_service = QuantResearchService::default();

    let report = execute(
        &mut engine,
        &mut governance_service,
        &mut compliance_service,
        &mut audit_service,
        &mut market_data_service,
        &mut research_service,
        &action,
        request,
    )
    .await
    .expect("promotion workflow");

    let summary_fixture: PipelineSummary = serde_json::from_str(include_str!(
        "../../../testing/fixtures/finance/run-2026-03-05/pipeline_summary.json"
    ))
    .expect("summary fixture");
    let gate_fixture: contracts::PromotionGateV1 = serde_json::from_str(include_str!(
        "../../../testing/fixtures/finance/run-2026-03-05/promotion_gate.json"
    ))
    .expect("gate fixture");
    let compliance_fixture: contracts::ComplianceReportV1 = serde_json::from_str(include_str!(
        "../../../testing/fixtures/finance/run-2026-03-05/compliance_report.json"
    ))
    .expect("compliance fixture");

    assert_eq!(report.summary, summary_fixture);
    assert_eq!(report.promotion_gate, gate_fixture);
    assert_eq!(report.recommendation.required_workflows.len(), 2);
    assert!(
        report
            .recommendation
            .required_workflows
            .contains(&strategy_review_boundary().workflow_name)
    );
    assert!(
        report
            .recommendation
            .required_workflows
            .contains(&compliance_attestation_boundary().workflow_name)
    );
    assert_compliance_report_semantics(&report.compliance_report, &compliance_fixture);
    assert_eq!(engine.recorded_evidence().await.unwrap().len(), 1);
    assert_eq!(governance_service.recommendations().len(), 1);
    assert_eq!(compliance_service.reports().len(), 1);
    assert_eq!(audit_service.audit_events().len(), 2);
}
