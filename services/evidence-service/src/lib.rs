use std::future::Future;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use contracts::{EvidenceManifestV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext, SourceErrorInfo};
use evidence_sdk::{EvidenceSink, MemoryEvidenceSink};
use serde::{Deserialize, Serialize};
use trading_core::{Clock, IdGenerator, SystemClock, SystemIdGenerator, hash_payload};
use trading_errors::TradingError;

const SERVICE_NAME: &str = "evidence-service";
const DOMAIN_NAME: &str = "audit_assurance";
const APPROVED_WORKFLOWS: &[&str] = &[
    "control_testing",
    "treasury_disbursement",
    "policy_exception",
    "quant_strategy_promotion",
];
const OWNED_AGGREGATES: &[&str] = &["evidence_manifest", "audit_event"];

fn map_trading_error(error: TradingError) -> InstitutionalError {
    InstitutionalError::persistence(
        OperationContext::new("services/evidence-service", "append_audit_event"),
        "failed to hash audit payload",
        SourceErrorInfo::new(
            "trading-core",
            Some("hash_payload".to_string()),
            error.to_string(),
        ),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub event_id: String,
    pub recorded_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub current_hash: String,
}

#[derive(Debug, Default, Clone)]
struct InMemoryAuditEventLog {
    events: Vec<AuditEvent>,
}

impl InMemoryAuditEventLog {
    fn last_hash(&self) -> Option<&str> {
        self.events.last().map(|event| event.current_hash.as_str())
    }

    fn append(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    fn events(&self) -> &[AuditEvent] {
        &self.events
    }
}

#[derive(Clone)]
pub struct EvidenceService {
    sink: MemoryEvidenceSink,
    audit_log: InMemoryAuditEventLog,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for EvidenceService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EvidenceService")
            .field("manifests", &self.sink.len().unwrap_or_default())
            .field("audit_events", &self.audit_log.events().len())
            .finish_non_exhaustive()
    }
}

impl Default for EvidenceService {
    fn default() -> Self {
        Self {
            sink: MemoryEvidenceSink::default(),
            audit_log: InMemoryAuditEventLog::default(),
            clock: Arc::new(SystemClock),
            ids: Arc::new(SystemIdGenerator),
        }
    }
}

impl EvidenceService {
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self {
            sink: MemoryEvidenceSink::default(),
            audit_log: InMemoryAuditEventLog::default(),
            clock,
            ids,
        }
    }

    pub fn append_audit_event(
        &mut self,
        payload: serde_json::Value,
    ) -> InstitutionalResult<AuditEvent> {
        let previous_hash = self
            .audit_log
            .last_hash()
            .map_or_else(|| "GENESIS".to_string(), str::to_owned);
        let current_hash =
            hash_payload(&(payload.clone(), &previous_hash)).map_err(map_trading_error)?;
        let event = AuditEvent {
            event_id: self.ids.next_id(),
            recorded_at: self.clock.now(),
            payload,
            previous_hash,
            current_hash,
        };
        self.audit_log.append(event.clone());
        Ok(event)
    }

    #[must_use]
    pub fn audit_events(&self) -> &[AuditEvent] {
        self.audit_log.events()
    }
}

impl EvidenceSink for EvidenceService {
    fn record(
        &self,
        manifest: EvidenceManifestV1,
    ) -> impl Future<Output = InstitutionalResult<()>> + Send + '_ {
        self.sink.record(manifest)
    }

    fn recorded(
        &self,
    ) -> impl Future<Output = InstitutionalResult<Vec<EvidenceManifestV1>>> + Send + '_ {
        self.sink.recorded()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.into(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES.iter().copied().map(Into::into).collect(),
    }
}
