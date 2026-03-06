use std::collections::BTreeMap;

use contracts::{
    ModelApprovalStatusV1, ModelApprovalV1, PromotionRecommendationV1, ServiceBoundaryV1,
};
use enforcement::ApprovedMutationContext;
use error_model::InstitutionalResult;

#[derive(Debug, Default, Clone)]
pub struct GovernanceService {
    approvals: BTreeMap<String, ModelApprovalV1>,
    recommendations: Vec<PromotionRecommendationV1>,
}

impl GovernanceService {
    pub fn submit_model(&mut self, model_id: &str, version: &str, notes: &str) {
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

    pub fn approve_model(&mut self, model_id: &str, version: &str, reviewer: &str) {
        if let Some(model) = self.approvals.get_mut(&format!("{model_id}:{version}")) {
            model.status = ModelApprovalStatusV1::Approved;
            model.approved_by = Some(reviewer.to_string());
        }
    }

    #[must_use]
    pub fn approved_models(&self) -> Vec<String> {
        self.approvals
            .values()
            .filter(|model| model.status == ModelApprovalStatusV1::Approved)
            .map(|model| format!("{}:{}", model.model_id, model.version))
            .collect()
    }

    pub fn record_recommendation(
        &mut self,
        context: &ApprovedMutationContext,
        recommendation: PromotionRecommendationV1,
    ) -> InstitutionalResult<PromotionRecommendationV1> {
        context.assert_workflow("quant_strategy_promotion")?;
        context.assert_target_service("governance-service")?;
        self.recommendations.push(recommendation.clone());
        Ok(recommendation)
    }

    #[must_use]
    pub fn recommendations(&self) -> &[PromotionRecommendationV1] {
        &self.recommendations
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "governance-service".to_owned(),
        domain: "strategy_governance".to_owned(),
        approved_workflows: vec![
            "strategy_review".to_owned(),
            "policy_exception".to_owned(),
            "quant_strategy_promotion".to_owned(),
        ],
        owned_aggregates: vec![
            "governance_decision".to_owned(),
            "institutional_invariant".to_owned(),
            "promotion_recommendation".to_owned(),
            "model_approval".to_owned(),
        ],
    }
}
