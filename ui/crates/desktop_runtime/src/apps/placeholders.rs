//! Compatibility placeholder apps for legacy shell identifiers that are no longer shipped.

use desktop_app_contract::AppMountContext;
use leptos::*;
use serde::{Deserialize, Serialize};
use system_ui::prelude::*;

/// Mounts the legacy calculator placeholder.
pub(super) fn mount_calculator_placeholder_app(context: AppMountContext) -> View {
    compatibility_placeholder(
        context,
        "Calculator",
        "The calculator sample app is not part of the Short Origin shell. Use Control Center for institutional workspace status instead.",
    )
}

/// Mounts the legacy explorer placeholder.
pub(super) fn mount_explorer_placeholder_app(context: AppMountContext) -> View {
    compatibility_placeholder(
        context,
        "Explorer",
        "Filesystem browsing remains a host capability, but the standalone explorer app is not shipped in this shell profile.",
    )
}

/// Mounts the legacy notepad placeholder.
pub(super) fn mount_notepad_placeholder_app(context: AppMountContext) -> View {
    compatibility_placeholder(
        context,
        "Notes",
        "The notepad sample app is not shipped in the institutional shell. Capture operational notes through the governed platform surfaces instead.",
    )
}

/// Mounts the legacy dial-up placeholder.
pub(super) fn mount_dialup_placeholder_app(context: AppMountContext) -> View {
    compatibility_placeholder(
        context,
        "Dial-up",
        "Legacy connectivity simulation is retired. Network posture is represented through host capability and runtime status surfaces.",
    )
}

/// Mounts the legacy UI showcase placeholder.
pub(super) fn mount_ui_showcase_placeholder_app(context: AppMountContext) -> View {
    compatibility_placeholder(
        context,
        "UI Showcase",
        "The design-system showcase is intentionally excluded from the integrated product shell. Shared primitives remain available to shipped apps only.",
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PlaceholderState {
    acknowledged: bool,
}

fn compatibility_placeholder(
    context: AppMountContext,
    title: &'static str,
    description: &'static str,
) -> View {
    view! {
        <CompatibilityPlaceholderApp context=context title=title description=description />
    }
    .into_view()
}

#[component]
fn CompatibilityPlaceholderApp(
    context: AppMountContext,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    let state = create_rw_signal(PlaceholderState {
        acknowledged: false,
    });
    hydrate_persisted_state(&context, state);

    view! {
        <AppShell>
            <Panel variant=SurfaceVariant::Standard>
                <Stack gap=LayoutGap::Md>
                    <Heading>{title}</Heading>
                    <Text tone=TextTone::Secondary>{description}</Text>
                    <Surface variant=SurfaceVariant::Inset elevation=Elevation::Inset>
                        <Stack gap=LayoutGap::Sm>
                            <Text role=TextRole::Label>"Supported shipped apps"</Text>
                            <Text>"Control Center, Terminal, and System Settings."</Text>
                        </Stack>
                    </Surface>
                    <ToggleRow
                        title="Acknowledge compatibility mode"
                        description="Persist this state per window so the shell can keep legacy launches non-destructive."
                        checked=Signal::derive(move || state.get().acknowledged)
                    >
                        <Switch
                            aria_label="Acknowledge compatibility mode"
                            checked=Signal::derive(move || state.get().acknowledged)
                            on_toggle=Callback::new(move |checked| {
                                state.update(|state| state.acknowledged = checked);
                            })
                        />
                    </ToggleRow>
                </Stack>
            </Panel>
            <StatusBar>
                <StatusBarItem>{move || format!("App: {title}")}</StatusBarItem>
                <StatusBarItem>
                    {move || if state.get().acknowledged {
                        "Compatibility mode acknowledged".to_string()
                    } else {
                        "Compatibility mode active".to_string()
                    }}
                </StatusBarItem>
            </StatusBar>
        </AppShell>
    }
}

fn hydrate_persisted_state<T>(context: &AppMountContext, state: RwSignal<T>)
where
    T: Clone + for<'de> Deserialize<'de> + Serialize + 'static,
{
    let restored_state = context.restored_state.clone();
    let services = context.services.clone();
    let last_saved = create_rw_signal::<Option<String>>(None);
    let hydrated = create_rw_signal(false);

    create_effect(move |_| {
        if restored_state.is_object() {
            if let Ok(restored) = serde_json::from_value::<T>(restored_state.clone()) {
                let serialized = serde_json::to_string(&restored).ok();
                state.set(restored);
                last_saved.set(serialized);
            }
        }
    });

    hydrated.set(true);

    create_effect(move |_| {
        if !hydrated.get() {
            return;
        }

        let snapshot = state.get();
        let serialized = match serde_json::to_string(&snapshot) {
            Ok(raw) => raw,
            Err(err) => {
                logging::warn!("compatibility placeholder serialize failed: {err}");
                return;
            }
        };

        if last_saved.get().as_deref() == Some(serialized.as_str()) {
            return;
        }
        last_saved.set(Some(serialized));

        if let Ok(value) = serde_json::to_value(snapshot) {
            services.state.persist_window_state(value);
        }
    });
}
