use std::collections::BTreeSet;

use contracts::{
    ImpactTier, PolicyDecisionOutcome, PolicyDecisionRequestV1, PolicyDecisionV1, ServiceBoundaryV1,
};
use error_model::InstitutionalResult;
use policy_sdk::PolicyDecisionPort;

#[derive(Debug, Clone)]
pub struct PolicyService {
    freeze_active: bool,
    expired_exceptions: BTreeSet<String>,
}

impl PolicyService {
    #[must_use]
    pub fn institutional_default() -> Self {
        Self {
            freeze_active: false,
            expired_exceptions: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn with_change_freeze(mut self) -> Self {
        self.freeze_active = true;
        self
    }

    pub fn mark_exception_expired(&mut self, exception_ref: impl Into<String>) {
        self.expired_exceptions.insert(exception_ref.into());
    }
}

impl PolicyDecisionPort for PolicyService {
    fn evaluate(&self, request: &PolicyDecisionRequestV1) -> InstitutionalResult<PolicyDecisionV1> {
        let mut denial_reasons = Vec::new();
        if request.policy_refs.is_empty() {
            denial_reasons.push("policy_refs must not be empty".to_owned());
        }
        if self.freeze_active && request.impact_tier != ImpactTier::Tier0 {
            denial_reasons.push("change freeze is active".to_owned());
        }
        if request
            .exception_refs
            .iter()
            .any(|exception_ref| self.expired_exceptions.contains(exception_ref))
        {
            denial_reasons.push("expired exception reference supplied".to_owned());
        }

        let allowed = denial_reasons.is_empty();
        let obligations = if allowed {
            let mut obligations = vec!["record_evidence".to_owned()];
            if request.impact_tier != ImpactTier::Tier0 {
                obligations.push("require_human_approval".to_owned());
            }
            obligations
        } else {
            Vec::new()
        };

        Ok(PolicyDecisionV1 {
            decision_id: format!("decision::{}", request.request_id),
            request_id: request.request_id.clone(),
            decision: if allowed {
                PolicyDecisionOutcome::Allow
            } else {
                PolicyDecisionOutcome::Deny
            },
            obligations,
            denial_reasons,
            evidence_refs: vec![format!("evidence::{}", request.request_id)],
        })
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "policy-service".to_owned(),
        domain: "strategy_governance".to_owned(),
        approved_workflows: vec![
            "knowledge_publication".to_owned(),
            "policy_exception".to_owned(),
            "release_approval".to_owned(),
            "treasury_disbursement".to_owned(),
            "quant_strategy_promotion".to_owned(),
        ],
        owned_aggregates: vec!["policy_decision".to_owned(), "policy_exception".to_owned()],
    }
}
