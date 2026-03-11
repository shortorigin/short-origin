use chrono::{DateTime, Utc};
use contracts::{WeatherAlertFeedV1, WeatherArtifactKindV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherProductPublishedV1 {
    pub product_ref: String,
    pub region_id: String,
    pub artifact_kind: WeatherArtifactKindV1,
    pub native_identifier: String,
    pub event_time: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherAlertUpdatedV1 {
    pub feed: WeatherAlertFeedV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherBackfillCompletedV1 {
    pub run_id: String,
    pub region_ids: Vec<String>,
    pub product_refs: Vec<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", content = "payload", rename_all = "snake_case")]
pub enum MeteorologicalEventPayloadV1 {
    WeatherProductPublished(WeatherProductPublishedV1),
    WeatherAlertUpdated(WeatherAlertUpdatedV1),
    WeatherBackfillCompleted(WeatherBackfillCompletedV1),
}
