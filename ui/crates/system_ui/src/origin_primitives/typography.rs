use leptos::prelude::*;

use crate::foundation::{TextRole, TextTone, merge_layout_class};

#[component]
pub fn Text(
    #[prop(default = TextRole::Body)] role: TextRole,
    #[prop(default = TextTone::Primary)] tone: TextTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=merge_layout_class("ui-text", layout_class)
            data-origin-primitive="text"
            data-ui-primitive="true"
            data-ui-kind="text"
            data-ui-slot=ui_slot
            data-ui-variant=role.token()
            data-ui-tone=tone.token()
        >
            {children()}
        </span>
    }
}

#[component]
pub fn Heading(
    #[prop(default = TextRole::Title)] role: TextRole,
    #[prop(default = TextTone::Primary)] tone: TextTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-heading", layout_class)
            data-origin-primitive="heading"
            data-ui-primitive="true"
            data-ui-kind="heading"
            data-ui-slot=ui_slot
            data-ui-variant=role.token()
            data-ui-tone=tone.token()
        >
            {children()}
        </div>
    }
}
