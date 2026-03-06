use std::collections::{BTreeMap, BTreeSet};

use contracts::{ApprovalDecisionV1, ApprovalRequestV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult};
use policy_sdk::ApprovalVerificationPort;

#[derive(Debug, Default, Clone)]
pub struct ApprovalService {
    approvals: BTreeMap<String, Vec<ApprovalDecisionV1>>,
}

impl ApprovalService {
    pub fn record_decision(&mut self, decision: ApprovalDecisionV1) {
        self.approvals
            .entry(decision.action_id.clone())
            .or_default()
            .push(decision);
    }
}

impl ApprovalVerificationPort for ApprovalService {
    fn verify(&self, request: &ApprovalRequestV1) -> InstitutionalResult<Vec<ApprovalDecisionV1>> {
        if request.required_approval_count() == 0 {
            return Ok(Vec::new());
        }

        let decisions = self
            .approvals
            .get(&request.action_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|decision| decision.approved && decision.decided_at <= request.expires_at)
            .collect::<Vec<_>>();

        let approved_roles = decisions
            .iter()
            .map(|decision| decision.approver_role)
            .collect::<BTreeSet<_>>();

        let minimum_met = decisions.len() >= request.required_approval_count();
        let roles_met = request
            .required_approver_roles
            .iter()
            .all(|role| approved_roles.contains(role));

        if minimum_met && roles_met {
            Ok(decisions)
        } else {
            Err(InstitutionalError::ApprovalMissing {
                action: request.action_id.clone(),
            })
        }
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "approval-service".to_owned(),
        domain: "strategy_governance".to_owned(),
        approved_workflows: vec![
            "policy_exception".to_owned(),
            "release_approval".to_owned(),
            "treasury_disbursement".to_owned(),
            "quant_strategy_promotion".to_owned(),
        ],
        owned_aggregates: vec![
            "approval_request".to_owned(),
            "approval_decision".to_owned(),
        ],
    }
}
