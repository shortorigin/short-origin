//! Built-in Control Center app for workspace posture, capabilities, and shell guidance.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use desktop_app_contract::{AppCapability, AppServices};
use leptos::*;
use sdk_rs::UiDashboardSnapshotV1;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_ui::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum ControlCenterSection {
    Overview,
    Host,
    Guidance,
}

impl ControlCenterSection {
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Host => "Host",
            Self::Guidance => "Guidance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ControlCenterState {
    active_section: ControlCenterSection,
}

impl Default for ControlCenterState {
    fn default() -> Self {
        Self {
            active_section: ControlCenterSection::Overview,
        }
    }
}

#[component]
/// Control Center window contents.
pub fn ControlCenterApp(
    /// Launch parameters supplied by the desktop runtime for deep-link targeting.
    launch_params: Value,
    /// Previously persisted per-window state restored by the desktop runtime.
    restored_state: Option<Value>,
    /// Capability-scoped host and platform services injected by the runtime.
    services: Option<AppServices>,
) -> impl IntoView {
    let services = services.expect("control center requires app services");
    let state = create_rw_signal(ControlCenterState::default());

    if let Some(restored_state) = restored_state {
        if let Ok(restored) = serde_json::from_value::<ControlCenterState>(restored_state) {
            state.set(restored);
        }
    }

    if let Some(section) = launch_params.get("section").and_then(Value::as_str) {
        let parsed = match section {
            "host" => Some(ControlCenterSection::Host),
            "guidance" => Some(ControlCenterSection::Guidance),
            "overview" => Some(ControlCenterSection::Overview),
            _ => None,
        };
        if let Some(section) = parsed {
            state.update(|state| state.active_section = section);
        }
    }

    create_effect(move |_| {
        if let Ok(serialized) = serde_json::to_value(state.get()) {
            services.state.persist_window_state(serialized);
        }
    });

    let capabilities = services.capabilities().clone();
    let overview_capabilities = capabilities.clone();
    let host_capabilities = capabilities.clone();
    let native_explorer_status = capability_flag(capabilities.supports_native_explorer());
    let terminal_backend_status = capability_flag(capabilities.supports_terminal_process());
    let platform_dashboard = services.platform.dashboard;
    let active_section = Signal::derive(move || state.get().active_section);

    view! {
        <AppShell>
            <ToolBar aria_label="Control Center sections">
                {[
                    ControlCenterSection::Overview,
                    ControlCenterSection::Host,
                    ControlCenterSection::Guidance,
                ]
                .into_iter()
                .map(|section| {
                    let selected = move || active_section.get() == section;
                    view! {
                        <Button
                            variant=ButtonVariant::Quiet
                            selected=Signal::derive(selected)
                            on_click=Callback::new(move |_| {
                                state.update(|state| state.active_section = section);
                            })
                        >
                            {section.label()}
                        </Button>
                    }
                })
                .collect_view()}
            </ToolBar>

            <Panel variant=SurfaceVariant::Standard>
                <Stack gap=LayoutGap::Md>
                    <Heading>"Short Origin Control Center"</Heading>
                    <Text tone=TextTone::Secondary>
                        "This shell profile ships only the governed desktop surfaces needed for operational visibility and host-aware workflows."
                    </Text>

                    <Show
                        when=move || active_section.get() == ControlCenterSection::Overview
                        fallback=move || {
                            let host_capabilities = host_capabilities.clone();
                            view! {
                                <Show
                                    when=move || active_section.get() == ControlCenterSection::Host
                                    fallback=move || view! { <GuidanceSection /> }
                                >
                                    <HostSection capabilities=host_capabilities.clone() />
                                </Show>
                            }
                        }
                    >
                        <OverviewSection
                            capabilities=overview_capabilities.clone()
                            dashboard=platform_dashboard
                        />
                    </Show>
                </Stack>
            </Panel>

            <StatusBar>
                <StatusBarItem>{move || format!("Section: {}", active_section.get().label())}</StatusBarItem>
                <StatusBarItem>
                    {format!("Native explorer: {native_explorer_status}")}
                </StatusBarItem>
                <StatusBarItem>
                    {format!("Terminal backend: {terminal_backend_status}")}
                </StatusBarItem>
            </StatusBar>
        </AppShell>
    }
}

#[component]
fn OverviewSection(
    capabilities: desktop_app_contract::CapabilitySet,
    dashboard: ReadSignal<UiDashboardSnapshotV1>,
) -> impl IntoView {
    view! {
        <Grid>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Deployment model"</Text>
                    <Text>"wasmCloud + Wasmtime workloads, SurrealDB system of record, Tauri desktop runtime."</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Product shell"</Text>
                    <Text>"Control Center, Terminal, and Settings are the release surfaces in this profile."</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Granted runtime capabilities"</Text>
                    <Text>{format_capabilities(capabilities.granted())}</Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Release shell apps"</Text>
                    <Text>
                        {move || dashboard
                            .get()
                            .release_apps
                            .into_iter()
                            .map(|app| app.display_name)
                            .collect::<Vec<_>>()
                            .join(", ")}
                    </Text>
                </Stack>
            </Surface>
            <Surface variant=SurfaceVariant::Muted elevation=Elevation::Raised>
                <Stack gap=LayoutGap::Sm>
                    <Text role=TextRole::Label>"Connected cache"</Text>
                    <Text>{move || capability_flag(dashboard.get().connected_cache)}</Text>
                </Stack>
            </Surface>
        </Grid>
    }
}

#[component]
fn HostSection(capabilities: desktop_app_contract::CapabilitySet) -> impl IntoView {
    let native_explorer = capability_flag(capabilities.supports_native_explorer());
    let terminal_backend = capability_flag(capabilities.supports_terminal_process());
    let commands = capability_status_text(capabilities.status(AppCapability::Commands));
    let wallpaper = capability_status_text(capabilities.status(AppCapability::Wallpaper));
    let notifications = capability_status_text(capabilities.status(AppCapability::Notifications));

    view! {
        <ListSurface>
            <div>
                <Text role=TextRole::Label>"Native explorer"</Text>
                <Text>{native_explorer}</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Host terminal process"</Text>
                <Text>{terminal_backend}</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Structured commands"</Text>
                <Text>{commands}</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Wallpaper library"</Text>
                <Text>{wallpaper}</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Notifications"</Text>
                <Text>{notifications}</Text>
            </div>
        </ListSurface>
    }
}

#[component]
fn GuidanceSection() -> impl IntoView {
    view! {
        <ListSurface>
            <div>
                <Text role=TextRole::Label>"Connected cache posture"</Text>
                <Text>"UI state stays local and typed, but governed data remains service-owned and contract-bound."</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Desktop authority"</Text>
                <Text>"Tauri is the release runtime. Browser/WASM remains useful for preview and parity checks."</Text>
            </div>
            <div>
                <Text role=TextRole::Label>"Legacy app ids"</Text>
                <Text>"Removed sample apps resolve to compatibility placeholders so older deep links stay non-destructive."</Text>
            </div>
        </ListSurface>
    }
}

fn capability_flag(value: bool) -> &'static str {
    if value {
        "available"
    } else {
        "unavailable"
    }
}

fn capability_status_text(status: platform_host::CapabilityStatus) -> &'static str {
    match status {
        platform_host::CapabilityStatus::Available => "available",
        platform_host::CapabilityStatus::RequiresUserActivation => "activation required",
        platform_host::CapabilityStatus::Unavailable => "unavailable",
    }
}

fn format_capabilities(capabilities: &[AppCapability]) -> String {
    capabilities
        .iter()
        .map(|capability| match capability {
            AppCapability::Window => "window",
            AppCapability::State => "state",
            AppCapability::Config => "config",
            AppCapability::Theme => "theme",
            AppCapability::Wallpaper => "wallpaper",
            AppCapability::Notifications => "notifications",
            AppCapability::Ipc => "ipc",
            AppCapability::ExternalUrl => "external-url",
            AppCapability::Commands => "commands",
        })
        .collect::<Vec<_>>()
        .join(", ")
}
