use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fmt::Write as _;
use std::time::Duration;

use codegen::embedded_surrealdb_schemas;
use contracts::{
    EvidenceManifestV1, KnowledgeCapsuleV1, KnowledgeChangeNotificationV1, KnowledgeEdgeV1,
    KnowledgePublicationStatusV1, KnowledgeRelationshipV1, KnowledgeRetrievalHitV1,
    KnowledgeRetrievalQueryV1, KnowledgeSourceV1, MacroFinancialAnalysisV1,
    TreasuryDisbursementRecordedV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use events::RecordedEventV1;
use futures::{StreamExt, stream::BoxStream};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::engine::any::{self, Any};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::opt::auth::Root;
use surrealdb::types::Value as SurrealData;
use surrealdb::{Connection, Notification, Surreal};
use surrealdb_model::{
    EventRecordV1, EvidenceManifestRecordV1, KnowledgeAnalysisRecordV1, KnowledgeCapsuleRecordV1,
    KnowledgeChangeNotificationRecordV1, KnowledgeChunkRecordV1, KnowledgeEdgeRecordV1,
    KnowledgePublicationStatusRecordV1, KnowledgeSourceRecordV1, TreasuryDisbursementRecordV1,
    WorkflowExecutionRecordV1,
};

pub const DEFAULT_NAMESPACE: &str = "short_origin";
pub const DEFAULT_DATABASE: &str = "institutional";
pub const LATEST_PUBLICATION_STATUS_ID: &str = "latest";
pub const ENV_ENDPOINT: &str = "ORIGIN_SURREALDB_ENDPOINT";
pub const ENV_NAMESPACE: &str = "ORIGIN_SURREALDB_NAMESPACE";
pub const ENV_DATABASE: &str = "ORIGIN_SURREALDB_DATABASE";
pub const ENV_USERNAME: &str = "ORIGIN_SURREALDB_USERNAME";
pub const ENV_PASSWORD: &str = "ORIGIN_SURREALDB_PASSWORD";

const FALLBACK_ENV_ENDPOINT: &str = "SURREALDB_ENDPOINT";
const FALLBACK_ENV_NAMESPACE: &str = "SURREALDB_NAMESPACE";
const FALLBACK_ENV_DATABASE: &str = "SURREALDB_DATABASE";
const FALLBACK_ENV_USERNAME: &str = "SURREALDB_USERNAME";
const FALLBACK_ENV_PASSWORD: &str = "SURREALDB_PASSWORD";

pub use surrealdb::Connection as BackendConnection;

pub type KnowledgeStoreBackend<C> = SurrealRepositoryContext<C>;
pub type InMemoryKnowledgeStoreBackend = SurrealRepositoryContext<Db>;
pub type DurableKnowledgeStoreBackend = SurrealRepositoryContext<Any>;
pub type KnowledgeNotificationStream =
    BoxStream<'static, InstitutionalResult<KnowledgeChangeNotificationRecordV1>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableCatalog {
    pub workflow_execution: &'static str,
    pub evidence_manifest: &'static str,
    pub recorded_event: &'static str,
    pub treasury_disbursement: &'static str,
    pub knowledge_source: &'static str,
    pub knowledge_capsule: &'static str,
    pub knowledge_analysis: &'static str,
    pub knowledge_edge: &'static str,
    pub knowledge_chunk: &'static str,
    pub knowledge_publication_status: &'static str,
    pub knowledge_change_notification: &'static str,
}

pub const TABLES: TableCatalog = TableCatalog {
    workflow_execution: "workflow_execution",
    evidence_manifest: "evidence_manifest",
    recorded_event: "recorded_event",
    treasury_disbursement: "treasury_disbursement",
    knowledge_source: "knowledge_source",
    knowledge_capsule: "knowledge_capsule",
    knowledge_analysis: "knowledge_analysis",
    knowledge_edge: "knowledge_edge",
    knowledge_chunk: "knowledge_chunk",
    knowledge_publication_status: "knowledge_publication_status",
    knowledge_change_notification: "knowledge_change_notification",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurrealConnectionConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl SurrealConnectionConfig {
    #[must_use]
    pub fn new(
        endpoint: impl Into<String>,
        namespace: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            namespace: namespace.into(),
            database: database.into(),
            username: None,
            password: None,
        }
    }

    #[must_use]
    pub fn with_root_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn from_env() -> InstitutionalResult<Self> {
        Self::from_env_with(|key| env::var(key).ok())
    }

    fn from_env_with<F>(mut read: F) -> InstitutionalResult<Self>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let endpoint = read_required_env(&mut read, ENV_ENDPOINT, Some(FALLBACK_ENV_ENDPOINT))?;
        let namespace = read_env(&mut read, ENV_NAMESPACE, Some(FALLBACK_ENV_NAMESPACE))
            .unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());
        let database = read_env(&mut read, ENV_DATABASE, Some(FALLBACK_ENV_DATABASE))
            .unwrap_or_else(|| DEFAULT_DATABASE.to_string());
        let username = read_env(&mut read, ENV_USERNAME, Some(FALLBACK_ENV_USERNAME));
        let password = read_env(&mut read, ENV_PASSWORD, Some(FALLBACK_ENV_PASSWORD));
        let config = Self {
            endpoint,
            namespace,
            database,
            username,
            password,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> InstitutionalResult<()> {
        if self.endpoint.trim().is_empty() {
            return Err(configuration_error(
                "validate_connection_config",
                format!("environment variable `{ENV_ENDPOINT}` must not be empty"),
            ));
        }
        if !supported_endpoint(self.endpoint.as_str()) {
            return Err(configuration_error(
                "validate_connection_config",
                format!(
                    "endpoint `{}` is not supported; use `ws://`, `wss://`, `mem://`, or `memory`",
                    self.endpoint
                ),
            ));
        }
        if self.namespace.trim().is_empty() {
            return Err(configuration_error(
                "validate_connection_config",
                "SurrealDB namespace must not be empty",
            ));
        }
        if self.database.trim().is_empty() {
            return Err(configuration_error(
                "validate_connection_config",
                "SurrealDB database must not be empty",
            ));
        }
        if self.username.is_some() ^ self.password.is_some() {
            return Err(configuration_error(
                "validate_connection_config",
                "username and password must be provided together or omitted together",
            ));
        }
        if requires_root_auth(self.endpoint.as_str())
            && (self.username.is_none() || self.password.is_none())
        {
            return Err(configuration_error(
                "validate_connection_config",
                format!(
                    "remote SurrealDB endpoints require both `{ENV_USERNAME}` and `{ENV_PASSWORD}`"
                ),
            ));
        }
        Ok(())
    }
}

pub struct SurrealRepositoryContext<C>
where
    C: Connection,
{
    db: Surreal<C>,
    notification_delivery: NotificationDeliveryMode,
}

impl<C> Clone for SurrealRepositoryContext<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            notification_delivery: self.notification_delivery,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum NotificationDeliveryMode {
    LiveQuery,
    Polling { poll_interval: Duration },
}

impl<C> SurrealRepositoryContext<C>
where
    C: Connection,
{
    #[must_use]
    pub fn new(db: Surreal<C>) -> Self {
        Self {
            db,
            notification_delivery: NotificationDeliveryMode::LiveQuery,
        }
    }

    #[must_use]
    fn with_notification_delivery(
        mut self,
        notification_delivery: NotificationDeliveryMode,
    ) -> Self {
        self.notification_delivery = notification_delivery;
        self
    }

    pub async fn use_namespace(&self, namespace: &str, database: &str) -> InstitutionalResult<()> {
        self.db
            .use_ns(namespace)
            .use_db(database)
            .await
            .map_err(surreal_error)?;
        Ok(())
    }

    pub async fn healthcheck(&self) -> InstitutionalResult<()> {
        self.db
            .health()
            .await
            .map_err(|error| surreal_operation_error("health", error))?;
        Ok(())
    }

    pub async fn bootstrap_schema(&self) -> InstitutionalResult<()> {
        for statement in surreal_schema_statements()? {
            let mut response = self.db.query(statement).await.map_err(surreal_error)?;
            let errors = response.take_errors();
            if !errors.is_empty() {
                return Err(query_errors_to_error(errors));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn workflow_executions(&self) -> WorkflowExecutionRepository<C> {
        WorkflowExecutionRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn evidence_manifests(&self) -> EvidenceManifestRepository<C> {
        EvidenceManifestRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn recorded_events(&self) -> RecordedEventRepository<C> {
        RecordedEventRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn treasury_disbursements(&self) -> TreasuryDisbursementRepository<C> {
        TreasuryDisbursementRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_sources(&self) -> KnowledgeSourceRepository<C> {
        KnowledgeSourceRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_capsules(&self) -> KnowledgeCapsuleRepository<C> {
        KnowledgeCapsuleRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_analyses(&self) -> KnowledgeAnalysisRepository<C> {
        KnowledgeAnalysisRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_edges(&self) -> KnowledgeEdgeRepository<C> {
        KnowledgeEdgeRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn knowledge_chunks(&self) -> KnowledgeChunkRepository<C> {
        KnowledgeChunkRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn publication_status(&self) -> KnowledgePublicationStatusRepository<C> {
        KnowledgePublicationStatusRepository {
            db: self.db.clone(),
        }
    }

    #[must_use]
    pub fn change_notifications(&self) -> KnowledgeChangeNotificationRepository<C> {
        KnowledgeChangeNotificationRepository {
            db: self.db.clone(),
            notification_delivery: self.notification_delivery,
        }
    }

    pub async fn store_sources_batch(
        &self,
        sources: Vec<KnowledgeSourceV1>,
        events: Vec<EventRecordV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        let source_records = sources
            .into_iter()
            .map(|source| KnowledgeSourceRecordV1 {
                id: source.source_id.clone(),
                source,
            })
            .collect::<Vec<_>>();
        let notification_records = notifications
            .into_iter()
            .map(KnowledgeChangeNotificationRecordV1::from_notification)
            .collect::<Vec<_>>();
        execute_batch_transaction(
            &self.db,
            &[
                TransactionBatch::records(TABLES.knowledge_source, source_records)?,
                TransactionBatch::append_only_records(TABLES.recorded_event, events)?,
                TransactionBatch::append_only_records(
                    TABLES.knowledge_change_notification,
                    notification_records,
                )?,
            ],
        )
        .await
    }

    pub async fn store_publication_bundle(
        &self,
        capsule: KnowledgeCapsuleV1,
        chunks: Vec<KnowledgeChunkRecordV1>,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        let capsule_record = KnowledgeCapsuleRecordV1 {
            id: capsule.capsule_id.clone(),
            capsule,
        };
        let edge_records = edges
            .into_iter()
            .map(edge_record_for_relation)
            .collect::<InstitutionalResult<Vec<_>>>()?;
        let notification_records = notifications
            .into_iter()
            .map(KnowledgeChangeNotificationRecordV1::from_notification)
            .collect::<Vec<_>>();
        execute_batch_transaction(
            &self.db,
            &[
                TransactionBatch::records(TABLES.knowledge_capsule, vec![capsule_record])?,
                TransactionBatch::records(TABLES.knowledge_chunk, chunks)?,
                TransactionBatch::relations(TABLES.knowledge_edge, edge_records),
                TransactionBatch::append_only_records(TABLES.recorded_event, events)?,
                TransactionBatch::append_only_records(
                    TABLES.knowledge_change_notification,
                    notification_records,
                )?,
            ],
        )
        .await
    }

    pub async fn store_analysis_bundle(
        &self,
        analysis: MacroFinancialAnalysisV1,
        evidence_id: String,
        manifest: EvidenceManifestV1,
        events: Vec<EventRecordV1>,
        edges: Vec<KnowledgeEdgeV1>,
        notifications: Vec<KnowledgeChangeNotificationV1>,
    ) -> InstitutionalResult<()> {
        let analysis_record = KnowledgeAnalysisRecordV1 {
            id: analysis.analysis_id.clone(),
            analysis,
        };
        let evidence_record = EvidenceManifestRecordV1 {
            id: evidence_id,
            evidence: manifest,
        };
        let edge_records = edges
            .into_iter()
            .map(edge_record_for_relation)
            .collect::<InstitutionalResult<Vec<_>>>()?;
        let notification_records = notifications
            .into_iter()
            .map(KnowledgeChangeNotificationRecordV1::from_notification)
            .collect::<Vec<_>>();
        execute_batch_transaction(
            &self.db,
            &[
                TransactionBatch::records(TABLES.knowledge_analysis, vec![analysis_record])?,
                TransactionBatch::records(TABLES.evidence_manifest, vec![evidence_record])?,
                TransactionBatch::relations(TABLES.knowledge_edge, edge_records),
                TransactionBatch::append_only_records(TABLES.recorded_event, events)?,
                TransactionBatch::append_only_records(
                    TABLES.knowledge_change_notification,
                    notification_records,
                )?,
            ],
        )
        .await
    }
}

pub async fn connect_in_memory() -> InstitutionalResult<SurrealRepositoryContext<Db>> {
    let db = Surreal::new::<Mem>(()).await.map_err(surreal_error)?;
    let context = SurrealRepositoryContext::new(db).with_notification_delivery(
        NotificationDeliveryMode::Polling {
            poll_interval: Duration::from_millis(25),
        },
    );
    context
        .use_namespace(DEFAULT_NAMESPACE, DEFAULT_DATABASE)
        .await?;
    context.bootstrap_schema().await?;
    Ok(context)
}

pub async fn connect_durable(
    config: &SurrealConnectionConfig,
) -> InstitutionalResult<SurrealRepositoryContext<Any>> {
    config.validate()?;
    let db = any::connect(config.endpoint.clone())
        .await
        .map_err(|error| surreal_operation_error("connect", error))?;
    if requires_root_auth(config.endpoint.as_str()) {
        let username = config.username.as_ref().ok_or_else(|| {
            configuration_error(
                "connect",
                format!("missing `{ENV_USERNAME}` for remote SurrealDB endpoint"),
            )
        })?;
        let password = config.password.as_ref().ok_or_else(|| {
            configuration_error(
                "connect",
                format!("missing `{ENV_PASSWORD}` for remote SurrealDB endpoint"),
            )
        })?;
        db.signin(Root {
            username: username.clone(),
            password: password.clone(),
        })
        .await
        .map_err(|error| surreal_operation_error("signin", error))?;
    }
    let context = SurrealRepositoryContext::new(db);
    context.healthcheck().await?;
    context
        .use_namespace(config.namespace.as_str(), config.database.as_str())
        .await?;
    context.bootstrap_schema().await?;
    Ok(context)
}

pub async fn connect_durable_from_env() -> InstitutionalResult<SurrealRepositoryContext<Any>> {
    let config = SurrealConnectionConfig::from_env()?;
    connect_durable(&config).await
}

pub struct WorkflowExecutionRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> WorkflowExecutionRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        record: WorkflowExecutionRecordV1,
    ) -> InstitutionalResult<WorkflowExecutionRecordV1> {
        create_record(
            &self.db,
            TABLES.workflow_execution,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<WorkflowExecutionRecordV1>> {
        select_record(&self.db, TABLES.workflow_execution, id).await
    }
}

pub struct EvidenceManifestRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> EvidenceManifestRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        id: impl Into<String>,
        evidence: EvidenceManifestV1,
    ) -> InstitutionalResult<EvidenceManifestRecordV1> {
        let record = EvidenceManifestRecordV1 {
            id: id.into(),
            evidence,
        };
        create_record(
            &self.db,
            TABLES.evidence_manifest,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<EvidenceManifestRecordV1>> {
        select_record(&self.db, TABLES.evidence_manifest, id).await
    }
}

pub struct RecordedEventRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> RecordedEventRepository<C>
where
    C: Connection,
{
    pub async fn append(
        &self,
        id: impl Into<String>,
        event: RecordedEventV1,
    ) -> InstitutionalResult<EventRecordV1> {
        let record = EventRecordV1 {
            id: id.into(),
            event,
        };
        create_record(&self.db, TABLES.recorded_event, record.id.clone(), record).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<EventRecordV1>> {
        select_record(&self.db, TABLES.recorded_event, id).await
    }
}

pub struct TreasuryDisbursementRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> TreasuryDisbursementRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        id: impl Into<String>,
        disbursement: TreasuryDisbursementRecordedV1,
    ) -> InstitutionalResult<TreasuryDisbursementRecordV1> {
        let record = TreasuryDisbursementRecordV1 {
            id: id.into(),
            disbursement,
        };
        create_record(
            &self.db,
            TABLES.treasury_disbursement,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(
        &self,
        id: &str,
    ) -> InstitutionalResult<Option<TreasuryDisbursementRecordV1>> {
        select_record(&self.db, TABLES.treasury_disbursement, id).await
    }
}

pub struct KnowledgeSourceRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeSourceRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        source: KnowledgeSourceV1,
    ) -> InstitutionalResult<KnowledgeSourceRecordV1> {
        let record = KnowledgeSourceRecordV1 {
            id: source.source_id.clone(),
            source,
        };
        create_record(&self.db, TABLES.knowledge_source, record.id.clone(), record).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeSourceRecordV1>> {
        select_record(&self.db, TABLES.knowledge_source, id).await
    }

    pub async fn load_many(
        &self,
        ids: &[String],
    ) -> InstitutionalResult<Vec<KnowledgeSourceRecordV1>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(record) = self.load(id).await? {
                out.push(record);
            }
        }
        Ok(out)
    }
}

pub struct KnowledgeCapsuleRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeCapsuleRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        capsule: KnowledgeCapsuleV1,
    ) -> InstitutionalResult<KnowledgeCapsuleRecordV1> {
        let record = KnowledgeCapsuleRecordV1 {
            id: capsule.capsule_id.clone(),
            capsule,
        };
        create_record(
            &self.db,
            TABLES.knowledge_capsule,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeCapsuleRecordV1>> {
        select_record(&self.db, TABLES.knowledge_capsule, id).await
    }
}

pub struct KnowledgeAnalysisRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeAnalysisRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        analysis: MacroFinancialAnalysisV1,
    ) -> InstitutionalResult<KnowledgeAnalysisRecordV1> {
        let record = KnowledgeAnalysisRecordV1 {
            id: analysis.analysis_id.clone(),
            analysis,
        };
        create_record(
            &self.db,
            TABLES.knowledge_analysis,
            record.id.clone(),
            record,
        )
        .await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeAnalysisRecordV1>> {
        select_record(&self.db, TABLES.knowledge_analysis, id).await
    }
}

pub struct KnowledgeEdgeRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeEdgeRepository<C>
where
    C: Connection,
{
    pub async fn store(&self, edge: KnowledgeEdgeV1) -> InstitutionalResult<KnowledgeEdgeRecordV1> {
        let record = edge_record_for_relation(edge)?;
        upsert_relation_record(&self.db, TABLES.knowledge_edge, &record).await?;
        Ok(record)
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeEdgeRecordV1>> {
        select_relation_record(&self.db, TABLES.knowledge_edge, id).await
    }

    pub async fn load_many(
        &self,
        ids: &[String],
    ) -> InstitutionalResult<Vec<KnowledgeEdgeRecordV1>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(record) = self.load(id).await? {
                out.push(record);
            }
        }
        Ok(out)
    }
}

pub struct KnowledgeChunkRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgeChunkRepository<C>
where
    C: Connection,
{
    pub async fn store(
        &self,
        chunk: KnowledgeChunkRecordV1,
    ) -> InstitutionalResult<KnowledgeChunkRecordV1> {
        create_record(&self.db, TABLES.knowledge_chunk, chunk.id.clone(), chunk).await
    }

    pub async fn load(&self, id: &str) -> InstitutionalResult<Option<KnowledgeChunkRecordV1>> {
        select_record(&self.db, TABLES.knowledge_chunk, id).await
    }

    pub async fn search(
        &self,
        query: KnowledgeRetrievalQueryV1,
    ) -> InstitutionalResult<Vec<KnowledgeRetrievalHitV1>> {
        let mut where_clauses = vec![
            "classification = $classification".to_string(),
            "search_text @0@ $query_text".to_string(),
        ];
        let mut binds = vec![
            (
                "classification".to_string(),
                serde_json::to_value(query.classification).map_err(json_error)?,
            ),
            (
                "query_text".to_string(),
                Value::String(query.query_text.clone()),
            ),
            (
                "top_k".to_string(),
                serde_json::to_value(query.top_k).map_err(json_error)?,
            ),
        ];
        if let Some(capsule_id) = query.selector.capsule_id.as_ref() {
            where_clauses.push("capsule_id = $capsule_id".to_string());
            binds.push(("capsule_id".to_string(), Value::String(capsule_id.clone())));
        }
        if !query.selector.source_ids.is_empty() {
            where_clauses.push("source_id INSIDE $source_ids".to_string());
            binds.push((
                "source_ids".to_string(),
                serde_json::to_value(query.selector.source_ids).map_err(json_error)?,
            ));
        }
        if !query.selector.country_areas.is_empty() {
            where_clauses.push("country_area INSIDE $country_areas".to_string());
            binds.push((
                "country_areas".to_string(),
                serde_json::to_value(query.selector.country_areas).map_err(json_error)?,
            ));
        }
        if !query.selector.source_kinds.is_empty() {
            where_clauses.push("source_kind INSIDE $source_kinds".to_string());
            binds.push((
                "source_kinds".to_string(),
                serde_json::to_value(query.selector.source_kinds).map_err(json_error)?,
            ));
        }

        let sql = format!(
            "SELECT *, type::string(id) AS id, search::score(0) AS score, search::highlight('<mark>', '</mark>', 0) AS highlight FROM {} WHERE {} ORDER BY score DESC, source_id ASC, chunk_index ASC LIMIT $top_k;",
            TABLES.knowledge_chunk,
            where_clauses.join(" AND "),
        );
        let mut response = bind_all(self.db.query(sql), binds)
            .await
            .map_err(surreal_error)?;
        let rows = take_rows::<KnowledgeChunkSearchRow>(&mut response, 0)?;
        Ok(rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| {
                row.record.to_hit(
                    truncate_snippet(
                        row.highlight
                            .unwrap_or_else(|| row.record.search_text.clone()),
                        query.snippet_chars,
                    ),
                    index + 1,
                    row.score,
                )
            })
            .collect())
    }
}

pub struct KnowledgePublicationStatusRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> KnowledgePublicationStatusRepository<C>
where
    C: Connection,
{
    pub async fn latest(&self) -> InstitutionalResult<Option<KnowledgePublicationStatusV1>> {
        Ok(select_record::<_, KnowledgePublicationStatusRecordV1>(
            &self.db,
            TABLES.knowledge_publication_status,
            LATEST_PUBLICATION_STATUS_ID,
        )
        .await?
        .map(|record| record.as_status()))
    }
}

pub struct KnowledgeChangeNotificationRepository<C>
where
    C: Connection,
{
    db: Surreal<C>,
    notification_delivery: NotificationDeliveryMode,
}

impl<C> KnowledgeChangeNotificationRepository<C>
where
    C: Connection,
{
    pub async fn recent(
        &self,
        limit: usize,
    ) -> InstitutionalResult<Vec<KnowledgeChangeNotificationRecordV1>> {
        load_recent_notification_records(&self.db, limit).await
    }

    pub async fn subscribe(&self) -> InstitutionalResult<KnowledgeNotificationStream> {
        match self.notification_delivery {
            NotificationDeliveryMode::LiveQuery => {
                let stream = self
                    .db
                    .select(TABLES.knowledge_change_notification)
                    .live()
                    .await
                    .map_err(surreal_error)?;
                Ok(stream
                    .map(|result| {
                        result
                            .map_err(surreal_error)
                            .and_then(deserialize_notification)
                    })
                    .boxed())
            }
            NotificationDeliveryMode::Polling { poll_interval } => {
                let seen_ids = self
                    .recent(256)
                    .await?
                    .into_iter()
                    .map(|record| record.id)
                    .collect::<HashSet<_>>();
                Ok(poll_notification_records(self.db.clone(), poll_interval, seen_ids).boxed())
            }
        }
    }
}

#[derive(Debug, Clone)]
enum TransactionBatch {
    Records {
        table: &'static str,
        rows: Vec<StoredRow>,
        mode: TransactionWriteMode,
    },
    Relations {
        table: &'static str,
        rows: Vec<KnowledgeEdgeRecordV1>,
    },
}

#[derive(Debug, Clone, Copy)]
enum TransactionWriteMode {
    Upsert,
    Create,
}

impl TransactionBatch {
    fn records<T>(table: &'static str, records: Vec<T>) -> InstitutionalResult<Self>
    where
        T: Clone + Serialize,
    {
        Self::records_with_mode(table, records, TransactionWriteMode::Upsert)
    }

    fn append_only_records<T>(table: &'static str, records: Vec<T>) -> InstitutionalResult<Self>
    where
        T: Clone + Serialize,
    {
        Self::records_with_mode(table, records, TransactionWriteMode::Create)
    }

    fn records_with_mode<T>(
        table: &'static str,
        records: Vec<T>,
        mode: TransactionWriteMode,
    ) -> InstitutionalResult<Self>
    where
        T: Clone + Serialize,
    {
        let rows = records
            .into_iter()
            .map(|record| {
                let value = serde_json::to_value(record.clone()).map_err(json_error)?;
                let id = value
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        InstitutionalError::invariant(
                            OperationContext::new("shared/surrealdb-access", "serialize_record"),
                            "serialized record missing `id` field",
                        )
                    })?
                    .to_string();
                Ok(StoredRow {
                    id,
                    content: remove_id_field(value),
                })
            })
            .collect::<InstitutionalResult<Vec<_>>>()?;
        Ok(Self::Records { table, rows, mode })
    }

    #[must_use]
    fn relations(table: &'static str, rows: Vec<KnowledgeEdgeRecordV1>) -> Self {
        Self::Relations { table, rows }
    }
}

#[derive(Debug, Clone)]
struct StoredRow {
    id: String,
    content: Value,
}

#[derive(Debug, Deserialize)]
struct KnowledgeChunkSearchRow {
    #[serde(flatten)]
    record: KnowledgeChunkRecordV1,
    score: Option<f32>,
    highlight: Option<String>,
}

async fn execute_batch_transaction<C>(
    db: &Surreal<C>,
    batches: &[TransactionBatch],
) -> InstitutionalResult<()>
where
    C: Connection,
{
    let mut sql = String::from("BEGIN TRANSACTION;\n");
    let mut binds = Vec::new();

    for batch in batches {
        match batch {
            TransactionBatch::Records { table, rows, mode } => {
                for (index, row) in rows.iter().enumerate() {
                    let id_key = format!("{table}_id_{index}");
                    let content_key = format!("{table}_content_{index}");
                    let statement = match mode {
                        TransactionWriteMode::Upsert => {
                            format!(
                                "UPSERT type::record('{table}', ${id_key}) CONTENT ${content_key};"
                            )
                        }
                        TransactionWriteMode::Create => {
                            format!(
                                "CREATE type::record('{table}', ${id_key}) CONTENT ${content_key};"
                            )
                        }
                    };
                    let _ = writeln!(sql, "{statement}");
                    binds.push((id_key, Value::String(row.id.clone())));
                    binds.push((content_key, row.content.clone()));
                }
            }
            TransactionBatch::Relations { table, rows } => {
                for (index, row) in rows.iter().enumerate() {
                    let prefix = format!("{table}_{index}");
                    let _ = writeln!(
                        sql,
                        "INSERT RELATION INTO {table} {{ id: ${prefix}_id, in: type::record(${prefix}_in_table, ${prefix}_from_id), out: type::record(${prefix}_out_table, ${prefix}_to_id), from_id: ${prefix}_from_id, to_id: ${prefix}_to_id, relationship: ${prefix}_relationship, rationale: ${prefix}_rationale }};"
                    );
                    binds.push((format!("{prefix}_id"), Value::String(row.id.clone())));
                    binds.push((
                        format!("{prefix}_in_table"),
                        Value::String(record_table_from_thing(&row.r#in)),
                    ));
                    binds.push((
                        format!("{prefix}_out_table"),
                        Value::String(record_table_from_thing(&row.r#out)),
                    ));
                    binds.push((
                        format!("{prefix}_from_id"),
                        Value::String(row.from_id.clone()),
                    ));
                    binds.push((format!("{prefix}_to_id"), Value::String(row.to_id.clone())));
                    binds.push((
                        format!("{prefix}_relationship"),
                        serde_json::to_value(row.relationship).map_err(json_error)?,
                    ));
                    binds.push((
                        format!("{prefix}_rationale"),
                        Value::String(row.rationale.clone()),
                    ));
                }
            }
        }
    }

    sql.push_str("COMMIT TRANSACTION;");
    let mut response = bind_all(db.query(sql), binds)
        .await
        .map_err(surreal_error)?;
    let errors = response.take_errors();
    if !errors.is_empty() {
        return Err(query_errors_to_error(errors));
    }
    Ok(())
}

async fn upsert_relation_record<C>(
    db: &Surreal<C>,
    table: &str,
    record: &KnowledgeEdgeRecordV1,
) -> InstitutionalResult<()>
where
    C: Connection,
{
    let sql = format!(
        "INSERT RELATION INTO {table} {{ id: $id, in: type::record($in_table, $from_id), out: type::record($out_table, $to_id), from_id: $from_id, to_id: $to_id, relationship: $relationship, rationale: $rationale }};"
    );
    let mut response = db
        .query(sql)
        .bind(("id", record.id.clone()))
        .bind(("in_table", record_table_from_thing(&record.r#in)))
        .bind(("out_table", record_table_from_thing(&record.r#out)))
        .bind(("from_id", record.from_id.clone()))
        .bind(("to_id", record.to_id.clone()))
        .bind((
            "relationship",
            serde_json::to_value(record.relationship).map_err(json_error)?,
        ))
        .bind(("rationale", record.rationale.clone()))
        .await
        .map_err(surreal_error)?;
    let errors = response.take_errors();
    if !errors.is_empty() {
        return Err(query_errors_to_error(errors));
    }
    Ok(())
}

async fn create_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: String,
    record: T,
) -> InstitutionalResult<T>
where
    C: Connection,
    T: Clone + Serialize + DeserializeOwned + 'static,
{
    let content = remove_id_field(serde_json::to_value(record.clone()).map_err(json_error)?);
    let sql = format!("UPSERT type::record('{table}', $id) CONTENT $content;");
    let mut response = db
        .query(sql)
        .bind(("id", id))
        .bind(("content", content))
        .await
        .map_err(surreal_error)?;
    let errors = response.take_errors();
    if !errors.is_empty() {
        return Err(query_errors_to_error(errors));
    }
    Ok(record)
}

async fn select_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: &str,
) -> InstitutionalResult<Option<T>>
where
    C: Connection,
    T: DeserializeOwned,
{
    let sql = format!(
        "SELECT *, type::string(id) AS id FROM {table} WHERE id = type::record('{table}', $id) LIMIT 1;"
    );
    let mut response = db
        .query(sql)
        .bind(("id", id.to_string()))
        .await
        .map_err(surreal_error)?;
    let mut rows = take_rows::<T>(&mut response, 0)?;
    Ok(rows.pop())
}

async fn select_relation_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: &str,
) -> InstitutionalResult<Option<T>>
where
    C: Connection,
    T: DeserializeOwned,
{
    let sql = format!(
        "SELECT *, type::string(id) AS id, type::string(in) AS in, type::string(out) AS out FROM {table} WHERE id = type::record('{table}', $id) LIMIT 1;"
    );
    let mut response = db
        .query(sql)
        .bind(("id", id.to_string()))
        .await
        .map_err(surreal_error)?;
    let mut rows = take_rows::<T>(&mut response, 0)?;
    Ok(rows.pop())
}

fn surreal_schema_statements() -> InstitutionalResult<Vec<String>> {
    let mut statements = Vec::new();
    for schema in embedded_surrealdb_schemas()? {
        if let Some(entries) = schema.document.get("statements").and_then(Value::as_array) {
            for entry in entries {
                let statement = entry.as_str().ok_or_else(|| {
                    InstitutionalError::parse(
                        schema.name.clone(),
                        "surrealdb schema statements must be string entries",
                    )
                })?;
                statements.push(statement.to_string());
            }
        }
    }
    Ok(statements)
}

fn bind_all<C>(
    mut query: surrealdb::method::Query<'_, C>,
    binds: Vec<(String, Value)>,
) -> surrealdb::method::Query<'_, C>
where
    C: Connection,
{
    for (key, value) in binds {
        query = query.bind((key, value));
    }
    query
}

fn edge_record_for_relation(edge: KnowledgeEdgeV1) -> InstitutionalResult<KnowledgeEdgeRecordV1> {
    let (in_table, out_table) = match edge.relationship {
        KnowledgeRelationshipV1::DerivedFrom => (TABLES.knowledge_capsule, TABLES.knowledge_source),
        KnowledgeRelationshipV1::Supports => (TABLES.knowledge_analysis, TABLES.knowledge_capsule),
        KnowledgeRelationshipV1::Cites => (TABLES.knowledge_analysis, TABLES.knowledge_source),
        KnowledgeRelationshipV1::RetainedBy => {
            return Err(InstitutionalError::validation(
                OperationContext::new("shared/surrealdb-access", "edge_record_for_relation"),
                "retained_by knowledge relations are not mapped in the pilot data plane",
            ));
        }
    };
    let in_record = format!("{in_table}:{}", edge.from_id);
    let out_record = format!("{out_table}:{}", edge.to_id);
    Ok(KnowledgeEdgeRecordV1::from_edge(
        edge, in_record, out_record,
    ))
}

fn record_table_from_thing(thing: &str) -> String {
    thing.split(':').next().unwrap_or_default().to_string()
}

fn remove_id_field(value: Value) -> Value {
    match value {
        Value::Object(mut map) => {
            map.remove("id");
            Value::Object(
                map.into_iter()
                    .filter_map(|(key, value)| strip_null_object_field(key, value))
                    .collect(),
            )
        }
        other => other,
    }
}

fn strip_null_object_field(key: String, value: Value) -> Option<(String, Value)> {
    let normalized = normalize_content_value(value);
    if normalized.is_null() {
        None
    } else {
        Some((key, normalized))
    }
}

fn normalize_content_value(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter_map(|(key, value)| strip_null_object_field(key, value))
                .collect(),
        ),
        Value::Array(values) => {
            Value::Array(values.into_iter().map(normalize_content_value).collect())
        }
        other => other,
    }
}

fn read_required_env<F>(
    read: &mut F,
    primary_key: &str,
    fallback_key: Option<&str>,
) -> InstitutionalResult<String>
where
    F: FnMut(&str) -> Option<String>,
{
    read_env(read, primary_key, fallback_key).ok_or_else(|| {
        configuration_error(
            "load-connection-config",
            format!("environment variable `{primary_key}` is required"),
        )
    })
}

fn read_env<F>(read: &mut F, primary_key: &str, fallback_key: Option<&str>) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    read_env_key(read, primary_key).or_else(|| fallback_key.and_then(|key| read_env_key(read, key)))
}

fn read_env_key<F>(read: &mut F, key: &str) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    read(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn supported_endpoint(endpoint: &str) -> bool {
    let endpoint = endpoint.trim().to_ascii_lowercase();
    endpoint == "memory"
        || endpoint == "mem://"
        || endpoint.starts_with("ws://")
        || endpoint.starts_with("wss://")
}

fn requires_root_auth(endpoint: &str) -> bool {
    let endpoint = endpoint.trim().to_ascii_lowercase();
    endpoint.starts_with("ws://") || endpoint.starts_with("wss://")
}

fn truncate_snippet(snippet: String, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    snippet.chars().take(max_chars).collect()
}

fn json_error(error: serde_json::Error) -> InstitutionalError {
    InstitutionalError::parse_with_parser(
        "surrealdb-serialization",
        "serde_json",
        error.to_string(),
    )
}

fn configuration_error(operation: &'static str, message: impl Into<String>) -> InstitutionalError {
    InstitutionalError::configuration(
        OperationContext::new("shared/surrealdb-access", operation),
        message,
    )
}

fn surreal_error(error: surrealdb::Error) -> InstitutionalError {
    InstitutionalError::external("surrealdb", None, error.to_string())
}

fn surreal_operation_error(operation: &'static str, error: surrealdb::Error) -> InstitutionalError {
    InstitutionalError::external("surrealdb", Some(operation.to_string()), error.to_string())
}

fn query_errors_to_error(errors: HashMap<usize, surrealdb::Error>) -> InstitutionalError {
    let mut entries = errors.into_iter().collect::<Vec<_>>();
    entries.sort_by_key(|(index, _)| *index);
    let details = entries
        .into_iter()
        .map(|(index, error)| format!("statement {index}: {error}"))
        .collect::<Vec<_>>()
        .join(" | ");
    InstitutionalError::external("surrealdb", None, details)
}

fn deserialize_row<T>(value: Value) -> InstitutionalResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(value).map_err(json_error)
}

fn take_rows<T>(
    response: &mut surrealdb::IndexedResults,
    index: usize,
) -> InstitutionalResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let rows: Vec<Value> = response.take(index).map_err(surreal_error)?;
    rows.into_iter().map(deserialize_row::<T>).collect()
}

fn deserialize_notification(
    notification: Notification<SurrealData>,
) -> InstitutionalResult<KnowledgeChangeNotificationRecordV1> {
    let value = serde_json::to_value(notification.data).map_err(json_error)?;
    deserialize_row(normalize_live_notification_value(value))
}

fn normalize_live_notification_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            if let Some(record_id) = stringify_record_id_object(&map) {
                Value::String(record_id)
            } else {
                Value::Object(
                    map.into_iter()
                        .map(|(key, value)| (key, normalize_live_notification_value(value)))
                        .collect(),
                )
            }
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(normalize_live_notification_value)
                .collect(),
        ),
        other => other,
    }
}

fn stringify_record_id_object(map: &serde_json::Map<String, Value>) -> Option<String> {
    let table = map.get("table")?.as_str()?;
    let key = map.get("key")?;
    Some(format!("{table}:{}", stringify_record_id_key(key)?))
}

fn stringify_record_id_key(value: &Value) -> Option<String> {
    match value {
        Value::String(string) => Some(string.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Null => None,
        other => Some(serde_json::to_string(other).ok()?),
    }
}

fn poll_notification_records<C>(
    db: Surreal<C>,
    poll_interval: Duration,
    seen_ids: HashSet<String>,
) -> impl futures::Stream<Item = InstitutionalResult<KnowledgeChangeNotificationRecordV1>>
where
    C: Connection,
{
    futures::stream::unfold(
        PollingNotificationState {
            db,
            poll_interval,
            seen_ids,
            pending: VecDeque::new(),
        },
        |mut state| async move {
            loop {
                if let Some(record) = state.pending.pop_front() {
                    return Some((Ok(record), state));
                }

                tokio::time::sleep(state.poll_interval).await;
                match load_recent_notification_records(&state.db, 256).await {
                    Ok(records) => {
                        let mut unseen = records
                            .into_iter()
                            .filter(|record| state.seen_ids.insert(record.id.clone()))
                            .collect::<Vec<_>>();
                        unseen.sort_by(|left, right| {
                            left.published_at
                                .cmp(&right.published_at)
                                .then_with(|| left.id.cmp(&right.id))
                        });
                        state.pending = unseen.into();
                    }
                    Err(error) => return Some((Err(error), state)),
                }
            }
        },
    )
}

#[derive(Debug)]
struct PollingNotificationState<C>
where
    C: Connection,
{
    db: Surreal<C>,
    poll_interval: Duration,
    seen_ids: HashSet<String>,
    pending: VecDeque<KnowledgeChangeNotificationRecordV1>,
}

async fn load_recent_notification_records<C>(
    db: &Surreal<C>,
    limit: usize,
) -> InstitutionalResult<Vec<KnowledgeChangeNotificationRecordV1>>
where
    C: Connection,
{
    let sql = format!(
        "SELECT *, type::string(id) AS id FROM {} ORDER BY published_at DESC LIMIT $limit;",
        TABLES.knowledge_change_notification
    );
    let mut response = db
        .query(sql)
        .bind(("limit", limit))
        .await
        .map_err(surreal_error)?;
    take_rows(&mut response, 0)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use contracts::{
        AnalysisAssumptionV1, AnalysisCoverageV1, AnalysisHorizonV1, AnalysisImplicationsV1,
        AnalysisObjectiveV1, ClaimEvidenceV1, ClaimKindV1, Classification, ConfidenceV1,
        DataRegisterEntryV1, DirectionalBiasV1, DriverBucketV1, EvidenceManifestV1,
        ExecutiveBriefV1, ExternalAccountsBalanceSheetMapV1, FxDriverAssessmentV1,
        GlobalLiquidityFundingConditionsV1, GlobalLiquidityPhaseV1, InferenceStepV1,
        KnowledgeAppendixV1, KnowledgeChangeKindV1, KnowledgeChangeNotificationV1,
        KnowledgeDocumentFormatV1, KnowledgeEvidenceUseV1, KnowledgeRelationshipV1,
        KnowledgeRetrievalQueryV1, KnowledgeRetrievalSelectorV1, KnowledgeSourceKindV1,
        KnowledgeSourceProvenanceV1, MacroFinancialAnalysisV1, MechanismMapV1, PayloadRefV1,
        PipelineStepIdV1, PipelineStepTraceV1, PolicyFrictionObservationV1,
        PolicyRegimeDiagnosisV1, ProbabilityV1, ProblemContractV1, RankedRiskV1,
        RiskRegisterEntryV1, ScenarioCaseV1, ScenarioKindV1, SignalMagnitudeV1,
        SignalSummaryEntryV1, SovereignSystemicRiskV1, TransmissionChannelV1,
        TreasuryDisbursementRecordedV1, WatchlistIndicatorV1,
    };
    use events::EventEnvelopeV1;
    use futures::StreamExt;
    use identity::ActorRef;

    use super::*;

    #[tokio::test]
    async fn repositories_round_trip_core_records_and_search() {
        let context = connect_in_memory().await.expect("memory db");
        let workflow = context
            .workflow_executions()
            .store(WorkflowExecutionRecordV1 {
                id: "wf-1".to_string(),
                workflow_name: "treasury_disbursement".to_string(),
                trace_ref: "trace-1".to_string(),
            })
            .await
            .expect("store workflow");
        assert_eq!(workflow.workflow_name, "treasury_disbursement");

        let evidence = context
            .evidence_manifests()
            .store(
                "evidence-1",
                EvidenceManifestV1 {
                    evidence_id: "evidence-1".into(),
                    producer: "tests".to_string(),
                    artifact_hash: "abc".to_string(),
                    storage_ref: "surrealdb:evidence/evidence-1".to_string(),
                    retention_class: "standard".to_string(),
                    classification: Classification::Internal,
                    related_decision_refs: vec!["decision-1".into()],
                },
            )
            .await
            .expect("store evidence");
        assert_eq!(evidence.evidence.producer, "tests");

        let event = context
            .recorded_events()
            .append(
                "event-1",
                RecordedEventV1 {
                    envelope: EventEnvelopeV1::new(
                        "workflow.started",
                        ActorRef("ops:user-1".to_string()),
                        "corr-1",
                        None,
                        Classification::Internal,
                        "schemas/events/v1/workflow-started",
                        "deadbeef",
                    ),
                    payload_ref: PayloadRefV1 {
                        schema_ref: "schemas/contracts/v1/workflow-execution".to_string(),
                        record_id: "event-1".to_string(),
                    },
                },
            )
            .await
            .expect("append event");
        assert_eq!(event.id, "event-1");

        let disbursement = context
            .treasury_disbursements()
            .store(
                "disbursement-1",
                TreasuryDisbursementRecordedV1 {
                    disbursement_id: "disbursement-1".to_string(),
                    workflow_execution_id: "wf-1".to_string(),
                    ledger_ref: "ledger:primary".to_string(),
                    amount_minor: 5000,
                    currency: "USD".to_string(),
                    beneficiary: "Vendor".to_string(),
                    approved_by_roles: Vec::new(),
                },
            )
            .await
            .expect("store disbursement");
        assert_eq!(disbursement.disbursement.currency, "USD");

        let source = KnowledgeSourceV1 {
            source_id: "source-1".to_string(),
            ingestion_id: "ingest-1".to_string(),
            kind: KnowledgeSourceKindV1::Imf,
            title: "IMF External Accounts".to_string(),
            country_area: "Japan".to_string(),
            series_name: Some("BOP".to_string()),
            source_url: "https://www.imf.org/example".to_string(),
            source_domain: "www.imf.org".to_string(),
            format: KnowledgeDocumentFormatV1::Json,
            mime_type: "application/json".to_string(),
            classification: Classification::Internal,
            acquired_at: Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0).single().unwrap(),
            content_digest: "digest-1".to_string(),
            content_text: "Current account surplus remains positive and liquidity is steady."
                .to_string(),
            provenance_tier: KnowledgeSourceProvenanceV1::Primary,
            evidence_use: KnowledgeEvidenceUseV1::Evidence,
            last_observation: Some("2026-02".to_string()),
            units: Some("USD bn".to_string()),
            transform: Some("yoy".to_string()),
            release_lag: Some("T+30d".to_string()),
            quality_flags: Vec::new(),
            notes: vec!["Primary source".to_string()],
            governance_notes: vec!["Primary IMF source".to_string()],
            provider_metadata: std::collections::BTreeMap::new(),
        };
        context
            .store_sources_batch(
                vec![source.clone()],
                vec![],
                vec![KnowledgeChangeNotificationV1 {
                    notification_id: "notification-source".to_string(),
                    kind: KnowledgeChangeKindV1::SourceIngested,
                    record_id: source.source_id.clone(),
                    publication_id: None,
                    capsule_id: None,
                    source_id: Some(source.source_id.clone()),
                    analysis_id: None,
                    published_at: source.acquired_at,
                    classification: Classification::Internal,
                }],
            )
            .await
            .expect("store sources batch");

        let capsule = KnowledgeCapsuleV1 {
            capsule_id: "capsule-1".to_string(),
            publication_id: "publication-1".to_string(),
            title: "Macro capsule".to_string(),
            source_ids: vec![source.source_id.clone()],
            source_count: 1,
            storage_ref: "surrealdb:capsule/capsule-1".to_string(),
            artifact_hash: "capsule-hash".to_string(),
            version: "v2".to_string(),
            memvid_version: "surrealdb-3.0.3".to_string(),
            published_at: source.acquired_at + Duration::minutes(5),
            classification: Classification::Internal,
            retention_class: "institutional_record".to_string(),
        };
        let chunk = KnowledgeChunkRecordV1 {
            id: "chunk-1".to_string(),
            chunk_id: "chunk-1".to_string(),
            capsule_id: capsule.capsule_id.clone(),
            source_id: source.source_id.clone(),
            chunk_index: 0,
            title: source.title.clone(),
            uri: format!("knowledge://source/{}", source.source_id),
            country_area: source.country_area.clone(),
            classification: Classification::Internal,
            source_kind: source.kind,
            search_text: source.content_text.clone(),
            content_digest: source.content_digest.clone(),
            acquired_at: source.acquired_at,
        };
        context
            .store_publication_bundle(
                capsule.clone(),
                vec![chunk],
                vec![],
                vec![KnowledgeEdgeV1 {
                    edge_id: "edge-1".to_string(),
                    from_id: capsule.capsule_id.clone(),
                    to_id: source.source_id.clone(),
                    relationship: KnowledgeRelationshipV1::DerivedFrom,
                    rationale: "Capsule compiled from source.".to_string(),
                }],
                vec![KnowledgeChangeNotificationV1 {
                    notification_id: "notification-capsule".to_string(),
                    kind: KnowledgeChangeKindV1::CapsulePublished,
                    record_id: capsule.capsule_id.clone(),
                    publication_id: Some(capsule.publication_id.clone()),
                    capsule_id: Some(capsule.capsule_id.clone()),
                    source_id: None,
                    analysis_id: None,
                    published_at: capsule.published_at,
                    classification: capsule.classification,
                }],
            )
            .await
            .expect("store publication bundle");

        let hits = context
            .knowledge_chunks()
            .search(KnowledgeRetrievalQueryV1 {
                query_id: "query-1".to_string(),
                actor_ref: ActorRef("ops:user-1".to_string()),
                purpose: "analysis".to_string(),
                classification: Classification::Internal,
                selector: KnowledgeRetrievalSelectorV1 {
                    capsule_id: Some(capsule.capsule_id.clone()),
                    ..KnowledgeRetrievalSelectorV1::default()
                },
                query_text: "liquidity".to_string(),
                top_k: 4,
                snippet_chars: 80,
            })
            .await
            .expect("search chunks");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].source_id, source.source_id);

        let latest = context
            .publication_status()
            .latest()
            .await
            .expect("latest status")
            .expect("projection exists");
        assert_eq!(latest.capsule_id, capsule.capsule_id);
    }

    #[tokio::test]
    async fn publication_bundle_rolls_back_on_invalid_chunk() {
        let context = connect_in_memory().await.expect("memory db");
        let capsule = KnowledgeCapsuleV1 {
            capsule_id: "capsule-fail".to_string(),
            publication_id: "publication-fail".to_string(),
            title: "Broken capsule".to_string(),
            source_ids: vec!["source-1".to_string()],
            source_count: 1,
            storage_ref: "surrealdb:capsule/capsule-fail".to_string(),
            artifact_hash: "capsule-hash".to_string(),
            version: "v2".to_string(),
            memvid_version: "surrealdb-3.0.3".to_string(),
            published_at: Utc.with_ymd_and_hms(2026, 3, 9, 12, 5, 0).single().unwrap(),
            classification: Classification::Internal,
            retention_class: "institutional_record".to_string(),
        };
        let invalid_chunk = KnowledgeChunkRecordV1 {
            id: "chunk-bad".to_string(),
            chunk_id: "chunk-bad".to_string(),
            capsule_id: capsule.capsule_id.clone(),
            source_id: "source-1".to_string(),
            chunk_index: 0,
            title: "Empty".to_string(),
            uri: "knowledge://source/source-1".to_string(),
            country_area: "Japan".to_string(),
            classification: Classification::Internal,
            source_kind: KnowledgeSourceKindV1::Imf,
            search_text: String::new(),
            content_digest: "digest".to_string(),
            acquired_at: Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0).single().unwrap(),
        };

        let error = context
            .store_publication_bundle(capsule.clone(), vec![invalid_chunk], vec![], vec![], vec![])
            .await
            .expect_err("invalid bundle should fail");
        assert_eq!(
            error.category(),
            error_model::InstitutionalErrorCategory::Transport
        );
        assert!(
            context
                .knowledge_capsules()
                .load(&capsule.capsule_id)
                .await
                .expect("load capsule")
                .is_none()
        );
        assert!(
            context
                .publication_status()
                .latest()
                .await
                .expect("load latest")
                .is_none()
        );
    }

    #[tokio::test]
    async fn live_notifications_emit_once_per_committed_transaction() {
        let context = connect_in_memory().await.expect("memory db");
        let mut stream = context
            .change_notifications()
            .subscribe()
            .await
            .expect("subscribe");
        let source = KnowledgeSourceV1 {
            source_id: "source-live".to_string(),
            ingestion_id: "ingest-live".to_string(),
            kind: KnowledgeSourceKindV1::Imf,
            title: "IMF live".to_string(),
            country_area: "Japan".to_string(),
            series_name: None,
            source_url: "https://www.imf.org/live".to_string(),
            source_domain: "www.imf.org".to_string(),
            format: KnowledgeDocumentFormatV1::Text,
            mime_type: "text/plain".to_string(),
            classification: Classification::Internal,
            acquired_at: Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0).single().unwrap(),
            content_digest: "digest-live".to_string(),
            content_text: "Liquidity remains stable.".to_string(),
            provenance_tier: KnowledgeSourceProvenanceV1::Primary,
            evidence_use: KnowledgeEvidenceUseV1::Evidence,
            last_observation: None,
            units: None,
            transform: None,
            release_lag: None,
            quality_flags: Vec::new(),
            notes: Vec::new(),
            governance_notes: Vec::new(),
            provider_metadata: std::collections::BTreeMap::new(),
        };
        context
            .store_sources_batch(
                vec![source.clone()],
                vec![],
                vec![KnowledgeChangeNotificationV1 {
                    notification_id: "notification-live".to_string(),
                    kind: KnowledgeChangeKindV1::SourceIngested,
                    record_id: source.source_id.clone(),
                    publication_id: None,
                    capsule_id: None,
                    source_id: Some(source.source_id.clone()),
                    analysis_id: None,
                    published_at: source.acquired_at,
                    classification: source.classification,
                }],
            )
            .await
            .expect("store live batch");

        let notification = stream
            .next()
            .await
            .expect("notification item")
            .expect("notification result");
        assert_eq!(notification.kind, KnowledgeChangeKindV1::SourceIngested);
        assert_eq!(notification.source_id.as_deref(), Some("source-live"));
    }

    #[test]
    fn durable_connection_config_requires_complete_fields() {
        let error = SurrealConnectionConfig {
            endpoint: String::new(),
            namespace: "ns".to_string(),
            database: "db".to_string(),
            username: None,
            password: None,
        }
        .validate()
        .expect_err("empty endpoint should fail");
        assert_eq!(
            error.category(),
            error_model::InstitutionalErrorCategory::Configuration
        );

        let error = SurrealConnectionConfig {
            endpoint: "127.0.0.1:8000".to_string(),
            namespace: "ns".to_string(),
            database: "db".to_string(),
            username: Some("root".to_string()),
            password: None,
        }
        .validate()
        .expect_err("partial auth should fail");
        assert_eq!(
            error.category(),
            error_model::InstitutionalErrorCategory::Configuration
        );
    }

    #[test]
    fn durable_config_from_env_defaults_namespace_and_database() {
        let values = std::collections::BTreeMap::from([
            (ENV_ENDPOINT.to_string(), "ws://127.0.0.1:8000".to_string()),
            (ENV_USERNAME.to_string(), "root".to_string()),
            (ENV_PASSWORD.to_string(), "root-password".to_string()),
        ]);
        let config =
            SurrealConnectionConfig::from_env_with(|key| values.get(key).cloned()).expect("config");

        assert_eq!(config.endpoint, "ws://127.0.0.1:8000");
        assert_eq!(config.namespace, DEFAULT_NAMESPACE);
        assert_eq!(config.database, DEFAULT_DATABASE);
    }

    #[test]
    fn durable_config_requires_remote_credentials() {
        let error = SurrealConnectionConfig {
            endpoint: "ws://127.0.0.1:8000".to_string(),
            namespace: DEFAULT_NAMESPACE.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            username: None,
            password: None,
        }
        .validate()
        .expect_err("remote credentials required");

        assert_eq!(
            error.category(),
            error_model::InstitutionalErrorCategory::Configuration
        );
        assert!(error.to_string().contains(ENV_USERNAME));
    }

    #[tokio::test]
    async fn durable_connection_supports_memory_endpoint_for_runtime_path() {
        let context = connect_durable(&SurrealConnectionConfig {
            endpoint: "mem://".to_string(),
            namespace: DEFAULT_NAMESPACE.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            username: None,
            password: None,
        })
        .await
        .expect("durable memory connection");

        context.healthcheck().await.expect("healthcheck");

        let stored = context
            .workflow_executions()
            .store(WorkflowExecutionRecordV1 {
                id: "wf-durable-1".to_string(),
                workflow_name: "knowledge_publication".to_string(),
                trace_ref: "trace-durable-1".to_string(),
            })
            .await
            .expect("store workflow");
        assert_eq!(stored.id, "wf-durable-1");

        let loaded = context
            .workflow_executions()
            .load("wf-durable-1")
            .await
            .expect("load workflow")
            .expect("workflow present");
        assert_eq!(loaded.workflow_name, "knowledge_publication");
    }

    #[allow(clippy::too_many_lines, dead_code)]
    fn sample_analysis() -> MacroFinancialAnalysisV1 {
        MacroFinancialAnalysisV1 {
            analysis_id: "analysis-1".to_string(),
            generated_at: Utc.with_ymd_and_hms(2026, 3, 9, 12, 0, 0).single().unwrap(),
            trace_ref: "trace-1".to_string(),
            objective: AnalysisObjectiveV1::PolicyEval,
            horizon: AnalysisHorizonV1::Nowcast,
            coverage: AnalysisCoverageV1 {
                countries: vec!["Japan".to_string()],
                regions: Vec::new(),
                currencies: vec!["JPY".to_string()],
                fx_pairs: vec!["USD/JPY".to_string()],
                asset_classes: vec!["rates".to_string()],
            },
            problem_contract: ProblemContractV1 {
                objective: AnalysisObjectiveV1::PolicyEval,
                horizon: AnalysisHorizonV1::Nowcast,
                target_countries: vec!["Japan".to_string()],
                target_regions: Vec::new(),
                target_currencies: vec!["JPY".to_string()],
                target_fx_pairs: vec!["USD/JPY".to_string()],
                asset_classes: vec!["rates".to_string()],
                dependent_variables: vec!["FX bilateral".to_string()],
                required_inputs: vec!["FX levels".to_string()],
                missing_inputs: vec!["missing".to_string()],
            },
            data_vintage: "2026-03-09".to_string(),
            required_inputs: vec!["FX levels".to_string()],
            dependent_variables: vec!["FX bilateral".to_string()],
            global_liquidity_phase: GlobalLiquidityPhaseV1::Tighten,
            global_liquidity_funding: GlobalLiquidityFundingConditionsV1 {
                phase: GlobalLiquidityPhaseV1::Tighten,
                dominant_transmission_channel: TransmissionChannelV1::CrossBorderBankCredit,
                dollar_funding_stress_state: "Contained".to_string(),
                backstop_availability: "Adequate reserves.".to_string(),
                missing_inputs: Vec::new(),
            },
            external_accounts_map: ExternalAccountsBalanceSheetMapV1 {
                current_account_pressures: "Surplus persists.".to_string(),
                financial_account_decomposition: "Portfolio flows dominate.".to_string(),
                external_debt_structure: "Mostly local currency debt.".to_string(),
                currency_mismatch_indicators: "Contained mismatch.".to_string(),
                marginal_financer: "Portfolio investors".to_string(),
                flow_reversal_vulnerability: "Portfolio reverses first".to_string(),
                missing_inputs: Vec::new(),
            },
            policy_regime_diagnosis: PolicyRegimeDiagnosisV1 {
                monetary_policy_regime: "Inflation targeting".to_string(),
                credibility_signals: "Credibility remains intact.".to_string(),
                exchange_rate_regime: "Managed float".to_string(),
                intervention_pattern: "Smoothing intervention".to_string(),
                frictions: vec![PolicyFrictionObservationV1 {
                    friction: "FX illiquidity".to_string(),
                    observable_indicators: vec!["volatility".to_string()],
                    confidence: ConfidenceV1::Moderate,
                }],
                missing_inputs: Vec::new(),
            },
            driver_decomposition: vec![FxDriverAssessmentV1 {
                bucket: DriverBucketV1::RateDifferentialsExpectedPolicyPaths,
                direction: DirectionalBiasV1::Positive,
                magnitude: SignalMagnitudeV1::Medium,
                confidence: ConfidenceV1::Moderate,
                evidence: "Policy spread widened.".to_string(),
            }],
            sovereign_systemic_risk: SovereignSystemicRiskV1 {
                debt_sustainability_state: "Stable".to_string(),
                gross_financing_needs: "Manageable".to_string(),
                rollover_risk: "Contained".to_string(),
                sovereign_bank_nonbank_nexus: "Present but stable".to_string(),
                key_amplifiers: vec!["Leverage".to_string()],
                cross_border_spillovers: "Portfolio and swaps.".to_string(),
                missing_inputs: Vec::new(),
            },
            executive_brief: ExecutiveBriefV1 {
                as_of_date: "2026-03-09".to_string(),
                as_of_timezone: "America/Los_Angeles".to_string(),
                data_vintage: "2026-03-09".to_string(),
                objective: AnalysisObjectiveV1::PolicyEval,
                horizon: AnalysisHorizonV1::Nowcast,
                coverage: AnalysisCoverageV1 {
                    countries: vec!["Japan".to_string()],
                    regions: Vec::new(),
                    currencies: vec!["JPY".to_string()],
                    fx_pairs: vec!["USD/JPY".to_string()],
                    asset_classes: vec!["rates".to_string()],
                },
                key_judgments_facts: vec!["FACT".to_string()],
                key_judgments_inferences: vec!["INFERENCE".to_string()],
                key_risks: vec![RankedRiskV1 {
                    risk: "Funding stress".to_string(),
                    summary: "Dollar funding availability tightens.".to_string(),
                    probability: ProbabilityV1::Medium,
                }],
                signal_summary: vec![SignalSummaryEntryV1 {
                    signal: "Rate differentials".to_string(),
                    direction: DirectionalBiasV1::Positive,
                    magnitude: SignalMagnitudeV1::Medium,
                    confidence: ConfidenceV1::Moderate,
                    evidence: "Policy spread widened.".to_string(),
                }],
                implications: AnalysisImplicationsV1 {
                    policy_evaluation: "Maintain monitoring.".to_string(),
                    investment_strategy: "Prefer hedged exposure.".to_string(),
                    risk_management: "Tighten liquidity limits.".to_string(),
                    long_horizon_strategy: "Track reserve adequacy.".to_string(),
                },
            },
            data_register: vec![DataRegisterEntryV1 {
                series_name: "BOP".to_string(),
                country_area: "Japan".to_string(),
                source: "IMF".to_string(),
                frequency: "Monthly".to_string(),
                last_obs: "2026-02".to_string(),
                units: "USD bn".to_string(),
                transform: "yoy".to_string(),
                lag: "T+30d".to_string(),
                quality_flag: String::new(),
                notes: "Primary".to_string(),
            }],
            mechanism_map: MechanismMapV1 {
                current_account_narrative: "Surplus persists.".to_string(),
                financial_account_funding_mix: "Portfolio flows dominate.".to_string(),
                reserves_and_backstops: "Adequate reserves.".to_string(),
                fx_swap_basis_state: "Basis mildly negative.".to_string(),
                dollar_funding_stress_state: "Contained.".to_string(),
                risk_sentiment_linkage: "High beta to global risk.".to_string(),
                spillover_channels: "Portfolio and swaps.".to_string(),
            },
            scenario_matrix: vec![ScenarioCaseV1 {
                scenario: ScenarioKindV1::Base,
                triggers: "Stable policy path".to_string(),
                transmission_path: "Accounts to funding to FX".to_string(),
                fx_outcome: "Range-bound".to_string(),
                capital_flows_outcome: "Steady".to_string(),
                liquidity_funding_outcome: "Stable".to_string(),
                systemic_risk_outcome: "Contained".to_string(),
                policy_response_space: "Moderate".to_string(),
                strategy_implications: "Keep hedges on.".to_string(),
                watchlist: vec![WatchlistIndicatorV1 {
                    indicator: "Basis".to_string(),
                    threshold: "< -20bp".to_string(),
                    rationale: "Funding stress".to_string(),
                }],
            }],
            risk_register: vec![RiskRegisterEntryV1 {
                risk: "Funding stress".to_string(),
                mechanism: "Basis widening".to_string(),
                early_indicators: "Cross-currency basis".to_string(),
                impact_channels: "FX and liquidity".to_string(),
                mitigants_or_hedges: "Shorten tenor".to_string(),
                probability: ProbabilityV1::Medium,
                confidence: ConfidenceV1::Moderate,
            }],
            knowledge_appendix: KnowledgeAppendixV1 {
                definitions: vec!["External accounts".to_string()],
                indicator_dictionary: vec!["Basis".to_string()],
                playbooks: vec!["Sudden stop".to_string()],
                common_failure_modes: vec!["Proxy drift".to_string()],
                source_note: "Primary sources only".to_string(),
                assumptions_log: vec!["1. Stable policy backdrop.".to_string()],
            },
            source_governance: vec![],
            assumptions: vec![AnalysisAssumptionV1 {
                assumption_id: "A1".to_string(),
                text: "Primary sources dominate evidence.".to_string(),
                stable: true,
            }],
            inference_steps: vec![InferenceStepV1 {
                inference_id: "INF-01".to_string(),
                label: "Funding inference".to_string(),
                assumption_ids: vec!["A1".to_string()],
                inputs_used: vec!["BOP".to_string()],
                resulting_judgment: "Funding remains stable".to_string(),
            }],
            claim_evidence: vec![ClaimEvidenceV1 {
                claim_id: "claim-1".to_string(),
                output_section: "Executive Brief".to_string(),
                claim_kind: ClaimKindV1::Fact,
                statement: "FACT".to_string(),
                source_ids: vec!["source-1".to_string()],
                inference_ids: Vec::new(),
            }],
            pipeline_trace: vec![PipelineStepTraceV1 {
                step: PipelineStepIdV1::StepA,
                ordinal: 1,
                summary: "Problem contract complete".to_string(),
            }],
            source_ids: vec!["source-1".to_string()],
            capsule_id: Some("capsule-1".to_string()),
            rendered_output: "analysis".to_string(),
            retrieval_context: vec!["Surplus persists.".to_string()],
        }
    }
}
