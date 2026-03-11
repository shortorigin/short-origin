use std::future::Future;

use contracts::{
    EvidenceManifestV1, KnowledgeCapsuleV1, KnowledgeEdgeV1, KnowledgePublicationStatusV1,
    KnowledgeSourceV1, MacroFinancialAnalysisV1,
};
use error_model::InstitutionalResult;
use events::RecordedEventV1;

pub trait KnowledgeStore: Send + Sync {
    fn store_analysis(
        &self,
        analysis: MacroFinancialAnalysisV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn load_analysis(
        &self,
        analysis_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<MacroFinancialAnalysisV1>>> + Send + '_;
    fn store_evidence(
        &self,
        id: String,
        manifest: EvidenceManifestV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn append_event(
        &self,
        id: String,
        event: RecordedEventV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn store_source(
        &self,
        source: KnowledgeSourceV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn load_sources(
        &self,
        ids: &[String],
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeSourceV1>>> + Send + '_;
    fn store_capsule(
        &self,
        capsule: KnowledgeCapsuleV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
    fn load_capsule(
        &self,
        capsule_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgeCapsuleV1>>> + Send + '_;
    fn latest_publication_status(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Option<KnowledgePublicationStatusV1>>> + Send + '_;
    fn store_edge(
        &self,
        edge: KnowledgeEdgeV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_;
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

pub async fn connect_from_env() -> InstitutionalResult<DurableKnowledgeStore> {
    Ok(GovernedKnowledgeStore::new(
        storage_backend::connect_durable_from_env().await?,
    ))
}

impl<C> GovernedKnowledgeStore<storage_backend::KnowledgeStoreBackend<C>>
where
    C: storage_backend::BackendConnection + Send + Sync,
{
    async fn store_analysis_inner(
        &self,
        analysis: MacroFinancialAnalysisV1,
    ) -> InstitutionalResult<()> {
        self.inner.knowledge_analyses().store(analysis).await?;
        Ok(())
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

    async fn store_evidence_inner(
        &self,
        id: String,
        manifest: EvidenceManifestV1,
    ) -> InstitutionalResult<()> {
        self.inner.evidence_manifests().store(id, manifest).await?;
        Ok(())
    }

    async fn append_event_inner(
        &self,
        id: String,
        event: RecordedEventV1,
    ) -> InstitutionalResult<()> {
        self.inner.recorded_events().append(id, event).await?;
        Ok(())
    }

    async fn store_source_inner(&self, source: KnowledgeSourceV1) -> InstitutionalResult<()> {
        self.inner.knowledge_sources().store(source).await?;
        Ok(())
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

    async fn store_capsule_inner(&self, capsule: KnowledgeCapsuleV1) -> InstitutionalResult<()> {
        self.inner.knowledge_capsules().store(capsule).await?;
        Ok(())
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
        self.inner.knowledge_capsules().latest_status().await
    }

    async fn store_edge_inner(&self, edge: KnowledgeEdgeV1) -> InstitutionalResult<()> {
        self.inner.knowledge_edges().store(edge).await?;
        Ok(())
    }
}

impl<C> KnowledgeStore for GovernedKnowledgeStore<storage_backend::KnowledgeStoreBackend<C>>
where
    C: storage_backend::BackendConnection + Send + Sync,
{
    fn store_analysis(
        &self,
        analysis: MacroFinancialAnalysisV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_analysis_inner(analysis)
    }

    fn load_analysis(
        &self,
        analysis_id: &str,
    ) -> impl Future<Output = InstitutionalResult<Option<MacroFinancialAnalysisV1>>> + Send + '_
    {
        self.load_analysis_inner(analysis_id.to_owned())
    }

    fn store_evidence(
        &self,
        id: String,
        manifest: EvidenceManifestV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_evidence_inner(id, manifest)
    }

    fn append_event(
        &self,
        id: String,
        event: RecordedEventV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.append_event_inner(id, event)
    }

    fn store_source(
        &self,
        source: KnowledgeSourceV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_source_inner(source)
    }

    fn load_sources(
        &self,
        ids: &[String],
    ) -> impl Future<Output = InstitutionalResult<Vec<KnowledgeSourceV1>>> + Send + '_ {
        self.load_sources_inner(ids.to_vec())
    }

    fn store_capsule(
        &self,
        capsule: KnowledgeCapsuleV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_capsule_inner(capsule)
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

    fn store_edge(
        &self,
        edge: KnowledgeEdgeV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.store_edge_inner(edge)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contracts::{Classification, KnowledgeCapsuleV1};

    use super::{connect_durable, KnowledgeStore, SurrealConnectionConfig};

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
            .store_capsule(KnowledgeCapsuleV1 {
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
            })
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
