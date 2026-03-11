use leptos::prelude::*;

use crate::foundation::{ElevationRole, LayoutPadding, SurfaceRole, merge_layout_class};

#[component]
pub fn Layer(
    #[prop(default = ElevationRole::Floating)] elevation_role: ElevationRole,
    #[prop(default = SurfaceRole::Menu)] surface_role: SurfaceRole,
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
            data-ui-surface-role=surface_role.token()
            data-ui-elevation-role=elevation_role.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Viewport(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-viewport", layout_class)
            data-origin-primitive="viewport"
            data-ui-primitive="true"
            data-ui-kind="viewport"
            data-ui-surface-role=SurfaceRole::Shell.token()
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}
