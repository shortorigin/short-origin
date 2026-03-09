use contracts::{KnowledgeCapsuleV1, KnowledgeEdgeV1, KnowledgeSourceV1, MacroFinancialAnalysisV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSourceRecordV1 {
    pub id: String,
    pub source: KnowledgeSourceV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeCapsuleRecordV1 {
    pub id: String,
    pub capsule: KnowledgeCapsuleV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeAnalysisRecordV1 {
    pub id: String,
    pub analysis: MacroFinancialAnalysisV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeEdgeRecordV1 {
    pub id: String,
    pub edge: KnowledgeEdgeV1,
}
