use contracts::{EvidenceManifestV1, TreasuryDisbursementRecordedV1};
use error_model::{InstitutionalError, InstitutionalResult};
use events::RecordedEventV1;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{Connection, Surreal};
use surrealdb_model::{
    EventRecordV1, EvidenceManifestRecordV1, TreasuryDisbursementRecordV1,
    WorkflowExecutionRecordV1,
};

pub const DEFAULT_NAMESPACE: &str = "short_origin";
pub const DEFAULT_DATABASE: &str = "institutional";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableCatalog {
    pub workflow_execution: &'static str,
    pub evidence_manifest: &'static str,
    pub recorded_event: &'static str,
    pub treasury_disbursement: &'static str,
}

pub const TABLES: TableCatalog = TableCatalog {
    workflow_execution: "workflow_execution",
    evidence_manifest: "evidence_manifest",
    recorded_event: "recorded_event",
    treasury_disbursement: "treasury_disbursement",
};

pub struct SurrealRepositoryContext<C>
where
    C: Connection,
{
    db: Surreal<C>,
}

impl<C> Clone for SurrealRepositoryContext<C>
where
    C: Connection,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<C> SurrealRepositoryContext<C>
where
    C: Connection,
{
    #[must_use]
    pub fn new(db: Surreal<C>) -> Self {
        Self { db }
    }

    pub async fn use_namespace(&self, namespace: &str, database: &str) -> InstitutionalResult<()> {
        self.db
            .use_ns(namespace)
            .use_db(database)
            .await
            .map_err(surreal_error)?;
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
}

pub async fn connect_in_memory() -> InstitutionalResult<SurrealRepositoryContext<Db>> {
    let db = Surreal::new::<Mem>(()).await.map_err(surreal_error)?;
    let context = SurrealRepositoryContext::new(db);
    context
        .use_namespace(DEFAULT_NAMESPACE, DEFAULT_DATABASE)
        .await?;
    Ok(context)
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

async fn create_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: String,
    record: T,
) -> InstitutionalResult<T>
where
    C: Connection,
    T: Clone + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    let content = serde_json::to_value(record.clone()).map_err(|error| {
        InstitutionalError::external("surrealdb", Some("create".to_string()), error.to_string())
    })?;
    let content = match content {
        serde_json::Value::Object(mut map) => {
            map.remove("id");
            serde_json::Value::Object(map)
        }
        other => other,
    };
    db.query("UPSERT type::thing($table, $id) CONTENT $content;")
        .bind(("table", table.to_string()))
        .bind(("id", id))
        .bind(("content", content))
        .await
        .map_err(surreal_error)?;
    Ok(record)
}

async fn select_record<C, T>(
    db: &Surreal<C>,
    table: &str,
    id: &str,
) -> InstitutionalResult<Option<T>>
where
    C: Connection,
    T: serde::de::DeserializeOwned,
{
    let mut response = db
        .query("SELECT *, type::string(id) AS id FROM ONLY type::thing($table, $id);")
        .bind(("table", table.to_string()))
        .bind(("id", id.to_string()))
        .await
        .map_err(surreal_error)?;
    response.take(0).map_err(surreal_error)
}

fn surreal_error(error: surrealdb::Error) -> InstitutionalError {
    InstitutionalError::external("surrealdb", None, error.to_string())
}

#[cfg(test)]
mod tests {
    use contracts::{Classification, EvidenceManifestV1, TreasuryDisbursementRecordedV1};
    use events::EventEnvelopeV1;
    use identity::ActorRef;

    use super::*;

    #[tokio::test]
    async fn repositories_round_trip_core_records() {
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
                    evidence_id: "evidence-1".to_string(),
                    producer: "tests".to_string(),
                    artifact_hash: "abc".to_string(),
                    storage_ref: "surrealdb:evidence/evidence-1".to_string(),
                    retention_class: "standard".to_string(),
                    classification: Classification::Internal,
                    related_decision_refs: vec!["decision-1".to_string()],
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
                    payload_ref: contracts::PayloadRefV1 {
                        schema_ref: "schemas/contracts/v1/workflow-execution".to_string(),
                        record_id: "wf-1".to_string(),
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

        let loaded = context
            .workflow_executions()
            .load("wf-1")
            .await
            .expect("load workflow")
            .expect("workflow present");
        assert_eq!(loaded.trace_ref, "trace-1");
    }
}
