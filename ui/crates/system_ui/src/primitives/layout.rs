use super::*;

#[component]
/// Vertical layout stack.
pub fn Stack(
    #[prop(default = LayoutGap::Md)] gap: LayoutGap,
    #[prop(default = LayoutAlign::Stretch)] align: LayoutAlign,
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-stack", layout_class)
            data-ui-primitive="true"
            data-ui-kind="stack"
            data-ui-slot=ui_slot
            data-ui-gap=gap.token()
            data-ui-align=align.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Horizontal wrapping cluster.
pub fn Cluster(
    #[prop(default = LayoutGap::Md)] gap: LayoutGap,
    #[prop(default = LayoutAlign::Center)] align: LayoutAlign,
    #[prop(default = LayoutJustify::Start)] justify: LayoutJustify,
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-cluster", layout_class)
            data-ui-primitive="true"
            data-ui-kind="cluster"
            data-ui-slot=ui_slot
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
/// Grid layout primitive.
pub fn Grid(
    #[prop(default = LayoutGap::Md)] gap: LayoutGap,
    #[prop(default = LayoutPadding::None)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-grid", layout_class)
            data-ui-primitive="true"
            data-ui-kind="grid"
            data-ui-slot=ui_slot
            data-ui-gap=gap.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Split layout for two or three pane surfaces.
pub fn SplitLayout(
    #[prop(default = LayoutGap::Md)] gap: LayoutGap,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional)] tabindex: Option<i32>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-split-layout", layout_class)
            data-ui-primitive="true"
            data-ui-kind="split-layout"
            data-ui-slot=ui_slot
            data-ui-gap=gap.token()
            tabindex=tabindex
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
                }
            }
        >
            {children()}
        </div>
    }
}
