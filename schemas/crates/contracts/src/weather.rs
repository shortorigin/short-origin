use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherSourceKindV1 {
    Hrrr,
    Gfs,
    Goes,
    Nexrad,
    Surface,
    NwsAlert,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherArtifactKindV1 {
    ForecastGrid,
    SatelliteScene,
    RadarVolume,
    SurfaceObservation,
    AlertFeed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherLayerKindV1 {
    Temperature,
    Precipitation,
    Wind,
    RadarReflectivity,
    CloudCover,
    SurfaceObservations,
    AlertOverlay,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherFeatureKindV1 {
    Temperature2m,
    WindSpeed10m,
    PrecipitationRate,
    Visibility,
    ConvectiveRisk,
    AlertSeverityScore,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherQcFlagV1 {
    Passed,
    Estimated,
    Missing,
    OutOfRange,
    UpstreamFlagged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeoBoundsV1 {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeoPointV1 {
    pub longitude: f64,
    pub latitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherProvenanceV1 {
    pub source_kind: WeatherSourceKindV1,
    pub source_dataset: String,
    pub source_object_ref: String,
    pub retrieved_at: DateTime<Utc>,
    pub raw_artifact_hash: String,
    pub transform_version: String,
    pub config_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherLayerAvailabilityV1 {
    pub layer: WeatherLayerKindV1,
    pub latest_event_time: DateTime<Utc>,
    pub latest_valid_time: DateTime<Utc>,
    pub horizon_hours: u16,
    pub product_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherAvailabilityV1 {
    pub catalog_id: String,
    pub region_id: String,
    pub bounds: GeoBoundsV1,
    pub generated_at: DateTime<Utc>,
    pub source_kinds: Vec<WeatherSourceKindV1>,
    pub available_layers: Vec<WeatherLayerAvailabilityV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherViewLayerV1 {
    pub layer: WeatherLayerKindV1,
    pub title: String,
    pub asset_ref: String,
    pub native_identifier: String,
    pub event_time: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
    pub qc_flags: Vec<WeatherQcFlagV1>,
    pub provenance: Vec<WeatherProvenanceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherViewV1 {
    pub view_id: String,
    pub region_id: String,
    pub bounds: GeoBoundsV1,
    pub generated_at: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
    pub horizon_hours: u16,
    pub layers: Vec<WeatherViewLayerV1>,
    pub alert_feed_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherFeatureValueV1 {
    pub feature: WeatherFeatureKindV1,
    pub value: f64,
    pub units: String,
    pub probability: Option<f64>,
    pub qc_flags: Vec<WeatherQcFlagV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherFeatureSliceV1 {
    pub slice_id: String,
    pub region_id: String,
    pub event_time: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
    pub lead_hours: u16,
    pub source_product_refs: Vec<String>,
    pub features: Vec<WeatherFeatureValueV1>,
    pub provenance: Vec<WeatherProvenanceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherAlertV1 {
    pub alert_id: String,
    pub region_id: String,
    pub headline: String,
    pub severity: String,
    pub effective_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub source: String,
    pub bounds: GeoBoundsV1,
    pub provenance: Vec<WeatherProvenanceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherAlertFeedV1 {
    pub feed_id: String,
    pub region_id: String,
    pub generated_at: DateTime<Utc>,
    pub alerts: Vec<WeatherAlertV1>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherMapSourceEncodingV1 {
    RasterTile,
    VectorTile,
    GeoJson,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum WeatherMapLayerRenderModeV1 {
    Raster,
    Fill,
    Line,
    Circle,
    Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherMapLegendStopV1 {
    pub label: String,
    pub color: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherMapInteractionV1 {
    pub popup_title: String,
    pub property_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherMapSourceV1 {
    pub source_id: String,
    pub layer: WeatherLayerKindV1,
    pub title: String,
    pub encoding: WeatherMapSourceEncodingV1,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub attribution: String,
    pub promote_id: Option<String>,
    pub cluster: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherMapFrameSourceBindingV1 {
    pub source_id: String,
    pub revision: String,
    pub tilejson_url: Option<String>,
    pub data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherMapFrameV1 {
    pub frame_id: String,
    pub label: String,
    pub event_time: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
    pub horizon_hours: u16,
    pub source_bindings: Vec<WeatherMapFrameSourceBindingV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherMapLayerV1 {
    pub layer_id: String,
    pub source_id: String,
    pub layer: WeatherLayerKindV1,
    pub title: String,
    pub render_mode: WeatherMapLayerRenderModeV1,
    pub source_layer: Option<String>,
    pub visible_by_default: bool,
    pub legend: Vec<WeatherMapLegendStopV1>,
    pub interaction: Option<WeatherMapInteractionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherMapSceneV1 {
    pub scene_id: String,
    pub region_id: String,
    pub scene_revision: String,
    pub bounds: GeoBoundsV1,
    pub default_center: GeoPointV1,
    pub default_zoom: f64,
    pub generated_at: DateTime<Utc>,
    pub active_frame_id: String,
    pub refresh_interval_seconds: u16,
    pub frames: Vec<WeatherMapFrameV1>,
    pub sources: Vec<WeatherMapSourceV1>,
    pub layers: Vec<WeatherMapLayerV1>,
}
