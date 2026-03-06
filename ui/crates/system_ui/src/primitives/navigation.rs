use super::*;

#[component]
/// Shared menubar primitive.
pub fn MenuBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = LayoutGap::Sm)] gap: LayoutGap,
    #[prop(default = LayoutPadding::Sm)] padding: LayoutPadding,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-menubar", layout_class)
            data-ui-primitive="true"
            data-ui-kind="menubar"
            data-ui-variant="standard"
            data-ui-gap=gap.token()
            data-ui-padding=padding.token()
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared toolbar primitive.
pub fn ToolBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = LayoutGap::Sm)] gap: LayoutGap,
    #[prop(default = LayoutPadding::Sm)] padding: LayoutPadding,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-toolbar", layout_class)
            data-ui-primitive="true"
            data-ui-kind="toolbar"
            data-ui-variant="standard"
            data-ui-gap=gap.token()
            data-ui-padding=padding.token()
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared status bar primitive.
pub fn StatusBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = LayoutGap::Sm)] gap: LayoutGap,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-statusbar", layout_class)
            data-ui-primitive="true"
            data-ui-kind="statusbar"
            data-ui-variant="standard"
            data-ui-gap=gap.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared disclosure panel for secondary or advanced controls.
pub fn DisclosurePanel(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional, into)] expanded: MaybeSignal<bool>,
    #[prop(optional)] on_toggle: Option<Callback<MouseEvent>>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-disclosure", layout_class)
            data-ui-primitive="true"
            data-ui-kind="disclosure"
            data-ui-state=move || if expanded.get() { "open" } else { "closed" }
            data-ui-expanded=move || bool_token(expanded.get())
        >
            <Button
                layout_class="ui-disclosure-toggle"
                ui_slot="toggle"
                variant=ButtonVariant::Quiet
                selected=expanded
                aria_label=title.to_string()
                on_click=Callback::new(move |ev| {
                    if let Some(on_toggle) = on_toggle.as_ref() {
                        on_toggle.call(ev);
                    }
                })
            >
                <span data-ui-slot="copy">
                    <span data-ui-slot="title">{title}</span>
                    {description.map(|description| view! { <span data-ui-slot="description">{description}</span> })}
                </span>
                <span data-ui-slot="indicator" aria-hidden="true">
                    {move || if expanded.get() { "Hide" } else { "Show" }}
                </span>
            </Button>
            <Show when=move || expanded.get() fallback=|| ()>
                <div data-ui-slot="body">{children()}</div>
            </Show>
        </section>
    }
}

#[component]
/// Root container for a guided multi-step flow.
pub fn StepFlow(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-step-flow", layout_class)
            data-ui-primitive="true"
            data-ui-kind="step-flow"
        >
            {children()}
        </section>
    }
}

#[component]
/// Header section for a guided multi-step flow.
pub fn StepFlowHeader(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
) -> impl IntoView {
    view! {
        <header
            class=merge_layout_class("ui-step-flow-header", layout_class)
            data-ui-primitive="true"
            data-ui-kind="step-flow-header"
        >
            <div data-ui-slot="title">{title}</div>
            {description.map(|description| view! { <div data-ui-slot="description">{description}</div> })}
        </header>
    }
}

#[component]
/// Individual step block within a guided flow.
pub fn StepFlowStep(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(into)] status: MaybeSignal<StepStatus>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-step-flow-step", layout_class)
            data-ui-primitive="true"
            data-ui-kind="step-flow-step"
            data-ui-state=move || status.get().token()
        >
            <div data-ui-slot="header">
                <span data-ui-slot="badge">{move || status.get().token()}</span>
                <div data-ui-slot="copy">
                    <div data-ui-slot="title">{title}</div>
                    {description.map(|description| view! { <div data-ui-slot="description">{description}</div> })}
                </div>
            </div>
            <div data-ui-slot="body">{children()}</div>
        </section>
    }
}

#[component]
/// Shared action row for guided flows.
pub fn StepFlowActions(
    #[prop(default = LayoutJustify::Between)] justify: LayoutJustify,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-step-flow-actions", layout_class)
            data-ui-primitive="true"
            data-ui-kind="step-flow-actions"
            data-ui-justify=justify.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared tab list primitive.
pub fn TabList(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-tab-list", layout_class)
            data-ui-primitive="true"
            data-ui-kind="tab-list"
            role="tablist"
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared tab trigger primitive.
pub fn Tab(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(into)] id: MaybeSignal<String>,
    #[prop(into)] controls: MaybeSignal<String>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    #[prop(into)] tabindex: MaybeSignal<i32>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
            layout_class=layout_class.unwrap_or("")
            id=id.get()
            role="tab".to_string()
            aria_controls=controls.get()
            selected=selected
            tabindex=tabindex.get()
            ui_slot="tab"
            variant=ButtonVariant::Quiet
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
                }
            })
            on_keydown=Callback::new(move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
                }
            })
        >
            {children()}
        </Button>
    }
}

#[component]
/// Shared launcher menu wrapper.
pub fn LauncherMenu(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <MenuSurface layout_class=layout_class.unwrap_or("") role="menu".to_string() aria_label="Application launcher".to_string()>
            {children()}
        </MenuSurface>
    }
}
