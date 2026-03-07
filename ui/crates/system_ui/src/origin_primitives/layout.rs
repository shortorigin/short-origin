use leptos::*;

use crate::foundation::{merge_layout_class, LayoutAlign, LayoutGap, LayoutJustify, LayoutPadding};

#[component]
pub fn Inline(
    #[prop(default = LayoutGap::Md)] gap: LayoutGap,
    #[prop(default = LayoutAlign::Center)] align: LayoutAlign,
    #[prop(default = LayoutJustify::Start)] justify: LayoutJustify,
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-inline", layout_class)
            data-origin-primitive="inline"
            data-ui-primitive="true"
            data-ui-kind="inline"
            data-ui-gap=gap.token()
            data-ui-align=align.token()
            data-ui-justify=justify.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Center(
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-center", layout_class)
            data-origin-primitive="center"
            data-ui-primitive="true"
            data-ui-kind="center"
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Inset(
    #[prop(default = LayoutPadding::Md)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("origin-inset", layout_class)
            data-origin-primitive="inset"
            data-ui-primitive="true"
            data-ui-kind="inset"
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}
