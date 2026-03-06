use std::sync::Arc;

use chrono::{DateTime, Utc};
use contracts::{EvidenceManifestV1, ServiceBoundaryV1};
use error_model::{InstitutionalError, InstitutionalResult};
use evidence_sdk::{EvidenceSink, MemoryEvidenceSink};
use serde::{Deserialize, Serialize};
use trading_core::{hash_payload, Clock, IdGenerator, SystemClock, SystemIdGenerator};
use trading_errors::TradingError;

fn map_trading_error(error: TradingError) -> InstitutionalError {
    InstitutionalError::InvariantViolation {
        invariant: error.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub event_id: String,
    pub recorded_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub current_hash: String,
}

#[derive(Clone)]
pub struct EvidenceService {
    sink: MemoryEvidenceSink,
    audit_events: Vec<AuditEvent>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl std::fmt::Debug for EvidenceService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EvidenceService")
            .field("manifests", &self.sink.recorded().len())
            .field("audit_events", &self.audit_events.len())
            .finish_non_exhaustive()
    }
}

impl Default for EvidenceService {
    fn default() -> Self {
        Self {
            sink: MemoryEvidenceSink::default(),
            audit_events: Vec::new(),
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
            audit_events: Vec::new(),
            clock,
            ids,
        }
    }

    pub fn append_audit_event(
        &mut self,
        payload: serde_json::Value,
    ) -> InstitutionalResult<AuditEvent> {
        let previous_hash = self
            .audit_events
            .last()
            .map_or_else(|| "GENESIS".to_string(), |event| event.current_hash.clone());
        let current_hash =
            hash_payload(&(payload.clone(), &previous_hash)).map_err(map_trading_error)?;
        let event = AuditEvent {
            event_id: self.ids.next_id(),
            recorded_at: self.clock.now(),
            payload,
            previous_hash,
            current_hash,
        };
        self.audit_events.push(event.clone());
        Ok(event)
    }

    #[must_use]
    pub fn audit_events(&self) -> &[AuditEvent] {
        &self.audit_events
    }
}

impl EvidenceSink for EvidenceService {
    fn record(&mut self, manifest: EvidenceManifestV1) -> InstitutionalResult<()> {
        self.sink.record(manifest)
    }

    fn recorded(&self) -> Vec<EvidenceManifestV1> {
        self.sink.recorded()
    }
}

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "evidence-service".to_owned(),
        domain: "audit_assurance".to_owned(),
        approved_workflows: vec![
            "control_testing".to_owned(),
            "treasury_disbursement".to_owned(),
            "policy_exception".to_owned(),
            "quant_strategy_promotion".to_owned(),
        ],
        owned_aggregates: vec!["evidence_manifest".to_owned(), "audit_event".to_owned()],
    }
}
