use leptos::ev::KeyboardEvent;
use leptos::*;

use crate::foundation::{merge_layout_class, ButtonVariant};

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
            data-ui-variant="standard"
            data-ui-elevation="flat"
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
                    on_click.call(ev);
                }
            }
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.call(ev);
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
            role=role
            aria-label=aria_label
            aria-keyshortcuts=aria_keyshortcuts
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.call(ev);
                }
            }
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
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
    #[prop(optional, into)] aria_controls: MaybeSignal<String>,
    #[prop(optional, into)] aria_haspopup: MaybeSignal<String>,
    #[prop(optional, into)] aria_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] aria_pressed: MaybeSignal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: MaybeSignal<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] data_app: MaybeSignal<String>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    #[prop(optional, into)] pressed: MaybeSignal<bool>,
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
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.call(ev);
                }
            })
            on_contextmenu=Callback::new(move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.call(ev);
                }
            })
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
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
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] aria_controls: MaybeSignal<String>,
    #[prop(optional, into)] aria_haspopup: MaybeSignal<String>,
    #[prop(optional, into)] aria_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: MaybeSignal<String>,
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
                    on_click.call(ev);
                }
            })
        >
            {children()}
        </TaskbarButton>
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
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TrayButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] pressed: MaybeSignal<bool>,
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
                    on_click.call(ev);
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
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] aria_controls: MaybeSignal<String>,
    #[prop(optional, into)] aria_haspopup: MaybeSignal<String>,
    #[prop(optional, into)] aria_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: MaybeSignal<String>,
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
                    on_click.call(ev);
                }
            })
        >
            {children()}
        </TaskbarButton>
    }
}
