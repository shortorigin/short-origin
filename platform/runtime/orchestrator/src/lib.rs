use contracts::EvidenceManifestV1;
use enforcement::{ApprovedMutationContext, GuardedMutationRequest, MutationEnforcer};
use error_model::InstitutionalResult;
use evidence_sdk::EvidenceSink;
use policy_sdk::{ApprovalVerificationPort, PolicyDecisionPort};

pub struct WorkflowEngine<P, A, E> {
    policy_port: P,
    approval_port: A,
    evidence_sink: E,
}

impl<P, A, E> WorkflowEngine<P, A, E> {
    #[must_use]
    pub fn new(policy_port: P, approval_port: A, evidence_sink: E) -> Self {
        Self {
            policy_port,
            approval_port,
            evidence_sink,
        }
    }
}

impl<P, A, E> WorkflowEngine<P, A, E>
where
    P: PolicyDecisionPort,
    A: ApprovalVerificationPort,
    E: EvidenceSink,
{
    pub fn execute_mutation<T, F>(
        &mut self,
        request: GuardedMutationRequest,
        action: F,
    ) -> InstitutionalResult<T>
    where
        F: FnOnce(&ApprovedMutationContext) -> InstitutionalResult<T>,
    {
        let mut enforcer = MutationEnforcer::new(
            &self.policy_port,
            &self.approval_port,
            &mut self.evidence_sink,
        );
        let context = enforcer.authorize(&request)?;
        action(&context)
    }

    #[must_use]
    pub fn recorded_evidence(&self) -> Vec<EvidenceManifestV1> {
        self.evidence_sink.recorded()
    }
}
