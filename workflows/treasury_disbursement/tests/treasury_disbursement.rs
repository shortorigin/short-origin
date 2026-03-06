use approval_service::ApprovalService;
use chrono::Utc;
use contracts::{
    AgentActionRequestV1, ApprovalDecisionV1, Classification, ImpactTier,
    TreasuryDisbursementRequestV1,
};
use evidence_service::EvidenceService;
use finance_service::FinanceService;
use identity::{ActorRef, InstitutionalRole};
use orchestrator::WorkflowEngine;
use policy_service::PolicyService;
use treasury_disbursement::execute;

fn build_action(action_id: &str) -> AgentActionRequestV1 {
    AgentActionRequestV1 {
        action_id: action_id.to_owned(),
        actor_ref: ActorRef("agent.finance_analyst".to_owned()),
        objective: "Release approved treasury disbursement".to_owned(),
        requested_workflow: "treasury_disbursement".to_owned(),
        impact_tier: ImpactTier::Tier3,
        classification: Classification::Restricted,
        required_approver_roles: vec![InstitutionalRole::CFO, InstitutionalRole::CHRO],
        policy_refs: vec!["finance.treasury.disbursement.v1".to_owned()],
    }
}

fn build_request() -> TreasuryDisbursementRequestV1 {
    TreasuryDisbursementRequestV1 {
        ledger_ref: "ledger/treasury/main".to_owned(),
        amount_minor: 25_000,
        currency: "USD".to_owned(),
        beneficiary: "vendor:critical-recovery-provider".to_owned(),
        justification: "Approved resilience spend".to_owned(),
    }
}

#[test]
fn treasury_disbursement_rejects_missing_approval() {
    let policy_service = PolicyService::institutional_default();
    let approval_service = ApprovalService::default();
    let evidence_service = EvidenceService::default();
    let mut engine = WorkflowEngine::new(policy_service, approval_service, evidence_service);
    let mut finance_service = FinanceService::default();

    let result = execute(
        &mut engine,
        &mut finance_service,
        &build_action("action::treasury::missing"),
        build_request(),
    );

    assert!(matches!(
        result,
        Err(error_model::InstitutionalError::ApprovalMissing { .. })
    ));
    assert_eq!(finance_service.disbursements().len(), 0);
}

#[test]
fn treasury_disbursement_accepts_policy_and_dual_approval() {
    let policy_service = PolicyService::institutional_default();
    let mut approval_service = ApprovalService::default();
    let evidence_service = EvidenceService::default();
    let action = build_action("action::treasury::approved");
    let request = build_request();
    let decided_at = Utc::now();

    approval_service.record_decision(ApprovalDecisionV1 {
        action_id: action.action_id.clone(),
        approver: ActorRef("human.cfo".to_owned()),
        approver_role: InstitutionalRole::CFO,
        approved: true,
        rationale: "Treasury allocation approved".to_owned(),
        decided_at,
    });
    approval_service.record_decision(ApprovalDecisionV1 {
        action_id: action.action_id.clone(),
        approver: ActorRef("human.chro".to_owned()),
        approver_role: InstitutionalRole::CHRO,
        approved: true,
        rationale: "Compensation impact reviewed".to_owned(),
        decided_at,
    });

    let mut engine = WorkflowEngine::new(policy_service, approval_service, evidence_service);
    let mut finance_service = FinanceService::default();
    let result = execute(&mut engine, &mut finance_service, &action, request).unwrap();

    assert_eq!(result.amount_minor, 25_000);
    assert_eq!(result.approved_by_roles.len(), 2);
    assert_eq!(finance_service.disbursements().len(), 1);
    assert_eq!(engine.recorded_evidence().len(), 1);
}
