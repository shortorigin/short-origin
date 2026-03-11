use std::future::Future;

use chrono::{Duration, Utc};
use contracts::{
    AgentActionRequestV1, ApprovalRequestV1, PolicyDecisionRequestV1, ServiceBoundaryV1,
};
use error_model::InstitutionalResult;
use identity::EnvironmentId;
use uuid::Uuid;

pub trait PolicyDecisionPort {
    fn evaluate(
        &self,
        request: &PolicyDecisionRequestV1,
    ) -> impl Future<Output = InstitutionalResult<contracts::PolicyDecisionV1>> + Send + '_;
}

pub trait ApprovalVerificationPort {
    fn verify(
        &self,
        request: &ApprovalRequestV1,
    ) -> impl Future<Output = InstitutionalResult<Vec<contracts::ApprovalDecisionV1>>> + Send + '_;
}

#[must_use]
pub fn build_policy_request(
    action: &AgentActionRequestV1,
    service_boundary: &ServiceBoundaryV1,
    environment: impl Into<EnvironmentId>,
    cross_domain: bool,
) -> PolicyDecisionRequestV1 {
    PolicyDecisionRequestV1 {
        request_id: Uuid::new_v4().to_string().into(),
        actor_ref: action.actor_ref.clone(),
        action: action.requested_workflow.clone(),
        resource: service_boundary.service_name.clone().into(),
        environment: environment.into(),
        impact_tier: action.impact_tier,
        classification: action.classification,
        cross_domain,
        policy_refs: action.policy_refs.clone(),
        exception_refs: Vec::new(),
    }
}

#[must_use]
pub fn build_approval_request(action: &AgentActionRequestV1) -> ApprovalRequestV1 {
    ApprovalRequestV1 {
        action_id: action.action_id.clone(),
        approval_scope: action.requested_workflow.clone(),
        required_approver_roles: action.required_approver_roles.clone(),
        minimum_approvals: action.required_approver_roles.len().min(2),
        impact_tier: action.impact_tier,
        expires_at: Utc::now() + Duration::hours(24),
        rationale: action.objective.clone(),
    }
}
