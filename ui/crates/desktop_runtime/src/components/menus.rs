use super::*;
use crate::model::{DesktopNotification, ThemeMode};
use leptos::ev::MouseEvent;
use system_ui::components::{
    LauncherPanel as SystemLauncherPanel, MenuItem, MenuSurface,
    NotificationCenter as SystemNotificationCenter, QuickSettingTile, SidePanel as SystemSidePanel,
};
use system_ui::primitives::{ButtonVariant, Icon, IconName, IconSize};

#[component]
pub(super) fn DesktopContextMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    desktop_context_menu: RwSignal<Option<DesktopContextMenuState>>,
    open_system_settings: Callback<()>,
) -> impl IntoView {
    let _ = (state, runtime);

    view! {
        <Show when=move || desktop_context_menu.get().is_some() fallback=|| ()>
            {move || {
                let Some(menu) = desktop_context_menu.get() else {
                    return ().into_view();
                };
                let menu_style = format!("left:{}px;top:{}px;", menu.x, menu.y);

                view! {
                    <MenuSurface
                        id="desktop-context-menu"
                        role="menu"
                        aria_label="Desktop context menu"
                        style=menu_style
                        on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                            if handle_menu_roving_keydown(&ev, "desktop-context-menu") {
                                return;
                            }
                            if ev.key() == "Escape" {
                                ev.prevent_default();
                                ev.stop_propagation();
                                desktop_context_menu.set(None);
                                let _ = focus_element_by_id("desktop-shell-root");
                            }
                        })
                        on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
                        on_click=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
                    >
                        <MenuItem
                            id="desktop-context-menu-item-refresh"
                            role="menuitem"
                            on_click=Callback::new(move |ev| {
                                stop_mouse_event(&ev);
                                desktop_context_menu.set(None);
                            })
                        >
                            "Refresh"
                        </MenuItem>
                        <MenuItem
                            id="desktop-context-menu-item-properties"
                            role="menuitem"
                            on_click=Callback::new(move |ev| {
                                stop_mouse_event(&ev);
                                desktop_context_menu.set(None);
                                open_system_settings.call(());
                            })
                        >
                            "Properties..."
                        </MenuItem>
                    </MenuSurface>
                }
                    .into_view()
            }}
        </Show>
    }
}

#[component]
pub(super) fn StartMenu(
    launcher_open: Signal<bool>,
    close_launcher: Callback<()>,
    activate_app: Callback<ApplicationId>,
    return_focus_id: &'static str,
) -> impl IntoView {
    view! {
        <Show when=move || launcher_open.get() fallback=|| ()>
            <SystemLauncherPanel
                id="desktop-launcher-menu"
                aria_label="Application launcher"
                style="left:50%;transform:translateX(-50%);bottom:calc(var(--origin-shell-taskbar-height) + var(--origin-space-3));"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if handle_menu_roving_keydown(&ev, "desktop-launcher-menu") {
                        return;
                    }
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        close_launcher.call(());
                        let _ = focus_element_by_id(return_focus_id);
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
                <div data-ui-slot="launcher-panel-header">
                    <span aria-hidden="true">
                        <Icon icon=IconName::Search size=IconSize::Sm />
                    </span>
                    <span>"Applications"</span>
                </div>
                <div data-ui-slot="launcher-grid">
                <For each=move || apps::launcher_apps() key=|app| app.app_id.to_string() let:app>
                    {{
                        let app_id = app.app_id.clone();
                        let app_dom_id = format!("desktop-launcher-item-{}", app_id.as_str());
                        let app_icon = app_icon_name(&app_id);
                        let launcher_label = app.launcher_label;
                        view! {
                            <MenuItem
                                id=app_dom_id
                                role="menuitem"
                                on_click=Callback::new(move |_| {
                                    activate_app.call(app_id.clone());
                                })
                            >
                                <span aria-hidden="true">
                                    <Icon icon=app_icon size=IconSize::Md />
                                </span>
                                <span>{launcher_label}</span>
                            </MenuItem>
                        }
                    }}
                </For>
                </div>
                <MenuItem
                    id="desktop-launcher-item-close"
                    role="menuitem"
                    on_click=Callback::new(move |_| close_launcher.call(()))
                >
                    "Close"
                </MenuItem>
            </SystemLauncherPanel>
        </Show>
    }
}

#[component]
pub(super) fn OverflowMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    viewport_width: RwSignal<i32>,
    selected_running_window: RwSignal<Option<WindowId>>,
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    overflow_menu_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || overflow_menu_open.get() fallback=|| ()>
            <MenuSurface
                id="taskbar-overflow-menu"
                role="menu"
                aria_label="Hidden taskbar windows"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if handle_menu_roving_keydown(&ev, "taskbar-overflow-menu") {
                        return;
                    }
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        overflow_menu_open.set(false);
                        let _ = focus_element_by_id("taskbar-overflow-button");
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
                <For
                    each=move || {
                        let desktop = state.get();
                        let layout = compute_taskbar_layout(
                            viewport_width.get(),
                            pinned_taskbar_apps().len(),
                            ordered_taskbar_windows(&desktop).len(),
                        );
                        ordered_taskbar_windows(&desktop)
                            .into_iter()
                            .skip(layout.visible_running_count)
                            .collect::<Vec<_>>()
                    }
                    key=|win| win.id.0
                    let:win
                >
                    <MenuItem
                        id=format!("taskbar-overflow-menu-item-{}", win.id.0)
                        role="menuitem"
                        selected=Signal::derive(move || {
                            selected_running_window.get() == Some(win.id)
                        })
                        on_click=Callback::new(move |_| {
                            selected_running_window.set(Some(win.id));
                            overflow_menu_open.set(false);
                            window_context_menu.set(None);
                            runtime.dispatch_action(DesktopAction::CloseStartMenu);
                            let desktop = runtime.state.get_untracked();
                            focus_or_unminimize_window(runtime, &desktop, win.id);
                        })
                        on_contextmenu=Callback::new(move |ev: MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            selected_running_window.set(Some(win.id));
                            overflow_menu_open.set(false);
                            runtime.dispatch_action(DesktopAction::CloseStartMenu);
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
                            <Icon
                                icon=app_icon_name(&win.app_id)
                                size=IconSize::Sm
                            />
                        </span>
                        <span>{win.title.clone()}</span>
                    </MenuItem>
                </For>
            </MenuSurface>
        </Show>
    }
}

#[component]
pub(super) fn ControlCenterPanel(
    open: Signal<bool>,
    theme_mode: Signal<ThemeMode>,
    high_contrast: Signal<bool>,
    reduced_motion: Signal<bool>,
    open_window_count: Signal<usize>,
    unread_notification_count: Signal<usize>,
    close_panel: Callback<()>,
    open_settings: Callback<()>,
    toggle_theme_mode: Callback<()>,
    toggle_high_contrast: Callback<()>,
    toggle_reduced_motion: Callback<()>,
    return_focus_id: &'static str,
) -> impl IntoView {
    view! {
        <Show when=move || open.get() fallback=|| ()>
            <SystemSidePanel
                id="desktop-control-center"
                aria_label="Control center"
                ui_slot="control-center"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        close_panel.call(());
                        let _ = focus_element_by_id(return_focus_id);
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
                <div data-ui-slot="panel-header">
                    <div>
                        <strong>"Control Center"</strong>
                        <p>"Quick appearance and system controls."</p>
                    </div>
                    <MenuItem
                        id="control-center-open-settings"
                        role="button"
                        on_click=Callback::new(move |_| open_settings.call(()))
                    >
                        "Settings"
                    </MenuItem>
                </div>

                <div data-ui-slot="quick-setting-grid">
                    <QuickSettingTile
                        aria_label="Toggle dark theme"
                        selected=Signal::derive(move || matches!(theme_mode.get(), ThemeMode::Dark))
                        on_click=Callback::new(move |_| toggle_theme_mode.call(()))
                    >
                        <span aria-hidden="true">
                            {move || {
                                if matches!(theme_mode.get(), ThemeMode::Dark) {
                                    view! { <Icon icon=IconName::Moon size=IconSize::Sm /> }.into_view()
                                } else {
                                    view! { <Icon icon=IconName::Sun size=IconSize::Sm /> }.into_view()
                                }
                            }}
                        </span>
                        <span>{move || if matches!(theme_mode.get(), ThemeMode::Dark) { "Dark" } else { "Light" }}</span>
                    </QuickSettingTile>

                    <QuickSettingTile
                        aria_label="Toggle high contrast"
                        selected=high_contrast
                        on_click=Callback::new(move |_| toggle_high_contrast.call(()))
                    >
                        <span aria-hidden="true">
                            <Icon icon=IconName::WifiOn size=IconSize::Sm />
                        </span>
                        <span>"High Contrast"</span>
                    </QuickSettingTile>

                    <QuickSettingTile
                        aria_label="Toggle reduced motion"
                        selected=reduced_motion
                        on_click=Callback::new(move |_| toggle_reduced_motion.call(()))
                    >
                        <span aria-hidden="true">
                            <Icon icon=IconName::MotionOff size=IconSize::Sm />
                        </span>
                        <span>"Reduced Motion"</span>
                    </QuickSettingTile>
                </div>

                <div data-ui-slot="panel-section">
                    <strong>"Open Windows"</strong>
                    <p>{move || open_window_count.get().to_string()}</p>
                </div>

                <div data-ui-slot="panel-section">
                    <strong>"Unread Notifications"</strong>
                    <p>{move || unread_notification_count.get().to_string()}</p>
                </div>
            </SystemSidePanel>
        </Show>
    }
}

#[component]
pub(super) fn NotificationCenterPanel(
    open: Signal<bool>,
    notifications: Signal<Vec<DesktopNotification>>,
    close_panel: Callback<()>,
    clear_notifications: Callback<()>,
    dismiss_notification: Callback<u64>,
    return_focus_id: &'static str,
) -> impl IntoView {
    view! {
        <Show when=move || open.get() fallback=|| ()>
            <SystemNotificationCenter
                id="desktop-notification-center"
                aria_label="Notification center"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        close_panel.call(());
                        let _ = focus_element_by_id(return_focus_id);
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
                <div data-ui-slot="panel-header">
                    <div>
                        <strong>"Notifications"</strong>
                        <p>"Recent system and app activity."</p>
                    </div>
                    <MenuItem
                        id="notification-center-clear"
                        role="button"
                        on_click=Callback::new(move |_| clear_notifications.call(()))
                    >
                        "Clear All"
                    </MenuItem>
                </div>

                <div data-ui-slot="notification-list">
                    <Show
                        when=move || !notifications.get().is_empty()
                        fallback=|| view! { <p>"No notifications yet."</p> }
                    >
                        <For
                            each=move || notifications.get()
                            key=|notification| notification.id
                            let:notification
                        >
                            <div
                                data-ui-primitive="true"
                                data-ui-kind="notification-item"
                                data-ui-state=if notification.unread { "unread" } else { "read" }
                            >
                                <div>
                                    <strong>{notification.title.clone()}</strong>
                                    <p>{notification.body.clone()}</p>
                                </div>
                                <MenuItem
                                    id=format!("notification-dismiss-{}", notification.id)
                                    role="button"
                                    on_click=Callback::new(move |_| {
                                        dismiss_notification.call(notification.id);
                                    })
                                >
                                    "Dismiss"
                                </MenuItem>
                            </div>
                        </For>
                    </Show>
                </div>
            </SystemNotificationCenter>
        </Show>
    }
}

#[component]
pub(super) fn TaskbarWindowContextMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    selected_running_window: RwSignal<Option<WindowId>>,
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
) -> impl IntoView {
    view! {
        <Show
            when=move || {
                window_context_menu
                    .get()
                    .and_then(|menu| {
                        state
                            .get()
                            .windows
                            .into_iter()
                            .find(|win| win.id == menu.window_id)
                            .map(|win| (menu, win))
                    })
                    .is_some()
            }
            fallback=|| ()
        >
            {move || {
                let Some((menu, win)) = window_context_menu.get().and_then(|menu| {
                    state
                        .get()
                        .windows
                        .into_iter()
                        .find(|win| win.id == menu.window_id)
                        .map(|win| (menu, win))
                }) else {
                    return ().into_view();
                };

                let menu_style = format!("left:{}px;top:{}px;", menu.x, menu.y);
                let can_focus = !win.is_focused && !win.minimized;
                let can_restore = win.minimized || win.maximized;
                let can_minimize = win.flags.minimizable && !win.minimized;
                let can_maximize = win.flags.maximizable && !win.maximized;
                let restore_label = if win.minimized {
                    "Restore"
                } else {
                    "Restore Size"
                };
                let window_id = win.id;

                view! {
                    <MenuSurface
                        id="taskbar-window-context-menu"
                        role="menu"
                        aria_label=format!("Window menu for {}", win.title)
                        style=menu_style
                        on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                            if handle_menu_roving_keydown(&ev, "taskbar-window-context-menu") {
                                return;
                            }
                            if ev.key() == "Escape" {
                                ev.prevent_default();
                                ev.stop_propagation();
                                window_context_menu.set(None);
                                let focus_target = selected_running_window
                                    .get_untracked()
                                    .unwrap_or(window_id);
                                let _ = focus_element_by_id(&taskbar_window_button_dom_id(focus_target));
                            }
                        })
                        on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
                    >
                        <MenuItem
                            id=format!("taskbar-window-menu-focus-{}", window_id.0)
                            role="menuitem"
                            disabled=!can_focus
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                let desktop = runtime.state.get_untracked();
                                focus_or_unminimize_window(runtime, &desktop, window_id);
                            })
                        >
                            "Focus"
                        </MenuItem>
                        <MenuItem
                            id=format!("taskbar-window-menu-restore-{}", window_id.0)
                            role="menuitem"
                            disabled=!can_restore
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                runtime.dispatch_action(DesktopAction::RestoreWindow { window_id });
                            })
                        >
                            {restore_label}
                        </MenuItem>
                        <MenuItem
                            id=format!("taskbar-window-menu-minimize-{}", window_id.0)
                            role="menuitem"
                            disabled=!can_minimize
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                runtime.dispatch_action(DesktopAction::MinimizeWindow { window_id });
                            })
                        >
                            "Minimize"
                        </MenuItem>
                        <MenuItem
                            id=format!("taskbar-window-menu-maximize-{}", window_id.0)
                            role="menuitem"
                            disabled=!can_maximize
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                runtime.dispatch_action(DesktopAction::MaximizeWindow {
                                    window_id,
                                    viewport: runtime.host.get_value().desktop_viewport_rect(TASKBAR_HEIGHT_PX),
                                });
                            })
                        >
                            "Maximize"
                        </MenuItem>
                        <MenuItem
                            id=format!("taskbar-window-menu-close-{}", window_id.0)
                            role="menuitem"
                            variant=ButtonVariant::Danger
                            on_click=Callback::new(move |_| {
                                window_context_menu.set(None);
                                runtime.dispatch_action(DesktopAction::CloseWindow { window_id });
                            })
                        >
                            "Close"
                        </MenuItem>
                    </MenuSurface>
                }
                    .into_view()
            }}
        </Show>
    }
}
