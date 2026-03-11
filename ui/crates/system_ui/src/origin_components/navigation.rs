use leptos::callback::Callable;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;

use crate::foundation::{
    ButtonVariant, ControlTone, ElevationRole, LayoutGap, LayoutJustify, LayoutPadding,
    SurfaceRole, bool_token, merge_layout_class,
};

#[component]
pub fn Toolbar(
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
            role=role
            aria-label=aria_label
            data-origin-component="toolbar"
            data-ui-primitive="true"
            data-ui-kind="toolbar"
            data-ui-surface-role=SurfaceRole::WindowInactive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
            data-ui-gap=gap.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn StatusBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = LayoutGap::Sm)] gap: LayoutGap,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-statusbar", layout_class)
            data-origin-component="status-bar"
            data-ui-primitive="true"
            data-ui-kind="statusbar"
            data-ui-surface-role=SurfaceRole::WindowInactive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
            data-ui-gap=gap.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn StatusBarItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=merge_layout_class("ui-statusbar-item", layout_class)
            data-origin-component="status-bar-item"
            data-ui-primitive="true"
            data-ui-kind="statusbar-item"
        >
            {children()}
        </span>
    }
}

#[component]
pub fn MenuSurface(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] style: Signal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-menu-surface", layout_class)
            id=id
            role=role
            aria-label=move || aria_label.get()
            style=move || style.get()
            data-origin-component="menu-surface"
            data-ui-primitive="true"
            data-ui-kind="menu-surface"
            data-ui-surface-role=SurfaceRole::Menu.token()
            data-ui-elevation-role=ElevationRole::Floating.token()
            data-ui-slot=ui_slot
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            }
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn MenuItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(default = ButtonVariant::Quiet)] variant: ButtonVariant,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] aria_checked: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            id=id.unwrap_or_default()
            role=role.unwrap_or_default()
            aria_label=aria_label
            title=title
            aria_checked=aria_checked
            disabled=disabled
            selected=selected
            ui_slot="menu-item"
            variant=variant
            control_tone=ControlTone::Neutral
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.run(ev);
                }
            })
            on_contextmenu=Callback::new(move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.run(ev);
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

#[component]
pub fn MenuSeparator(#[prop(optional)] layout_class: Option<&'static str>) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-menu-separator", layout_class)
            role="separator"
            aria-hidden="true"
            data-origin-component="menu-separator"
            data-ui-primitive="true"
            data-ui-kind="menu-separator"
        ></div>
    }
}

#[component]
pub fn DisclosurePanel(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional, into)] expanded: Signal<bool>,
    #[prop(optional)] on_toggle: Option<Callback<leptos::ev::MouseEvent>>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-disclosure", layout_class)
            data-origin-component="disclosure-panel"
            data-ui-primitive="true"
            data-ui-kind="disclosure"
            data-ui-surface-role=SurfaceRole::WindowInactive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
            data-ui-state=move || if expanded.get() { "open" } else { "closed" }
            data-ui-expanded=move || bool_token(expanded.get())
        >
            <crate::origin_components::Button
                layout_class="ui-disclosure-toggle"
                ui_slot="toggle"
                variant=ButtonVariant::Quiet
                control_tone=ControlTone::Neutral
                selected=expanded
                aria_label=title.to_string()
                on_click=Callback::new(move |ev| {
                    if let Some(on_toggle) = on_toggle.as_ref() {
                        on_toggle.run(ev);
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
            </crate::origin_components::Button>
            <Show when=move || expanded.get() fallback=|| ()>
                <div data-ui-slot="body">{children()}</div>
            </Show>
        </section>
    }
}

#[component]
pub fn StepFlow(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-step-flow", layout_class)
            data-origin-component="step-flow"
            data-ui-primitive="true"
            data-ui-kind="step-flow"
        >
            {children()}
        </section>
    }
}

#[component]
pub fn StepFlowHeader(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
) -> impl IntoView {
    view! {
        <header
            class=merge_layout_class("ui-step-flow-header", layout_class)
            data-origin-component="step-flow-header"
            data-ui-primitive="true"
            data-ui-kind="step-flow-header"
        >
            <div data-ui-slot="title">{title}</div>
            {description.map(|description| view! { <div data-ui-slot="description">{description}</div> })}
        </header>
    }
}

#[component]
pub fn StepFlowStep(
    #[prop(optional)] layout_class: Option<&'static str>,
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(into)] status: Signal<StepStatus>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-step-flow-step", layout_class)
            data-origin-component="step-flow-step"
            data-ui-primitive="true"
            data-ui-kind="step-flow-step"
            data-ui-surface-role=SurfaceRole::WindowInactive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Current,
    Complete,
    Pending,
    Error,
}

impl StepStatus {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Complete => "complete",
            Self::Pending => "pending",
            Self::Error => "error",
        }
    }
}

#[component]
pub fn StepFlowActions(
    #[prop(default = LayoutJustify::Between)] justify: LayoutJustify,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-step-flow-actions", layout_class)
            data-origin-component="step-flow-actions"
            data-ui-primitive="true"
            data-ui-kind="step-flow-actions"
            data-ui-justify=justify.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn ToggleRow(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] checked: Signal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <label
            class=merge_layout_class("ui-toggle-row", layout_class)
            data-origin-component="toggle-row"
            data-ui-primitive="true"
            data-ui-kind="toggle-row"
            data-ui-surface-role=SurfaceRole::WindowInactive.token()
            data-ui-elevation-role=ElevationRole::Raised.token()
            data-ui-selected=move || bool_token(checked.get())
        >
            <span data-ui-slot="copy">
                {title.map(|title| view! { <span data-ui-slot="title">{title}</span> })}
                {description.map(|description| view! { <span data-ui-slot="description">{description}</span> })}
            </span>
            <span data-ui-slot="control">{children()}</span>
        </label>
    }
}
