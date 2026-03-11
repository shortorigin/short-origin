use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use contracts::{
    GeoBoundsV1, GeoPointV1, ServiceBoundaryV1, WeatherAlertFeedV1, WeatherArtifactKindV1,
    WeatherAvailabilityV1, WeatherFeatureKindV1, WeatherFeatureSliceV1, WeatherLayerKindV1,
    WeatherMapFrameSourceBindingV1, WeatherMapFrameV1, WeatherMapInteractionV1,
    WeatherMapLayerRenderModeV1, WeatherMapLayerV1, WeatherMapLegendStopV1, WeatherMapSceneV1,
    WeatherMapSourceEncodingV1, WeatherMapSourceV1, WeatherProvenanceV1, WeatherQcFlagV1,
};
use error_model::{InstitutionalError, InstitutionalResult, OperationContext};
use events::{WeatherAlertUpdatedV1, WeatherProductPublishedV1};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const SERVICE_NAME: &str = "meteorological-service";
const DOMAIN_NAME: &str = "meteorological_intelligence";
const APPROVED_WORKFLOWS: &[&str] = &["weather_ingestion"];
const OWNED_AGGREGATES: &[&str] = &[
    "weather_dataset",
    "weather_feature_product",
    "weather_alert",
    "weather_map_scene",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawSourceAsset {
    pub asset_id: String,
    pub source_kind: contracts::WeatherSourceKindV1,
    pub source_ref: String,
    pub bytes_sha256: String,
    pub retrieved_at: DateTime<Utc>,
    pub upstream_qc_notes: Vec<String>,
    pub provenance: WeatherProvenanceV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedWeatherProduct {
    pub product_id: String,
    pub region_id: String,
    pub artifact_kind: WeatherArtifactKindV1,
    pub native_identifier: String,
    pub event_time: DateTime<Utc>,
    pub valid_time: DateTime<Utc>,
    pub lead_hours: u16,
    pub bounds: GeoBoundsV1,
    pub location: Option<GeoPointV1>,
    pub raw_asset_ids: Vec<String>,
    pub qc_flags: Vec<WeatherQcFlagV1>,
    pub provenance: Vec<WeatherProvenanceV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherFixtureBatchV1 {
    pub batch_id: String,
    pub region_id: String,
    pub raw_assets: Vec<RawSourceAsset>,
    pub normalized_products: Vec<NormalizedWeatherProduct>,
    pub availability: WeatherAvailabilityV1,
    pub view: contracts::WeatherViewV1,
    pub feature_slices: Vec<WeatherFeatureSliceV1>,
    pub alerts: WeatherAlertFeedV1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherIngestionReport {
    pub batch_id: String,
    pub raw_asset_count: usize,
    pub normalized_product_count: usize,
    pub feature_slice_count: usize,
    pub alert_count: usize,
    pub latest_availability: WeatherAvailabilityV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WeatherTileJsonDocumentV1 {
    pub tilejson: String,
    pub name: String,
    pub scheme: String,
    pub tiles: Vec<String>,
    pub minzoom: u8,
    pub maxzoom: u8,
    pub bounds: [f64; 4],
    pub attribution: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherMapTilePayloadFormatV1 {
    Webp,
    Mvt,
}

impl WeatherMapTilePayloadFormatV1 {
    fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Mvt => "mvt",
        }
    }

    fn content_type(self) -> &'static str {
        match self {
            Self::Webp => "image/webp",
            Self::Mvt => "application/vnd.mapbox-vector-tile",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherMapSourceRequestV1 {
    pub scene_id: String,
    pub source_id: String,
    pub frame_id: String,
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherMapTileRequestV1 {
    pub source: WeatherMapSourceRequestV1,
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub format: WeatherMapTilePayloadFormatV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherMapHttpRouteV1 {
    pub method: &'static str,
    pub path_template: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherMapHttpResponseV1 {
    pub content_type: String,
    pub cache_control: String,
    pub body: Vec<u8>,
}

pub struct WeatherMapHttpAdapter<'a> {
    service: &'a MeteorologicalService,
}

impl<'a> WeatherMapHttpAdapter<'a> {
    const ROUTES: [WeatherMapHttpRouteV1; 4] = [
        WeatherMapHttpRouteV1 {
            method: "GET",
            path_template: "/api/weather/maps/scenes/{region_id}",
            description: "Resolve the latest weather map scene manifest for a governed region.",
        },
        WeatherMapHttpRouteV1 {
            method: "GET",
            path_template: "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/tilejson.json",
            description: "Resolve a revisioned TileJSON document for a weather source binding.",
        },
        WeatherMapHttpRouteV1 {
            method: "GET",
            path_template: "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/tiles/{z}/{x}/{y}.{mvt|webp}",
            description: "Resolve a revisioned vector or raster weather tile payload.",
        },
        WeatherMapHttpRouteV1 {
            method: "GET",
            path_template: "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/features.geojson",
            description: "Resolve a revisioned GeoJSON overlay for a weather source binding.",
        },
    ];

    #[must_use]
    pub fn new(service: &'a MeteorologicalService) -> Self {
        Self { service }
    }

    #[must_use]
    pub fn routes() -> &'static [WeatherMapHttpRouteV1; 4] {
        &Self::ROUTES
    }

    pub fn get_scene(&self, region_id: &str) -> InstitutionalResult<WeatherMapHttpResponseV1> {
        let scene = self
            .service
            .weather_map_scene(region_id)
            .ok_or_else(|| weather_not_found("get_scene", "weather map scene not found"))?;
        json_response(scene, "application/json", scene_manifest_cache_control())
    }

    pub fn get_tilejson(
        &self,
        request: &WeatherMapSourceRequestV1,
    ) -> InstitutionalResult<WeatherMapHttpResponseV1> {
        self.validate_binding(request)?;
        let tilejson = self
            .service
            .weather_map_tilejson(&request.scene_id, &request.source_id, &request.frame_id)
            .ok_or_else(|| weather_not_found("get_tilejson", "weather tilejson not found"))?;
        json_response(
            tilejson,
            "application/json",
            immutable_asset_cache_control(),
        )
    }

    pub fn get_geojson(
        &self,
        request: &WeatherMapSourceRequestV1,
    ) -> InstitutionalResult<WeatherMapHttpResponseV1> {
        self.validate_binding(request)?;
        let geojson = self
            .service
            .weather_map_geojson(&request.scene_id, &request.source_id, &request.frame_id)
            .ok_or_else(|| weather_not_found("get_geojson", "weather geojson not found"))?;
        json_response(
            geojson,
            "application/geo+json",
            immutable_asset_cache_control(),
        )
    }

    pub fn get_tile(
        &self,
        request: &WeatherMapTileRequestV1,
    ) -> InstitutionalResult<WeatherMapHttpResponseV1> {
        self.validate_binding(&request.source)?;
        let payload = json!({
            "scene_id": request.source.scene_id,
            "source_id": request.source.source_id,
            "frame_id": request.source.frame_id,
            "revision": request.source.revision,
            "z": request.z,
            "x": request.x,
            "y": request.y,
            "format": request.format.extension(),
            "note": "Fixture delivery advertises revisioned tile endpoints; binary tile materialization is provided by deployment adapters.",
        });
        let body = serde_json::to_vec(&payload).map_err(|err| {
            InstitutionalError::invariant(
                OperationContext::new("services/meteorological-service", "get_tile"),
                format!("failed to serialize tile placeholder payload: {err}"),
            )
        })?;

        Ok(WeatherMapHttpResponseV1 {
            content_type: request.format.content_type().to_string(),
            cache_control: immutable_asset_cache_control().to_string(),
            body,
        })
    }

    fn validate_binding(&self, request: &WeatherMapSourceRequestV1) -> InstitutionalResult<()> {
        let scene = self
            .service
            .scene_by_id(&request.scene_id)
            .ok_or_else(|| weather_not_found("validate_binding", "weather map scene not found"))?;
        let frame = scene
            .frames
            .iter()
            .find(|frame| frame.frame_id == request.frame_id)
            .ok_or_else(|| weather_not_found("validate_binding", "weather map frame not found"))?;
        let _source = scene
            .sources
            .iter()
            .find(|source| source.source_id == request.source_id)
            .ok_or_else(|| weather_not_found("validate_binding", "weather map source not found"))?;
        let binding = frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == request.source_id)
            .ok_or_else(|| {
                weather_not_found("validate_binding", "weather map source binding not found")
            })?;

        if binding.revision != request.revision {
            return Err(weather_not_found(
                "validate_binding",
                "weather map revision does not match the active source binding",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct MeteorologicalCatalog {
    raw_assets: Vec<RawSourceAsset>,
    normalized_products: Vec<NormalizedWeatherProduct>,
    availability_by_region: BTreeMap<String, WeatherAvailabilityV1>,
    view_by_region: BTreeMap<String, contracts::WeatherViewV1>,
    feature_slices_by_region: BTreeMap<String, Vec<WeatherFeatureSliceV1>>,
    alerts_by_region: BTreeMap<String, WeatherAlertFeedV1>,
    map_scene_by_region: BTreeMap<String, WeatherMapSceneV1>,
    tilejson_by_frame: BTreeMap<(String, String, String), WeatherTileJsonDocumentV1>,
    geojson_by_frame: BTreeMap<(String, String, String), Value>,
    published_products: Vec<WeatherProductPublishedV1>,
    updated_alerts: Vec<WeatherAlertUpdatedV1>,
}

#[derive(Debug, Default, Clone)]
pub struct MeteorologicalService {
    catalog: MeteorologicalCatalog,
}

impl MeteorologicalService {
    pub fn ingest_fixture_batch(
        &mut self,
        batch: WeatherFixtureBatchV1,
    ) -> InstitutionalResult<WeatherIngestionReport> {
        if batch.raw_assets.is_empty() {
            return Err(InstitutionalError::invariant(
                OperationContext::new("services/meteorological-service", "ingest_fixture_batch"),
                "weather ingest requires at least one raw source asset",
            ));
        }
        if batch.normalized_products.is_empty() {
            return Err(InstitutionalError::invariant(
                OperationContext::new("services/meteorological-service", "ingest_fixture_batch"),
                "weather ingest requires at least one normalized weather product",
            ));
        }

        let map_bundle = build_map_scene_bundle(&batch);

        self.catalog
            .raw_assets
            .extend(batch.raw_assets.iter().cloned());
        self.catalog
            .normalized_products
            .extend(batch.normalized_products.iter().cloned());
        self.catalog
            .availability_by_region
            .insert(batch.region_id.clone(), batch.availability.clone());
        self.catalog
            .view_by_region
            .insert(batch.region_id.clone(), batch.view.clone());
        self.catalog
            .feature_slices_by_region
            .insert(batch.region_id.clone(), batch.feature_slices.clone());
        self.catalog
            .alerts_by_region
            .insert(batch.region_id.clone(), batch.alerts.clone());
        self.catalog
            .map_scene_by_region
            .insert(batch.region_id.clone(), map_bundle.scene);
        self.catalog
            .tilejson_by_frame
            .extend(map_bundle.tilejson_by_frame);
        self.catalog
            .geojson_by_frame
            .extend(map_bundle.geojson_by_frame);

        self.catalog
            .published_products
            .extend(
                batch
                    .normalized_products
                    .iter()
                    .map(|product| WeatherProductPublishedV1 {
                        product_ref: product.product_id.clone(),
                        region_id: product.region_id.clone(),
                        artifact_kind: product.artifact_kind,
                        native_identifier: product.native_identifier.clone(),
                        event_time: product.event_time,
                        valid_time: product.valid_time,
                    }),
            );
        self.catalog.updated_alerts.push(WeatherAlertUpdatedV1 {
            feed: batch.alerts.clone(),
        });

        Ok(WeatherIngestionReport {
            batch_id: batch.batch_id,
            raw_asset_count: batch.raw_assets.len(),
            normalized_product_count: batch.normalized_products.len(),
            feature_slice_count: batch.feature_slices.len(),
            alert_count: batch.alerts.alerts.len(),
            latest_availability: batch.availability,
        })
    }

    pub fn weather_availability(&self, region_id: &str) -> Option<&WeatherAvailabilityV1> {
        self.catalog.availability_by_region.get(region_id)
    }

    pub fn weather_view(&self, region_id: &str) -> Option<&contracts::WeatherViewV1> {
        self.catalog.view_by_region.get(region_id)
    }

    pub fn weather_feature_slices(&self, region_id: &str) -> &[WeatherFeatureSliceV1] {
        self.catalog
            .feature_slices_by_region
            .get(region_id)
            .map_or(&[], Vec::as_slice)
    }

    pub fn weather_alert_feed(&self, region_id: &str) -> Option<&WeatherAlertFeedV1> {
        self.catalog.alerts_by_region.get(region_id)
    }

    pub fn weather_map_scene(&self, region_id: &str) -> Option<&WeatherMapSceneV1> {
        self.catalog.map_scene_by_region.get(region_id)
    }

    pub fn weather_map_tilejson(
        &self,
        scene_id: &str,
        source_id: &str,
        frame_id: &str,
    ) -> Option<&WeatherTileJsonDocumentV1> {
        self.catalog.tilejson_by_frame.get(&(
            scene_id.to_string(),
            source_id.to_string(),
            frame_id.to_string(),
        ))
    }

    pub fn weather_map_geojson(
        &self,
        scene_id: &str,
        source_id: &str,
        frame_id: &str,
    ) -> Option<&Value> {
        self.catalog.geojson_by_frame.get(&(
            scene_id.to_string(),
            source_id.to_string(),
            frame_id.to_string(),
        ))
    }

    #[must_use]
    pub fn weather_map_http_adapter(&self) -> WeatherMapHttpAdapter<'_> {
        WeatherMapHttpAdapter::new(self)
    }

    pub fn published_products(&self) -> &[WeatherProductPublishedV1] {
        &self.catalog.published_products
    }

    pub fn updated_alerts(&self) -> &[WeatherAlertUpdatedV1] {
        &self.catalog.updated_alerts
    }

    fn scene_by_id(&self, scene_id: &str) -> Option<&WeatherMapSceneV1> {
        self.catalog
            .map_scene_by_region
            .values()
            .find(|scene| scene.scene_id == scene_id)
    }
}

#[derive(Debug)]
struct WeatherMapSceneBundle {
    scene: WeatherMapSceneV1,
    tilejson_by_frame: BTreeMap<(String, String, String), WeatherTileJsonDocumentV1>,
    geojson_by_frame: BTreeMap<(String, String, String), Value>,
}

#[derive(Debug, Clone)]
struct SceneSourceDescriptor {
    source: WeatherMapSourceV1,
    layer: WeatherMapLayerV1,
    activation_time: DateTime<Utc>,
    event_time: DateTime<Utc>,
}

fn build_map_scene_bundle(batch: &WeatherFixtureBatchV1) -> WeatherMapSceneBundle {
    let scene_id = format!("weather-map-scene-{}", batch.region_id);
    let scene_revision = format!(
        "{}-{}",
        batch.region_id,
        batch.availability.generated_at.timestamp()
    );
    let frame_times = collect_frame_times(batch);
    let sources = build_scene_sources(batch);
    let active_frame_time = frame_times
        .last()
        .copied()
        .unwrap_or(batch.availability.generated_at);
    let mut frames = Vec::new();
    let mut tilejson_by_frame = BTreeMap::new();
    let mut geojson_by_frame = BTreeMap::new();

    for (index, valid_time) in frame_times.iter().enumerate() {
        let frame_id = format!("frame-{:02}", index + 1);
        let mut source_bindings = Vec::new();

        for descriptor in &sources {
            if descriptor.activation_time > *valid_time {
                continue;
            }
            let revision = format!("{}-{}", descriptor.source.source_id, valid_time.timestamp());

            let tilejson_url = match descriptor.source.encoding {
                WeatherMapSourceEncodingV1::RasterTile | WeatherMapSourceEncodingV1::VectorTile => {
                    Some(weather_map_tilejson_path(
                        &scene_id,
                        &descriptor.source.source_id,
                        &frame_id,
                        &revision,
                    ))
                }
                WeatherMapSourceEncodingV1::GeoJson => None,
            };
            let data_url = match descriptor.source.encoding {
                WeatherMapSourceEncodingV1::RasterTile => None,
                WeatherMapSourceEncodingV1::VectorTile | WeatherMapSourceEncodingV1::GeoJson => {
                    Some(weather_map_geojson_path(
                        &scene_id,
                        &descriptor.source.source_id,
                        &frame_id,
                        &revision,
                    ))
                }
            };

            if tilejson_url.is_some() {
                tilejson_by_frame.insert(
                    (
                        scene_id.clone(),
                        descriptor.source.source_id.clone(),
                        frame_id.clone(),
                    ),
                    build_tilejson_document(
                        &scene_id,
                        &descriptor.source,
                        &frame_id,
                        &revision,
                        &batch.availability.bounds,
                    ),
                );
            }

            if let Some(data_url) = &data_url {
                geojson_by_frame.insert(
                    (
                        scene_id.clone(),
                        descriptor.source.source_id.clone(),
                        frame_id.clone(),
                    ),
                    build_geojson_document(
                        batch,
                        &descriptor.source.source_id,
                        *valid_time,
                        data_url,
                    ),
                );
            }

            source_bindings.push(WeatherMapFrameSourceBindingV1 {
                source_id: descriptor.source.source_id.clone(),
                revision,
                tilejson_url,
                data_url,
            });
        }

        let event_time = sources
            .iter()
            .filter(|descriptor| descriptor.activation_time <= *valid_time)
            .map(|descriptor| descriptor.event_time)
            .max()
            .unwrap_or(batch.availability.generated_at);
        let horizon_hours = batch
            .feature_slices
            .iter()
            .filter(|slice| slice.valid_time <= *valid_time)
            .map(|slice| slice.lead_hours)
            .max()
            .unwrap_or(0);

        frames.push(WeatherMapFrameV1 {
            frame_id: frame_id.clone(),
            label: valid_time.format("%H:%MZ").to_string(),
            event_time,
            valid_time: *valid_time,
            horizon_hours,
            source_bindings,
        });
    }

    let scene = WeatherMapSceneV1 {
        scene_id: scene_id.clone(),
        region_id: batch.region_id.clone(),
        scene_revision,
        bounds: batch.availability.bounds.clone(),
        default_center: bounds_center(&batch.availability.bounds),
        default_zoom: 4.4,
        generated_at: batch.availability.generated_at,
        active_frame_id: frames
            .iter()
            .find(|frame| frame.valid_time == active_frame_time)
            .map_or_else(|| "frame-01".to_string(), |frame| frame.frame_id.clone()),
        refresh_interval_seconds: 300,
        frames,
        sources: sources
            .iter()
            .map(|descriptor| descriptor.source.clone())
            .collect(),
        layers: sources
            .iter()
            .map(|descriptor| descriptor.layer.clone())
            .collect(),
    };

    WeatherMapSceneBundle {
        scene,
        tilejson_by_frame,
        geojson_by_frame,
    }
}

fn collect_frame_times(batch: &WeatherFixtureBatchV1) -> Vec<DateTime<Utc>> {
    let mut frame_times = BTreeSet::new();
    frame_times.insert(batch.availability.generated_at);
    frame_times.extend(
        batch
            .normalized_products
            .iter()
            .map(|product| product.valid_time),
    );
    frame_times.extend(batch.feature_slices.iter().map(|slice| slice.valid_time));
    frame_times.insert(batch.alerts.generated_at);
    frame_times.into_iter().collect()
}

fn build_scene_sources(batch: &WeatherFixtureBatchV1) -> Vec<SceneSourceDescriptor> {
    let precipitation_product = batch
        .normalized_products
        .iter()
        .find(|product| product.native_identifier == "APCP");
    let cloud_product = batch
        .normalized_products
        .iter()
        .find(|product| product.artifact_kind == WeatherArtifactKindV1::SatelliteScene);
    let radar_product = batch
        .normalized_products
        .iter()
        .find(|product| product.artifact_kind == WeatherArtifactKindV1::RadarVolume);
    let surface_product = batch
        .normalized_products
        .iter()
        .find(|product| product.artifact_kind == WeatherArtifactKindV1::SurfaceObservation);
    let precipitation_slice = batch.feature_slices.iter().find(|slice| {
        slice
            .features
            .iter()
            .any(|feature| feature.feature == WeatherFeatureKindV1::PrecipitationRate)
    });

    let mut descriptors = Vec::new();

    if let Some(product) = precipitation_product {
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "precipitation-raster".to_string(),
                layer: WeatherLayerKindV1::Precipitation,
                title: "Forecast Precipitation".to_string(),
                encoding: WeatherMapSourceEncodingV1::RasterTile,
                min_zoom: 0,
                max_zoom: 9,
                attribution: "NOAA HRRR".to_string(),
                promote_id: None,
                cluster: false,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "precipitation-raster".to_string(),
                source_id: "precipitation-raster".to_string(),
                layer: WeatherLayerKindV1::Precipitation,
                title: "Forecast Precipitation".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Raster,
                source_layer: None,
                visible_by_default: true,
                legend: vec![
                    legend_stop("Light", "#6baed6", Some(0.0), Some(2.0)),
                    legend_stop("Moderate", "#3182bd", Some(2.0), Some(6.0)),
                    legend_stop("Heavy", "#08519c", Some(6.0), None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Forecast precipitation".to_string(),
                    property_keys: vec!["native_identifier".to_string(), "valid_time".to_string()],
                }),
            },
            activation_time: product.valid_time,
            event_time: product.event_time,
        });
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "precipitation-contours".to_string(),
                layer: WeatherLayerKindV1::Precipitation,
                title: "Precipitation Isobands".to_string(),
                encoding: WeatherMapSourceEncodingV1::VectorTile,
                min_zoom: 2,
                max_zoom: 10,
                attribution: "NOAA HRRR".to_string(),
                promote_id: Some("contour_id".to_string()),
                cluster: false,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "precipitation-contours".to_string(),
                source_id: "precipitation-contours".to_string(),
                layer: WeatherLayerKindV1::Precipitation,
                title: "Precipitation Isobands".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Fill,
                source_layer: Some("precipitation_contours".to_string()),
                visible_by_default: false,
                legend: vec![
                    legend_stop("Watch", "#9ecae1", Some(0.0), Some(1.0)),
                    legend_stop("Advisory", "#6baed6", Some(1.0), Some(3.0)),
                    legend_stop("Severe", "#2171b5", Some(3.0), None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Precipitation band".to_string(),
                    property_keys: vec![
                        "precipitation_rate".to_string(),
                        "probability".to_string(),
                    ],
                }),
            },
            activation_time: precipitation_slice
                .map_or(product.valid_time, |slice| slice.valid_time),
            event_time: precipitation_slice.map_or(product.event_time, |slice| slice.event_time),
        });
    }

    if let Some(product) = radar_product {
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "radar-reflectivity".to_string(),
                layer: WeatherLayerKindV1::RadarReflectivity,
                title: "Radar Reflectivity".to_string(),
                encoding: WeatherMapSourceEncodingV1::RasterTile,
                min_zoom: 0,
                max_zoom: 10,
                attribution: "NOAA NEXRAD".to_string(),
                promote_id: None,
                cluster: false,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "radar-reflectivity".to_string(),
                source_id: "radar-reflectivity".to_string(),
                layer: WeatherLayerKindV1::RadarReflectivity,
                title: "Radar Reflectivity".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Raster,
                source_layer: None,
                visible_by_default: true,
                legend: vec![
                    legend_stop("Light", "#c7e9b4", None, Some(20.0)),
                    legend_stop("Moderate", "#41ab5d", Some(20.0), Some(40.0)),
                    legend_stop("Strong", "#005a32", Some(40.0), None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Radar".to_string(),
                    property_keys: vec!["native_identifier".to_string(), "valid_time".to_string()],
                }),
            },
            activation_time: product.valid_time,
            event_time: product.event_time,
        });
    }

    if let Some(product) = cloud_product {
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "cloud-cover".to_string(),
                layer: WeatherLayerKindV1::CloudCover,
                title: "Cloud Cover".to_string(),
                encoding: WeatherMapSourceEncodingV1::RasterTile,
                min_zoom: 0,
                max_zoom: 8,
                attribution: "NOAA GOES".to_string(),
                promote_id: None,
                cluster: false,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "cloud-cover".to_string(),
                source_id: "cloud-cover".to_string(),
                layer: WeatherLayerKindV1::CloudCover,
                title: "Cloud Cover".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Raster,
                source_layer: None,
                visible_by_default: false,
                legend: vec![
                    legend_stop("Sparse", "#bdd7e7", None, Some(0.4)),
                    legend_stop("Broken", "#6baed6", Some(0.4), Some(0.8)),
                    legend_stop("Overcast", "#2171b5", Some(0.8), None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Cloud cover".to_string(),
                    property_keys: vec!["native_identifier".to_string(), "valid_time".to_string()],
                }),
            },
            activation_time: product.valid_time,
            event_time: product.event_time,
        });
    }

    if let Some(product) = surface_product {
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "surface-stations".to_string(),
                layer: WeatherLayerKindV1::SurfaceObservations,
                title: "Surface Stations".to_string(),
                encoding: WeatherMapSourceEncodingV1::VectorTile,
                min_zoom: 3,
                max_zoom: 12,
                attribution: "NOAA METAR".to_string(),
                promote_id: Some("station_id".to_string()),
                cluster: true,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "surface-stations".to_string(),
                source_id: "surface-stations".to_string(),
                layer: WeatherLayerKindV1::SurfaceObservations,
                title: "Surface Stations".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Circle,
                source_layer: Some("surface_stations".to_string()),
                visible_by_default: true,
                legend: vec![
                    legend_stop("Visibility < 5 mi", "#fdae6b", None, Some(5.0)),
                    legend_stop("Visibility 5-10 mi", "#fd8d3c", Some(5.0), Some(10.0)),
                    legend_stop("Visibility > 10 mi", "#e6550d", Some(10.0), None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Surface station".to_string(),
                    property_keys: vec![
                        "station_id".to_string(),
                        "visibility".to_string(),
                        "wind_speed10m".to_string(),
                    ],
                }),
            },
            activation_time: product.valid_time,
            event_time: product.event_time,
        });
    }

    if !batch.alerts.alerts.is_empty() {
        descriptors.push(SceneSourceDescriptor {
            source: WeatherMapSourceV1 {
                source_id: "alert-overlay".to_string(),
                layer: WeatherLayerKindV1::AlertOverlay,
                title: "Active Alerts".to_string(),
                encoding: WeatherMapSourceEncodingV1::VectorTile,
                min_zoom: 2,
                max_zoom: 10,
                attribution: "NWS Alerts".to_string(),
                promote_id: Some("alert_id".to_string()),
                cluster: false,
            },
            layer: WeatherMapLayerV1 {
                layer_id: "alert-overlay".to_string(),
                source_id: "alert-overlay".to_string(),
                layer: WeatherLayerKindV1::AlertOverlay,
                title: "Active Alerts".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Line,
                source_layer: Some("alerts".to_string()),
                visible_by_default: true,
                legend: vec![
                    legend_stop("Moderate", "#fdae6b", None, None),
                    legend_stop("Severe", "#e6550d", None, None),
                ],
                interaction: Some(WeatherMapInteractionV1 {
                    popup_title: "Weather alert".to_string(),
                    property_keys: vec![
                        "headline".to_string(),
                        "severity".to_string(),
                        "effective_at".to_string(),
                    ],
                }),
            },
            activation_time: batch.alerts.generated_at,
            event_time: batch.alerts.generated_at,
        });
    }

    descriptors.push(SceneSourceDescriptor {
        source: WeatherMapSourceV1 {
            source_id: "selection-overlay".to_string(),
            layer: WeatherLayerKindV1::SurfaceObservations,
            title: "Selection Overlay".to_string(),
            encoding: WeatherMapSourceEncodingV1::GeoJson,
            min_zoom: 0,
            max_zoom: 12,
            attribution: "Origin Weather".to_string(),
            promote_id: Some("selection_id".to_string()),
            cluster: false,
        },
        layer: WeatherMapLayerV1 {
            layer_id: "selection-overlay".to_string(),
            source_id: "selection-overlay".to_string(),
            layer: WeatherLayerKindV1::SurfaceObservations,
            title: "Selection Overlay".to_string(),
            render_mode: WeatherMapLayerRenderModeV1::Symbol,
            source_layer: None,
            visible_by_default: false,
            legend: Vec::new(),
            interaction: Some(WeatherMapInteractionV1 {
                popup_title: "Selection".to_string(),
                property_keys: vec!["selection_label".to_string(), "selection_kind".to_string()],
            }),
        },
        activation_time: batch.availability.generated_at,
        event_time: batch.availability.generated_at,
    });

    descriptors
}

fn build_tilejson_document(
    scene_id: &str,
    source: &WeatherMapSourceV1,
    frame_id: &str,
    revision: &str,
    bounds: &GeoBoundsV1,
) -> WeatherTileJsonDocumentV1 {
    let format = match source.encoding {
        WeatherMapSourceEncodingV1::RasterTile => WeatherMapTilePayloadFormatV1::Webp,
        WeatherMapSourceEncodingV1::VectorTile | WeatherMapSourceEncodingV1::GeoJson => {
            WeatherMapTilePayloadFormatV1::Mvt
        }
    };

    WeatherTileJsonDocumentV1 {
        tilejson: "3.0.0".to_string(),
        name: source.title.clone(),
        scheme: "xyz".to_string(),
        tiles: vec![weather_map_tile_path(
            scene_id,
            &source.source_id,
            frame_id,
            revision,
            format,
        )],
        minzoom: source.min_zoom,
        maxzoom: source.max_zoom,
        bounds: [bounds.west, bounds.south, bounds.east, bounds.north],
        attribution: source.attribution.clone(),
    }
}

fn build_geojson_document(
    batch: &WeatherFixtureBatchV1,
    source_id: &str,
    frame_time: DateTime<Utc>,
    data_url: &str,
) -> Value {
    let features = match source_id {
        "alert-overlay" => alert_geojson_features(&batch.alerts, frame_time),
        "surface-stations" => {
            station_geojson_features(&batch.normalized_products, &batch.feature_slices)
        }
        "precipitation-contours" => contour_geojson_features(
            &batch.availability.bounds,
            &batch.feature_slices,
            frame_time,
        ),
        "selection-overlay" => selection_geojson_features(batch, frame_time),
        _ => Vec::new(),
    };

    json!({
        "type": "FeatureCollection",
        "metadata": {
            "region_id": batch.region_id,
            "frame_time": frame_time.to_rfc3339(),
            "source_id": source_id,
            "data_url": data_url,
        },
        "features": features,
    })
}

fn alert_geojson_features(alerts: &WeatherAlertFeedV1, frame_time: DateTime<Utc>) -> Vec<Value> {
    alerts
        .alerts
        .iter()
        .filter(|alert| alert.effective_at <= frame_time && alert.expires_at >= frame_time)
        .map(|alert| {
            json!({
                "type": "Feature",
                "id": alert.alert_id,
                "properties": {
                    "alert_id": alert.alert_id,
                    "headline": alert.headline,
                    "severity": alert.severity,
                    "effective_at": alert.effective_at.to_rfc3339(),
                    "expires_at": alert.expires_at.to_rfc3339(),
                },
                "geometry": bounds_polygon(&alert.bounds),
            })
        })
        .collect()
}

fn station_geojson_features(
    normalized_products: &[NormalizedWeatherProduct],
    feature_slices: &[WeatherFeatureSliceV1],
) -> Vec<Value> {
    normalized_products
        .iter()
        .filter(|product| product.artifact_kind == WeatherArtifactKindV1::SurfaceObservation)
        .filter_map(|product| {
            let location = product.location.as_ref()?;
            let slice = feature_slices.iter().find(|slice| {
                slice
                    .source_product_refs
                    .iter()
                    .any(|id| id == &product.product_id)
            });
            let visibility = slice.and_then(|slice| {
                slice
                    .features
                    .iter()
                    .find(|feature| feature.feature == WeatherFeatureKindV1::Visibility)
                    .map(|feature| feature.value)
            });
            let wind_speed10m = slice.and_then(|slice| {
                slice
                    .features
                    .iter()
                    .find(|feature| feature.feature == WeatherFeatureKindV1::WindSpeed10m)
                    .map(|feature| feature.value)
            });

            Some(json!({
                "type": "Feature",
                "id": product.native_identifier,
                "properties": {
                    "station_id": product.native_identifier,
                    "visibility": visibility,
                    "wind_speed10m": wind_speed10m,
                    "valid_time": product.valid_time.to_rfc3339(),
                },
                "geometry": {
                    "type": "Point",
                    "coordinates": [location.longitude, location.latitude],
                },
            }))
        })
        .collect()
}

fn contour_geojson_features(
    bounds: &GeoBoundsV1,
    feature_slices: &[WeatherFeatureSliceV1],
    frame_time: DateTime<Utc>,
) -> Vec<Value> {
    feature_slices
        .iter()
        .filter(|slice| slice.valid_time <= frame_time)
        .filter_map(|slice| {
            let precipitation_rate = slice
                .features
                .iter()
                .find(|feature| feature.feature == WeatherFeatureKindV1::PrecipitationRate)?;
            let probability = precipitation_rate.probability.unwrap_or_default();
            let inset = 0.15 + (probability.min(0.8) * 0.2);
            let contour_bounds = inset_bounds(bounds, inset);

            Some(json!({
                "type": "Feature",
                "id": format!("contour-{}", slice.slice_id),
                "properties": {
                    "contour_id": format!("contour-{}", slice.slice_id),
                    "precipitation_rate": precipitation_rate.value,
                    "probability": precipitation_rate.probability,
                    "valid_time": slice.valid_time.to_rfc3339(),
                },
                "geometry": bounds_polygon(&contour_bounds),
            }))
        })
        .collect()
}

fn selection_geojson_features(
    batch: &WeatherFixtureBatchV1,
    frame_time: DateTime<Utc>,
) -> Vec<Value> {
    let mut features = station_geojson_features(&batch.normalized_products, &batch.feature_slices);
    let mut alerts = alert_geojson_features(&batch.alerts, frame_time);
    features.append(&mut alerts);

    features
        .into_iter()
        .enumerate()
        .map(|(index, feature)| {
            let mut feature = feature;
            if let Some(properties) = feature.get_mut("properties").and_then(Value::as_object_mut) {
                properties.insert(
                    "selection_id".to_string(),
                    Value::String(format!("selection-{index:02}")),
                );
                properties.insert(
                    "selection_label".to_string(),
                    properties
                        .get("headline")
                        .cloned()
                        .or_else(|| properties.get("station_id").cloned())
                        .unwrap_or_else(|| Value::String("selection".to_string())),
                );
                properties.insert(
                    "selection_kind".to_string(),
                    if properties.contains_key("headline") {
                        Value::String("alert".to_string())
                    } else {
                        Value::String("station".to_string())
                    },
                );
            }
            feature
        })
        .collect()
}

pub fn weather_map_scene_path(region_id: &str) -> String {
    format!("/api/weather/maps/scenes/{region_id}")
}

fn weather_map_tilejson_path(
    scene_id: &str,
    source_id: &str,
    frame_id: &str,
    revision: &str,
) -> String {
    format!(
        "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/tilejson.json?frame_id={frame_id}&revision={revision}"
    )
}

fn weather_map_tile_path(
    scene_id: &str,
    source_id: &str,
    frame_id: &str,
    revision: &str,
    format: WeatherMapTilePayloadFormatV1,
) -> String {
    format!(
        "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/tiles/{{z}}/{{x}}/{{y}}.{}?frame_id={frame_id}&revision={revision}",
        format.extension()
    )
}

fn weather_map_geojson_path(
    scene_id: &str,
    source_id: &str,
    frame_id: &str,
    revision: &str,
) -> String {
    format!(
        "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/features.geojson?frame_id={frame_id}&revision={revision}"
    )
}

fn scene_manifest_cache_control() -> &'static str {
    "public, max-age=30, stale-while-revalidate=120"
}

fn immutable_asset_cache_control() -> &'static str {
    "public, max-age=31536000, immutable"
}

fn json_response<T: Serialize>(
    payload: &T,
    content_type: &str,
    cache_control: &str,
) -> InstitutionalResult<WeatherMapHttpResponseV1> {
    let body = serde_json::to_vec(payload).map_err(|err| {
        InstitutionalError::invariant(
            OperationContext::new("services/meteorological-service", "json_response"),
            format!("failed to serialize weather map response: {err}"),
        )
    })?;

    Ok(WeatherMapHttpResponseV1 {
        content_type: content_type.to_string(),
        cache_control: cache_control.to_string(),
        body,
    })
}

fn weather_not_found(operation: &'static str, message: &'static str) -> InstitutionalError {
    InstitutionalError::not_found(
        OperationContext::new("services/meteorological-service", operation),
        message,
    )
}

fn bounds_center(bounds: &GeoBoundsV1) -> GeoPointV1 {
    GeoPointV1 {
        longitude: f64::midpoint(bounds.west, bounds.east),
        latitude: f64::midpoint(bounds.south, bounds.north),
    }
}

fn bounds_polygon(bounds: &GeoBoundsV1) -> Value {
    json!({
        "type": "Polygon",
        "coordinates": [[
            [bounds.west, bounds.south],
            [bounds.east, bounds.south],
            [bounds.east, bounds.north],
            [bounds.west, bounds.north],
            [bounds.west, bounds.south],
        ]],
    })
}

fn inset_bounds(bounds: &GeoBoundsV1, fraction: f64) -> GeoBoundsV1 {
    let lat_inset = (bounds.north - bounds.south) * fraction;
    let lon_inset = (bounds.east - bounds.west).abs() * fraction;
    GeoBoundsV1 {
        north: bounds.north - lat_inset,
        south: bounds.south + lat_inset,
        east: bounds.east - lon_inset,
        west: bounds.west + lon_inset,
    }
}

fn legend_stop(
    label: &str,
    color: &str,
    min_value: Option<f64>,
    max_value: Option<f64>,
) -> WeatherMapLegendStopV1 {
    WeatherMapLegendStopV1 {
        label: label.to_string(),
        color: color.to_string(),
        min_value,
        max_value,
    }
}

pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: SERVICE_NAME.to_owned(),
        domain: DOMAIN_NAME.to_owned(),
        approved_workflows: APPROVED_WORKFLOWS.iter().copied().map(Into::into).collect(),
        owned_aggregates: OWNED_AGGREGATES.iter().copied().map(Into::into).collect(),
    }
}

#[cfg(test)]
mod tests {
    mod contract_parity {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/contract_parity.rs"
        ));
    }

    use contract_parity::assert_service_boundary_matches_catalog;

    use super::{
        DOMAIN_NAME, MeteorologicalService, WeatherFixtureBatchV1, WeatherMapHttpAdapter,
        WeatherMapSourceRequestV1, WeatherMapTilePayloadFormatV1, WeatherMapTileRequestV1,
        immutable_asset_cache_control, scene_manifest_cache_control, service_boundary,
        weather_map_scene_path,
    };

    fn load_fixture() -> WeatherFixtureBatchV1 {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../testing/fixtures/weather/run-2026-03-10/noaa_weather_batch.json"
        )))
        .expect("weather fixture json")
    }

    #[test]
    fn service_boundary_matches_enterprise_catalog() {
        let source = include_str!(
            "../../../enterprise/domains/meteorological_intelligence/service_boundaries.toml"
        );
        let boundary = service_boundary();

        assert_service_boundary_matches_catalog(&boundary, DOMAIN_NAME, source);
    }

    #[test]
    fn ingest_fixture_populates_queryable_weather_products() {
        let fixture = load_fixture();
        let mut service = MeteorologicalService::default();

        let report = service.ingest_fixture_batch(fixture).expect("ingest");

        assert_eq!(report.raw_asset_count, 6);
        assert_eq!(report.normalized_product_count, 4);
        assert_eq!(report.feature_slice_count, 2);
        assert_eq!(report.alert_count, 1);
        assert_eq!(
            service
                .weather_availability("us-west")
                .expect("availability")
                .available_layers
                .len(),
            3
        );
        assert_eq!(service.weather_feature_slices("us-west").len(), 2);
        assert_eq!(service.published_products().len(), 4);
        assert_eq!(service.updated_alerts().len(), 1);
        let scene = service.weather_map_scene("us-west").expect("map scene");
        assert_eq!(scene.region_id, "us-west");
        assert_eq!(scene.sources.len(), 7);
        assert_eq!(scene.layers.len(), 7);
        assert_eq!(scene.frames.len(), 4);
        assert!(scene.sources.iter().any(|source| source.cluster));
        assert!(
            scene
                .frames
                .iter()
                .all(|frame| !frame.source_bindings.is_empty())
        );
    }

    #[test]
    fn view_and_features_share_weather_provenance_roots() {
        let fixture = load_fixture();
        let mut service = MeteorologicalService::default();
        service.ingest_fixture_batch(fixture).expect("ingest");

        let view_hashes = service
            .weather_view("us-west")
            .expect("view")
            .layers
            .iter()
            .flat_map(|layer| {
                layer
                    .provenance
                    .iter()
                    .map(|record| record.raw_artifact_hash.clone())
            })
            .collect::<Vec<_>>();
        let feature_hashes = service
            .weather_feature_slices("us-west")
            .iter()
            .flat_map(|slice| {
                slice
                    .provenance
                    .iter()
                    .map(|record| record.raw_artifact_hash.clone())
            })
            .collect::<Vec<_>>();

        assert!(view_hashes.iter().any(|hash| feature_hashes.contains(hash)));
    }

    #[test]
    fn map_scene_exposes_tilejson_and_geojson_delivery_docs() {
        let fixture = load_fixture();
        let mut service = MeteorologicalService::default();
        service.ingest_fixture_batch(fixture).expect("ingest");

        let scene = service.weather_map_scene("us-west").expect("scene");
        let active_frame = scene
            .frames
            .iter()
            .find(|frame| frame.frame_id == scene.active_frame_id)
            .expect("active frame");
        let precipitation = active_frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == "precipitation-raster")
            .expect("precipitation");
        let alerts = active_frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == "alert-overlay")
            .expect("alerts");

        let tilejson = service
            .weather_map_tilejson(
                &scene.scene_id,
                "precipitation-raster",
                &active_frame.frame_id,
            )
            .expect("tilejson");
        let alert_geojson = service
            .weather_map_geojson(&scene.scene_id, "alert-overlay", &active_frame.frame_id)
            .expect("alert geojson");
        let expected_tilejson_url = format!(
            "/api/weather/maps/scenes/{}/sources/precipitation-raster/tilejson.json?frame_id={}&revision={}",
            scene.scene_id, active_frame.frame_id, precipitation.revision
        );
        let expected_geojson_url = format!(
            "/api/weather/maps/scenes/{}/sources/alert-overlay/features.geojson?frame_id={}&revision={}",
            scene.scene_id, active_frame.frame_id, alerts.revision
        );
        let expected_tile_url = format!(
            "/api/weather/maps/scenes/{}/sources/precipitation-raster/tiles/{{z}}/{{x}}/{{y}}.webp?frame_id={}&revision={}",
            scene.scene_id, active_frame.frame_id, precipitation.revision
        );

        assert_eq!(
            precipitation.tilejson_url.as_deref(),
            Some(expected_tilejson_url.as_str())
        );
        assert_eq!(
            alerts.data_url.as_deref(),
            Some(expected_geojson_url.as_str())
        );
        assert_eq!(tilejson.tiles.len(), 1);
        assert_eq!(tilejson.tiles[0], expected_tile_url);
        assert_eq!(alert_geojson["type"], "FeatureCollection");
        assert_eq!(alert_geojson["features"].as_array().map_or(0, Vec::len), 1);
    }

    #[test]
    fn map_http_adapter_serves_scene_and_revisioned_assets() {
        let fixture = load_fixture();
        let mut service = MeteorologicalService::default();
        service.ingest_fixture_batch(fixture).expect("ingest");

        let adapter = service.weather_map_http_adapter();
        let scene_response = adapter.get_scene("us-west").expect("scene response");
        assert_eq!(scene_response.content_type, "application/json");
        assert_eq!(scene_response.cache_control, scene_manifest_cache_control());
        let scene: contracts::WeatherMapSceneV1 =
            serde_json::from_slice(&scene_response.body).expect("scene json");
        assert_eq!(
            weather_map_scene_path("us-west"),
            "/api/weather/maps/scenes/us-west"
        );

        let active_frame = scene
            .frames
            .iter()
            .find(|frame| frame.frame_id == scene.active_frame_id)
            .expect("active frame");
        let precipitation = active_frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == "precipitation-raster")
            .expect("precipitation binding");
        let alerts = active_frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == "alert-overlay")
            .expect("alerts binding");

        let tilejson_response = adapter
            .get_tilejson(&WeatherMapSourceRequestV1 {
                scene_id: scene.scene_id.clone(),
                source_id: "precipitation-raster".to_string(),
                frame_id: active_frame.frame_id.clone(),
                revision: precipitation.revision.clone(),
            })
            .expect("tilejson response");
        assert_eq!(tilejson_response.content_type, "application/json");
        assert_eq!(
            tilejson_response.cache_control,
            immutable_asset_cache_control()
        );

        let geojson_response = adapter
            .get_geojson(&WeatherMapSourceRequestV1 {
                scene_id: scene.scene_id.clone(),
                source_id: "alert-overlay".to_string(),
                frame_id: active_frame.frame_id.clone(),
                revision: alerts.revision.clone(),
            })
            .expect("geojson response");
        assert_eq!(geojson_response.content_type, "application/geo+json");
        assert_eq!(
            geojson_response.cache_control,
            immutable_asset_cache_control()
        );

        let tile_response = adapter
            .get_tile(&WeatherMapTileRequestV1 {
                source: WeatherMapSourceRequestV1 {
                    scene_id: scene.scene_id.clone(),
                    source_id: "precipitation-raster".to_string(),
                    frame_id: active_frame.frame_id.clone(),
                    revision: precipitation.revision.clone(),
                },
                z: 4,
                x: 2,
                y: 6,
                format: WeatherMapTilePayloadFormatV1::Webp,
            })
            .expect("tile response");
        assert_eq!(tile_response.content_type, "image/webp");
        assert_eq!(tile_response.cache_control, immutable_asset_cache_control());
        assert_eq!(
            WeatherMapHttpAdapter::routes()[0].path_template,
            "/api/weather/maps/scenes/{region_id}"
        );
        assert_eq!(
            WeatherMapHttpAdapter::routes()[1].path_template,
            "/api/weather/maps/scenes/{scene_id}/sources/{source_id}/tilejson.json"
        );
    }
}
