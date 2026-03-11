use std::future::Future;

use contracts::{
    EvidenceManifestV1, KnowledgeCapsuleV1, KnowledgeChangeNotificationV1, KnowledgeEdgeV1,
    KnowledgePublicationStatusV1, KnowledgeRetrievalHitV1, KnowledgeRetrievalQueryV1,
    KnowledgeSourceV1, MacroFinancialAnalysisV1,
};
use error_model::InstitutionalResult;
use futures::{StreamExt, stream::BoxStream};
use surrealdb_model::{EventRecordV1, KnowledgeChunkRecordV1};

pub type KnowledgeChangeStream =
    BoxStream<'static, InstitutionalResult<KnowledgeChangeNotificationV1>>;

pub trait KnowledgeStore: Send + Sync {
    fn store_sources_batch(
        &self,
        sources: Vec<KnowledgeSourceV1>,
        events: Vec<EventRecordV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;

    fn store_publication_bundle(
        &self,
        capsule: KnowledgeCapsuleV1,
        chunks: Vec<KnowledgeChunkRecordV1>,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;

    fn store_analysis_bundle(
        &self,
        analysis: MacroFinancialAnalysisV1,
        evidence_id: String,
        manifest: EvidenceManifestV1,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;

    fn load_analysis(
        &self,
        analysis_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<MacroFinancialAnalysisV1>>> + Send + '_;

    fn load_sources(
        &self,
        ids: &[String],
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeSourceV1>>> + Send + '_;

    fn load_capsule(
        &self,
        capsule_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgeCapsuleV1>>> + Send + '_;

    fn latest_publication_status(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgePublicationStatusV1>>> + Send + '_;

    fn search_capsule(
        &self,
        query: KnowledgeRetrievalQueryV1,
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeRetrievalHitV1>>> + Send + '_;

    fn load_change_notifications(
        &self,
        limit: usize,
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeChangeNotificationV1>>> + Send + '_;

    fn subscribe_change_notifications(
        &self,
    ) -> impl Future<Output = InstitutionalResult<KnowledgeChangeStream>> + Send + '_;
}

#[derive(Clone)]
pub struct GovernedKnowledgeStore<B> {
    inner: B,
}

impl<B> GovernedKnowledgeStore<B> {
    #[must_use]
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

pub type InMemoryKnowledgeStore =
    GovernedKnowledgeStore<storage_backend::InMemoryKnowledgeStoreBackend>;
pub type DurableKnowledgeStore =
    GovernedKnowledgeStore<storage_backend::DurableKnowledgeStoreBackend>;

pub use storage_backend::SurrealConnectionConfig;

pub async fn connect_in_memory() -> InstitutionalResult<InMemoryKnowledgeStore> {
    Ok(GovernedKnowledgeStore::new(
        storage_backend::connect_in_memory().await?,
    ))
}

pub async fn connect_durable(
    config: &SurrealConnectionConfig,
) -> InstitutionalResult<DurableKnowledgeStore> {
    Ok(GovernedKnowledgeStore::new(
        storage_backend::connect_durable(config).await?,
    ))
}

pub async fn connect_durable_from_env() -> InstitutionalResult<DurableKnowledgeStore> {
    Ok(GovernedKnowledgeStore::new(
        storage_backend::connect_durable_from_env().await?,
    ))
}

pub async fn connect_from_env() -> InstitutionalResult<DurableKnowledgeStore> {
    connect_durable_from_env().await
}

impl<C> GovernedKnowledgeStore<storage_backend::KnowledgeStoreBackend<C>>
where
    C: storage_backend::BackendConnection + Send + Sync,
{
    async fn store_sources_batch_inner(
        &self,
        sources: Vec<KnowledgeSourceV1>,
        events: Vec<EventRecordV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        self.inner
            .store_sources_batch(sources, events, notifications)
            .await
    }

    async fn store_publication_bundle_inner(
        &self,
        capsule: KnowledgeCapsuleV1,
        chunks: Vec<KnowledgeChunkRecordV1>,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        self.inner
            .store_publication_bundle(capsule, chunks, events, edges, notifications)
            .await
    }

    async fn store_analysis_bundle_inner(
        &self,
        analysis: MacroFinancialAnalysisV1,
        evidence_id: String,
        manifest: EvidenceManifestV1,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        self.inner
            .store_analysis_bundle(
                analysis,
                evidence_id,
                manifest,
                events,
                edges,
                notifications,
            )
            .await
    }

    async fn load_analysis_inner(
        &self,
        analysis_id: String,
    ) -> InstitutionalResult<Option<MacroFinancialAnalysisV1>> {
        Ok(self
            .inner
            .knowledge_analyses()
            .load(&analysis_id)
            .await?
            .map(|record| record.analysis))
    }

    async fn load_sources_inner(
        &self,
        ids: Vec<String>,
    ) -> InstitutionalResult<Vec<KnowledgeSourceV1>> {
        Ok(self
            .inner
            .knowledge_sources()
            .load_many(&ids)
            .await?
            .into_iter()
            .map(|record| record.source)
            .collect())
    }

    async fn load_capsule_inner(
        &self,
        capsule_id: String,
    ) -> InstitutionalResult<Option<KnowledgeCapsuleV1>> {
        Ok(self
            .inner
            .knowledge_capsules()
            .load(&capsule_id)
            .await?
            .map(|record| record.capsule))
    }

    async fn latest_publication_status_inner(
        &self,
    ) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>> {
        self.inner.publication_status().latest().await
    }

    async fn search_capsule_inner(
        &self,
        query: KnowledgeRetrievalQueryV1,
    ) -> InstitutionalResult<Vec<KnowledgeRetrievalHitV1>> {
        self.inner.knowledge_chunks().search(query).await
    }

    async fn load_change_notifications_inner(
        &self,
        limit: usize,
    ) -> InstitutionalResult<Vec<KnowledgeChangeNotificationV1>> {
        Ok(self
            .inner
            .change_notifications()
            .recent(limit)
            .await?
            .into_iter()
            .map(|record| record.as_notification())
            .collect())
    }

    async fn subscribe_change_notifications_inner(
        &self,
    ) -> InstitutionalResult<KnowledgeChangeStream> {
        let stream = self.inner.change_notifications().subscribe().await?;
        Ok(stream
            .map(|result| result.map(|record| record.as_notification()))
            .boxed())
    }
}

impl<C> KnowledgeStore for GovernedKnowledgeStore<storage_backend::KnowledgeStoreBackend<C>>
where
    C: storage_backend::BackendConnection + Send + Sync,
{
    fn store_sources_batch(
        &self,
        sources: Vec<KnowledgeSourceV1>,
        events: Vec<EventRecordV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_sources_batch_inner(sources, events, notifications)
    }

    fn store_publication_bundle(
        &self,
        capsule: KnowledgeCapsuleV1,
        chunks: Vec<KnowledgeChunkRecordV1>,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_publication_bundle_inner(capsule, chunks, events, edges, notifications)
    }

    fn store_analysis_bundle(
        &self,
        analysis: MacroFinancialAnalysisV1,
        evidence_id: String,
        manifest: EvidenceManifestV1,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_analysis_bundle_inner(
            analysis,
            evidence_id,
            manifest,
            events,
            edges,
            notifications,
        )
    }

    fn load_analysis(
        &self,
        analysis_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<MacroFinancialAnalysisV1>>> + Send + '_
    {
        self.load_analysis_inner(analysis_id.to_owned())
    }

    fn load_sources(
        &self,
        ids: &[String],
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeSourceV1>>> + Send + '_ {
        self.load_sources_inner(ids.to_vec())
    }

    fn load_capsule(
        &self,
        capsule_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgeCapsuleV1>>> + Send + '_ {
        self.load_capsule_inner(capsule_id.to_owned())
    }

    fn latest_publication_status(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgePublicationStatusV1>>> + Send + '_
    {
        self.latest_publication_status_inner()
    }

    fn search_capsule(
        &self,
        query: KnowledgeRetrievalQueryV1,
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeRetrievalHitV1>>> + Send + '_ {
        self.search_capsule_inner(query)
    }

    fn load_change_notifications(
        &self,
        limit: usize,
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeChangeNotificationV1>>> + Send + '_
    {
        self.load_change_notifications_inner(limit)
    }

    fn subscribe_change_notifications(
        &self,
    ) -> impl Future<Output = InstitutionalResult<KnowledgeChangeStream>> + Send + '_ {
        self.subscribe_change_notifications_inner()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{Classification, KnowledgeCapsuleV1};

    use super::{KnowledgeStore, SurrealConnectionConfig, connect_durable};

    #[tokio::test]
    async fn durable_governed_store_supports_runtime_connection_path() {
        let store = connect_durable(&SurrealConnectionConfig {
            endpoint: "mem://".to_string(),
            namespace: "runtime".to_string(),
            database: "governed".to_string(),
            username: None,
            password: None,
        })
        .await
        .expect("store");

        store
            .store_publication_bundle(
                KnowledgeCapsuleV1 {
                    capsule_id: "capsule-runtime-1".to_string(),
                    publication_id: "publication-runtime-1".to_string(),
                    title: "Runtime capsule".to_string(),
                    source_ids: vec!["source-runtime-1".to_string()],
                    source_count: 1,
                    storage_ref: "memvid:capsule-runtime-1".to_string(),
                    artifact_hash: "capsule-runtime-hash".to_string(),
                    version: "v1".to_string(),
                    memvid_version: "2.0.138".to_string(),
                    published_at: Utc
                        .with_ymd_and_hms(2026, 3, 10, 12, 0, 0)
                        .single()
                        .expect("time"),
                    classification: Classification::Internal,
                    retention_class: "institutional_record".to_string(),
                },
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .await
            .expect("store capsule");

        let loaded = store
            .load_capsule("capsule-runtime-1")
            .await
            .expect("load capsule")
            .expect("capsule present");
        assert_eq!(loaded.title, "Runtime capsule");
    }
}
