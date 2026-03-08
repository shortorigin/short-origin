use std::collections::{BTreeMap, BTreeSet};

use contracts::{ApprovalDecisionV1, ApprovalRequestV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult};
use policy_sdk::ApprovalVerificationPort;

const SERVICE_NAME: &str = "approval-service";
const DOMAIN_NAME: &str = "strategy_governance";
const APPROVED_WORKFLOWS: &[&str] = &[
    "policy_exception",
    "release_approval",
    "treasury_disbursement",
    "quant_strategy_promotion",
];
const OWNED_AGGREGATES: &[&str] = &["approval_request", "approval_decision"];

#[derive(Debug, Default, Clone)]
struct InMemoryApprovalStore {
    approvals: BTreeMap<String, Vec<ApprovalDecisionV1>>,
}

impl InMemoryApprovalStore {
    fn record(&mut self, decision: ApprovalDecisionV1) {
        self.approvals
            .entry(decision.action_id.clone())
            .or_default()
            .push(decision);
    }

    fn decisions_for(&self, action_id: &str) -> Vec<ApprovalDecisionV1> {
        self.approvals.get(action_id).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ApprovalService {
    store: InMemoryApprovalStore,
}

impl ApprovalService {
    pub fn record_decision(&mut self, decision: ApprovalDecisionV1) {
        self.store.record(decision);
    }
}

impl ApprovalVerificationPort for ApprovalService {
    fn verify(&self, request: &ApprovalRequestV1) -> InstitutionalResult<Vec<ApprovalDecisionV1>> {
        if request.required_approval_count() == 0 {
            return Ok(Vec::new());
        }

        let decisions = self
            .store
            .decisions_for(&request.action_id)
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
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        owned_aggregates: OWNED_AGGREGATES
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}
