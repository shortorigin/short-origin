use leptos::*;

use crate::foundation::{bool_token, merge_layout_class};

#[component]
pub fn WindowFrame(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] style: MaybeSignal<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] focused: MaybeSignal<bool>,
    #[prop(optional, into)] minimized: MaybeSignal<bool>,
    #[prop(optional, into)] maximized: MaybeSignal<bool>,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-window-frame", layout_class)
            style=move || style.get()
            role="dialog"
            aria-label=move || aria_label.get()
            data-origin-component="window-frame"
            data-ui-primitive="true"
            data-ui-kind="window-frame"
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
pub fn WindowTitleBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    #[prop(optional)] on_dblclick: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <header
            class=merge_layout_class("ui-window-titlebar", layout_class)
            data-origin-component="window-titlebar"
            data-ui-primitive="true"
            data-ui-kind="window-titlebar"
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
pub fn WindowControls(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-window-controls", layout_class)
            data-origin-component="window-controls"
            data-ui-primitive="true"
            data-ui-kind="window-controls"
        >
            {children()}
        </div>
    }
}
