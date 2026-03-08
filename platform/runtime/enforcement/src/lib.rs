use chrono::Utc;
use contracts::{
    ApprovalDecisionV1, ApprovalRequestV1, Classification, EvidenceManifestV1, ImpactTier,
    PolicyDecisionOutcome, PolicyDecisionRequestV1, PolicyDecisionV1,
};
use error_model::{InstitutionalError, InstitutionalResult};
use evidence_sdk::EvidenceSink;
use identity::{ActorRef, InstitutionalRole};
use policy_sdk::{ApprovalVerificationPort, PolicyDecisionPort};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use telemetry::{DecisionRef, TraceContext};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardedMutationRequest {
    pub action_id: String,
    pub workflow_name: String,
    pub target_service: String,
    pub target_aggregate: String,
    pub actor_ref: ActorRef,
    pub impact_tier: ImpactTier,
    pub classification: Classification,
    pub policy_refs: Vec<String>,
    pub required_approver_roles: Vec<InstitutionalRole>,
    pub environment: String,
    pub cross_domain: bool,
}

impl GuardedMutationRequest {
    #[must_use]
    pub fn to_policy_request(&self) -> PolicyDecisionRequestV1 {
        PolicyDecisionRequestV1 {
            request_id: self.action_id.clone(),
            actor_ref: self.actor_ref.clone(),
            action: self.workflow_name.clone(),
            resource: self.target_service.clone(),
            environment: self.environment.clone(),
            impact_tier: self.impact_tier,
            classification: self.classification,
            cross_domain: self.cross_domain,
            policy_refs: self.policy_refs.clone(),
            exception_refs: Vec::new(),
        }
    }

    #[must_use]
    pub fn to_approval_request(&self) -> ApprovalRequestV1 {
        ApprovalRequestV1 {
            action_id: self.action_id.clone(),
            approval_scope: self.workflow_name.clone(),
            required_approver_roles: self.required_approver_roles.clone(),
            minimum_approvals: self.required_approver_roles.len().min(2),
            impact_tier: self.impact_tier,
            expires_at: Utc::now() + chrono::Duration::hours(24),
            rationale: format!(
                "Authorize {} against {}",
                self.workflow_name, self.target_aggregate
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedMutationContext {
    workflow_name: String,
    target_service: String,
    target_aggregate: String,
    decision: PolicyDecisionV1,
    approvals: Vec<ApprovalDecisionV1>,
    trace_context: TraceContext,
}

impl ApprovedMutationContext {
    pub fn assert_workflow(&self, expected: &str) -> InstitutionalResult<()> {
        if self.workflow_name == expected {
            Ok(())
        } else {
            Err(InstitutionalError::InvariantViolation {
                invariant: format!(
                    "workflow `{}` cannot execute mutation reserved for `{expected}`",
                    self.workflow_name
                ),
            })
        }
    }

    pub fn assert_target_service(&self, expected: &str) -> InstitutionalResult<()> {
        if self.target_service == expected {
            Ok(())
        } else {
            Err(InstitutionalError::InvariantViolation {
                invariant: format!(
                    "service `{}` cannot mutate target service `{expected}`",
                    self.target_service
                ),
            })
        }
    }

    #[must_use]
    pub fn approvals(&self) -> &[ApprovalDecisionV1] {
        &self.approvals
    }

    #[must_use]
    pub fn trace_context(&self) -> &TraceContext {
        &self.trace_context
    }
}

pub struct MutationEnforcer<'a, P, A, E> {
    policy_port: &'a P,
    approval_port: &'a A,
    evidence_sink: &'a mut E,
}

impl<'a, P, A, E> MutationEnforcer<'a, P, A, E> {
    #[must_use]
    pub fn new(policy_port: &'a P, approval_port: &'a A, evidence_sink: &'a mut E) -> Self {
        Self {
            policy_port,
            approval_port,
            evidence_sink,
        }
    }
}

impl<P, A, E> MutationEnforcer<'_, P, A, E>
where
    P: PolicyDecisionPort,
    A: ApprovalVerificationPort,
    E: EvidenceSink,
{
    pub fn authorize(
        &mut self,
        request: &GuardedMutationRequest,
    ) -> InstitutionalResult<ApprovedMutationContext> {
        let policy_request = request.to_policy_request();
        let decision = self.policy_port.evaluate(&policy_request)?;
        let trace_context =
            TraceContext::new().with_decision_ref(DecisionRef::new(decision.decision_id.clone()));
        let approvals = if request.impact_tier == ImpactTier::Tier0 {
            Vec::new()
        } else {
            let approval_request = request.to_approval_request();
            self.approval_port.verify(&approval_request)?
        };

        if decision.decision == PolicyDecisionOutcome::Deny {
            self.record_evidence(request, &decision)?;
            let reason = decision
                .denial_reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "policy denied mutation".to_owned());
            return Err(InstitutionalError::PolicyDenied { reason });
        }

        self.record_evidence(request, &decision)?;

        Ok(ApprovedMutationContext {
            workflow_name: request.workflow_name.clone(),
            target_service: request.target_service.clone(),
            target_aggregate: request.target_aggregate.clone(),
            decision,
            approvals,
            trace_context,
        })
    }

    fn record_evidence(
        &mut self,
        request: &GuardedMutationRequest,
        decision: &PolicyDecisionV1,
    ) -> InstitutionalResult<()> {
        let mut digest = Sha256::new();
        digest.update(request.workflow_name.as_bytes());
        digest.update(request.target_service.as_bytes());
        digest.update(decision.decision_id.as_bytes());
        let artifact_hash = hex::encode(digest.finalize());

        let manifest = EvidenceManifestV1 {
            evidence_id: format!("evidence::{}", decision.decision_id),
            producer: "platform/runtime/enforcement".to_owned(),
            artifact_hash,
            storage_ref: format!("surrealdb:evidence/{}", decision.decision_id),
            retention_class: "institutional_record".to_owned(),
            classification: request.classification,
            related_decision_refs: vec![decision.decision_id.clone()],
        };

        self.evidence_sink.record(manifest)
    }
}
