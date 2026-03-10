use std::collections::BTreeSet;

use contracts::{
    GeoBoundsV1, GeoPointV1, WeatherMapLayerV1, WeatherMapSceneV1, WeatherMapSourceEncodingV1,
    WeatherMapSourceV1,
};
use serde::Serialize;

const MAX_DYNAMIC_SOURCES: usize = 10;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WeatherMapRenderSourcePlan {
    pub source_id: String,
    pub encoding: WeatherMapSourceEncodingV1,
    pub title: String,
    pub tilejson_url: Option<String>,
    pub data_url: Option<String>,
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub promote_id: Option<String>,
    pub cluster: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WeatherMapRenderPlan {
    pub scene_id: String,
    pub bounds: GeoBoundsV1,
    pub default_center: GeoPointV1,
    pub default_zoom: f64,
    pub sources: Vec<WeatherMapRenderSourcePlan>,
    pub layers: Vec<WeatherMapLayerV1>,
}

pub fn build_render_plan(
    scene: &WeatherMapSceneV1,
    active_frame_id: &str,
    hidden_layer_ids: &[String],
) -> Result<WeatherMapRenderPlan, String> {
    let active_frame = scene
        .frames
        .iter()
        .find(|frame| frame.frame_id == active_frame_id)
        .or_else(|| scene.frames.first())
        .ok_or_else(|| format!("Weather map scene {} has no frames.", scene.scene_id))?;

    let mut seen_sources = BTreeSet::new();
    let mut sources = Vec::new();
    let mut layers = Vec::new();

    for layer in &scene.layers {
        if hidden_layer_ids
            .iter()
            .any(|hidden| hidden == &layer.layer_id)
        {
            continue;
        }

        let Some(source) = scene
            .sources
            .iter()
            .find(|source| source.source_id == layer.source_id)
        else {
            continue;
        };
        let Some(binding) = active_frame
            .source_bindings
            .iter()
            .find(|binding| binding.source_id == layer.source_id)
        else {
            continue;
        };

        if seen_sources.insert(source.source_id.clone()) {
            sources.push(source_plan(
                source,
                binding.tilejson_url.clone(),
                binding.data_url.clone(),
            ));
        }
        layers.push(layer.clone());
    }

    if sources.len() > MAX_DYNAMIC_SOURCES {
        return Err(format!(
            "Weather map scene {} requires {} dynamic sources, exceeding the budget of {}.",
            scene.scene_id,
            sources.len(),
            MAX_DYNAMIC_SOURCES
        ));
    }

    Ok(WeatherMapRenderPlan {
        scene_id: scene.scene_id.clone(),
        bounds: scene.bounds.clone(),
        default_center: scene.default_center.clone(),
        default_zoom: scene.default_zoom,
        sources,
        layers,
    })
}

fn source_plan(
    source: &WeatherMapSourceV1,
    tilejson_url: Option<String>,
    data_url: Option<String>,
) -> WeatherMapRenderSourcePlan {
    WeatherMapRenderSourcePlan {
        source_id: source.source_id.clone(),
        encoding: source.encoding,
        title: source.title.clone(),
        tilejson_url,
        data_url,
        min_zoom: source.min_zoom,
        max_zoom: source.max_zoom,
        promote_id: source.promote_id.clone(),
        cluster: source.cluster,
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = r#"
const weatherMapRegistry = new Map();

function mapboxGlobal() {
  return globalThis.mapboxgl ?? null;
}

function layerColor(layer) {
  const stops = layer.legend ?? [];
  if (stops.length === 0) {
    return '#3b82f6';
  }
  return stops[stops.length - 1].color || '#3b82f6';
}

function buildSource(source) {
  if (source.encoding === 'raster_tile' && source.tilejson_url) {
    return { type: 'raster', url: source.tilejson_url, tileSize: 256 };
  }
  if (source.encoding === 'vector_tile' && source.tilejson_url) {
    return { type: 'vector', url: source.tilejson_url };
  }
  if ((source.encoding === 'geojson' || source.encoding === 'geo_json') && source.data_url) {
    return { type: 'geojson', data: source.data_url };
  }
  if (source.data_url) {
    return { type: 'geojson', data: source.data_url };
  }
  return null;
}

function buildLayer(layer, sourceId) {
  const base = {
    id: layer.layer_id,
    source: sourceId,
    layout: {
      visibility: layer.visible_by_default ? 'visible' : 'none',
    },
  };

  if (layer.source_layer) {
    base['source-layer'] = layer.source_layer;
  }

  switch (layer.render_mode) {
    case 'raster':
      return { ...base, type: 'raster', paint: { 'raster-opacity': 0.72 } };
    case 'fill':
      return {
        ...base,
        type: 'fill',
        paint: {
          'fill-color': layerColor(layer),
          'fill-opacity': 0.24,
          'fill-outline-color': layerColor(layer),
        },
      };
    case 'line':
      return {
        ...base,
        type: 'line',
        paint: {
          'line-color': layerColor(layer),
          'line-width': 2,
          'line-opacity': 0.8,
        },
      };
    case 'circle':
      return {
        ...base,
        type: 'circle',
        paint: {
          'circle-radius': 5,
          'circle-color': layerColor(layer),
          'circle-stroke-width': 1,
          'circle-stroke-color': '#ffffff',
        },
      };
    case 'symbol':
      return {
        ...base,
        type: 'symbol',
        layout: {
          ...base.layout,
          'text-field': ['coalesce', ['get', 'selection_label'], ['get', 'headline'], ['get', 'station_id'], layer.title],
          'text-size': 11,
          'text-offset': [0, 1.2],
        },
        paint: {
          'text-color': layerColor(layer),
          'text-halo-color': '#ffffff',
          'text-halo-width': 1,
        },
      };
    default:
      return null;
  }
}

function clearWeatherLayers(record) {
  if (!record || !record.map) {
    return;
  }
  const map = record.map;
  for (const layerId of [...record.layerIds].reverse()) {
    if (map.getLayer(layerId)) {
      map.removeLayer(layerId);
    }
  }
  for (const sourceId of [...record.sourceIds].reverse()) {
    if (map.getSource(sourceId)) {
      map.removeSource(sourceId);
    }
  }
  record.layerIds = [];
  record.sourceIds = [];
}

function applyScene(record, plan) {
  const map = record.map;
  if (!plan) {
    clearWeatherLayers(record);
    return;
  }

  clearWeatherLayers(record);

  for (const source of plan.sources) {
    const sourceConfig = buildSource(source);
    if (!sourceConfig) {
      continue;
    }
    map.addSource(source.source_id, sourceConfig);
    record.sourceIds.push(source.source_id);
  }

  for (const layer of plan.layers) {
    if (!map.getSource(layer.source_id)) {
      continue;
    }
    const layerConfig = buildLayer(layer, layer.source_id);
    if (!layerConfig) {
      continue;
    }
    map.addLayer(layerConfig);
    record.layerIds.push(layer.layer_id);
  }

  if (!record.didFitBounds && plan.bounds) {
    map.fitBounds(
      [
        [plan.bounds.west, plan.bounds.south],
        [plan.bounds.east, plan.bounds.north],
      ],
      { padding: 28, duration: 0 }
    );
    record.didFitBounds = true;
  }

  map.resize();
}

function ensureRecord(containerId) {
  let record = weatherMapRegistry.get(containerId);
  if (!record) {
    record = {
      map: null,
      sourceIds: [],
      layerIds: [],
      didFitBounds: false,
      sceneRevision: null,
      frameId: null,
      styleUrl: null,
    };
    weatherMapRegistry.set(containerId, record);
  }
  return record;
}

export function jsMapboxAvailable() {
  return mapboxGlobal() !== null;
}

export function jsRenderWeatherMap(containerId, token, styleUrl, planJson) {
  const mapboxgl = mapboxGlobal();
  if (!mapboxgl) {
    return false;
  }

  const container = globalThis.document?.getElementById(containerId);
  if (!container) {
    return false;
  }

  const plan = JSON.parse(planJson);
  const record = ensureRecord(containerId);

  if (!record.map || record.styleUrl !== styleUrl) {
    if (record.map) {
      record.map.remove();
    }
    mapboxgl.accessToken = token;
    record.map = new mapboxgl.Map({
      container,
      style: styleUrl,
      center: [plan.default_center.longitude, plan.default_center.latitude],
      zoom: plan.default_zoom,
      attributionControl: true,
    });
    record.sourceIds = [];
    record.layerIds = [];
    record.didFitBounds = false;
    record.styleUrl = styleUrl;
    record.map.once('load', () => applyScene(record, plan));
    return true;
  }

  if (record.map.loaded()) {
    applyScene(record, plan);
  } else {
    record.map.once('load', () => applyScene(record, plan));
  }
  return true;
}

export function jsResizeWeatherMap(containerId) {
  const record = weatherMapRegistry.get(containerId);
  if (record?.map) {
    record.map.resize();
  }
}
"#)]
    extern "C" {
        #[wasm_bindgen(js_name = jsMapboxAvailable)]
        fn js_mapbox_available() -> bool;

        #[wasm_bindgen(js_name = jsRenderWeatherMap)]
        fn js_render_weather_map(
            container_id: &str,
            token: &str,
            style_url: &str,
            plan_json: &str,
        ) -> bool;

        #[wasm_bindgen(js_name = jsResizeWeatherMap)]
        fn js_resize_weather_map(container_id: &str);
    }

    pub fn mapbox_available() -> bool {
        js_mapbox_available()
    }

    pub fn render_weather_map(
        container_id: &str,
        token: &str,
        style_url: &str,
        plan_json: &str,
    ) -> bool {
        js_render_weather_map(container_id, token, style_url, plan_json)
    }

    pub fn resize_weather_map(container_id: &str) {
        js_resize_weather_map(container_id);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    pub fn mapbox_available() -> bool {
        false
    }

    pub fn render_weather_map(
        _container_id: &str,
        _token: &str,
        _style_url: &str,
        _plan_json: &str,
    ) -> bool {
        false
    }

    pub fn resize_weather_map(_container_id: &str) {}
}

pub use imp::{mapbox_available, render_weather_map, resize_weather_map};
