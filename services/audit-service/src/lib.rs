use contracts::{ApprovalDecisionV1, EvidenceManifestV1, RiskRecordV1, ServiceBoundaryV1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditSnapshot {
    pub evidence_count: usize,
    pub risk_count: usize,
    pub approval_count: usize,
}

pub struct AuditService;

impl AuditService {
    #[must_use]
    pub fn snapshot(
        &self,
        evidence: &[EvidenceManifestV1],
        risks: &[RiskRecordV1],
        approvals: &[ApprovalDecisionV1],
    ) -> AuditSnapshot {
        AuditSnapshot {
            evidence_count: evidence.len(),
            risk_count: risks.len(),
            approval_count: approvals.len(),
        }
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "audit-service".to_owned(),
        domain: "audit_assurance".to_owned(),
        approved_workflows: vec!["control_testing".to_owned(), "evidence_review".to_owned()],
        owned_aggregates: vec!["audit_finding".to_owned(), "evidence_manifest".to_owned()],
    }
}
