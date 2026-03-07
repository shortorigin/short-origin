#![allow(clippy::clone_on_copy)]

use super::*;
use crate::app_runtime::ensure_window_session;
use crate::apps;
use crate::shell;
use desktop_app_contract::{AppMountContext, AppServices, ApplicationId, CapabilitySet};
use leptos::ev::MouseEvent;
use sdk_rs::{InstitutionalPlatformClientV1, ReleasedUiAppV1};
use system_ui::components::{
    WindowControls as SystemWindowControls, WindowFrame as SystemWindowFrame,
    WindowTitleBar as SystemWindowTitleBar,
};
use system_ui::primitives::{
    Icon, IconName, IconSize, WindowBody as SystemWindowBody,
    WindowControlButton as SystemWindowControlButton, WindowTitle as SystemWindowTitle,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
fn try_set_pointer_capture(ev: &web_sys::PointerEvent) {
    if let Some(target) = ev.current_target() {
        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
            let _ = element.set_pointer_capture(ev.pointer_id());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn try_set_pointer_capture(_: &web_sys::PointerEvent) {}

#[component]
pub(super) fn DesktopWindow(window_id: WindowId) -> impl IntoView {
    let runtime = use_desktop_runtime();

    let window = Signal::derive(move || {
        runtime
            .state
            .get()
            .windows
            .into_iter()
            .find(|w| w.id == window_id)
    });

    let focus = move |_| {
        let should_focus = window
            .get()
            .map(|w| !w.is_focused || w.minimized)
            .unwrap_or(false);
        if should_focus {
            runtime.dispatch_action(DesktopAction::FocusWindow { window_id });
        }
    };
    let minimize = move |_| runtime.dispatch_action(DesktopAction::MinimizeWindow { window_id });
    let close = move |_| runtime.dispatch_action(DesktopAction::CloseWindow { window_id });
    let toggle_maximize = move |_| {
        if let Some(win) = window.get() {
            if win.maximized {
                runtime.dispatch_action(DesktopAction::RestoreWindow { window_id });
            } else if win.flags.maximizable {
                runtime.dispatch_action(DesktopAction::MaximizeWindow {
                    window_id,
                    viewport: runtime
                        .host
                        .get_value()
                        .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
                });
            }
        }
    };
    let begin_move = move |ev: web_sys::PointerEvent| {
        if ev.pointer_type() == "mouse" && ev.button() != 0 {
            return;
        }
        if ev.pointer_type() != "mouse" && !ev.is_primary() {
            return;
        }
        try_set_pointer_capture(&ev);
        if ev.button() != 0 {
            return;
        }
        ev.prevent_default();
        ev.stop_propagation();
        runtime.dispatch_action(DesktopAction::BeginMove {
            window_id,
            pointer: pointer_from_pointer_event(&ev),
        });
    };
    let titlebar_double_click = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        ev.stop_propagation();
        if let Some(win) = window.get() {
            if win.flags.maximizable {
                if win.maximized {
                    runtime.dispatch_action(DesktopAction::RestoreWindow { window_id });
                } else {
                    runtime.dispatch_action(DesktopAction::MaximizeWindow {
                        window_id,
                        viewport: runtime
                            .host
                            .get_value()
                            .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
                    });
                }
            }
        }
    };

    view! {
        <Show when=move || window.get().is_some() fallback=|| ()>
            <SystemWindowFrame
                style=Signal::derive(move || {
                    let win = window.get().expect("window exists while shown");
                    format!(
                        "left:{}px;top:{}px;width:{}px;height:{}px;z-index:{};",
                        win.rect.x, win.rect.y, win.rect.w, win.rect.h, win.z_index
                    )
                })
                on_pointerdown=Callback::new(focus)
                focused=Signal::derive(move || {
                    window.get().map(|win| win.is_focused).unwrap_or(false)
                })
                minimized=Signal::derive(move || {
                    window.get().map(|win| win.minimized).unwrap_or(false)
                })
                maximized=Signal::derive(move || {
                    window.get().map(|win| win.maximized).unwrap_or(false)
                })
                aria_label=Signal::derive(move || {
                    window
                        .get()
                        .map(|win| win.title)
                        .unwrap_or_default()
                })
            >
                <SystemWindowTitleBar
                    on_pointerdown=Callback::new(begin_move)
                    on_dblclick=Callback::new(titlebar_double_click)
                >
                    <SystemWindowTitle>
                        <span aria-hidden="true">
                            <Icon
                                icon={{
                                    let app_id = window
                                        .get_untracked()
                                        .expect("window exists while shown")
                                        .app_id;
                                    app_icon_name(&app_id)
                                }}
                                size=IconSize::Sm
                            />
                        </span>
                        <span>
                            {move || {
                                window
                                    .get()
                                    .map(|win| win.title)
                                .unwrap_or_default()
                            }}
                        </span>
                    </SystemWindowTitle>
                    <SystemWindowControls>
                        <SystemWindowControlButton
                            disabled=Signal::derive(move || {
                                !window
                                    .get()
                                    .expect("window exists while shown")
                                    .flags
                                    .minimizable
                            })
                            aria_label="Minimize window"
                            on_pointerdown=Callback::new(move |ev: web_sys::PointerEvent| {
                                ev.prevent_default();
                                ev.stop_propagation();
                            })
                            on_mousedown=Callback::new(move |ev: MouseEvent| stop_mouse_event(&ev))
                            on_click=Callback::new(move |ev| {
                                stop_mouse_event(&ev);
                                minimize(ev);
                            })
                        >
                            <Icon icon=IconName::WindowMinimize size=IconSize::Xs />
                        </SystemWindowControlButton>
                        <SystemWindowControlButton
                            disabled=Signal::derive(move || {
                                !window
                                    .get()
                                    .expect("window exists while shown")
                                    .flags
                                    .maximizable
                            })
                            aria_label=Signal::derive(move || {
                                if window
                                    .get()
                                    .expect("window exists while shown")
                                    .maximized
                                {
                                    "Restore window".to_string()
                                } else {
                                    "Maximize window".to_string()
                                }
                            })
                            on_pointerdown=Callback::new(move |ev: web_sys::PointerEvent| {
                                ev.prevent_default();
                                ev.stop_propagation();
                            })
                            on_mousedown=Callback::new(move |ev: MouseEvent| stop_mouse_event(&ev))
                            on_click=Callback::new(move |ev| {
                                stop_mouse_event(&ev);
                                toggle_maximize(ev);
                            })
                        >
                            {move || {
                                if window
                                    .get()
                                    .expect("window exists while shown")
                                    .maximized
                                {
                                    view! { <Icon icon=IconName::WindowRestore size=IconSize::Xs /> }
                                } else {
                                    view! { <Icon icon=IconName::WindowMaximize size=IconSize::Xs /> }
                                }
                            }}
                        </SystemWindowControlButton>
                        <SystemWindowControlButton
                            aria_label="Close window"
                            on_pointerdown=Callback::new(move |ev: web_sys::PointerEvent| {
                                ev.prevent_default();
                                ev.stop_propagation();
                            })
                            on_mousedown=Callback::new(move |ev: MouseEvent| stop_mouse_event(&ev))
                            on_click=Callback::new(move |ev| {
                                stop_mouse_event(&ev);
                                close(ev);
                            })
                        >
                            <Icon icon=IconName::Dismiss size=IconSize::Xs />
                        </SystemWindowControlButton>
                    </SystemWindowControls>
                </SystemWindowTitleBar>
                <SystemWindowBody>
                    <ManagedWindowBody window_id=window_id />
                </SystemWindowBody>
                <Show
                    when=move || {
                        window
                            .get()
                            .map(|w| w.flags.resizable && !w.maximized)
                            .unwrap_or(false)
                    }
                    fallback=|| ()
                >
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::North />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::South />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::East />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::West />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::NorthEast />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::NorthWest />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::SouthEast />
                    <WindowResizeHandle window_id=window_id edge=ResizeEdge::SouthWest />
                </Show>
            </SystemWindowFrame>
        </Show>
    }
}

#[component]
fn WindowResizeHandle(window_id: WindowId, edge: ResizeEdge) -> impl IntoView {
    let runtime = use_desktop_runtime();

    let on_pointerdown = move |ev: web_sys::PointerEvent| {
        if ev.pointer_type() == "mouse" && ev.button() != 0 {
            return;
        }
        if ev.pointer_type() != "mouse" && !ev.is_primary() {
            return;
        }
        try_set_pointer_capture(&ev);
        ev.prevent_default();
        ev.stop_propagation();
        runtime.dispatch_action(DesktopAction::BeginResize {
            window_id,
            edge,
            pointer: pointer_from_pointer_event(&ev),
            viewport: runtime
                .host
                .get_value()
                .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
        });
    };

    view! {
        <div
            aria-hidden="true"
            data-ui-primitive="true"
            data-ui-kind="resize-handle"
            data-ui-slot=resize_edge_class(edge)
            on:pointerdown=on_pointerdown
        />
    }
}

#[component]
fn ManagedWindowBody(window_id: WindowId) -> impl IntoView {
    let runtime = use_desktop_runtime();
    let state = runtime.state;
    let session = ensure_window_session(runtime.app_runtime, window_id);
    let lifecycle = session.lifecycle.read_only();
    let inbox = session.inbox;
    let theme_high_contrast = create_rw_signal(runtime.state.get_untracked().theme.high_contrast);
    let theme_reduced_motion = create_rw_signal(runtime.state.get_untracked().theme.reduced_motion);
    let wallpaper_current = create_rw_signal(runtime.state.get_untracked().wallpaper);
    let wallpaper_preview = create_rw_signal(runtime.state.get_untracked().wallpaper_preview);
    let wallpaper_library = create_rw_signal(runtime.state.get_untracked().wallpaper_library);
    let terminal_history = create_rw_signal(runtime.state.get_untracked().terminal_history);
    create_effect(move |_| {
        let desktop = runtime.state.get();
        theme_high_contrast.set(desktop.theme.high_contrast);
        theme_reduced_motion.set(desktop.theme.reduced_motion);
        wallpaper_current.set(desktop.wallpaper);
        wallpaper_preview.set(desktop.wallpaper_preview);
        wallpaper_library.set(desktop.wallpaper_library);
        terminal_history.set(desktop.terminal_history);
    });
    let command_sender = Callback::new(move |command| {
        spawn_local(async move {
            runtime.dispatch_action(DesktopAction::HandleAppCommand { window_id, command });
        });
    });
    let app_id = state
        .get_untracked()
        .windows
        .iter()
        .find(|w| w.id == window_id)
        .map(|w| w.app_id.clone())
        .expect("window app id");
    let capabilities = create_rw_signal(CapabilitySet::new(
        apps::app_requested_capabilities_by_id(&app_id).to_vec(),
        runtime.host.get_value().host_capabilities(),
    ));
    let platform_dashboard = create_rw_signal(
        InstitutionalPlatformClientV1 {
            client_name: "origin-shell".to_string(),
            supported_services: Vec::new(),
            supported_workflows: Vec::new(),
            lattice_config: None,
        }
        .dashboard_snapshot(
            apps::app_registry()
                .iter()
                .filter(|entry| entry.show_in_launcher || entry.show_on_desktop)
                .map(|entry| ReleasedUiAppV1 {
                    app_id: entry.app_id.to_string(),
                    display_name: entry.launcher_label.to_string(),
                    desktop_enabled: entry.show_on_desktop,
                })
                .collect(),
            true,
        ),
    );
    let services = store_value(AppServices::new(
        command_sender,
        capabilities.get_untracked(),
        runtime.host.get_value().app_state_store(),
        runtime.host.get_value().prefs_store(),
        runtime.host.get_value().explorer_fs_service(),
        runtime.host.get_value().content_cache(),
        theme_high_contrast.read_only(),
        theme_reduced_motion.read_only(),
        wallpaper_current.read_only(),
        wallpaper_preview.read_only(),
        wallpaper_library.read_only(),
        platform_dashboard.read_only(),
        shell::build_command_service(
            runtime.clone(),
            app_id.clone(),
            window_id,
            terminal_history.read_only(),
        ),
    ));
    let mounted_window = state
        .get_untracked()
        .windows
        .into_iter()
        .find(|window| window.id == window_id)
        .expect("window exists while body is mounted");
    let contents = view! {
        <MountedManagedApp
            app_id=mounted_window.app_id.clone()
            context=AppMountContext {
                app_id: mounted_window.app_id.clone(),
                window_id: mounted_window.id.0,
                launch_params: mounted_window.launch_params.clone(),
                restored_state: mounted_window.app_state.clone(),
                lifecycle,
                inbox,
                capabilities: capabilities.read_only(),
                services: services.get_value(),
            }
        />
    };

    view! {
        <div data-ui-slot="window-body-content">
            {contents}
        </div>
    }
}

#[component]
fn MountedManagedApp(app_id: ApplicationId, context: AppMountContext) -> impl IntoView {
    apps::app_module_by_id(&app_id).mount(context)
}
