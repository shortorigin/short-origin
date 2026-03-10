use chrono::Utc;
use contracts::{
    ApprovalDecisionV1, ApprovalRequestV1, Classification, EvidenceManifestV1, ImpactTier,
    PolicyDecisionOutcome, PolicyDecisionRequestV1, PolicyDecisionV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use evidence_sdk::EvidenceSink;
use identity::{
    ActionId, ActorRef, AggregateId, EnvironmentId, EvidenceId, InstitutionalRole, ServiceId,
    WorkflowId,
};
use policy_sdk::{ApprovalVerificationPort, PolicyDecisionPort};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use telemetry::{DecisionRef, TraceContext};
use tokio::time::{timeout, Duration};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuardedMutationRequest {
    pub action_id: ActionId,
    pub workflow_name: WorkflowId,
    pub target_service: ServiceId,
    pub target_aggregate: AggregateId,
    pub actor_ref: ActorRef,
    pub impact_tier: ImpactTier,
    pub classification: Classification,
    pub policy_refs: Vec<String>,
    pub required_approver_roles: Vec<InstitutionalRole>,
    pub environment: EnvironmentId,
    pub cross_domain: bool,
}

impl GuardedMutationRequest {
    fn operation_context(&self, operation: &str) -> OperationContext {
        OperationContext::new("platform/runtime/enforcement", operation)
            .with_service_id(self.target_service.clone())
            .with_workflow_id(self.workflow_name.clone())
            .with_correlation_id(self.action_id.as_str())
    }

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
    workflow_name: WorkflowId,
    target_service: ServiceId,
    target_aggregate: AggregateId,
    decision: PolicyDecisionV1,
    approvals: Vec<ApprovalDecisionV1>,
    trace_context: TraceContext,
}

impl ApprovedMutationContext {
    pub fn assert_workflow(&self, expected: &WorkflowId) -> InstitutionalResult<()> {
        if &self.workflow_name == expected {
            Ok(())
        } else {
            Err(InstitutionalError::invariant(
                self.operation_context("assert_workflow"),
                format!(
                    "workflow `{}` cannot execute mutation reserved for `{expected}`",
                    self.workflow_name
                ),
            ))
        }
    }

    pub fn assert_target_service(&self, expected: &ServiceId) -> InstitutionalResult<()> {
        if &self.target_service == expected {
            Ok(())
        } else {
            Err(InstitutionalError::invariant(
                self.operation_context("assert_target_service"),
                format!(
                    "service `{}` cannot mutate target service `{expected}`",
                    self.target_service
                ),
            ))
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

    #[must_use]
    pub fn workflow_id(&self) -> &WorkflowId {
        &self.workflow_name
    }

    fn operation_context(&self, operation: &str) -> OperationContext {
        OperationContext::new("platform/runtime/enforcement", operation)
            .with_service_id(self.target_service.clone())
            .with_workflow_id(self.workflow_name.clone())
            .with_correlation_id(self.trace_context.correlation_id.clone())
    }
}

pub struct MutationEnforcer<'a, P, A, E> {
    policy_port: &'a P,
    approval_port: &'a A,
    evidence_sink: &'a E,
}

impl<'a, P, A, E> MutationEnforcer<'a, P, A, E> {
    #[must_use]
    pub fn new(policy_port: &'a P, approval_port: &'a A, evidence_sink: &'a E) -> Self {
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
    pub async fn authorize(
        &mut self,
        request: &GuardedMutationRequest,
    ) -> InstitutionalResult<ApprovedMutationContext> {
        self.authorize_inner(request).await
    }

    pub async fn authorize_with_timeout(
        &mut self,
        request: &GuardedMutationRequest,
        limit: Duration,
    ) -> InstitutionalResult<ApprovedMutationContext> {
        timeout(limit, self.authorize_inner(request))
            .await
            .map_err(|_| {
                let error = InstitutionalError::Timeout {
                    context: Box::new(request.operation_context("authorize")),
                    message: format!("mutation authorization exceeded {} ms", limit.as_millis()),
                    source_info: None,
                };
                trace_failure(&error);
                error
            })?
    }

    async fn authorize_inner(
        &mut self,
        request: &GuardedMutationRequest,
    ) -> InstitutionalResult<ApprovedMutationContext> {
        let policy_request = request.to_policy_request();
        let decision = self
            .policy_port
            .evaluate(&policy_request)
            .await
            .map_err(traced)?;
        let trace_context = TraceContext::new()
            .with_causation_id(request.action_id.as_str())
            .with_decision_ref(DecisionRef::new(decision.decision_id.as_str()));
        let approvals = if request.impact_tier == ImpactTier::Tier0 {
            Vec::new()
        } else {
            let approval_request = request.to_approval_request();
            self.approval_port
                .verify(&approval_request)
                .await
                .map_err(traced)?
        };

        if decision.decision == PolicyDecisionOutcome::Deny {
            self.record_evidence(request, &decision).await?;
            let error = InstitutionalError::policy_denied(
                request.operation_context("authorize"),
                decision
                    .denial_reasons
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "policy denied mutation".to_owned()),
            );
            trace_failure(&error);
            return Err(error);
        }

        self.record_evidence(request, &decision).await?;

        Ok(ApprovedMutationContext {
            workflow_name: request.workflow_name.clone(),
            target_service: request.target_service.clone(),
            target_aggregate: request.target_aggregate.clone(),
            decision,
            approvals,
            trace_context,
        })
    }

    async fn record_evidence(
        &self,
        request: &GuardedMutationRequest,
        decision: &PolicyDecisionV1,
    ) -> InstitutionalResult<()> {
        let mut digest = Sha256::new();
        digest.update(request.workflow_name.as_str().as_bytes());
        digest.update(request.target_service.as_str().as_bytes());
        digest.update(decision.decision_id.as_str().as_bytes());
        let artifact_hash = hex::encode(digest.finalize());

        let manifest = EvidenceManifestV1 {
            evidence_id: EvidenceId::from(format!("evidence::{}", decision.decision_id)),
            producer: "platform/runtime/enforcement".to_owned(),
            artifact_hash,
            storage_ref: format!("surrealdb:evidence/{}", decision.decision_id),
            retention_class: "institutional_record".to_owned(),
            classification: request.classification,
            related_decision_refs: vec![DecisionRef::new(decision.decision_id.as_str())],
        };

        self.evidence_sink.record(manifest).await.map_err(traced)
    }
}

fn traced(error: InstitutionalError) -> InstitutionalError {
    trace_failure(&error);
    error
}

fn trace_failure(error: &InstitutionalError) {
    let context = error.context();
    error!(
        subsystem = context.subsystem,
        service_id = context
            .service_id
            .as_ref()
            .map_or("", identity::ServiceId::as_str),
        workflow_id = context
            .workflow_id
            .as_ref()
            .map_or("", identity::WorkflowId::as_str),
        correlation_id = context
            .correlation_id
            .as_ref()
            .map_or("", telemetry::CorrelationId::as_str),
        operation = context.operation,
        error_category = ?error.category(),
        message = error.message(),
        "mutation enforcement failed"
    );
}
