//! Built-in Weather app for platform-managed meteorological intelligence surfaces.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod mapbox;

use contracts::{WeatherMapLayerV1, WeatherMapSceneV1};
use desktop_app_contract::AppServices;
use leptos::prelude::*;
use leptos::task::spawn_local;
use sdk_rs::WeatherPlatformSnapshotV1;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_ui::components::{AppShell, Button, StatusBar, StatusBarItem, Toolbar};
use system_ui::primitives::{
    ButtonVariant, DataTable, Elevation, Grid, Heading, LayoutGap, Panel, Stack, Surface,
    SurfaceVariant, Text, TextRole, TextTone,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum WeatherSection {
    Map,
    Overview,
    Features,
    Alerts,
}

impl WeatherSection {
    fn label(self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Overview => "Overview",
            Self::Features => "Features",
            Self::Alerts => "Alerts",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WeatherAppState {
    active_section: WeatherSection,
    selected_frame_id: Option<String>,
    hidden_layer_ids: Vec<String>,
}

impl Default for WeatherAppState {
    fn default() -> Self {
        Self {
            active_section: WeatherSection::Map,
            selected_frame_id: None,
            hidden_layer_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WeatherFeatureRow {
    label: String,
    valid_time: String,
    value: String,
    qc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WeatherMapLayerRow {
    layer_id: String,
    title: String,
    visible: bool,
    legend_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct MapboxConfigState {
    loaded: bool,
    token: Option<String>,
    style_url: Option<String>,
    error: Option<String>,
}

fn snapshot_region(snapshot: &WeatherPlatformSnapshotV1) -> String {
    snapshot
        .availability
        .as_ref()
        .map(|availability| availability.region_id.clone())
        .or_else(|| snapshot.view.as_ref().map(|view| view.region_id.clone()))
        .or_else(|| {
            snapshot
                .map_scene
                .as_ref()
                .map(|scene| scene.region_id.clone())
        })
        .unwrap_or_else(|| "unavailable".to_string())
}

fn snapshot_summary(snapshot: &WeatherPlatformSnapshotV1) -> String {
    let region = snapshot_region(snapshot);
    let layer_count = snapshot
        .availability
        .as_ref()
        .map_or(0, |availability| availability.available_layers.len());
    let alert_count = snapshot.alerts.as_ref().map_or(0, |feed| feed.alerts.len());
    let frame_count = snapshot
        .map_scene
        .as_ref()
        .map_or(0, |scene| scene.frames.len());
    format!(
        "{region} has {layer_count} active layers, {alert_count} alert(s), and {frame_count} map frame(s)."
    )
}

fn feature_rows(snapshot: &WeatherPlatformSnapshotV1) -> Vec<WeatherFeatureRow> {
    snapshot
        .feature_slices
        .iter()
        .flat_map(|slice| {
            slice.features.iter().map(move |feature| WeatherFeatureRow {
                label: format!("{:?}", feature.feature),
                valid_time: slice.valid_time.to_rfc3339(),
                value: format!("{:.2} {}", feature.value, feature.units),
                qc: feature
                    .qc_flags
                    .iter()
                    .map(|flag| format!("{flag:?}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            })
        })
        .collect()
}

fn alert_headlines(snapshot: &WeatherPlatformSnapshotV1) -> Vec<String> {
    snapshot
        .alerts
        .as_ref()
        .map(|feed| {
            feed.alerts
                .iter()
                .map(|alert| format!("{} ({})", alert.headline, alert.severity))
                .collect()
        })
        .unwrap_or_default()
}

fn latest_valid_time(snapshot: &WeatherPlatformSnapshotV1) -> String {
    snapshot
        .map_scene
        .as_ref()
        .and_then(|scene| scene.frames.last())
        .map(|frame| frame.valid_time.to_rfc3339())
        .or_else(|| {
            snapshot
                .view
                .as_ref()
                .map(|view| view.valid_time.to_rfc3339())
        })
        .or_else(|| {
            snapshot
                .availability
                .as_ref()
                .and_then(|availability| availability.available_layers.first())
                .map(|layer| layer.latest_valid_time.to_rfc3339())
        })
        .unwrap_or_else(|| "n/a".to_string())
}

fn current_scene(snapshot: &WeatherPlatformSnapshotV1) -> Option<WeatherMapSceneV1> {
    snapshot.map_scene.clone()
}

fn active_frame_id(scene: &WeatherMapSceneV1, state: &WeatherAppState) -> String {
    state
        .selected_frame_id
        .clone()
        .filter(|selected| scene.frames.iter().any(|frame| frame.frame_id == *selected))
        .unwrap_or_else(|| scene.active_frame_id.clone())
}

fn layer_rows(scene: &WeatherMapSceneV1, hidden_layer_ids: &[String]) -> Vec<WeatherMapLayerRow> {
    scene
        .layers
        .iter()
        .map(|layer| WeatherMapLayerRow {
            layer_id: layer.layer_id.clone(),
            title: layer.title.clone(),
            visible: layer.visible_by_default && !hidden_layer_ids.contains(&layer.layer_id),
            legend_summary: legend_summary(layer),
        })
        .collect()
}

fn legend_summary(layer: &WeatherMapLayerV1) -> String {
    if layer.legend.is_empty() {
        return "No legend".to_string();
    }

    layer
        .legend
        .iter()
        .map(|stop| stop.label.clone())
        .collect::<Vec<_>>()
        .join(" / ")
}

fn frame_labels(scene: &WeatherMapSceneV1) -> Vec<(String, String)> {
    scene
        .frames
        .iter()
        .map(|frame| (frame.frame_id.clone(), frame.label.clone()))
        .collect()
}

fn inspector_lines(
    scene: &WeatherMapSceneV1,
    active_frame_id: &str,
    hidden_layer_ids: &[String],
) -> Vec<String> {
    let Some(frame) = scene
        .frames
        .iter()
        .find(|frame| frame.frame_id == active_frame_id)
    else {
        return Vec::new();
    };

    frame
        .source_bindings
        .iter()
        .filter_map(|binding| {
            let layer = scene
                .layers
                .iter()
                .find(|layer| layer.source_id == binding.source_id)?;
            if hidden_layer_ids.contains(&layer.layer_id) {
                return None;
            }

            Some(format!(
                "{} · {} · {}",
                layer.title,
                binding.revision,
                binding
                    .tilejson_url
                    .as_deref()
                    .or(binding.data_url.as_deref())
                    .unwrap_or("no endpoint")
            ))
        })
        .collect()
}

fn map_status(scene: Option<&WeatherMapSceneV1>, config: &MapboxConfigState) -> String {
    if scene.is_none() {
        return "No weather map scene is published for this region yet.".to_string();
    }
    if !config.loaded {
        return "Loading map configuration…".to_string();
    }
    if let Some(error) = &config.error {
        return error.clone();
    }
    if config.token.is_none() || config.style_url.is_none() {
        return "Mapbox is not configured. Set maps.mapbox_public_token and maps.mapbox_style_url."
            .to_string();
    }
    if !mapbox::mapbox_available() {
        return "Mapbox GL JS is not available in this runtime.".to_string();
    }

    "Map scene ready.".to_string()
}

fn map_render_status(
    scene: Option<&WeatherMapSceneV1>,
    active_frame_id: &str,
    hidden_layer_ids: &[String],
    config: &MapboxConfigState,
) -> String {
    let status = map_status(scene, config);
    if status != "Map scene ready." {
        return status;
    }

    let Some(scene) = scene else {
        return "No weather map scene is published for this region yet.".to_string();
    };

    match mapbox::build_render_plan(scene, active_frame_id, hidden_layer_ids) {
        Ok(_) => status,
        Err(err) => err,
    }
}

fn map_container_id(scene: Option<&WeatherMapSceneV1>) -> String {
    scene.map_or_else(
        || "weather-map-missing".to_string(),
        |scene| format!("weather-map-{}", scene.region_id),
    )
}

#[component]
/// Weather window contents.
pub fn WeatherApp(
    /// Launch parameters supplied by the desktop runtime.
    launch_params: Value,
    /// Previously persisted per-window state restored by the desktop runtime.
    restored_state: Option<Value>,
    /// Capability-scoped host and platform services injected by the runtime.
    services: Option<AppServices>,
) -> impl IntoView {
    let services = services.expect("weather app requires app services");
    let app_state = RwSignal::new(WeatherAppState::default());
    let mapbox_config = RwSignal::new(MapboxConfigState::default());

    if let Some(restored_state) = restored_state
        && let Ok(restored) = serde_json::from_value::<WeatherAppState>(restored_state)
    {
        app_state.set(restored);
    }

    if let Some(section) = launch_params.get("section").and_then(Value::as_str) {
        let parsed = match section {
            "map" => Some(WeatherSection::Map),
            "features" => Some(WeatherSection::Features),
            "alerts" => Some(WeatherSection::Alerts),
            "overview" => Some(WeatherSection::Overview),
            _ => None,
        };
        if let Some(section) = parsed {
            app_state.update(|state| state.active_section = section);
        }
    }

    {
        let config_service = services.config.clone();
        let config_state = mapbox_config;
        spawn_local(async move {
            let token = config_service
                .load::<String>("maps", "mapbox_public_token")
                .await;
            let style_url = config_service
                .load::<String>("maps", "mapbox_style_url")
                .await;

            match (token, style_url) {
                (Ok(token), Ok(style_url)) => config_state.set(MapboxConfigState {
                    loaded: true,
                    token,
                    style_url,
                    error: None,
                }),
                (Err(err), _) | (_, Err(err)) => config_state.set(MapboxConfigState {
                    loaded: true,
                    token: None,
                    style_url: None,
                    error: Some(format!("Failed to load map configuration: {err}")),
                }),
            }
        });
    }

    let state_service = services.state.clone();
    Effect::new(move |_| {
        if let Ok(serialized) = serde_json::to_value(app_state.get()) {
            state_service.persist_window_state(serialized);
        }
    });

    let weather_snapshot = Signal::derive({
        let services = services.clone();
        move || services.platform.weather.get()
    });

    Effect::new(move |_| {
        let Some(scene) = weather_snapshot.get().map_scene else {
            return;
        };

        app_state.update(|state| {
            let needs_default = state.selected_frame_id.as_ref().is_none_or(|selected| {
                !scene.frames.iter().any(|frame| frame.frame_id == *selected)
            });
            if needs_default {
                state.selected_frame_id = Some(scene.active_frame_id.clone());
            }
        });
    });

    view! {
        <AppShell>
            <Toolbar aria_label="Weather sections">
                {[WeatherSection::Map, WeatherSection::Overview, WeatherSection::Features, WeatherSection::Alerts]
                    .into_iter()
                    .map(|section| {
                        let selected = move || app_state.get().active_section == section;
                        view! {
                            <Button
                                variant=ButtonVariant::Quiet
                                selected=Signal::derive(selected)
                                on_click=Callback::new(move |_| {
                                    app_state.update(|state| state.active_section = section);
                                })
                            >
                                {section.label()}
                            </Button>
                        }
                    })
                    .collect_view()}
            </Toolbar>

            <Panel variant=SurfaceVariant::Standard>
                <Stack gap=LayoutGap::Md>
                    <Heading>"Weather Intelligence"</Heading>
                    <Text tone=TextTone::Secondary>{move || snapshot_summary(&weather_snapshot.get())}</Text>

                    <Show
                        when=move || app_state.get().active_section == WeatherSection::Map
                        fallback=move || {
                            view! {
                                <Show
                                    when=move || app_state.get().active_section == WeatherSection::Overview
                                    fallback=move || {
                                        view! {
                                            <Show
                                                when=move || app_state.get().active_section == WeatherSection::Features
                                                fallback=move || {
                                                    let alerts = Signal::derive(move || alert_headlines(&weather_snapshot.get()));
                                                    view! {
                                                        <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                                                            <Stack gap=LayoutGap::Sm>
                                                                <Heading role=TextRole::Title>"Active Alerts"</Heading>
                                                                <For
                                                                    each=move || alerts.get()
                                                                    key=|headline| headline.clone()
                                                                    let:headline
                                                                >
                                                                    <Text>{headline}</Text>
                                                                </For>
                                                            </Stack>
                                                        </Surface>
                                                    }
                                                }
                                            >
                                                <WeatherFeaturesTable rows=Signal::derive(move || feature_rows(&weather_snapshot.get())) />
                                            </Show>
                                        }
                                    }
                                >
                                    <WeatherOverview snapshot=weather_snapshot />
                                </Show>
                            }
                        }
                    >
                        <WeatherMapSection
                            snapshot=weather_snapshot
                            app_state=app_state
                            mapbox_config=mapbox_config.read_only()
                        />
                    </Show>
                </Stack>
            </Panel>

            <StatusBar>
                <StatusBarItem>{move || format!("Region: {}", snapshot_region(&weather_snapshot.get()))}</StatusBarItem>
                <StatusBarItem>{move || format!("Latest valid: {}", latest_valid_time(&weather_snapshot.get()))}</StatusBarItem>
                <StatusBarItem>{move || format!("Feature slices: {}", weather_snapshot.get().feature_slices.len())}</StatusBarItem>
            </StatusBar>
        </AppShell>
    }
}

#[component]
fn WeatherMapSection(
    snapshot: Signal<WeatherPlatformSnapshotV1>,
    app_state: RwSignal<WeatherAppState>,
    mapbox_config: ReadSignal<MapboxConfigState>,
) -> impl IntoView {
    let scene = Signal::derive(move || current_scene(&snapshot.get()));
    let active_frame_id = Signal::derive(move || {
        scene.get().map_or_else(
            || "frame-00".to_string(),
            |scene| active_frame_id(&scene, &app_state.get()),
        )
    });
    let hidden_layer_ids = Signal::derive(move || app_state.get().hidden_layer_ids.clone());
    let status = Signal::derive(move || {
        let scene_value = scene.get();
        let hidden_layers = hidden_layer_ids.get();
        let active_frame = active_frame_id.get();
        map_render_status(
            scene_value.as_ref(),
            &active_frame,
            &hidden_layers,
            &mapbox_config.get(),
        )
    });
    let frame_options = Signal::derive(move || {
        scene
            .get()
            .map_or_else(Vec::new, |scene| frame_labels(&scene))
    });
    let layers = Signal::derive(move || {
        scene.get().map_or_else(Vec::new, |scene| {
            layer_rows(&scene, &app_state.get().hidden_layer_ids)
        })
    });
    let inspector = Signal::derive(move || {
        scene.get().map_or_else(Vec::new, |scene| {
            inspector_lines(
                &scene,
                &active_frame_id.get(),
                &app_state.get().hidden_layer_ids,
            )
        })
    });

    view! {
        <div style="display:grid;grid-template-columns:minmax(0,2fr) minmax(280px,1fr);gap:1rem;">
            <WeatherMapViewport
                scene=scene
                active_frame_id=active_frame_id
                hidden_layer_ids=hidden_layer_ids
                mapbox_config=mapbox_config
                status=status
            />

            <Stack gap=LayoutGap::Sm>
                <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                    <Stack gap=LayoutGap::Sm>
                        <Heading role=TextRole::Title>"Timeline"</Heading>
                        <Toolbar aria_label="Weather frames">
                            <For
                                each=move || frame_options.get()
                                key=|(frame_id, _)| frame_id.clone()
                                let:frame
                            >
                                <Button
                                    variant=ButtonVariant::Quiet
                                    selected=Signal::derive({
                                        let frame_id = frame.0.clone();
                                        move || active_frame_id.get() == frame_id
                                    })
                                    on_click=Callback::new({
                                        let frame_id = frame.0.clone();
                                        move |_| {
                                            app_state.update(|state| state.selected_frame_id = Some(frame_id.clone()));
                                        }
                                    })
                                >
                                    {frame.1.clone()}
                                </Button>
                            </For>
                        </Toolbar>
                    </Stack>
                </Surface>

                <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                    <Stack gap=LayoutGap::Sm>
                        <Heading role=TextRole::Title>"Layers"</Heading>
                        <For
                            each=move || layers.get()
                            key=|row| row.layer_id.clone()
                            let:row
                        >
                            <Button
                                variant=ButtonVariant::Quiet
                                selected=Signal::derive({
                                    let visible = row.visible;
                                    move || visible
                                })
                                on_click=Callback::new({
                                    let layer_id = row.layer_id.clone();
                                    move |_| {
                                        app_state.update(|state| {
                                            if let Some(index) = state
                                                .hidden_layer_ids
                                                .iter()
                                                .position(|hidden| hidden == &layer_id)
                                            {
                                                state.hidden_layer_ids.remove(index);
                                            } else {
                                                state.hidden_layer_ids.push(layer_id.clone());
                                            }
                                        });
                                    }
                                })
                            >
                                {format!("{} · {}", row.title, row.legend_summary)}
                            </Button>
                        </For>
                    </Stack>
                </Surface>

                <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                    <Stack gap=LayoutGap::Sm>
                        <Heading role=TextRole::Title>"Inspector"</Heading>
                        <For
                            each=move || inspector.get()
                            key=|line| line.clone()
                            let:line
                        >
                            <Text>{line}</Text>
                        </For>
                    </Stack>
                </Surface>
            </Stack>
        </div>
    }
}

#[component]
fn WeatherMapViewport(
    scene: Signal<Option<WeatherMapSceneV1>>,
    active_frame_id: Signal<String>,
    hidden_layer_ids: Signal<Vec<String>>,
    mapbox_config: ReadSignal<MapboxConfigState>,
    status: Signal<String>,
) -> impl IntoView {
    let container_id = Signal::derive(move || map_container_id(scene.get().as_ref()));

    Effect::new(move |_| {
        let Some(scene) = scene.get() else {
            return;
        };
        let config = mapbox_config.get();
        let (Some(token), Some(style_url)) = (config.token.as_deref(), config.style_url.as_deref())
        else {
            return;
        };
        let Ok(render_plan) =
            mapbox::build_render_plan(&scene, &active_frame_id.get(), &hidden_layer_ids.get())
        else {
            return;
        };

        let render_plan_json = serde_json::to_string(&render_plan).expect("render plan json");
        let _ =
            mapbox::render_weather_map(&container_id.get(), token, style_url, &render_plan_json);
        mapbox::resize_weather_map(&container_id.get());
    });

    view! {
        <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
            <Stack gap=LayoutGap::Sm>
                <Heading role=TextRole::Title>"Map Scene"</Heading>
                <div
                    id=move || container_id.get()
                    style="min-height: 480px; width: 100%; border-radius: 18px; overflow: hidden; background: linear-gradient(180deg, rgba(15, 23, 36, 0.08) 0%, rgba(15, 23, 36, 0.02) 100%); border: 1px solid rgba(15, 23, 36, 0.08);"
                ></div>
                <Text tone=TextTone::Secondary>{move || status.get()}</Text>
            </Stack>
        </Surface>
    }
}

#[component]
fn WeatherOverview(snapshot: Signal<WeatherPlatformSnapshotV1>) -> impl IntoView {
    view! {
        <Grid>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Region"</Text>
                    <Text>{move || snapshot_region(&snapshot.get())}</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Latest valid time"</Text>
                    <Text>{move || latest_valid_time(&snapshot.get())}</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Available layers"</Text>
                    <Text>{move || snapshot.get().availability.map_or(0, |availability| availability.available_layers.len()).to_string()}</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Map frames"</Text>
                    <Text>{move || snapshot.get().map_scene.map_or(0, |scene| scene.frames.len()).to_string()}</Text>
                </Stack>
            </Surface>
        </Grid>
    }
}

#[component]
fn WeatherFeaturesTable(rows: Signal<Vec<WeatherFeatureRow>>) -> impl IntoView {
    view! {
        <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
            <Stack gap=LayoutGap::Sm>
                <Heading role=TextRole::Title>"Feature Products"</Heading>
                <DataTable aria_label="Weather feature table">
                    <thead>
                        <tr>
                            <th>"Feature"</th>
                            <th>"Valid time"</th>
                            <th>"Value"</th>
                            <th>"QC"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For each=move || rows.get() key=|row| format!("{}{}", row.label, row.valid_time) let:row>
                            <tr>
                                <td>{row.label}</td>
                                <td>{row.valid_time}</td>
                                <td>{row.value}</td>
                                <td>{row.qc}</td>
                            </tr>
                        </For>
                    </tbody>
                </DataTable>
            </Stack>
        </Surface>
    }
}

#[cfg(test)]
mod tests {
    use contracts::{
        GeoBoundsV1, GeoPointV1, WeatherLayerKindV1, WeatherMapFrameSourceBindingV1,
        WeatherMapFrameV1, WeatherMapLayerRenderModeV1, WeatherMapLayerV1, WeatherMapSceneV1,
        WeatherMapSourceEncodingV1, WeatherMapSourceV1,
    };
    use sdk_rs::WeatherPlatformSnapshotV1;

    use super::{
        MapboxConfigState, WeatherAppState, active_frame_id, alert_headlines, feature_rows,
        frame_labels, layer_rows, map_status, snapshot_summary,
    };

    fn load_snapshot() -> WeatherPlatformSnapshotV1 {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../testing/fixtures/weather/run-2026-03-10/platform_snapshot.json"
        )))
        .expect("weather snapshot fixture")
    }

    fn fixture_scene() -> WeatherMapSceneV1 {
        WeatherMapSceneV1 {
            scene_id: "scene-west".to_string(),
            region_id: "us-west".to_string(),
            scene_revision: "rev-1".to_string(),
            bounds: GeoBoundsV1 {
                north: 49.0,
                south: 31.0,
                east: -109.0,
                west: -125.0,
            },
            default_center: GeoPointV1 {
                longitude: -117.0,
                latitude: 40.0,
            },
            default_zoom: 4.4,
            generated_at: chrono::Utc::now(),
            active_frame_id: "frame-02".to_string(),
            refresh_interval_seconds: 300,
            frames: vec![
                WeatherMapFrameV1 {
                    frame_id: "frame-01".to_string(),
                    label: "11:10Z".to_string(),
                    event_time: chrono::Utc::now(),
                    valid_time: chrono::Utc::now(),
                    horizon_hours: 0,
                    source_bindings: vec![WeatherMapFrameSourceBindingV1 {
                        source_id: "alerts".to_string(),
                        revision: "a".to_string(),
                        tilejson_url: None,
                        data_url: Some("/alerts.geojson".to_string()),
                    }],
                },
                WeatherMapFrameV1 {
                    frame_id: "frame-02".to_string(),
                    label: "15:00Z".to_string(),
                    event_time: chrono::Utc::now(),
                    valid_time: chrono::Utc::now(),
                    horizon_hours: 4,
                    source_bindings: vec![WeatherMapFrameSourceBindingV1 {
                        source_id: "precip".to_string(),
                        revision: "b".to_string(),
                        tilejson_url: Some("/precip.tilejson".to_string()),
                        data_url: None,
                    }],
                },
            ],
            sources: vec![WeatherMapSourceV1 {
                source_id: "precip".to_string(),
                layer: contracts::WeatherLayerKindV1::Precipitation,
                title: "Precipitation".to_string(),
                encoding: WeatherMapSourceEncodingV1::RasterTile,
                min_zoom: 0,
                max_zoom: 8,
                attribution: "NOAA".to_string(),
                promote_id: None,
                cluster: false,
            }],
            layers: vec![WeatherMapLayerV1 {
                layer_id: "precip".to_string(),
                source_id: "precip".to_string(),
                layer: contracts::WeatherLayerKindV1::Precipitation,
                title: "Precipitation".to_string(),
                render_mode: WeatherMapLayerRenderModeV1::Raster,
                source_layer: None,
                visible_by_default: true,
                legend: Vec::new(),
                interaction: None,
            }],
        }
    }

    #[test]
    fn snapshot_summary_mentions_region_and_map_frames() {
        let summary = snapshot_summary(&load_snapshot());
        assert!(summary.contains("us-west"));
        assert!(summary.contains("map frame"));
    }

    #[test]
    fn feature_rows_flatten_weather_feature_products() {
        let rows = feature_rows(&load_snapshot());
        assert_eq!(rows.len(), 4);
        assert!(
            rows.iter()
                .any(|row| row.label.contains("PrecipitationRate"))
        );
    }

    #[test]
    fn alert_headlines_surface_active_alerts() {
        let headlines = alert_headlines(&load_snapshot());
        assert_eq!(headlines, vec!["Flood Watch (moderate)"]);
    }

    #[test]
    fn frame_labels_follow_scene_order() {
        let labels = frame_labels(&fixture_scene());
        assert_eq!(
            labels,
            vec![
                ("frame-01".to_string(), "11:10Z".to_string()),
                ("frame-02".to_string(), "15:00Z".to_string())
            ]
        );
    }

    #[test]
    fn active_frame_defaults_to_scene_active_frame() {
        let frame_id = active_frame_id(&fixture_scene(), &WeatherAppState::default());
        assert_eq!(frame_id, "frame-02");
    }

    #[test]
    fn layer_rows_respect_hidden_layers() {
        let rows = layer_rows(&fixture_scene(), &["precip".to_string()]);
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].visible);
    }

    #[test]
    fn map_status_reports_missing_config() {
        let status = map_status(Some(&fixture_scene()), &MapboxConfigState::default());
        assert!(status.contains("Loading map configuration"));
    }

    #[test]
    fn build_render_plan_translates_active_frame_sources() {
        let scene = load_snapshot().map_scene.expect("map scene");
        let plan = super::mapbox::build_render_plan(
            &scene,
            "frame-04",
            &["selection-overlay".to_string()],
        )
        .expect("render plan");

        assert_eq!(plan.sources.len(), 6);
        assert_eq!(plan.layers.len(), 6);
        assert!(plan.sources.iter().any(|source| {
            source.encoding == WeatherMapSourceEncodingV1::RasterTile
                && source.tilejson_url.as_deref().is_some_and(|url| {
                    url.contains("/api/weather/maps/scenes/weather-map-scene-us-west/sources/")
                })
        }));
        assert!(plan.sources.iter().any(|source| {
            source.encoding == WeatherMapSourceEncodingV1::VectorTile
                && source
                    .data_url
                    .as_deref()
                    .is_some_and(|url| url.contains("features.geojson"))
        }));
        assert!(
            !plan
                .layers
                .iter()
                .any(|layer| layer.layer_id == "selection-overlay")
        );
    }

    #[test]
    fn build_render_plan_rejects_source_budget_breach() {
        let frame_bindings = (0..11)
            .map(|index| WeatherMapFrameSourceBindingV1 {
                source_id: format!("source-{index}"),
                revision: format!("rev-{index}"),
                tilejson_url: Some(format!("/tilejson/{index}.json")),
                data_url: None,
            })
            .collect::<Vec<_>>();
        let sources = (0..11)
            .map(|index| WeatherMapSourceV1 {
                source_id: format!("source-{index}"),
                layer: WeatherLayerKindV1::Precipitation,
                title: format!("Source {index}"),
                encoding: WeatherMapSourceEncodingV1::RasterTile,
                min_zoom: 0,
                max_zoom: 8,
                attribution: "NOAA".to_string(),
                promote_id: None,
                cluster: false,
            })
            .collect::<Vec<_>>();
        let layers = (0..11)
            .map(|index| WeatherMapLayerV1 {
                layer_id: format!("layer-{index}"),
                source_id: format!("source-{index}"),
                layer: WeatherLayerKindV1::Precipitation,
                title: format!("Layer {index}"),
                render_mode: WeatherMapLayerRenderModeV1::Raster,
                source_layer: None,
                visible_by_default: true,
                legend: Vec::new(),
                interaction: None,
            })
            .collect::<Vec<_>>();
        let scene = WeatherMapSceneV1 {
            scene_id: "scene-budget".to_string(),
            region_id: "us-west".to_string(),
            scene_revision: "rev-budget".to_string(),
            bounds: GeoBoundsV1 {
                north: 49.0,
                south: 31.0,
                east: -109.0,
                west: -125.0,
            },
            default_center: GeoPointV1 {
                longitude: -117.0,
                latitude: 40.0,
            },
            default_zoom: 4.4,
            generated_at: chrono::Utc::now(),
            active_frame_id: "frame-01".to_string(),
            refresh_interval_seconds: 300,
            frames: vec![WeatherMapFrameV1 {
                frame_id: "frame-01".to_string(),
                label: "11:10Z".to_string(),
                event_time: chrono::Utc::now(),
                valid_time: chrono::Utc::now(),
                horizon_hours: 0,
                source_bindings: frame_bindings,
            }],
            sources,
            layers,
        };
        let status = super::mapbox::build_render_plan(&scene, "frame-01", &[])
            .expect_err("source budget should be enforced");

        assert!(status.contains("exceeding the budget of 10"));
    }
}
