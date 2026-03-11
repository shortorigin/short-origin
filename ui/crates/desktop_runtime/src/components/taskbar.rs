use super::*;
use leptos::ev;
use leptos::ev::MouseEvent;
use leptos::prelude::GetValue;
use system_ui::components::{
    Dock as SystemDock, DockButton as SystemDockButton, DockSection as SystemDockSection,
};
use system_ui::primitives::{Icon, IconName, IconSize};

#[component]
pub(super) fn Taskbar() -> impl IntoView {
    let runtime = use_desktop_runtime();
    let state = runtime.state;

    let viewport_width = RwSignal::new(
        runtime
            .host
            .get_value()
            .desktop_viewport_rect(TASKBAR_HEIGHT_PX)
            .w,
    );
    let clock_now = RwSignal::new(TaskbarClockSnapshot::now());
    let selected_running_window = RwSignal::new(None::<WindowId>);
    let window_context_menu = RwSignal::new(None::<TaskbarWindowContextMenuState>);
    let overflow_menu_open = RwSignal::new(false);
    let start_menu_was_open = RwSignal::new(false);
    let overflow_menu_was_open = RwSignal::new(false);
    let window_menu_was_open = RwSignal::new(false);
    let taskbar_layout = Memo::new(move |_| {
        let desktop = state.get();
        compute_taskbar_layout(
            viewport_width.get(),
            pinned_taskbar_apps().len(),
            ordered_taskbar_windows(&desktop).len(),
        )
    });
    let launcher_open = Signal::derive(move || state.get().panels.launcher_open);
    let control_center_open = Signal::derive(move || state.get().panels.control_center_open);
    let notification_center_open =
        Signal::derive(move || state.get().panels.notification_center_open);
    let theme_mode = Signal::derive(move || state.get().theme.mode);
    let high_contrast = Signal::derive(move || state.get().theme.high_contrast);
    let reduced_motion = Signal::derive(move || state.get().theme.reduced_motion);
    let open_window_count = Signal::derive(move || state.get().windows.len());
    let notifications = Signal::derive(move || state.get().notifications.clone());
    let unread_notification_count = Signal::derive(move || {
        state
            .get()
            .notifications
            .iter()
            .filter(|notification| notification.unread)
            .count()
    });
    let close_launcher =
        Callback::new(move |_| runtime.dispatch_action(DesktopAction::CloseStartMenu));
    let close_control_center =
        Callback::new(move |_| runtime.dispatch_action(DesktopAction::CloseControlCenter));
    let close_notification_center = Callback::new(move |_| {
        runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
    });
    let activate_launcher_app = Callback::new(move |app_id: ApplicationId| {
        window_context_menu.set(None);
        overflow_menu_open.set(false);
        runtime.dispatch_action(DesktopAction::CloseControlCenter);
        runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
        runtime.dispatch_action(DesktopAction::ActivateApp {
            app_id,
            viewport: Some(
                runtime
                    .host
                    .get_value()
                    .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
            ),
        });
    });
    let open_settings = Callback::new(move |_| {
        runtime.dispatch_action(DesktopAction::CloseControlCenter);
        runtime.dispatch_action(DesktopAction::OpenWindow(browser_e2e_window_request(
            apps::settings_application_id(),
            runtime,
            Value::Null,
        )));
    });
    let toggle_theme_mode = Callback::new(move |_| {
        let next = if matches!(
            runtime.state.get_untracked().theme.mode,
            crate::model::ThemeMode::Dark
        ) {
            crate::model::ThemeMode::Light
        } else {
            crate::model::ThemeMode::Dark
        };
        runtime.dispatch_action(DesktopAction::SetThemeMode { mode: next });
    });
    let toggle_high_contrast = Callback::new(move |_| {
        let enabled = runtime.state.get_untracked().theme.high_contrast;
        runtime.dispatch_action(DesktopAction::SetHighContrast { enabled: !enabled });
    });
    let toggle_reduced_motion = Callback::new(move |_| {
        let enabled = runtime.state.get_untracked().theme.reduced_motion;
        runtime.dispatch_action(DesktopAction::SetReducedMotion { enabled: !enabled });
    });
    let clear_notifications =
        Callback::new(move |_| runtime.dispatch_action(DesktopAction::ClearNotifications));
    let dismiss_notification = Callback::new(move |id| {
        runtime.dispatch_action(DesktopAction::DismissNotification { id });
    });

    let resize_listener = window_event_listener(ev::resize, move |_| {
        viewport_width.set(
            runtime
                .host
                .get_value()
                .desktop_viewport_rect(TASKBAR_HEIGHT_PX)
                .w,
        );
    });
    on_cleanup(move || resize_listener.remove());

    if let Ok(interval) = set_interval_with_handle(
        move || clock_now.set(TaskbarClockSnapshot::now()),
        Duration::from_secs(1),
    ) {
        on_cleanup(move || interval.clear());
    }

    let outside_click_listener = window_event_listener(ev::mousedown, move |_| {
        let had_window_menu = window_context_menu.get_untracked().is_some();
        let had_overflow_menu = overflow_menu_open.get_untracked();
        let had_launcher = runtime.state.get_untracked().panels.launcher_open;

        if had_window_menu {
            window_context_menu.set(None);
        }
        if had_overflow_menu {
            overflow_menu_open.set(false);
        }

        if had_launcher {
            runtime.dispatch_action(DesktopAction::CloseStartMenu);
        }
        if runtime.state.get_untracked().panels.control_center_open {
            runtime.dispatch_action(DesktopAction::CloseControlCenter);
        }
        if runtime
            .state
            .get_untracked()
            .panels
            .notification_center_open
        {
            runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
        }
    });
    on_cleanup(move || outside_click_listener.remove());

    let global_shortcut_listener = window_event_listener(ev::keydown, move |ev| {
        if ev.default_prevented() {
            return;
        }
        if try_handle_taskbar_shortcuts(&runtime, window_context_menu, overflow_menu_open, &ev) {}
    });
    on_cleanup(move || global_shortcut_listener.remove());

    Effect::new(move |_| {
        let desktop = state.get();
        let running = ordered_taskbar_windows(&desktop);
        let focused = running
            .iter()
            .find(|win| win.is_focused && !win.minimized)
            .map(|win| win.id);

        selected_running_window.update(|selected| {
            let selected_exists = selected
                .and_then(|id| running.iter().find(|win| win.id == id))
                .is_some();
            if selected_exists {
                return;
            }
            *selected = focused.or_else(|| running.first().map(|win| win.id));
        });

        window_context_menu.update(|menu| {
            if let Some(current) = *menu
                && running.iter().all(|win| win.id != current.window_id)
            {
                *menu = None;
            }
        });
    });

    Effect::new(move |_| {
        let is_open = state.get().panels.launcher_open;
        let was_open = start_menu_was_open.get_untracked();
        if is_open && !was_open {
            start_menu_was_open.set(true);
            let _ = focus_first_menu_item("desktop-launcher-menu");
        } else if !is_open && was_open {
            start_menu_was_open.set(false);
        }
    });

    Effect::new(move |_| {
        let is_open = overflow_menu_open.get();
        let was_open = overflow_menu_was_open.get_untracked();
        if is_open && !was_open {
            overflow_menu_was_open.set(true);
            let _ = focus_first_menu_item("taskbar-overflow-menu");
        } else if !is_open && was_open {
            overflow_menu_was_open.set(false);
        }
    });

    Effect::new(move |_| {
        let is_open = window_context_menu.get().is_some();
        let was_open = window_menu_was_open.get_untracked();
        if is_open && !was_open {
            window_menu_was_open.set(true);
            let _ = focus_first_menu_item("taskbar-window-context-menu");
        } else if !is_open && was_open {
            window_menu_was_open.set(false);
        }
    });

    let on_taskbar_keydown = move |ev: web_sys::KeyboardEvent| {
        if try_handle_taskbar_shortcuts(&runtime, window_context_menu, overflow_menu_open, &ev) {
            return;
        }

        match ev.key().as_str() {
            "ArrowRight" => {
                let desktop = runtime.state.get_untracked();
                let running = ordered_taskbar_windows(&desktop);
                if let Some(next) = cycle_selected_running_window(
                    &running,
                    selected_running_window.get_untracked(),
                    1,
                ) {
                    ev.prevent_default();
                    selected_running_window.set(Some(next));
                }
            }
            "ArrowLeft" => {
                let desktop = runtime.state.get_untracked();
                let running = ordered_taskbar_windows(&desktop);
                if let Some(next) = cycle_selected_running_window(
                    &running,
                    selected_running_window.get_untracked(),
                    -1,
                ) {
                    ev.prevent_default();
                    selected_running_window.set(Some(next));
                }
            }
            "Home" => {
                let desktop = runtime.state.get_untracked();
                let running = ordered_taskbar_windows(&desktop);
                if let Some(first) = running.first() {
                    ev.prevent_default();
                    selected_running_window.set(Some(first.id));
                }
            }
            "End" => {
                let desktop = runtime.state.get_untracked();
                let running = ordered_taskbar_windows(&desktop);
                if let Some(last) = running.last() {
                    ev.prevent_default();
                    selected_running_window.set(Some(last.id));
                }
            }
            _ => {
                if is_activation_key(&ev) {
                    if let Some(window_id) = selected_running_window.get_untracked() {
                        ev.prevent_default();
                        ev.stop_propagation();
                        window_context_menu.set(None);
                        overflow_menu_open.set(false);
                        runtime.dispatch_action(DesktopAction::ToggleTaskbarWindow { window_id });
                    }
                } else if is_context_menu_shortcut(&ev)
                    && let Some(window_id) = selected_running_window.get_untracked()
                {
                    ev.prevent_default();
                    ev.stop_propagation();
                    window_context_menu.set(None);
                    overflow_menu_open.set(false);
                    runtime.dispatch_action(DesktopAction::CloseStartMenu);
                    runtime.dispatch_action(DesktopAction::CloseControlCenter);
                    runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                    let viewport = runtime
                        .host
                        .get_value()
                        .desktop_viewport_rect(TASKBAR_HEIGHT_PX);
                    let x = (viewport.w / 2).max(24);
                    let y = (viewport.h + TASKBAR_HEIGHT_PX - 180).max(24);
                    open_taskbar_window_context_menu(
                        runtime.host.get_value(),
                        window_context_menu,
                        window_id,
                        x,
                        y,
                    );
                }
            }
        }
    };

    view! {
        <>
            <div data-ui-slot="taskbar-overlay">
                <SystemDock
                    role="toolbar"
                    aria_label="Desktop dock"
                    aria_keyshortcuts="Ctrl+Escape Alt+1 Alt+2 Alt+3 Alt+4 Alt+5 Alt+6 Alt+7 Alt+8 Alt+9"
                    on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
                    on_keydown=Callback::new(on_taskbar_keydown)
                >
                    <SystemDockSection ui_slot="left">
                        <SystemDockButton
                            id="taskbar-start-button"
                            ui_slot="dock-launcher-button"
                            aria_label="Open application launcher"
                            aria_haspopup="menu"
                            aria_controls="desktop-launcher-menu"
                            aria_expanded=launcher_open
                            aria_keyshortcuts="Ctrl+Escape"
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                overflow_menu_open.set(false);
                                runtime.dispatch_action(DesktopAction::CloseControlCenter);
                                runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                                runtime.dispatch_action(DesktopAction::ToggleStartMenu);
                            })
                        >
                            <span aria-hidden="true">
                                <Icon icon=IconName::Launcher size=IconSize::Md />
                            </span>
                        </SystemDockButton>

                        <div role="group" aria-label="Pinned apps" data-ui-slot="dock-pinned-apps">
                            <For
                                each=move || pinned_taskbar_apps().to_vec()
                                key=|app_id| app_id.to_string()
                                let:app_id
                            >
                                {{
                                    let app_id_for_selected = app_id.clone();
                                    let app_id_for_pressed = app_id.clone();
                                    let app_id_for_title = app_id.clone();
                                    let app_id_for_aria = app_id.clone();
                                    let app_id_for_click = app_id.clone();
                                    let app_icon_name_value = app_icon_name(&app_id);
                                    let app_data_id = apps::app_icon_id_by_id(&app_id).to_string();
                                    view! {
                                        <SystemDockButton
                                            data_app=app_data_id.clone()
                                            title=Signal::derive(move || {
                                                let desktop = state.get();
                                                let status = pinned_taskbar_app_state(&desktop, &app_id_for_title);
                                                taskbar_pinned_aria_label(&app_id_for_title, status)
                                            })
                                            aria_label=Signal::derive(move || {
                                                let desktop = state.get();
                                                let status = pinned_taskbar_app_state(&desktop, &app_id_for_aria);
                                                taskbar_pinned_aria_label(&app_id_for_aria, status)
                                            })
                                            selected=Signal::derive(move || {
                                                let desktop = state.get();
                                                let app_state =
                                                    pinned_taskbar_app_state(&desktop, &app_id_for_selected);
                                                app_state.focused
                                            })
                                            pressed=Signal::derive(move || {
                                                let desktop = state.get();
                                                let app_state =
                                                    pinned_taskbar_app_state(&desktop, &app_id_for_pressed);
                                                app_state.running_count > 0 && !app_state.all_minimized
                                            })
                                            on_click=Callback::new(move |_| {
                                                window_context_menu.set(None);
                                                overflow_menu_open.set(false);
                                                runtime.dispatch_action(DesktopAction::CloseStartMenu);
                                                runtime.dispatch_action(DesktopAction::CloseControlCenter);
                                                runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                                                activate_pinned_taskbar_app(&runtime, app_id_for_click.clone());
                                            })
                                        >
                                            <span aria-hidden="true">
                                                <Icon icon=app_icon_name_value size=IconSize::Md />
                                            </span>
                                        </SystemDockButton>
                                    }
                                }}
                            </For>
                        </div>
                    </SystemDockSection>

                    <SystemDockSection
                        ui_slot="running"
                        role="group"
                        aria_label="Running windows"
                    >
                        <For
                            each=move || {
                                let desktop = state.get();
                                let layout = taskbar_layout.get();
                                ordered_taskbar_windows(&desktop)
                                    .into_iter()
                                    .take(layout.visible_running_count)
                                    .collect::<Vec<_>>()
                            }
                            key=|win| win.id.0
                            let:win
                        >
                            <SystemDockButton
                                id=taskbar_window_button_dom_id(win.id)
                                data_app=win.icon_id.clone()
                                aria_pressed=Signal::derive(move || win.is_focused && !win.minimized)
                                aria_label=taskbar_window_aria_label(&win)
                                title=taskbar_window_aria_label(&win)
                                selected=Signal::derive(move || {
                                    selected_running_window.get() == Some(win.id)
                                })
                                pressed=Signal::derive(move || win.is_focused && !win.minimized)
                                on_click=Callback::new(move |_| {
                                    selected_running_window.set(Some(win.id));
                                    window_context_menu.set(None);
                                    overflow_menu_open.set(false);
                                    runtime.dispatch_action(DesktopAction::CloseStartMenu);
                                    runtime.dispatch_action(DesktopAction::CloseControlCenter);
                                    runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                                    runtime.dispatch_action(DesktopAction::ToggleTaskbarWindow {
                                        window_id: win.id,
                                    });
                                })
                                on_contextmenu=Callback::new(move |ev: MouseEvent| {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    selected_running_window.set(Some(win.id));
                                    overflow_menu_open.set(false);
                                    runtime.dispatch_action(DesktopAction::CloseStartMenu);
                                    runtime.dispatch_action(DesktopAction::CloseControlCenter);
                                    runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                                    open_taskbar_window_context_menu(
                                        runtime.host.get_value(),
                                        window_context_menu,
                                        win.id,
                                        ev.client_x(),
                                        ev.client_y(),
                                    );
                                })
                            >
                                <span aria-hidden="true">
                                    <Icon icon=app_icon_name(&win.app_id) size=IconSize::Md />
                                </span>
                            </SystemDockButton>
                        </For>

                        <Show
                            when=move || {
                                let desktop = state.get();
                                ordered_taskbar_windows(&desktop).len()
                                    > taskbar_layout.get().visible_running_count
                            }
                            fallback=|| ()
                        >
                            <SystemDockButton
                                id="taskbar-overflow-button"
                                ui_slot="dock-overflow-button"
                                aria_haspopup="menu"
                                aria_controls="taskbar-overflow-menu"
                                aria_expanded=overflow_menu_open.read_only()
                                aria_label=Signal::derive(move || {
                                    let desktop = state.get();
                                    let hidden = ordered_taskbar_windows(&desktop)
                                        .len()
                                        .saturating_sub(taskbar_layout.get().visible_running_count);
                                    format!("Show {} more windows", hidden)
                                })
                                title=Signal::derive(move || {
                                    let desktop = state.get();
                                    let hidden = ordered_taskbar_windows(&desktop)
                                        .len()
                                        .saturating_sub(taskbar_layout.get().visible_running_count);
                                    format!("Show {} more windows", hidden)
                                })
                                on_click=Callback::new(move |_| {
                                    window_context_menu.set(None);
                                    runtime.dispatch_action(DesktopAction::CloseStartMenu);
                                    runtime.dispatch_action(DesktopAction::CloseControlCenter);
                                    runtime.dispatch_action(DesktopAction::CloseNotificationCenter);
                                    overflow_menu_open.update(|open| *open = !*open);
                                })
                            >
                                <span aria-hidden="true">
                                    <Icon icon=IconName::ChevronDown size=IconSize::Sm />
                                </span>
                            </SystemDockButton>
                        </Show>
                    </SystemDockSection>

                    <SystemDockSection ui_slot="right">
                        <time
                            id="taskbar-clock"
                            data-ui-slot="dock-clock"
                            aria-label=move || format!("Time {}", format_taskbar_clock_time(clock_now.get()))
                            datetime=move || {
                                let now = clock_now.get();
                                format!(
                                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                                    now.year, now.month, now.day, now.hour, now.minute, now.second
                                )
                            }
                        >
                            {move || format_taskbar_clock_time(clock_now.get())}
                        </time>
                    </SystemDockSection>
                </SystemDock>
            </div>

            <super::menus::OverflowMenu
                state
                runtime
                viewport_width
                selected_running_window
                window_context_menu
                overflow_menu_open
            />

            <super::menus::StartMenu
                launcher_open
                close_launcher
                activate_app=activate_launcher_app
                return_focus_id="taskbar-start-button"
            />

            <super::menus::ControlCenterPanel
                open=control_center_open
                theme_mode
                high_contrast
                reduced_motion
                open_window_count
                unread_notification_count
                close_panel=close_control_center
                open_settings
                toggle_theme_mode
                toggle_high_contrast
                toggle_reduced_motion
                return_focus_id="desktop-shell-root"
            />

            <super::menus::NotificationCenterPanel
                open=notification_center_open
                notifications
                close_panel=close_notification_center
                clear_notifications
                dismiss_notification
                return_focus_id="desktop-shell-root"
            />

            <super::menus::TaskbarWindowContextMenu
                state
                runtime
                selected_running_window
                window_context_menu
            />
        </>
    }
}
