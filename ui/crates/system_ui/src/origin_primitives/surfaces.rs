use leptos::*;

use crate::foundation::{merge_layout_class, Elevation, LayoutPadding};

#[component]
pub fn Layer(
    #[prop(default = Elevation::Overlay)] elevation: Elevation,
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-layer", layout_class)
            data-origin-primitive="layer"
            data-ui-primitive="true"
            data-ui-kind="layer"
            data-ui-elevation=elevation.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Viewport(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-viewport", layout_class)
            data-origin-primitive="viewport"
            data-ui-primitive="true"
            data-ui-kind="viewport"
        >
            {children()}
        </div>
    }
}
