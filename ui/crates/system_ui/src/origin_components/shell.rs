use leptos::callback::Callable;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

use crate::foundation::{
    ButtonVariant, ControlTone, ElevationRole, SurfaceRole, merge_layout_class,
};

#[component]
pub fn AppShell(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-app-shell", layout_class)
            data-origin-component="app-shell"
            data-ui-primitive="true"
            data-ui-kind="app-shell"
            data-ui-surface-role=SurfaceRole::Shell.token()
            data-ui-elevation-role=ElevationRole::Embedded.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DesktopBackdrop(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("desktop-backdrop", layout_class)
            data-origin-component="desktop-backdrop"
            data-ui-primitive="true"
            data-ui-kind="desktop-backdrop"
            data-ui-surface-role=SurfaceRole::Shell.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DesktopIconGrid(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-desktop-icon-grid", layout_class)
            data-origin-component="desktop-icon-grid"
            data-ui-primitive="true"
            data-ui-kind="desktop-icon-grid"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DesktopIcon(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_dblclick: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=merge_layout_class("ui-desktop-icon-button", layout_class)
            title=title
            aria-label=aria_label
            data-origin-component="desktop-icon"
            data-ui-primitive="true"
            data-ui-kind="desktop-icon-button"
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            }
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.run(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn DesktopWindowLayer(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-window-layer", layout_class)
            data-origin-component="desktop-window-layer"
            data-ui-primitive="true"
            data-ui-kind="desktop-window-layer"
            data-ui-elevation-role=ElevationRole::Raised.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Taskbar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] aria_keyshortcuts: Option<String>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <footer
            class=merge_layout_class("ui-taskbar", layout_class)
            data-origin-component="taskbar"
            data-ui-primitive="true"
            data-ui-kind="taskbar"
            data-ui-surface-role=SurfaceRole::Taskbar.token()
            data-ui-elevation-role=ElevationRole::Embedded.token()
            role=role
            aria-label=aria_label
            aria-keyshortcuts=aria_keyshortcuts
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
        >
            {children()}
        </footer>
    }
}

#[component]
pub fn TaskbarSection(
    ui_slot: &'static str,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-taskbar-section", layout_class)
            data-origin-component="taskbar-section"
            data-ui-primitive="true"
            data-ui-kind="taskbar-section"
            data-ui-slot=ui_slot
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TaskbarButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_controls: Signal<String>,
    #[prop(optional, into)] aria_haspopup: Signal<String>,
    #[prop(optional, into)] aria_expanded: Signal<bool>,
    #[prop(optional, into)] aria_pressed: Signal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: Signal<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] data_app: Signal<String>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional, into)] pressed: Signal<bool>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            id=id.unwrap_or_default()
            aria_controls=aria_controls
            aria_haspopup=aria_haspopup
            aria_expanded=aria_expanded
            aria_pressed=aria_pressed
            aria_keyshortcuts=aria_keyshortcuts
            aria_label=aria_label
            title=title
            data_app=data_app
            selected=selected
            pressed=pressed
            ui_slot=ui_slot.unwrap_or("taskbar-button")
            variant=ButtonVariant::Quiet
            control_tone=ControlTone::Neutral
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            })
            on_contextmenu=Callback::new(move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.run(ev);
                }
            })
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </crate::origin_components::Button>
    }
}

#[component]
pub fn TaskbarOverflowButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] aria_controls: Signal<String>,
    #[prop(optional, into)] aria_haspopup: Signal<String>,
    #[prop(optional, into)] aria_expanded: Signal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: Signal<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <TaskbarButton
            layout_class=layout_class.unwrap_or("")
            id=id.unwrap_or_default()
            aria_label=aria_label
            aria_controls=aria_controls
            aria_haspopup=aria_haspopup
            aria_expanded=aria_expanded
            aria_keyshortcuts=aria_keyshortcuts
            ui_slot="taskbar-overflow-button"
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </TaskbarButton>
    }
}

#[component]
pub fn Dock(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] aria_keyshortcuts: Option<String>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <footer
            class=merge_layout_class("ui-dock", layout_class)
            data-origin-component="dock"
            data-ui-primitive="true"
            data-ui-kind="dock"
            data-ui-surface-role=SurfaceRole::Taskbar.token()
            data-ui-elevation-role=ElevationRole::Floating.token()
            role=role
            aria-label=aria_label
            aria-keyshortcuts=aria_keyshortcuts
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
        >
            {children()}
        </footer>
    }
}

#[component]
pub fn DockSection(
    ui_slot: &'static str,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-dock-section", layout_class)
            data-origin-component="dock-section"
            data-ui-primitive="true"
            data-ui-kind="dock-section"
            data-ui-slot=ui_slot
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DockButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_controls: Signal<String>,
    #[prop(optional, into)] aria_haspopup: Signal<String>,
    #[prop(optional, into)] aria_expanded: Signal<bool>,
    #[prop(optional, into)] aria_pressed: Signal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: Signal<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] data_app: Signal<String>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional, into)] pressed: Signal<bool>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            id=id.unwrap_or_default()
            aria_controls=aria_controls
            aria_haspopup=aria_haspopup
            aria_expanded=aria_expanded
            aria_pressed=aria_pressed
            aria_keyshortcuts=aria_keyshortcuts
            aria_label=aria_label
            title=title
            data_app=data_app
            selected=selected
            pressed=pressed
            ui_slot=ui_slot.unwrap_or("dock-button")
            variant=ButtonVariant::Quiet
            control_tone=ControlTone::Neutral
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            })
            on_contextmenu=Callback::new(move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.run(ev);
                }
            })
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </crate::origin_components::Button>
    }
}

#[component]
pub fn SystemTray(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-tray-list", layout_class)
            data-origin-component="system-tray"
            data-ui-primitive="true"
            data-ui-kind="tray-list"
            data-ui-control-tone=ControlTone::Neutral.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TrayButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] pressed: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <TaskbarButton
            layout_class=layout_class.unwrap_or("")
            aria_label=aria_label
            title=title
            pressed=pressed
            ui_slot="tray-button"
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </TaskbarButton>
    }
}

#[component]
pub fn ClockButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] aria_controls: Signal<String>,
    #[prop(optional, into)] aria_haspopup: Signal<String>,
    #[prop(optional, into)] aria_expanded: Signal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: Signal<String>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <TaskbarButton
            layout_class=layout_class.unwrap_or("")
            id=id.unwrap_or_default()
            aria_label=aria_label
            aria_controls=aria_controls
            aria_haspopup=aria_haspopup
            aria_expanded=aria_expanded
            aria_keyshortcuts=aria_keyshortcuts
            ui_slot="clock-button"
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </TaskbarButton>
    }
}

#[component]
pub fn LauncherPanel(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] style: Signal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-launcher-panel", layout_class)
            id=id
            role="menu"
            aria-label=move || aria_label.get()
            style=move || style.get()
            data-origin-component="launcher-panel"
            data-ui-primitive="true"
            data-ui-kind="launcher-panel"
            data-ui-surface-role=SurfaceRole::Menu.token()
            data-ui-elevation-role=ElevationRole::Floating.token()
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn SidePanel(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <aside
            class=merge_layout_class("ui-side-panel", layout_class)
            id=id
            role="complementary"
            aria-label=move || aria_label.get()
            data-origin-component="side-panel"
            data-ui-primitive="true"
            data-ui-kind="side-panel"
            data-ui-slot=ui_slot
            data-ui-surface-role=SurfaceRole::Menu.token()
            data-ui-elevation-role=ElevationRole::Floating.token()
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
        >
            {children()}
        </aside>
    }
}

#[component]
pub fn NotificationCenter(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <aside
            class=merge_layout_class("ui-notification-center", layout_class)
            id=id
            role="complementary"
            aria-label=move || aria_label.get()
            data-origin-component="notification-center"
            data-ui-primitive="true"
            data-ui-kind="notification-center"
            data-ui-slot="notification-center"
            data-ui-surface-role=SurfaceRole::Menu.token()
            data-ui-elevation-role=ElevationRole::Floating.token()
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
        >
            {children()}
        </aside>
    }
}

#[component]
pub fn QuickSettingTile(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            aria_label=aria_label
            title=title
            selected=selected
            ui_slot="quick-setting-tile"
            variant=ButtonVariant::Quiet
            control_tone=ControlTone::Accent
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            })
        >
            {children()}
        </crate::origin_components::Button>
    }
}

#[component]
pub fn SystemOverlay(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] visible: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-system-overlay", layout_class)
            hidden=move || !visible.get()
            data-origin-component="system-overlay"
            data-ui-primitive="true"
            data-ui-kind="system-overlay"
        >
            {children()}
        </div>
    }
}
