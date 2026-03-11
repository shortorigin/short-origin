use leptos::callback::Callable;
use leptos::prelude::*;
use leptos::web_sys;

use crate::foundation::{
    ButtonSize, ButtonVariant, ControlTone, ElevationRole, SurfaceRole, bool_token,
    merge_layout_class,
};

#[component]
pub fn WindowSurface(
    #[prop(default = SurfaceRole::Modal)] surface_role: SurfaceRole,
    #[prop(default = ElevationRole::Modal)] elevation_role: ElevationRole,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] style: Signal<String>,
    #[prop(optional, into)] focused: Signal<bool>,
    #[prop(optional, into)] minimized: Signal<bool>,
    #[prop(optional, into)] maximized: Signal<bool>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("origin-window-surface", layout_class)
            style=move || style.get()
            role="dialog"
            aria-label=move || aria_label.get()
            data-origin-primitive="window-surface"
            data-ui-primitive="true"
            data-ui-kind="window-surface"
            data-ui-surface-role=surface_role.token()
            data-ui-elevation-role=elevation_role.token()
            data-ui-focused=move || bool_token(focused.get())
            data-ui-minimized=move || bool_token(minimized.get())
            data-ui-maximized=move || bool_token(maximized.get())
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.run(ev);
                }
            }
        >
            {children()}
        </section>
    }
}

#[component]
pub fn TitlebarRegion(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = SurfaceRole::WindowActive)] surface_role: SurfaceRole,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    #[prop(optional)] on_dblclick: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <header
            class=merge_layout_class("origin-titlebar-region", layout_class)
            data-origin-primitive="titlebar-region"
            data-ui-primitive="true"
            data-ui-kind="titlebar-region"
            data-ui-surface-role=surface_role.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.run(ev);
                }
            }
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.run(ev);
                }
            }
        >
            {children()}
        </header>
    }
}

#[component]
pub fn ResizeHandleRegion(
    edge: &'static str,
    #[prop(optional)] layout_class: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-resize-handle-region", layout_class)
            data-origin-primitive="resize-handle-region"
            data-ui-primitive="true"
            data-ui-kind="resize-handle"
            data-ui-slot=edge
            aria-hidden="true"
        />
    }
}

#[component]
pub fn WindowTitle(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-window-title", layout_class)
            data-origin-primitive="window-title"
            data-ui-primitive="true"
            data-ui-kind="window-title"
            data-ui-variant="label"
        >
            {children()}
        </div>
    }
}

#[component]
pub fn WindowBody(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-window-body", layout_class)
            data-origin-primitive="window-body"
            data-ui-primitive="true"
            data-ui-kind="window-body"
            data-ui-surface-role=SurfaceRole::WindowActive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn WindowControlButton(
    #[prop(default = ControlTone::Neutral)] control_tone: ControlTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            aria_label=aria_label
            title=title
            disabled=disabled
            variant=ButtonVariant::Quiet
            size=ButtonSize::Sm
            control_tone=control_tone
            ui_slot="window-control"
            on_pointerdown=Callback::new(move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.run(ev);
                }
            })
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
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
