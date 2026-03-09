use error_model::{InstitutionalError, InstitutionalResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedSchema {
    pub name: String,
    pub document: Value,
}

const CONTRACT_FILES: [(&str, &str); 35] = [
    (
        "command-v1",
        include_str!("../../../contracts/v1/command-v1.json"),
    ),
    (
        "query-v1",
        include_str!("../../../contracts/v1/query-v1.json"),
    ),
    (
        "policy-decision-v1",
        include_str!("../../../contracts/v1/policy-decision-v1.json"),
    ),
    (
        "approval-v1",
        include_str!("../../../contracts/v1/approval-v1.json"),
    ),
    (
        "exception-v1",
        include_str!("../../../contracts/v1/exception-v1.json"),
    ),
    (
        "risk-v1",
        include_str!("../../../contracts/v1/risk-v1.json"),
    ),
    (
        "evidence-manifest-v1",
        include_str!("../../../contracts/v1/evidence-manifest-v1.json"),
    ),
    (
        "agent-handoff-v1",
        include_str!("../../../contracts/v1/agent-handoff-v1.json"),
    ),
    (
        "work-item-v1",
        include_str!("../../../contracts/v1/work-item-v1.json"),
    ),
    (
        "research-synthesis-v1",
        include_str!("../../../contracts/v1/research-synthesis-v1.json"),
    ),
    (
        "requirements-spec-v1",
        include_str!("../../../contracts/v1/requirements-spec-v1.json"),
    ),
    (
        "architecture-design-v1",
        include_str!("../../../contracts/v1/architecture-design-v1.json"),
    ),
    (
        "implementation-plan-v1",
        include_str!("../../../contracts/v1/implementation-plan-v1.json"),
    ),
    (
        "change-batch-v1",
        include_str!("../../../contracts/v1/change-batch-v1.json"),
    ),
    (
        "validation-report-v1",
        include_str!("../../../contracts/v1/validation-report-v1.json"),
    ),
    (
        "refinement-record-v1",
        include_str!("../../../contracts/v1/refinement-record-v1.json"),
    ),
    (
        "treasury-disbursement-v1",
        include_str!("../../../contracts/v1/treasury-disbursement-v1.json"),
    ),
    (
        "market-data-v1",
        include_str!("../../../contracts/v1/market-data-v1.json"),
    ),
    (
        "order-lifecycle-v1",
        include_str!("../../../contracts/v1/order-lifecycle-v1.json"),
    ),
    (
        "portfolio-snapshot-v1",
        include_str!("../../../contracts/v1/portfolio-snapshot-v1.json"),
    ),
    (
        "research-backtest-v1",
        include_str!("../../../contracts/v1/research-backtest-v1.json"),
    ),
    (
        "trading-risk-v1",
        include_str!("../../../contracts/v1/trading-risk-v1.json"),
    ),
    (
        "compliance-pack-v1",
        include_str!("../../../contracts/v1/compliance-pack-v1.json"),
    ),
    (
        "model-approval-v1",
        include_str!("../../../contracts/v1/model-approval-v1.json"),
    ),
    (
        "promotion-gate-v1",
        include_str!("../../../contracts/v1/promotion-gate-v1.json"),
    ),
    (
        "knowledge-source-ingest-v1",
        include_str!("../../../contracts/v1/knowledge-source-ingest-v1.json"),
    ),
    (
        "knowledge-publication-v1",
        include_str!("../../../contracts/v1/knowledge-publication-v1.json"),
    ),
    (
        "macro-financial-analysis-request-v1",
        include_str!("../../../contracts/v1/macro-financial-analysis-request-v1.json"),
    ),
    (
        "executive-brief-v1",
        include_str!("../../../contracts/v1/executive-brief-v1.json"),
    ),
    (
        "data-register-entry-v1",
        include_str!("../../../contracts/v1/data-register-entry-v1.json"),
    ),
    (
        "mechanism-map-v1",
        include_str!("../../../contracts/v1/mechanism-map-v1.json"),
    ),
    (
        "scenario-case-v1",
        include_str!("../../../contracts/v1/scenario-case-v1.json"),
    ),
    (
        "risk-register-entry-v1",
        include_str!("../../../contracts/v1/risk-register-entry-v1.json"),
    ),
    (
        "knowledge-appendix-v1",
        include_str!("../../../contracts/v1/knowledge-appendix-v1.json"),
    ),
    (
        "macro-financial-analysis-v1",
        include_str!("../../../contracts/v1/macro-financial-analysis-v1.json"),
    ),
];

const EVENT_FILES: [(&str, &str); 12] = [
    (
        "event-envelope-v1",
        include_str!("../../../events/envelope/v1/event-envelope-v1.json"),
    ),
    (
        "market-data-normalized-v1",
        include_str!("../../../events/capital_markets/v1/market-data-normalized-v1.json"),
    ),
    (
        "signal-generated-v1",
        include_str!("../../../events/capital_markets/v1/signal-generated-v1.json"),
    ),
    (
        "order-submitted-v1",
        include_str!("../../../events/capital_markets/v1/order-submitted-v1.json"),
    ),
    (
        "fill-recorded-v1",
        include_str!("../../../events/capital_markets/v1/fill-recorded-v1.json"),
    ),
    (
        "risk-limit-breached-v1",
        include_str!("../../../events/capital_markets/v1/risk-limit-breached-v1.json"),
    ),
    (
        "portfolio-snapshotted-v1",
        include_str!("../../../events/capital_markets/v1/portfolio-snapshotted-v1.json"),
    ),
    (
        "experiment-ranked-v1",
        include_str!("../../../events/capital_markets/v1/experiment-ranked-v1.json"),
    ),
    (
        "promotion-gate-evaluated-v1",
        include_str!("../../../events/capital_markets/v1/promotion-gate-evaluated-v1.json"),
    ),
    (
        "knowledge-source-ingested-v1",
        include_str!("../../../events/data_knowledge/v1/knowledge-source-ingested-v1.json"),
    ),
    (
        "knowledge-capsule-published-v1",
        include_str!("../../../events/data_knowledge/v1/knowledge-capsule-published-v1.json"),
    ),
    (
        "knowledge-analysis-generated-v1",
        include_str!("../../../events/data_knowledge/v1/knowledge-analysis-generated-v1.json"),
    ),
];

const SURREALDB_FILES: [(&str, &str); 1] = [(
    "record-types-v1",
    include_str!("../../../surrealdb/v1/record-types-v1.json"),
)];

fn parse(name: &str, document: &str) -> InstitutionalResult<EmbeddedSchema> {
    let parsed = serde_json::from_str(document)
        .map_err(|error| InstitutionalError::parse(name, error.to_string()))?;
    Ok(EmbeddedSchema {
        name: name.to_owned(),
        document: parsed,
    })
}

pub fn embedded_contract_schemas() -> InstitutionalResult<Vec<EmbeddedSchema>> {
    CONTRACT_FILES
        .iter()
        .map(|(name, document)| parse(name, document))
        .collect()
}

pub fn embedded_event_schemas() -> InstitutionalResult<Vec<EmbeddedSchema>> {
    EVENT_FILES
        .iter()
        .map(|(name, document)| parse(name, document))
        .collect()
}

pub fn embedded_surrealdb_schemas() -> InstitutionalResult<Vec<EmbeddedSchema>> {
    SURREALDB_FILES
        .iter()
        .map(|(name, document)| parse(name, document))
        .collect()
}
