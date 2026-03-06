use super::*;
use crate::wallpaper;
use leptos::ev::MouseEvent;
use platform_host::{WallpaperConfig, WallpaperMediaKind, WallpaperSelection};
use system_ui::{ButtonVariant, Icon, IconName, IconSize, MenuItem, MenuSeparator, MenuSurface};

#[component]
pub(super) fn DesktopContextMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    desktop_context_menu: RwSignal<Option<DesktopContextMenuState>>,
    open_system_settings: Callback<()>,
) -> impl IntoView {
    view! {
        <Show when=move || desktop_context_menu.get().is_some() fallback=|| ()>
            {move || {
                let Some(menu) = desktop_context_menu.get() else {
                    return ().into_view();
                };
                let active_id = match &state.get().wallpaper.selection {
                    WallpaperSelection::BuiltIn { wallpaper_id } => wallpaper_id.clone(),
                    WallpaperSelection::Imported { asset_id } => asset_id.clone(),
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

                        <MenuSeparator />
                        <div data-ui-slot="menu-group-label">
                            "Quick Backgrounds"
                        </div>

                        <For
                            each=move || wallpaper::featured_builtin_wallpapers()
                            key=|asset| asset.asset_id.clone()
                            let:asset
                        >
                            {{
                                let active_id = active_id.clone();
                                move || {
                                let item_id = asset.asset_id.clone();
                                let is_active = active_id == item_id;
                                let display_name = asset.display_name.clone();
                                let media_label = match asset.media_kind {
                                    WallpaperMediaKind::Video => "Video",
                                    WallpaperMediaKind::AnimatedImage => "Animated",
                                    WallpaperMediaKind::Svg => "Vector",
                                    WallpaperMediaKind::StaticImage => "Image",
                                };

                                view! {
                                    <MenuItem
                                        id=format!("desktop-context-menu-wallpaper-{}", item_id)
                                        role="menuitemradio"
                                        aria_checked=if is_active { "true" } else { "false" }
                                        selected=is_active
                                        on_click=Callback::new(move |ev| {
                                            stop_mouse_event(&ev);
                                            desktop_context_menu.set(None);
                                            runtime.dispatch_action(DesktopAction::SetCurrentWallpaper {
                                                config: WallpaperConfig {
                                                    selection: WallpaperSelection::BuiltIn {
                                                        wallpaper_id: item_id.clone(),
                                                    },
                                                    ..WallpaperConfig::default()
                                                },
                                            });
                                        })
                                    >
                                        <span aria-hidden="true">
                                            {if is_active {
                                                view! { <Icon icon=IconName::Checkmark size=IconSize::Xs /> }.into_view()
                                            } else {
                                                ().into_view()
                                            }}
                                        </span>
                                        <span>
                                            <span>{display_name}</span>
                                            <span>{media_label}</span>
                                        </span>
                                    </MenuItem>
                                }
                                }
                            }}
                        </For>
                    </MenuSurface>
                }
                    .into_view()
            }}
        </Show>
    }
}

#[component]
pub(super) fn StartMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    overflow_menu_open: RwSignal<bool>,
    clock_menu_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || state.get().start_menu_open fallback=|| ()>
            <MenuSurface
                id="desktop-launcher-menu"
                ui_slot="launcher-menu"
                role="menu"
                aria_label="Application launcher"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if handle_menu_roving_keydown(&ev, "desktop-launcher-menu") {
                        return;
                    }
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        runtime.dispatch_action(DesktopAction::CloseStartMenu);
                        let _ = focus_element_by_id("taskbar-start-button");
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
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
                                    window_context_menu.set(None);
                                    overflow_menu_open.set(false);
                                    clock_menu_open.set(false);
                                    runtime.dispatch_action(DesktopAction::ActivateApp {
                                        app_id: app_id.clone(),
                                        viewport: Some(runtime.host.get_value().desktop_viewport_rect(TASKBAR_HEIGHT_PX)),
                                    });
                                })
                            >
                                <span aria-hidden="true">
                                    <Icon icon=app_icon size=IconSize::Sm />
                                </span>
                                <span>{format!("Open {}", launcher_label)}</span>
                            </MenuItem>
                        }
                    }}
                </For>
                <MenuItem
                    id="desktop-launcher-item-close"
                    role="menuitem"
                    on_click=Callback::new(move |_| runtime.dispatch_action(DesktopAction::CloseStartMenu))
                >
                    "Close"
                </MenuItem>
            </MenuSurface>
        </Show>
    }
}

#[component]
pub(super) fn OverflowMenu(
    state: RwSignal<DesktopState>,
    runtime: DesktopRuntimeContext,
    viewport_width: RwSignal<i32>,
    clock_config: RwSignal<TaskbarClockConfig>,
    selected_running_window: RwSignal<Option<WindowId>>,
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    overflow_menu_open: RwSignal<bool>,
    clock_menu_open: RwSignal<bool>,
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
                        let tray_count = build_taskbar_tray_widgets(&desktop).len();
                        let layout = compute_taskbar_layout(
                            viewport_width.get(),
                            pinned_taskbar_apps().len(),
                            ordered_taskbar_windows(&desktop).len(),
                            tray_count,
                            clock_config.get().show_date,
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
                            clock_menu_open.set(false);
                            runtime.dispatch_action(DesktopAction::CloseStartMenu);
                            let desktop = runtime.state.get_untracked();
                            focus_or_unminimize_window(runtime, &desktop, win.id);
                        })
                        on_contextmenu=Callback::new(move |ev: MouseEvent| {
                            ev.prevent_default();
                            ev.stop_propagation();
                            selected_running_window.set(Some(win.id));
                            overflow_menu_open.set(false);
                            clock_menu_open.set(false);
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
pub(super) fn ClockMenu(
    clock_config: RwSignal<TaskbarClockConfig>,
    clock_menu_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <Show when=move || clock_menu_open.get() fallback=|| ()>
            <MenuSurface
                id="taskbar-clock-menu"
                role="menu"
                aria_label="Clock settings"
                on_keydown=Callback::new(move |ev: web_sys::KeyboardEvent| {
                    if handle_menu_roving_keydown(&ev, "taskbar-clock-menu") {
                        return;
                    }
                    if ev.key() == "Escape" {
                        ev.prevent_default();
                        ev.stop_propagation();
                        clock_menu_open.set(false);
                        let _ = focus_element_by_id("taskbar-clock-button");
                    }
                })
                on_mousedown=Callback::new(move |ev: MouseEvent| ev.stop_propagation())
            >
                <MenuItem
                    id="taskbar-clock-menu-item-24h"
                    role="menuitemcheckbox"
                    aria_checked=Signal::derive(move || {
                        if clock_config.get().use_24_hour {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        }
                    })
                    selected=Signal::derive(move || clock_config.get().use_24_hour)
                    on_click=Callback::new(move |_| {
                        clock_config.update(|cfg| cfg.use_24_hour = !cfg.use_24_hour);
                    })
                >
                    "24-hour time"
                </MenuItem>
                <MenuItem
                    id="taskbar-clock-menu-item-close"
                    role="menuitem"
                    on_click=Callback::new(move |_| clock_menu_open.set(false))
                >
                    "Close"
                </MenuItem>
            </MenuSurface>
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
