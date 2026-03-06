//! Desktop shell UI composition and interaction surfaces.

mod a11y;
mod menus;
mod taskbar;
mod taskbar_input;
mod window;

use std::time::Duration;

use desktop_app_contract::ApplicationId;
use leptos::*;
use platform_host::{
    WallpaperAnimationPolicy, WallpaperDisplayMode, WallpaperMediaKind, WallpaperPosition,
};
use serde_json::{json, Value};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

use self::{
    a11y::{focus_element_by_id, focus_first_menu_item, handle_menu_roving_keydown},
    menus::DesktopContextMenu,
    taskbar::Taskbar,
    taskbar_input::{is_activation_key, is_context_menu_shortcut, try_handle_taskbar_shortcuts},
    window::DesktopWindow,
};

use crate::{
    apps,
    e2e::{BrowserE2eConfig, BrowserE2eScene},
    host::DesktopHostContext,
    model::{DesktopState, PointerPosition, ResizeEdge, WindowId, WindowRecord},
    reducer::DesktopAction,
    runtime_context::open_system_settings,
    wallpaper,
};
use system_ui::{
    DesktopBackdrop, DesktopIconButton, DesktopIconGrid, DesktopWindowLayer, Icon, IconName,
    IconSize,
};

const TASKBAR_HEIGHT_PX: i32 = 38;
#[cfg(target_arch = "wasm32")]
const E2E_START_BUTTON_ATTR: &str = "data-e2e-state";

fn app_icon_name(app_id: &ApplicationId) -> IconName {
    apps::app_icon_name_by_id(app_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DesktopContextMenuState {
    x: i32,
    y: i32,
}

fn taskbar_window_button_dom_id(window_id: WindowId) -> String {
    format!("taskbar-window-button-{}", window_id.0)
}

fn wallpaper_object_position(position: WallpaperPosition) -> &'static str {
    match position {
        WallpaperPosition::TopLeft => "left top",
        WallpaperPosition::Top => "center top",
        WallpaperPosition::TopRight => "right top",
        WallpaperPosition::Left => "left center",
        WallpaperPosition::Center => "center center",
        WallpaperPosition::Right => "right center",
        WallpaperPosition::BottomLeft => "left bottom",
        WallpaperPosition::Bottom => "center bottom",
        WallpaperPosition::BottomRight => "right bottom",
    }
}

fn wallpaper_background_position(position: WallpaperPosition) -> &'static str {
    wallpaper_object_position(position)
}

fn wallpaper_object_fit(display_mode: WallpaperDisplayMode) -> &'static str {
    match display_mode {
        WallpaperDisplayMode::Fill => "cover",
        WallpaperDisplayMode::Fit => "contain",
        WallpaperDisplayMode::Stretch => "fill",
        WallpaperDisplayMode::Center => "none",
        WallpaperDisplayMode::Tile => "none",
    }
}

pub use crate::runtime_context::{use_desktop_runtime, DesktopProvider, DesktopRuntimeContext};

fn browser_e2e_window_request(
    app_id: ApplicationId,
    runtime: DesktopRuntimeContext,
    launch_params: Value,
) -> crate::model::OpenWindowRequest {
    let viewport = runtime
        .host
        .get_value()
        .desktop_viewport_rect(TASKBAR_HEIGHT_PX);
    let mut request = apps::default_open_request_by_id(&app_id, Some(viewport))
        .unwrap_or_else(|| crate::model::OpenWindowRequest::new(app_id));
    request.launch_params = launch_params;
    request
}

#[cfg(target_arch = "wasm32")]
fn set_start_button_e2e_state(state: Option<&str>, focus_button: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.get_element_by_id("taskbar-start-button") else {
        return;
    };
    match state {
        Some(value) => {
            let _ = element.set_attribute(E2E_START_BUTTON_ATTR, value);
        }
        None => {
            let _ = element.remove_attribute(E2E_START_BUTTON_ATTR);
        }
    }
    if focus_button {
        if let Some(html_element) = element.dyn_ref::<web_sys::HtmlElement>() {
            let _ = html_element.focus();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_start_button_e2e_state(_state: Option<&str>, _focus_button: bool) {}

#[cfg(target_arch = "wasm32")]
fn mark_browser_e2e_ready() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(performance) = js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("performance"))
    else {
        return;
    };
    let Ok(mark) = js_sys::Reflect::get(&performance, &JsValue::from_str("mark")) else {
        return;
    };
    let Some(mark_fn) = mark.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let _ = mark_fn.call1(&performance, &JsValue::from_str("os:e2e-ready"));
}

#[cfg(not(target_arch = "wasm32"))]
fn mark_browser_e2e_ready() {}

#[component]
fn DesktopWallpaperRenderer(state: RwSignal<DesktopState>) -> impl IntoView {
    let active_config = Signal::derive(move || {
        let desktop = state.get();
        desktop.wallpaper_preview.unwrap_or(desktop.wallpaper)
    });
    let resolved_source = Signal::derive(move || {
        let desktop = state.get();
        wallpaper::resolve_wallpaper_source(&active_config.get(), &desktop.wallpaper_library)
    });
    let reduced_motion = Signal::derive(move || state.get().theme.reduced_motion);

    view! {
        <Show when=move || resolved_source.get().is_some() fallback=|| ()>
            {move || {
                let config = active_config.get();
                let source = resolved_source.get().expect("wallpaper source");
                let fit = wallpaper_object_fit(config.display_mode);
                let position = wallpaper_object_position(config.position);
                let background_position = wallpaper_background_position(config.position);
                let allow_animation = !reduced_motion.get()
                    && config.animation == WallpaperAnimationPolicy::LoopMuted;

                match (source.media_kind, config.display_mode) {
                    (WallpaperMediaKind::StaticImage | WallpaperMediaKind::Svg, WallpaperDisplayMode::Tile) => {
                        view! {
                            <div
                                data-ui-slot="wallpaper-layer"
                                data-ui-kind="wallpaper-layer"
                                style=format!(
                                    "background-image:url('{}');background-position:{};",
                                    source.primary_url,
                                    background_position
                                )
                            />
                        }
                            .into_view()
                    }
                    (WallpaperMediaKind::Video, _) | (WallpaperMediaKind::AnimatedImage, _)
                        if !allow_animation =>
                    {
                        let fallback_url = source
                            .poster_url
                            .clone()
                            .unwrap_or_else(|| source.primary_url.clone());
                        view! {
                            <img
                                data-ui-slot="wallpaper-layer"
                                data-ui-kind="wallpaper-layer"
                                src=fallback_url
                                alt=""
                                style=format!("object-fit:{};object-position:{};", fit, position)
                            />
                        }
                            .into_view()
                    }
                    (WallpaperMediaKind::Video, _) => {
                        view! {
                            <video
                                data-ui-slot="wallpaper-layer"
                                data-ui-kind="wallpaper-layer"
                                src=source.primary_url
                                poster=source.poster_url.unwrap_or_default()
                                autoplay=true
                                muted=true
                                loop=true
                                playsinline=true
                                style=format!("object-fit:{};object-position:{};", fit, position)
                            />
                        }
                            .into_view()
                    }
                    _ => {
                        view! {
                            <img
                                data-ui-slot="wallpaper-layer"
                                data-ui-kind="wallpaper-layer"
                                src=source.primary_url
                                alt=""
                                style=format!("object-fit:{};object-position:{};", fit, position)
                            />
                        }
                            .into_view()
                    }
                }
            }}
        </Show>
    }
}

#[component]
/// Renders the full desktop shell UI and processes queued [`crate::RuntimeEffect`] values.
pub fn DesktopShell() -> impl IntoView {
    let runtime = use_desktop_runtime();
    let state = runtime.state;
    let browser_e2e = use_context::<BrowserE2eConfig>();
    let desktop_context_menu = create_rw_signal(None::<DesktopContextMenuState>);
    let desktop_context_menu_was_open = create_rw_signal(false);
    let browser_e2e_scene_applied = create_rw_signal(false);
    let browser_e2e_ready = create_rw_signal(browser_e2e.is_none());
    let browser_e2e_marked_ready = create_rw_signal(false);
    let browser_e2e_for_scene_setup = browser_e2e.clone();
    let browser_e2e_for_readiness = browser_e2e.clone();
    let browser_e2e_for_scene_attr = browser_e2e.clone();
    let browser_e2e_for_ready_attr = browser_e2e.clone();

    create_effect(move |_| {
        let is_open = desktop_context_menu.get().is_some();
        let was_open = desktop_context_menu_was_open.get_untracked();
        if is_open && !was_open {
            desktop_context_menu_was_open.set(true);
            let _ = focus_first_menu_item("desktop-context-menu");
        } else if !is_open && was_open {
            desktop_context_menu_was_open.set(false);
        }
    });

    let escape_listener = window_event_listener(ev::keydown, move |ev| {
        if ev.default_prevented() || ev.key() != "Escape" {
            return;
        }

        if desktop_context_menu.get_untracked().is_some() {
            ev.prevent_default();
            ev.stop_propagation();
            desktop_context_menu.set(None);
            let _ = focus_element_by_id("desktop-shell-root");
        }
    });
    on_cleanup(move || escape_listener.remove());

    let on_pointer_move = move |ev: web_sys::PointerEvent| {
        let pointer = pointer_from_pointer_event(&ev);
        let interaction = runtime.interaction.get_untracked();

        if interaction.dragging.is_some() {
            runtime.dispatch_action(DesktopAction::UpdateMove { pointer });
        }
        if interaction.resizing.is_some() {
            runtime.dispatch_action(DesktopAction::UpdateResize { pointer });
        }
    };
    let on_pointer_end = move |_| end_active_pointer_interaction(runtime);
    let open_system_settings = Callback::new(move |_| {
        desktop_context_menu.set(None);
        runtime.dispatch_action(DesktopAction::CloseStartMenu);
        open_system_settings(runtime, TASKBAR_HEIGHT_PX);
    });

    create_effect(move |_| {
        let Some(config) = browser_e2e_for_scene_setup.clone() else {
            return;
        };
        if !state.get().boot_hydrated || browser_e2e_scene_applied.get() {
            return;
        }

        browser_e2e_scene_applied.set(true);
        runtime.dispatch_action(DesktopAction::CloseStartMenu);
        desktop_context_menu.set(None);
        set_start_button_e2e_state(None, false);

        if let Some(skin) = config.skin {
            runtime.dispatch_action(DesktopAction::SetSkin { skin });
        }
        if let Some(enabled) = config.high_contrast {
            runtime.dispatch_action(DesktopAction::SetHighContrast { enabled });
        }
        if let Some(enabled) = config.reduced_motion {
            runtime.dispatch_action(DesktopAction::SetReducedMotion { enabled });
        }

        match config.scene {
            BrowserE2eScene::ShellDefault
            | BrowserE2eScene::ShellHighContrast
            | BrowserE2eScene::ShellReducedMotion => {}
            BrowserE2eScene::ShellContextMenuOpen => {
                let viewport = runtime
                    .host
                    .get_value()
                    .desktop_viewport_rect(TASKBAR_HEIGHT_PX);
                open_desktop_context_menu(
                    runtime.host.get_value(),
                    desktop_context_menu,
                    viewport.x + (viewport.w / 2),
                    viewport.y + (viewport.h / 2),
                );
            }
            BrowserE2eScene::SettingsAppearance => {
                runtime.dispatch_action(DesktopAction::OpenWindow(browser_e2e_window_request(
                    apps::settings_application_id(),
                    runtime,
                    json!({ "section": "appearance" }),
                )));
            }
            BrowserE2eScene::SettingsAccessibility => {
                runtime.dispatch_action(DesktopAction::OpenWindow(browser_e2e_window_request(
                    apps::settings_application_id(),
                    runtime,
                    json!({ "section": "accessibility" }),
                )));
            }
            BrowserE2eScene::StartButtonHover => {
                set_start_button_e2e_state(Some("hover"), false);
            }
            BrowserE2eScene::StartButtonFocus => {
                set_start_button_e2e_state(Some("focus-visible"), true);
            }
            BrowserE2eScene::UiShowcaseControls => {
                runtime.dispatch_action(DesktopAction::OpenWindow(browser_e2e_window_request(
                    ApplicationId::trusted("system.ui-showcase"),
                    runtime,
                    Value::Null,
                )));
            }
            BrowserE2eScene::TerminalDefault => {
                runtime.dispatch_action(DesktopAction::OpenWindow(browser_e2e_window_request(
                    ApplicationId::trusted("system.terminal"),
                    runtime,
                    Value::Null,
                )));
            }
        }
    });

    create_effect(move |_| {
        let Some(config) = browser_e2e_for_readiness.clone() else {
            return;
        };
        if !state.get().boot_hydrated || !browser_e2e_scene_applied.get() {
            browser_e2e_ready.set(false);
            return;
        }

        let desktop = state.get();
        let ready = match config.scene {
            BrowserE2eScene::ShellDefault => {
                desktop.windows.is_empty() && desktop_context_menu.get().is_none()
            }
            BrowserE2eScene::ShellContextMenuOpen => desktop_context_menu.get().is_some(),
            BrowserE2eScene::SettingsAppearance => desktop.windows.iter().any(|window| {
                window.app_id == apps::settings_application_id()
                    && window.launch_params.get("section").and_then(Value::as_str)
                        == Some("appearance")
            }),
            BrowserE2eScene::SettingsAccessibility => desktop.windows.iter().any(|window| {
                window.app_id == apps::settings_application_id()
                    && window.launch_params.get("section").and_then(Value::as_str)
                        == Some("accessibility")
            }),
            BrowserE2eScene::StartButtonHover | BrowserE2eScene::StartButtonFocus => {
                desktop_context_menu.get().is_none()
            }
            BrowserE2eScene::ShellHighContrast => {
                desktop.theme.high_contrast && desktop.windows.is_empty()
            }
            BrowserE2eScene::ShellReducedMotion => {
                desktop.theme.reduced_motion && desktop.windows.is_empty()
            }
            BrowserE2eScene::UiShowcaseControls => desktop
                .windows
                .iter()
                .any(|window| window.app_id == ApplicationId::trusted("system.ui-showcase")),
            BrowserE2eScene::TerminalDefault => desktop
                .windows
                .iter()
                .any(|window| window.app_id == ApplicationId::trusted("system.terminal")),
        };
        browser_e2e_ready.set(ready);
    });

    create_effect(move |_| {
        if browser_e2e_ready.get() && !browser_e2e_marked_ready.get() {
            browser_e2e_marked_ready.set(true);
            mark_browser_e2e_ready();
        }
    });

    view! {
        <div
            id="desktop-shell-root"
            class="desktop-shell"
            tabindex="-1"
            data-ui-primitive="true"
            data-ui-kind="desktop-root"
            data-e2e-scene=browser_e2e_for_scene_attr
                .as_ref()
                .map(|config| config.scene.id().to_string())
            data-e2e-ready=move || {
                browser_e2e_for_ready_attr
                    .as_ref()
                    .map(|_| browser_e2e_ready.get().to_string())
            }
            data-skin=move || state.get().theme.skin.css_id()
            data-high-contrast=move || state.get().theme.high_contrast.to_string()
            data-reduced-motion=move || state.get().theme.reduced_motion.to_string()
            on:click=move |_| {
                if desktop_context_menu.get_untracked().is_some() {
                    desktop_context_menu.set(None);
                }
            }
            on:pointermove=on_pointer_move
            on:pointerup=on_pointer_end
            on:pointercancel=on_pointer_end
        >
            <DesktopBackdrop>
                <DesktopWallpaperRenderer state=state />
                <div data-ui-slot="atmosphere" aria-hidden="true"></div>
                <div
                    data-ui-slot="dismiss-layer"
                    on:mousedown=move |_| {
                        desktop_context_menu.set(None);
                        runtime.dispatch_action(DesktopAction::CloseStartMenu);
                    }
                    on:contextmenu=move |ev| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        runtime.dispatch_action(DesktopAction::CloseStartMenu);
                        open_desktop_context_menu(
                            runtime.host.get_value(),
                            desktop_context_menu,
                            ev.client_x(),
                            ev.client_y(),
                        );
                    }
                />
                <DesktopIconGrid>
                    <For each=move || apps::desktop_icon_apps() key=|app| app.app_id.to_string() let:app>
                        {{
                            let app_id = app.app_id.clone();
                            let app_icon = app_icon_name(&app_id);
                            let desktop_icon_label = app.desktop_icon_label;
                            view! {
                                <DesktopIconButton
                                    on_click=Callback::new(move |_| {
                                        runtime.dispatch_action(DesktopAction::ActivateApp {
                                            app_id: app_id.clone(),
                                            viewport: Some(runtime.host.get_value().desktop_viewport_rect(TASKBAR_HEIGHT_PX)),
                                        });
                                    })
                                >
                                    <span>
                                        <Icon icon=app_icon size=IconSize::Lg />
                                    </span>
                                    <span>{desktop_icon_label}</span>
                                </DesktopIconButton>
                            }
                        }}
                    </For>
                </DesktopIconGrid>

                <DesktopWindowLayer>
                    <For
                        each=move || state.get().windows
                        key=|win| win.id.0
                        let:win
                    >
                        <DesktopWindow window_id=win.id />
                    </For>
                </DesktopWindowLayer>

                <DesktopContextMenu
                    state
                    runtime
                    desktop_context_menu
                    open_system_settings
                />
            </DesktopBackdrop>

            <Taskbar />
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TaskbarClockConfig {
    use_24_hour: bool,
    show_date: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskbarClockSnapshot {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

impl TaskbarClockSnapshot {
    fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let date = js_sys::Date::new_0();
            return Self {
                year: date.get_full_year(),
                month: date.get_month() + 1,
                day: date.get_date(),
                hour: date.get_hours(),
                minute: date.get_minutes(),
                second: date.get_seconds(),
            };
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskbarLayoutPlan {
    show_pins: bool,
    visible_tray_widget_count: usize,
    show_clock_date: bool,
    compact_running_items: bool,
    visible_running_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskbarWindowContextMenuState {
    window_id: WindowId,
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct PinnedTaskbarAppState {
    running_count: usize,
    focused: bool,
    all_minimized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskbarShortcutTarget {
    Pinned(ApplicationId),
    Window(WindowId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskbarTrayWidgetAction {
    None,
    ToggleHighContrast,
    ToggleReducedMotion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskbarTrayWidget {
    id: &'static str,
    icon: IconName,
    label: &'static str,
    value: String,
    pressed: Option<bool>,
    action: TaskbarTrayWidgetAction,
}

fn pinned_taskbar_apps() -> Vec<ApplicationId> {
    apps::pinned_taskbar_app_ids()
}

fn ordered_taskbar_windows(state: &DesktopState) -> Vec<WindowRecord> {
    let mut windows = state.windows.clone();
    windows.sort_by_key(|win| (win.z_index, win.id.0));
    windows
}

pub(crate) fn preferred_window_for_app(
    state: &DesktopState,
    app_id: &ApplicationId,
) -> Option<WindowId> {
    state
        .windows
        .iter()
        .rev()
        .find(|win| win.app_id == *app_id && !win.minimized && win.is_focused)
        .or_else(|| {
            state
                .windows
                .iter()
                .rev()
                .find(|win| win.app_id == *app_id && !win.minimized)
        })
        .or_else(|| state.windows.iter().rev().find(|win| win.app_id == *app_id))
        .map(|win| win.id)
}

fn pinned_taskbar_app_state(state: &DesktopState, app_id: &ApplicationId) -> PinnedTaskbarAppState {
    let windows: Vec<&WindowRecord> = state
        .windows
        .iter()
        .filter(|win| win.app_id == *app_id)
        .collect();
    let running_count = windows.len();
    let focused = windows.iter().any(|win| win.is_focused && !win.minimized);
    let all_minimized = running_count > 0 && windows.iter().all(|win| win.minimized);

    PinnedTaskbarAppState {
        running_count,
        focused,
        all_minimized,
    }
}

fn compute_taskbar_layout(
    viewport_width: i32,
    pinned_count: usize,
    running_count: usize,
    tray_widget_count: usize,
    desired_clock_show_date: bool,
) -> TaskbarLayoutPlan {
    let viewport_width = viewport_width.max(320);
    let mut show_pins = pinned_count > 0 && viewport_width >= 560;
    let mut visible_tray_widget_count = if viewport_width >= 1320 {
        tray_widget_count.min(4)
    } else if viewport_width >= 1040 {
        tray_widget_count.min(3)
    } else if viewport_width >= 760 {
        tray_widget_count.min(2)
    } else {
        tray_widget_count.min(1)
    };
    let mut show_clock_date = desired_clock_show_date && viewport_width >= 920;

    // Priority order under pressure: hide date, reduce tray widgets, then collapse pinned strip.
    loop {
        let start_width = if viewport_width < 640 { 72 } else { 108 };
        let pins_width = if show_pins {
            (pinned_count as i32) * 42 + 8
        } else {
            0
        };
        let tray_width = 22 + ((visible_tray_widget_count as i32) * 56);
        let clock_width = if show_clock_date { 132 } else { 88 };
        let reserved = start_width + pins_width + tray_width + clock_width + 40;
        let available = viewport_width - reserved;

        if available >= 120 || (!show_clock_date && visible_tray_widget_count <= 1 && !show_pins) {
            break;
        }

        if show_clock_date {
            show_clock_date = false;
            continue;
        }
        if visible_tray_widget_count > 1 {
            visible_tray_widget_count -= 1;
            continue;
        }
        if show_pins {
            show_pins = false;
            continue;
        }
        break;
    }

    let start_width = if viewport_width < 640 { 72 } else { 108 };
    let pins_width = if show_pins {
        (pinned_count as i32) * 42 + 8
    } else {
        0
    };
    let tray_width = 22 + ((visible_tray_widget_count as i32) * 56);
    let clock_width = if show_clock_date { 132 } else { 88 };
    let reserved = start_width + pins_width + tray_width + clock_width + 40;
    let available = (viewport_width - reserved).max(0);

    if running_count == 0 {
        return TaskbarLayoutPlan {
            show_pins,
            visible_tray_widget_count,
            show_clock_date,
            compact_running_items: false,
            visible_running_count: 0,
        };
    }

    let full_item_width = if viewport_width >= 1200 {
        176
    } else if viewport_width >= 900 {
        156
    } else {
        138
    };
    let compact_item_width = if viewport_width >= 900 { 52 } else { 46 };
    let full_visible = (available / full_item_width).max(0) as usize;
    if full_visible >= running_count {
        return TaskbarLayoutPlan {
            show_pins,
            visible_tray_widget_count,
            show_clock_date,
            compact_running_items: false,
            visible_running_count: running_count,
        };
    }

    let mut compact_visible = (available / compact_item_width).max(0) as usize;
    if compact_visible == 0 && available >= 36 {
        compact_visible = 1;
    }

    TaskbarLayoutPlan {
        show_pins,
        visible_tray_widget_count,
        show_clock_date,
        compact_running_items: true,
        visible_running_count: compact_visible.min(running_count),
    }
}

fn taskbar_window_aria_label(win: &WindowRecord) -> String {
    let mut parts = vec![win.title.clone()];
    if win.is_focused && !win.minimized {
        parts.push("focused".to_string());
    }
    if win.minimized {
        parts.push("minimized".to_string());
    }
    if win.maximized {
        parts.push("maximized".to_string());
    }
    parts.join(", ")
}

fn taskbar_pinned_aria_label(app_id: &ApplicationId, status: PinnedTaskbarAppState) -> String {
    let title = apps::app_title_by_id(app_id);
    match status.running_count {
        0 => format!("Pinned {} (not running)", title),
        1 => format!("Pinned {} (1 window running)", title),
        count => format!("Pinned {} ({} windows running)", title, count),
    }
}

fn build_taskbar_shortcut_targets(state: &DesktopState) -> Vec<TaskbarShortcutTarget> {
    let mut targets: Vec<TaskbarShortcutTarget> = pinned_taskbar_apps()
        .iter()
        .cloned()
        .map(TaskbarShortcutTarget::Pinned)
        .collect();

    targets.extend(
        ordered_taskbar_windows(state)
            .into_iter()
            .map(|win| TaskbarShortcutTarget::Window(win.id)),
    );

    targets
}

fn activate_pinned_taskbar_app(runtime: DesktopRuntimeContext, app_id: ApplicationId) {
    let state = runtime.state.get_untracked();
    let descriptor = apps::app_descriptor_by_id(&app_id);

    if descriptor.single_instance {
        if let Some(window_id) = preferred_window_for_app(&state, &app_id) {
            focus_or_unminimize_window(runtime, &state, window_id);
            return;
        }
    }

    runtime.dispatch_action(DesktopAction::ActivateApp {
        app_id,
        viewport: Some(
            runtime
                .host
                .get_value()
                .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
        ),
    });
}

fn activate_taskbar_shortcut_target(runtime: DesktopRuntimeContext, target: TaskbarShortcutTarget) {
    match target {
        TaskbarShortcutTarget::Pinned(app_id) => activate_pinned_taskbar_app(runtime, app_id),
        TaskbarShortcutTarget::Window(window_id) => {
            let state = runtime.state.get_untracked();
            focus_or_unminimize_window(runtime, &state, window_id);
        }
    }
}

pub(crate) fn focus_or_unminimize_window(
    runtime: DesktopRuntimeContext,
    state: &DesktopState,
    window_id: WindowId,
) {
    if let Some(window) = state.windows.iter().find(|win| win.id == window_id) {
        if window.minimized {
            runtime.dispatch_action(DesktopAction::RestoreWindow { window_id });
        } else if !window.is_focused {
            runtime.dispatch_action(DesktopAction::FocusWindow { window_id });
        }
    }
}

fn cycle_selected_running_window(
    running_windows: &[WindowRecord],
    selected: Option<WindowId>,
    delta: i32,
) -> Option<WindowId> {
    if running_windows.is_empty() {
        return None;
    }

    let current_idx = selected
        .and_then(|id| running_windows.iter().position(|win| win.id == id))
        .unwrap_or_else(|| {
            running_windows
                .iter()
                .position(|win| win.is_focused && !win.minimized)
                .unwrap_or(0)
        });
    let len = running_windows.len() as i32;
    let next_idx = (current_idx as i32 + delta).rem_euclid(len) as usize;
    Some(running_windows[next_idx].id)
}

fn open_desktop_context_menu(
    host: DesktopHostContext,
    menu: RwSignal<Option<DesktopContextMenuState>>,
    x: i32,
    y: i32,
) {
    let (x, y) = clamp_desktop_popup_position(host, x, y, 260, 340);
    menu.set(Some(DesktopContextMenuState { x, y }));
}

fn clamp_desktop_popup_position(
    host: DesktopHostContext,
    x: i32,
    y: i32,
    popup_w: i32,
    popup_h: i32,
) -> (i32, i32) {
    let viewport = host.desktop_viewport_rect(TASKBAR_HEIGHT_PX);
    let max_x = (viewport.w - popup_w - 6).max(6);
    let max_y = (viewport.h - popup_h - 6).max(6);
    (x.clamp(6, max_x), y.clamp(6, max_y))
}

fn open_taskbar_window_context_menu(
    host: DesktopHostContext,
    menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    window_id: WindowId,
    x: i32,
    y: i32,
) {
    let (x, y) = clamp_taskbar_popup_position(host, x, y, 220, 190);
    menu.set(Some(TaskbarWindowContextMenuState { window_id, x, y }));
}

fn clamp_taskbar_popup_position(
    host: DesktopHostContext,
    x: i32,
    y: i32,
    popup_w: i32,
    popup_h: i32,
) -> (i32, i32) {
    let viewport = host.desktop_viewport_rect(TASKBAR_HEIGHT_PX);
    let max_x = (viewport.w - popup_w - 6).max(6);
    let max_y = (viewport.h + TASKBAR_HEIGHT_PX - popup_h - 6).max(6);
    (x.clamp(6, max_x), y.clamp(6, max_y))
}

fn build_taskbar_tray_widgets(state: &DesktopState) -> Vec<TaskbarTrayWidget> {
    let total_windows = state.windows.len();
    let minimized_windows = state.windows.iter().filter(|win| win.minimized).count();
    let dialup_online = state
        .windows
        .iter()
        .any(|win| apps::is_dialup_application_id(&win.app_id) && !win.minimized);

    vec![
        TaskbarTrayWidget {
            id: "win-count",
            icon: IconName::WindowMultiple,
            label: "Open windows",
            value: total_windows.to_string(),
            pressed: None,
            action: TaskbarTrayWidgetAction::None,
        },
        TaskbarTrayWidget {
            id: "bg-count",
            icon: IconName::DesktopArrowDown,
            label: "Minimized windows",
            value: minimized_windows.to_string(),
            pressed: None,
            action: TaskbarTrayWidgetAction::None,
        },
        TaskbarTrayWidget {
            id: "network",
            icon: if dialup_online {
                IconName::WifiOn
            } else {
                IconName::WifiOff
            },
            label: "Network status",
            value: if dialup_online { "ON" } else { "IDLE" }.to_string(),
            pressed: Some(dialup_online),
            action: TaskbarTrayWidgetAction::None,
        },
        TaskbarTrayWidget {
            id: "contrast",
            icon: if state.theme.high_contrast {
                IconName::Checkmark
            } else {
                IconName::Dismiss
            },
            label: "High contrast",
            value: if state.theme.high_contrast {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
            pressed: Some(state.theme.high_contrast),
            action: TaskbarTrayWidgetAction::ToggleHighContrast,
        },
        TaskbarTrayWidget {
            id: "motion",
            icon: if state.theme.reduced_motion {
                IconName::MotionOff
            } else {
                IconName::MotionOn
            },
            label: "Reduced motion",
            value: if state.theme.reduced_motion {
                "ON"
            } else {
                "OFF"
            }
            .to_string(),
            pressed: Some(state.theme.reduced_motion),
            action: TaskbarTrayWidgetAction::ToggleReducedMotion,
        },
    ]
}

fn activate_taskbar_tray_widget(runtime: DesktopRuntimeContext, action: TaskbarTrayWidgetAction) {
    match action {
        TaskbarTrayWidgetAction::None => {}
        TaskbarTrayWidgetAction::ToggleHighContrast => {
            let enabled = runtime.state.get_untracked().theme.high_contrast;
            runtime.dispatch_action(DesktopAction::SetHighContrast { enabled: !enabled });
        }
        TaskbarTrayWidgetAction::ToggleReducedMotion => {
            let enabled = runtime.state.get_untracked().theme.reduced_motion;
            runtime.dispatch_action(DesktopAction::SetReducedMotion { enabled: !enabled });
        }
    }
}

fn format_taskbar_clock_time(snapshot: TaskbarClockSnapshot, config: TaskbarClockConfig) -> String {
    if config.use_24_hour {
        format!(
            "{:02}:{:02}:{:02}",
            snapshot.hour, snapshot.minute, snapshot.second
        )
    } else {
        let mut hour = snapshot.hour % 12;
        if hour == 0 {
            hour = 12;
        }
        let suffix = if snapshot.hour >= 12 { "PM" } else { "AM" };
        format!(
            "{:02}:{:02}:{:02} {}",
            hour, snapshot.minute, snapshot.second, suffix
        )
    }
}

fn format_taskbar_clock_date(snapshot: TaskbarClockSnapshot) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        snapshot.year, snapshot.month, snapshot.day
    )
}

fn format_taskbar_clock_aria(snapshot: TaskbarClockSnapshot, config: TaskbarClockConfig) -> String {
    let time_text = format_taskbar_clock_time(snapshot, config);
    if config.show_date {
        format!("{}, {}", format_taskbar_clock_date(snapshot), time_text)
    } else {
        time_text
    }
}

fn stop_mouse_event(ev: &web_sys::MouseEvent) {
    ev.prevent_default();
    ev.stop_propagation();
}

fn pointer_from_pointer_event(ev: &web_sys::PointerEvent) -> PointerPosition {
    PointerPosition {
        x: ev.client_x(),
        y: ev.client_y(),
    }
}

fn end_active_pointer_interaction(runtime: DesktopRuntimeContext) {
    let interaction = runtime.interaction.get_untracked();
    if interaction.dragging.is_some() {
        runtime.dispatch_action(DesktopAction::EndMoveWithViewport {
            viewport: runtime
                .host
                .get_value()
                .desktop_viewport_rect(TASKBAR_HEIGHT_PX),
        });
    }
    if interaction.resizing.is_some() {
        runtime.dispatch_action(DesktopAction::EndResize);
    }
}

fn resize_edge_class(edge: ResizeEdge) -> &'static str {
    match edge {
        ResizeEdge::North => "edge-n",
        ResizeEdge::South => "edge-s",
        ResizeEdge::East => "edge-e",
        ResizeEdge::West => "edge-w",
        ResizeEdge::NorthEast => "edge-ne",
        ResizeEdge::NorthWest => "edge-nw",
        ResizeEdge::SouthEast => "edge-se",
        ResizeEdge::SouthWest => "edge-sw",
    }
}
