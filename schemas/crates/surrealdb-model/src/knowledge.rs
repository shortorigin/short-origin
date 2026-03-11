use chrono::{DateTime, Utc};
use contracts::{
    Classification, KnowledgeCapsuleV1, KnowledgeChangeKindV1, KnowledgeChangeNotificationV1,
    KnowledgeEdgeV1, KnowledgePublicationStatusV1, KnowledgeRelationshipV1,
    KnowledgeRetrievalHitV1, KnowledgeSourceKindV1, KnowledgeSourceV1, MacroFinancialAnalysisV1,
};
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
    pub r#in: String,
    pub r#out: String,
    pub from_id: String,
    pub to_id: String,
    pub relationship: KnowledgeRelationshipV1,
    pub rationale: String,
}

impl KnowledgeEdgeRecordV1 {
    #[must_use]
    pub fn from_edge(edge: KnowledgeEdgeV1, in_record: String, out_record: String) -> Self {
        Self {
            id: edge.edge_id.clone(),
            r#in: in_record,
            r#out: out_record,
            from_id: edge.from_id,
            to_id: edge.to_id,
            relationship: edge.relationship,
            rationale: edge.rationale,
        }
    }

    #[must_use]
    pub fn as_edge(&self) -> KnowledgeEdgeV1 {
        KnowledgeEdgeV1 {
            edge_id: self.id.clone(),
            from_id: self.from_id.clone(),
            to_id: self.to_id.clone(),
            relationship: self.relationship,
            rationale: self.rationale.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeChunkRecordV1 {
    pub id: String,
    pub chunk_id: String,
    pub capsule_id: String,
    pub source_id: String,
    pub chunk_index: usize,
    pub title: String,
    pub uri: String,
    pub country_area: String,
    pub classification: Classification,
    pub source_kind: KnowledgeSourceKindV1,
    pub search_text: String,
    pub content_digest: String,
    pub acquired_at: DateTime<Utc>,
}

impl KnowledgeChunkRecordV1 {
    #[must_use]
    pub fn to_hit(
        &self,
        snippet: String,
        rank: usize,
        score: Option<f32>,
    ) -> KnowledgeRetrievalHitV1 {
        KnowledgeRetrievalHitV1 {
            chunk_id: self.chunk_id.clone(),
            capsule_id: self.capsule_id.clone(),
            source_id: self.source_id.clone(),
            title: self.title.clone(),
            uri: self.uri.clone(),
            snippet,
            rank,
            score,
            classification: self.classification,
            source_kind: self.source_kind,
            country_area: self.country_area.clone(),
            content_digest: self.content_digest.clone(),
            acquired_at: self.acquired_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgePublicationStatusRecordV1 {
    pub id: String,
    pub publication_id: String,
    pub capsule_id: String,
    pub published_at: DateTime<Utc>,
    pub source_count: usize,
    pub storage_ref: String,
    pub artifact_hash: String,
    pub version: String,
}

impl KnowledgePublicationStatusRecordV1 {
    #[must_use]
    pub fn from_status(id: impl Into<String>, status: KnowledgePublicationStatusV1) -> Self {
        Self {
            id: id.into(),
            publication_id: status.publication_id,
            capsule_id: status.capsule_id,
            published_at: status.published_at,
            source_count: status.source_count,
            storage_ref: status.storage_ref,
            artifact_hash: status.artifact_hash,
            version: status.version,
        }
    }

    #[must_use]
    pub fn as_status(&self) -> KnowledgePublicationStatusV1 {
        KnowledgePublicationStatusV1 {
            publication_id: self.publication_id.clone(),
            capsule_id: self.capsule_id.clone(),
            published_at: self.published_at,
            source_count: self.source_count,
            storage_ref: self.storage_ref.clone(),
            artifact_hash: self.artifact_hash.clone(),
            version: self.version.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeChangeNotificationRecordV1 {
    pub id: String,
    pub kind: KnowledgeChangeKindV1,
    pub record_id: String,
    pub publication_id: Option<String>,
    pub capsule_id: Option<String>,
    pub source_id: Option<String>,
    pub analysis_id: Option<String>,
    pub published_at: DateTime<Utc>,
    pub classification: Classification,
}

impl KnowledgeChangeNotificationRecordV1 {
    #[must_use]
    pub fn from_notification(notification: KnowledgeChangeNotificationV1) -> Self {
        Self {
            id: notification.notification_id.clone(),
            kind: notification.kind,
            record_id: notification.record_id,
            publication_id: notification.publication_id,
            capsule_id: notification.capsule_id,
            source_id: notification.source_id,
            analysis_id: notification.analysis_id,
            published_at: notification.published_at,
            classification: notification.classification,
        }
    }

    #[must_use]
    pub fn as_notification(&self) -> KnowledgeChangeNotificationV1 {
        KnowledgeChangeNotificationV1 {
            notification_id: self.id.clone(),
            kind: self.kind,
            record_id: self.record_id.clone(),
            publication_id: self.publication_id.clone(),
            capsule_id: self.capsule_id.clone(),
            source_id: self.source_id.clone(),
            analysis_id: self.analysis_id.clone(),
            published_at: self.published_at,
            classification: self.classification,
        }
    }
}
