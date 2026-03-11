//! Built-in System Settings desktop app for shell appearance and accessibility preferences.
//!
//! The app consumes the injected v2 service surface from [`desktop_app_contract::AppServices`]
//! so theme and accessibility controls stay synchronized with the desktop runtime.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use desktop_app_contract::AppServices;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_ui::components::{AppShell, StatusBar, StatusBarItem, ToggleRow, Toolbar};
use system_ui::primitives::{
    ButtonVariant, CheckboxField, Heading, LayoutGap, Panel, Stack, Surface, SurfaceVariant, Text,
    TextRole, TextTone,
};

const BASELINE_STYLE_ID: &str = "origin-baseline";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum SettingsSection {
    Appearance,
    Accessibility,
}

impl SettingsSection {
    fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Accessibility => "Accessibility",
        }
    }

    fn from_launch_param(raw: &str) -> Option<Self> {
        match raw.trim() {
            "appearance" => Some(Self::Appearance),
            "accessibility" => Some(Self::Accessibility),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SettingsAppState {
    active_section: SettingsSection,
}

impl Default for SettingsAppState {
    fn default() -> Self {
        Self {
            active_section: SettingsSection::Appearance,
        }
    }
}

#[component]
/// Settings app window contents.
pub fn SettingsApp(
    /// Legacy launch params, retained for compatibility.
    launch_params: Value,
    /// Manager-restored app state payload.
    restored_state: Option<Value>,
    /// Injected desktop services bundle.
    services: Option<AppServices>,
) -> impl IntoView {
    let services = services.expect("settings requires app services");
    let settings_state = RwSignal::new(SettingsAppState::default());

    if let Some(restored_state) = restored_state
        && let Ok(restored) = serde_json::from_value::<SettingsAppState>(restored_state)
    {
        settings_state.set(restored);
    }

    if let Some(section) = launch_params
        .get("section")
        .and_then(Value::as_str)
        .and_then(SettingsSection::from_launch_param)
    {
        settings_state.update(|state| state.active_section = section);
    }

    let state_service = services.state.clone();
    Effect::new(move |_| {
        if let Ok(serialized) = serde_json::to_value(settings_state.get()) {
            state_service.persist_window_state(serialized);
        }
    });

    let theme_high_contrast = Signal::derive({
        let services = services.clone();
        move || services.theme.high_contrast.get()
    });
    let theme_dark_mode = Signal::derive({
        let services = services.clone();
        move || services.theme.dark_mode.get()
    });
    let theme_reduced_motion = Signal::derive({
        let services = services.clone();
        move || services.theme.reduced_motion.get()
    });

    view! {
        <AppShell>
            <div style="display:grid;grid-template-columns:minmax(220px, 280px) minmax(0, 1fr);gap:var(--origin-space-section);align-items:start;">
                <Panel variant=SurfaceVariant::Muted>
                    <Stack gap=LayoutGap::Md>
                        <Heading role=TextRole::Title>"Settings"</Heading>
                        <Text tone=TextTone::Secondary>
                            "Appearance and accessibility controls for the Origin shell."
                        </Text>
                        <Toolbar role="tablist" aria_label="Settings sections" layout_class="settings-sidebar">
                            <For
                                each=move || [SettingsSection::Appearance, SettingsSection::Accessibility]
                                key=|section| *section as u8
                                let:section
                            >
                                <system_ui::components::Button
                                    variant=ButtonVariant::Quiet
                                    selected=Signal::derive(move || settings_state.get().active_section == section)
                                    role="tab"
                                    on_click=Callback::new(move |_| {
                                        settings_state.update(|state| state.active_section = section);
                                    })
                                >
                                    {section.label()}
                                </system_ui::components::Button>
                            </For>
                        </Toolbar>
                    </Stack>
                </Panel>

                <div>
                    <Show when=move || settings_state.get().active_section == SettingsSection::Appearance fallback=|| ()>
                        <Surface variant=SurfaceVariant::Muted>
                            <Stack gap=LayoutGap::Md>
                                <Panel variant=SurfaceVariant::Standard>
                                    <Heading role=TextRole::Title>"Shell appearance"</Heading>
                                    <Text tone=TextTone::Secondary>
                                        "The desktop background is fixed to the Short Origin logo and is no longer configurable."
                                    </Text>
                                </Panel>

                                <Panel variant=SurfaceVariant::Standard>
                                    <Heading role=TextRole::Title>"Theme family"</Heading>
                                    <ToggleRow
                                        title="Dark appearance"
                                        description="Switch the shell between the light and dark token families."
                                        checked=theme_dark_mode
                                    >
                                        <CheckboxField
                                            aria_label="Dark appearance"
                                            checked=theme_dark_mode
                                            on_change=Callback::new(move |ev| {
                                                services.theme.set_dark_mode(event_target_checked(&ev))
                                            })
                                        />
                                    </ToggleRow>
                                </Panel>

                                <Panel variant=SurfaceVariant::Standard>
                                    <Heading role=TextRole::Title>"Baseline"</Heading>
                                    <Text role=TextRole::Label>"Active style"</Text>
                                    <Text>{BASELINE_STYLE_ID}</Text>
                                </Panel>
                            </Stack>
                        </Surface>
                    </Show>

                    <Show when=move || settings_state.get().active_section == SettingsSection::Accessibility fallback=|| ()>
                        <Surface variant=SurfaceVariant::Muted>
                            <Stack gap=LayoutGap::Md>
                                <Panel variant=SurfaceVariant::Standard>
                                    <Heading role=TextRole::Title>"Visibility"</Heading>
                                    <ToggleRow
                                        title="High contrast"
                                        description="Increase separation between borders, text, and focus states."
                                        checked=theme_high_contrast
                                    >
                                        <CheckboxField
                                            aria_label="High contrast"
                                            checked=theme_high_contrast
                                            on_change=Callback::new(move |ev| {
                                                services.theme.set_high_contrast(event_target_checked(&ev))
                                            })
                                        />
                                    </ToggleRow>
                                </Panel>

                                <Panel variant=SurfaceVariant::Standard>
                                    <Heading role=TextRole::Title>"Motion"</Heading>
                                    <ToggleRow
                                        title="Reduced motion"
                                        description="Shorten non-essential motion across shell surfaces."
                                        checked=theme_reduced_motion
                                    >
                                        <CheckboxField
                                            aria_label="Reduced motion"
                                            checked=theme_reduced_motion
                                            on_change=Callback::new(move |ev| {
                                                services.theme.set_reduced_motion(event_target_checked(&ev))
                                            })
                                        />
                                    </ToggleRow>
                                </Panel>
                            </Stack>
                        </Surface>
                    </Show>
                </div>
            </div>

            <StatusBar>
                <StatusBarItem>{format!("Style: {BASELINE_STYLE_ID}")}</StatusBarItem>
                <StatusBarItem>
                    {move || if theme_dark_mode.get() { "Theme: Dark" } else { "Theme: Light" }}
                </StatusBarItem>
                <StatusBarItem>
                    {move || {
                        if theme_high_contrast.get() {
                            "Contrast: High"
                        } else {
                            "Contrast: Standard"
                        }
                    }}
                </StatusBarItem>
                <StatusBarItem>
                    {move || {
                        if theme_reduced_motion.get() {
                            "Motion: Reduced"
                        } else {
                            "Motion: Standard"
                        }
                    }}
                </StatusBarItem>
            </StatusBar>
        </AppShell>
    }
}
