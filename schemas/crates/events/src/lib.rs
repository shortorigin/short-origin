use chrono::{DateTime, Utc};
use contracts::{
    Classification, ExperimentResultV1, FillV1, LimitBreachRecordV1, PayloadRefV1,
    PortfolioSnapshotV1, PromotionGateV1, PromotionRecommendationV1, SignalV1,
};
use identity::ActorRef;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod knowledge;

pub use knowledge::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelopeV1 {
    pub event_id: String,
    pub event_type: String,
    pub event_version: String,
    pub actor_ref: ActorRef,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub decision_ref: Option<String>,
    pub classification: Classification,
    pub schema_ref: String,
    pub integrity_hash: String,
}

impl EventEnvelopeV1 {
    #[must_use]
    pub fn new(
        event_type: impl Into<String>,
        actor_ref: ActorRef,
        correlation_id: impl Into<String>,
        decision_ref: Option<String>,
        classification: Classification,
        schema_ref: impl Into<String>,
        integrity_hash: impl Into<String>,
    ) -> Self {
        let timestamp = Utc::now();
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            event_version: "v1".to_owned(),
            actor_ref,
            occurred_at: timestamp,
            recorded_at: timestamp,
            correlation_id: correlation_id.into(),
            causation_id: None,
            decision_ref,
            classification,
            schema_ref: schema_ref.into(),
            integrity_hash: integrity_hash.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordedEventV1 {
    pub envelope: EventEnvelopeV1,
    pub payload_ref: PayloadRefV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketDataNormalizedV1 {
    pub dataset_id: String,
    pub dataset_name: String,
    pub event_count: usize,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignalGeneratedV1 {
    pub strategy_id: String,
    pub signal: SignalV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderSubmittedV1 {
    pub order: contracts::OrderRequestV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FillRecordedV1 {
    pub fill: FillV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskLimitBreachedV1 {
    pub breach: LimitBreachRecordV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortfolioSnapshottedV1 {
    pub snapshot: PortfolioSnapshotV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentRankedV1 {
    pub rank: usize,
    pub result: ExperimentResultV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionGateEvaluatedV1 {
    pub gate: PromotionGateV1,
    pub recommendation: PromotionRecommendationV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", content = "payload", rename_all = "snake_case")]
pub enum CapitalMarketsEventPayloadV1 {
    MarketDataNormalized(MarketDataNormalizedV1),
    SignalGenerated(SignalGeneratedV1),
    OrderSubmitted(OrderSubmittedV1),
    FillRecorded(FillRecordedV1),
    RiskLimitBreached(RiskLimitBreachedV1),
    PortfolioSnapshotted(PortfolioSnapshottedV1),
    ExperimentRanked(ExperimentRankedV1),
    PromotionGateEvaluated(PromotionGateEvaluatedV1),
}
