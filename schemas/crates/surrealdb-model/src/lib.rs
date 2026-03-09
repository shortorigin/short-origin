use contracts::{
    ApprovalDecisionV1, ComplianceReportV1, EvidenceManifestV1, ExceptionRecordV1,
    ExperimentResultV1, FillV1, MarketDataBatchV1, ModelApprovalV1, OrderRequestV1,
    PolicyDecisionV1, PortfolioSnapshotV1, PromotionRecommendationV1, RiskRecordV1,
    TradingRiskSnapshotV1, TreasuryDisbursementRecordedV1,
};
use events::RecordedEventV1;
use identity::ActorV1;
use serde::{Deserialize, Serialize};

mod knowledge;

pub use knowledge::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActorRecordV1 {
    pub id: String,
    pub actor: ActorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DecisionRecordV1 {
    pub id: String,
    pub decision: PolicyDecisionV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalRecordV1 {
    pub id: String,
    pub approval: ApprovalDecisionV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExceptionRecordStoreV1 {
    pub id: String,
    pub exception: ExceptionRecordV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskRecordStoreV1 {
    pub id: String,
    pub risk: RiskRecordV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowExecutionRecordV1 {
    pub id: String,
    pub workflow_name: String,
    pub trace_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRecordV1 {
    pub id: String,
    pub event: RecordedEventV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeRecordV1 {
    pub id: String,
    pub domain: String,
    pub title: String,
    pub classification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceManifestRecordV1 {
    pub id: String,
    pub evidence: EvidenceManifestV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TreasuryDisbursementRecordV1 {
    pub id: String,
    pub disbursement: TreasuryDisbursementRecordedV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDatasetRecordV1 {
    pub id: String,
    pub dataset: MarketDataBatchV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentResultRecordV1 {
    pub id: String,
    pub result: ExperimentResultV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderRecordStoreV1 {
    pub id: String,
    pub order: OrderRequestV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FillRecordStoreV1 {
    pub id: String,
    pub fill: FillV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioSnapshotRecordV1 {
    pub id: String,
    pub snapshot: PortfolioSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradingRiskSnapshotRecordV1 {
    pub id: String,
    pub snapshot: TradingRiskSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplianceReportRecordV1 {
    pub id: String,
    pub report: ComplianceReportV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelApprovalRecordV1 {
    pub id: String,
    pub approval: ModelApprovalV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionRecommendationRecordV1 {
    pub id: String,
    pub recommendation: PromotionRecommendationV1,
}
