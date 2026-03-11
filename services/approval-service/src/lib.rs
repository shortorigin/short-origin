use std::collections::{BTreeMap, BTreeSet};
use std::future::{Future, ready};

use contracts::{ApprovalDecisionV1, ApprovalRequestV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use identity::ActionId;
use policy_sdk::ApprovalVerificationPort;

const SERVICE_NAME: &str = "approval-service";
const DOMAIN_NAME: &str = "strategy_governance";
const APPROVED_WORKFLOWS: &[&str] = &[
    "knowledge_publication",
    "policy_exception",
    "release_approval",
    "treasury_disbursement",
    "quant_strategy_promotion",
];
const OWNED_AGGREGATES: &[&str] = &["approval_request", "approval_decision"];

#[derive(Debug, Default, Clone)]
struct InMemoryApprovalStore {
    approvals: BTreeMap<ActionId, Vec<ApprovalDecisionV1>>,
}

impl InMemoryApprovalStore {
    fn record(&mut self, decision: ApprovalDecisionV1) {
        self.approvals
            .entry(decision.action_id.clone())
            .or_default()
            .push(decision);
    }

    fn decisions_for(&self, action_id: &ActionId) -> Vec<ApprovalDecisionV1> {
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
    fn verify(
        &self,
        request: &ApprovalRequestV1,
    ) -> impl Future<Output = InstitutionalResult<Vec<ApprovalDecisionV1>>> + Send + '_ {
        let request = request.clone();
        if request.required_approval_count() == 0 {
            return ready(Ok(Vec::new()));
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
            ready(Ok(decisions))
        } else {
            ready(Err(InstitutionalError::approval_denied(
                OperationContext::new("services/approval-service", "verify")
                    .with_workflow_id(request.approval_scope.clone())
                    .with_correlation_id(request.action_id.as_str()),
                format!(
                    "missing required approvals for action `{}` within workflow `{}`",
                    request.action_id, request.approval_scope
                ),
            )))
        }
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.into(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES.iter().copied().map(Into::into).collect(),
    }
}
