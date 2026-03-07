use leptos::*;

pub use crate::legacy_primitives::Modal as ModalOverlay;

#[component]
pub fn ContextMenu(children: Children) -> impl IntoView {
    view! { <crate::legacy_primitives::MenuSurface>{children()}</crate::legacy_primitives::MenuSurface> }
}

#[component]
pub fn Launcher(children: Children) -> impl IntoView {
    view! {
        <crate::legacy_primitives::MenuSurface ui_slot="launcher-menu">
            {children()}
        </crate::legacy_primitives::MenuSurface>
    }
}
