//! Runtime provider and context wiring for the desktop shell.
//!
//! This module owns the long-lived reducer container, runtime effect queue, app-session state,
//! host bootstrap wiring, and built-in shell registration. UI composition stays in
//! [`crate::components`].
#![allow(clippy::clone_on_copy)]

use leptos::callback::Callable;
use leptos::logging;
use leptos::prelude::*;
use platform_host::HostServices;

use crate::{
    app_runtime::{AppRuntimeState, sync_runtime_sessions},
    apps, effect_executor,
    host::DesktopHostContext,
    model::{DeepLinkState, DesktopState, InteractionState},
    reducer::{DesktopAction, RuntimeEffect, reduce_desktop},
    shell,
};

#[derive(Clone, Copy)]
/// Leptos context for reading desktop runtime state and dispatching [`DesktopAction`] values.
pub struct DesktopRuntimeContext {
    /// Host service bundle for executing runtime side effects and environment queries.
    pub host: StoredValue<DesktopHostContext, LocalStorage>,
    /// Long-lived reactive owner for runtime-managed resources that must outlive transient app views.
    pub owner: StoredValue<Owner, LocalStorage>,
    /// Reactive desktop state signal.
    pub state: RwSignal<DesktopState>,
    /// Reactive pointer/drag/resize interaction state signal.
    pub interaction: RwSignal<InteractionState>,
    /// Queue of runtime effects emitted by the reducer and processed by the shell.
    pub effects: RwSignal<Vec<RuntimeEffect>>,
    /// Runtime app-session and pub/sub state.
    pub app_runtime: RwSignal<AppRuntimeState>,
    /// Reducer dispatch callback.
    pub dispatch: Callback<DesktopAction>,
    /// Shared shell engine and command registry.
    pub shell_engine: StoredValue<system_shell::ShellEngine, LocalStorage>,
}

impl DesktopRuntimeContext {
    /// Dispatches a reducer action through the runtime context callback.
    pub fn dispatch_action(&self, action: DesktopAction) {
        self.dispatch.run(action);
    }
}

fn install_runtime_orchestration(
    runtime: DesktopRuntimeContext,
    initial_deep_link: Option<DeepLinkState>,
) {
    runtime.host.with_value(|host| {
        host.install_boot_hydration(runtime.dispatch, initial_deep_link);
    });
    std::mem::forget(shell::register_builtin_commands(runtime));
    effect_executor::install(runtime);
}

#[component]
/// Provides [`DesktopRuntimeContext`] to descendant components and boots persisted state.
pub fn DesktopProvider(
    /// Injected browser or desktop host bundle assembled by the entry layer.
    host_services: HostServices,
    /// URL intent captured by the browser entrypoint before runtime boot begins.
    initial_deep_link: Option<DeepLinkState>,
    children: Children,
) -> impl IntoView {
    let host = StoredValue::new_local(DesktopHostContext::new(host_services));
    let owner = StoredValue::new_local(Owner::current().expect("DesktopProvider owner"));
    let state = RwSignal::new(DesktopState::default());
    let interaction = RwSignal::new(InteractionState::default());
    let effects = RwSignal::new(Vec::<RuntimeEffect>::new());
    let app_runtime = RwSignal::new(AppRuntimeState::default());
    let shell_engine = StoredValue::new_local(system_shell::ShellEngine::new());

    let dispatch = Callback::new(move |action: DesktopAction| {
        let mut desktop = state.get_untracked();
        let mut ui = interaction.get_untracked();
        let previous_desktop = desktop.clone();
        let previous_ui = ui.clone();

        match reduce_desktop(&mut desktop, &mut ui, action) {
            Ok(new_effects) => {
                let windows_changed = desktop.windows != previous_desktop.windows;
                if windows_changed {
                    sync_runtime_sessions(app_runtime, &desktop.windows);
                }
                if desktop != previous_desktop {
                    state.set(desktop);
                }
                if ui != previous_ui {
                    interaction.set(ui);
                }
                if !new_effects.is_empty() {
                    let mut queue = effects.get_untracked();
                    queue.extend(new_effects);
                    effects.set(queue);
                }
            }
            Err(err) => {
                if let crate::reducer::ReducerError::CapabilityDenied {
                    window_id,
                    diagnostic_event,
                    ..
                } = &err
                {
                    let mut queue = effects.get_untracked();
                    queue.push(RuntimeEffect::DeliverAppEvent {
                        window_id: *window_id,
                        event: diagnostic_event.as_ref().clone(),
                    });
                    effects.set(queue);
                }
                logging::warn!("desktop reducer error: {err}");
            }
        }
    });

    let runtime = DesktopRuntimeContext {
        host,
        owner,
        state,
        interaction,
        effects,
        app_runtime,
        dispatch,
        shell_engine,
    };

    provide_context(runtime.clone());

    install_runtime_orchestration(runtime, initial_deep_link);

    children().into_view()
}

/// Returns the current [`DesktopRuntimeContext`].
///
/// # Panics
///
/// Panics if called outside [`DesktopProvider`].
pub fn use_desktop_runtime() -> DesktopRuntimeContext {
    use_context::<DesktopRuntimeContext>().expect("DesktopRuntimeContext not provided")
}

/// Opens System Settings by focusing an existing window or creating a new one.
pub(crate) fn open_system_settings(runtime: &DesktopRuntimeContext, taskbar_height_px: i32) {
    let desktop = runtime.state.get_untracked();
    let app_id = apps::settings_application_id();
    if let Some(window_id) = crate::components::preferred_window_for_app(&desktop, &app_id) {
        crate::components::focus_or_unminimize_window(runtime, &desktop, window_id);
        return;
    }

    let viewport = runtime
        .host
        .get_value()
        .desktop_viewport_rect(taskbar_height_px);
    runtime.dispatch_action(DesktopAction::OpenWindow(
        apps::default_open_request_by_id(&app_id, Some(viewport))
            .expect("system settings app exists"),
    ));
}
