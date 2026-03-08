use leptos::*;

use crate::foundation::{
    bool_token, merge_layout_class, ButtonSize, ButtonVariant, Elevation, SurfaceVariant,
};

#[component]
pub fn WindowSurface(
    #[prop(default = SurfaceVariant::Modal)] variant: SurfaceVariant,
    #[prop(default = Elevation::Modal)] elevation: Elevation,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] style: MaybeSignal<String>,
    #[prop(optional, into)] focused: MaybeSignal<bool>,
    #[prop(optional, into)] minimized: MaybeSignal<bool>,
    #[prop(optional, into)] maximized: MaybeSignal<bool>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
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
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            data-ui-focused=move || bool_token(focused.get())
            data-ui-minimized=move || bool_token(minimized.get())
            data-ui-maximized=move || bool_token(maximized.get())
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.call(ev);
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
            data-ui-elevation=Elevation::Raised.token()
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.call(ev);
                }
            }
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.call(ev);
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
            data-ui-kind="resize-handle-region"
            data-ui-slot=edge
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
            data-ui-variant=SurfaceVariant::Standard.token()
            data-ui-elevation=Elevation::Embedded.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn WindowControlButton(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
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
            ui_slot="window-control"
            on_pointerdown=Callback::new(move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.call(ev);
                }
            })
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.call(ev);
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
