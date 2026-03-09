use contracts::{KnowledgeCapsuleV1, KnowledgeSourceV1, MacroFinancialAnalysisV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSourceIngestedV1 {
    pub source: KnowledgeSourceV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeCapsulePublishedV1 {
    pub capsule: KnowledgeCapsuleV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeAnalysisGeneratedV1 {
    pub analysis: MacroFinancialAnalysisV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "event_type", content = "payload", rename_all = "snake_case")]
pub enum KnowledgeEventPayloadV1 {
    KnowledgeSourceIngested(Box<KnowledgeSourceIngestedV1>),
    KnowledgeCapsulePublished(Box<KnowledgeCapsulePublishedV1>),
    KnowledgeAnalysisGenerated(Box<KnowledgeAnalysisGeneratedV1>),
}
