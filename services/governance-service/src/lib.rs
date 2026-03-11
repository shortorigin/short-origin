use std::collections::BTreeMap;

use contracts::{
    DecisionAuditRecordV1, DecisionContextV1, DecisionRecommendationV1, ModelApprovalStatusV1,
    ModelApprovalV1, PolicyGateResultV1, PromotionRecommendationV1, ServiceBoundaryV1,
};
use decision_core::{BaselineDecisionEngine, DecisionEngine};
use enforcement::ApprovedMutationContext;
use error_model::InstitutionalResult;
use identity::{ServiceId, WorkflowId};

const SERVICE_NAME: &str = "governance-service";
const DOMAIN_NAME: &str = "strategy_governance";
const APPROVED_WORKFLOWS: &[&str] = &[
    "strategy_review",
    "policy_exception",
    "quant_strategy_promotion",
    "decision_evaluation",
];
const OWNED_AGGREGATES: &[&str] = &[
    "governance_decision",
    "institutional_invariant",
    "promotion_recommendation",
    "model_approval",
    "decision_recommendation",
    "decision_audit_record",
];

fn service_id() -> ServiceId {
    SERVICE_NAME.into()
}

fn quant_strategy_promotion_workflow_id() -> WorkflowId {
    "quant_strategy_promotion".into()
}

#[derive(Debug, Default, Clone)]
struct InMemoryGovernanceStore {
    approvals: BTreeMap<String, ModelApprovalV1>,
    recommendations: Vec<PromotionRecommendationV1>,
    decision_recommendations: Vec<DecisionRecommendationV1>,
    decision_audit_records: Vec<DecisionAuditRecordV1>,
}

impl InMemoryGovernanceStore {
    fn submit_model(&mut self, model_id: &str, version: &str, notes: &str) {
        self.approvals.insert(
            format!("{model_id}:{version}"),
            ModelApprovalV1 {
                model_id: model_id.to_string(),
                version: version.to_string(),
                approved_by: None,
                status: ModelApprovalStatusV1::Pending,
                notes: notes.to_string(),
            },
        );
    }

    fn approve_model(&mut self, model_id: &str, version: &str, reviewer: &str) {
        if let Some(model) = self.approvals.get_mut(&format!("{model_id}:{version}")) {
            model.status = ModelApprovalStatusV1::Approved;
            model.approved_by = Some(reviewer.to_string());
        }
    }

    fn approved_models(&self) -> Vec<String> {
        self.approvals
            .values()
            .filter(|model| model.status == ModelApprovalStatusV1::Approved)
            .map(|model| format!("{}:{}", model.model_id, model.version))
            .collect()
    }

    fn record_recommendation(&mut self, recommendation: PromotionRecommendationV1) {
        self.recommendations.push(recommendation);
    }

    fn recommendations(&self) -> &[PromotionRecommendationV1] {
        &self.recommendations
    }

    fn record_decision_evaluation(
        &mut self,
        recommendation: DecisionRecommendationV1,
        audit_record: DecisionAuditRecordV1,
    ) {
        self.decision_recommendations.push(recommendation);
        self.decision_audit_records.push(audit_record);
    }

    fn decision_recommendations(&self) -> &[DecisionRecommendationV1] {
        &self.decision_recommendations
    }

    fn decision_audit_records(&self) -> &[DecisionAuditRecordV1] {
        &self.decision_audit_records
    }
}

/// Structured report returned by pure decision evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionEvaluationReport {
    /// Ranked recommendation output.
    pub recommendation: DecisionRecommendationV1,
    /// Structured audit record for the evaluation.
    pub audit_record: DecisionAuditRecordV1,
    /// Explicit policy-gate result attached to the recommendation.
    pub gate_result: PolicyGateResultV1,
}

#[derive(Debug, Default, Clone)]
pub struct GovernanceService {
    store: InMemoryGovernanceStore,
}

impl GovernanceService {
    pub fn submit_model(&mut self, model_id: &str, version: &str, notes: &str) {
        self.store.submit_model(model_id, version, notes);
    }

    pub fn approve_model(&mut self, model_id: &str, version: &str, reviewer: &str) {
        self.store.approve_model(model_id, version, reviewer);
    }

    #[must_use]
    pub fn approved_models(&self) -> Vec<String> {
        self.store.approved_models()
    }

    pub fn record_recommendation(
        &mut self,
        context: &ApprovedMutationContext,
        recommendation: PromotionRecommendationV1,
    ) -> InstitutionalResult<PromotionRecommendationV1> {
        context.assert_workflow(&quant_strategy_promotion_workflow_id())?;
        context.assert_target_service(&service_id())?;
        self.store.record_recommendation(recommendation.clone());
        Ok(recommendation)
    }

    #[must_use]
    pub fn recommendations(&self) -> &[PromotionRecommendationV1] {
        self.store.recommendations()
    }

    pub fn evaluate_decision(
        &mut self,
        context: DecisionContextV1,
    ) -> InstitutionalResult<DecisionEvaluationReport> {
        let evaluation = BaselineDecisionEngine::default().evaluate(&context)?;
        self.store.record_decision_evaluation(
            evaluation.recommendation.clone(),
            evaluation.audit_record.clone(),
        );
        Ok(DecisionEvaluationReport {
            recommendation: evaluation.recommendation,
            audit_record: evaluation.audit_record,
            gate_result: evaluation.gate_result,
        })
    }

    #[must_use]
    pub fn decision_recommendations(&self) -> &[DecisionRecommendationV1] {
        self.store.decision_recommendations()
    }

    #[must_use]
    pub fn decision_audit_records(&self) -> &[DecisionAuditRecordV1] {
        self.store.decision_audit_records()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .copied()
            .map(str::to_owned)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    mod contract_parity {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/contract_parity.rs"
        ));
    }

    use contract_parity::assert_service_boundary_matches_catalog;

    use chrono::{TimeZone, Utc};
    use contracts::{
        ApprovalRequirementV1, DecisionClassV1, DecisionConstraintV1, DecisionContextV1,
        DecisionOptionV1, DecisionStateV1, OutcomeDistributionV1, OutcomeEstimateV1,
        OutcomeScenarioV1, ProvenanceV1, ReversibilityClassV1, RiskAssessmentV1, RiskTierV1,
        UtilityBreakdownV1,
    };
    use identity::{ActorRef, DecisionId};

    use super::{DOMAIN_NAME, GovernanceService, service_boundary};

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source =
            include_str!("../../../enterprise/domains/strategy_governance/service_boundaries.toml");
        let boundary = service_boundary();

        assert_service_boundary_matches_catalog(&boundary, DOMAIN_NAME, source);
    }

    fn decision_context() -> DecisionContextV1 {
        DecisionContextV1 {
            decision_id: DecisionId::from("decision::service"),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
                .single()
                .expect("timestamp"),
            decision_class: DecisionClassV1::ReleaseRiskAssessment,
            state: DecisionStateV1::Pending,
            actor_ref: ActorRef("agent.strategist".to_owned()),
            subject: "release".to_owned(),
            objective: "Choose release path".to_owned(),
            evaluation_seed: 23,
            risk_tier: RiskTierV1::Tier1,
            approval_requirement: ApprovalRequirementV1::None,
            policy_refs: vec!["policy.release.v1".to_owned()],
            reversibility: ReversibilityClassV1::GuardedRollback,
            requested_learned_support: false,
            options: vec![
                DecisionOptionV1 {
                    option_id: "staged".to_owned(),
                    title: "staged".to_owned(),
                    description: "staged".to_owned(),
                    expected_outcomes: vec![OutcomeEstimateV1 {
                        estimate_id: "estimate-staged".to_owned(),
                        description: "bounded".to_owned(),
                        probability: 0.8,
                        expected_utility: 0.8,
                        risk_adjustment: -0.1,
                        confidence: 0.8,
                        rationale: "historical".to_owned(),
                    }],
                    outcome_distribution: OutcomeDistributionV1 {
                        distribution_id: "distribution-staged".to_owned(),
                        scenarios: vec![
                            OutcomeScenarioV1 {
                                label: "smooth".to_owned(),
                                probability: 0.8,
                                utility: 0.9,
                                risk: 0.2,
                            },
                            OutcomeScenarioV1 {
                                label: "rollback".to_owned(),
                                probability: 0.2,
                                utility: 0.2,
                                risk: 0.5,
                            },
                        ],
                        expected_value: 0.76,
                        variance: 0.05,
                        downside_probability: 0.2,
                        rationale: "bounded".to_owned(),
                    },
                    risk_assessment: RiskAssessmentV1 {
                        risk_score: 0.3,
                        downside_probability: 0.2,
                        tail_risk_score: 0.3,
                        confidence: 0.8,
                        rationale: "controlled".to_owned(),
                        mitigation: "rollback".to_owned(),
                    },
                    utility_breakdown: UtilityBreakdownV1 {
                        value_score: 0.8,
                        resilience_score: 0.9,
                        compliance_score: 0.8,
                        cost_score: 0.6,
                        reversibility_score: 0.9,
                        strategic_fit_score: 0.8,
                        rationale: "balanced".to_owned(),
                    },
                    rollback_plan: "rollback".to_owned(),
                    reversibility: ReversibilityClassV1::GuardedRollback,
                    historical_successes: 8,
                    historical_failures: 2,
                },
                DecisionOptionV1 {
                    option_id: "immediate".to_owned(),
                    title: "immediate".to_owned(),
                    description: "immediate".to_owned(),
                    expected_outcomes: vec![OutcomeEstimateV1 {
                        estimate_id: "estimate-immediate".to_owned(),
                        description: "faster".to_owned(),
                        probability: 0.9,
                        expected_utility: 0.9,
                        risk_adjustment: -0.3,
                        confidence: 0.7,
                        rationale: "historical".to_owned(),
                    }],
                    outcome_distribution: OutcomeDistributionV1 {
                        distribution_id: "distribution-immediate".to_owned(),
                        scenarios: vec![
                            OutcomeScenarioV1 {
                                label: "fast".to_owned(),
                                probability: 0.7,
                                utility: 0.95,
                                risk: 0.5,
                            },
                            OutcomeScenarioV1 {
                                label: "incident".to_owned(),
                                probability: 0.3,
                                utility: 0.1,
                                risk: 0.9,
                            },
                        ],
                        expected_value: 0.695,
                        variance: 0.20,
                        downside_probability: 0.3,
                        rationale: "bounded".to_owned(),
                    },
                    risk_assessment: RiskAssessmentV1 {
                        risk_score: 0.7,
                        downside_probability: 0.3,
                        tail_risk_score: 0.8,
                        confidence: 0.7,
                        rationale: "higher blast radius".to_owned(),
                        mitigation: "manual rollback".to_owned(),
                    },
                    utility_breakdown: UtilityBreakdownV1 {
                        value_score: 0.9,
                        resilience_score: 0.5,
                        compliance_score: 0.7,
                        cost_score: 0.7,
                        reversibility_score: 0.5,
                        strategic_fit_score: 0.8,
                        rationale: "faster path".to_owned(),
                    },
                    rollback_plan: "rollback".to_owned(),
                    reversibility: ReversibilityClassV1::GuardedRollback,
                    historical_successes: 9,
                    historical_failures: 3,
                },
            ],
            constraints: vec![DecisionConstraintV1 {
                constraint_id: "max-risk".to_owned(),
                kind: contracts::ConstraintKindV1::MaxRiskScore,
                description: "risk must stay below 0.8".to_owned(),
                threshold: 0.8,
                hard: true,
                rationale: "guardrail".to_owned(),
            }],
            normal_form_game: None,
            provenance: ProvenanceV1 {
                source_system: "tests".to_owned(),
                source_refs: Vec::new(),
                generated_by: "tests".to_owned(),
                assumptions: vec!["bounded scenarios".to_owned()],
            },
        }
    }

    #[test]
    fn governance_service_evaluates_and_stores_decision_artifacts() {
        let mut service = GovernanceService::default();
        let report = service
            .evaluate_decision(decision_context())
            .expect("decision evaluation");

        assert_eq!(
            report.recommendation.selected_option_id.as_deref(),
            Some("staged")
        );
        assert_eq!(service.decision_recommendations().len(), 1);
        assert_eq!(service.decision_audit_records().len(), 1);
    }
}
