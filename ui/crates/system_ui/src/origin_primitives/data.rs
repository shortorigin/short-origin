use leptos::callback::Callable;
use leptos::ev::{KeyboardEvent, MouseEvent};
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys;

use crate::foundation::{
    ButtonVariant, Elevation, LayoutAlign, LayoutGap, LayoutJustify, LayoutPadding, SurfaceVariant,
    TextTone, merge_layout_class,
};

#[component]
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
            data-origin-primitive="stack"
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
            data-origin-primitive="cluster"
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
            data-origin-primitive="grid"
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
pub fn Surface(
    #[prop(default = SurfaceVariant::Standard)] variant: SurfaceVariant,
    #[prop(default = Elevation::Embedded)] elevation: Elevation,
    #[prop(default = LayoutPadding::Md)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-surface", layout_class)
            data-origin-primitive="surface"
            data-ui-primitive="true"
            data-ui-kind="surface"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            data-ui-padding=padding.token()
            role=role
            aria-label=aria_label
        >
            {children()}
        </div>
    }
}

#[component]
pub fn Panel(
    #[prop(default = SurfaceVariant::Standard)] variant: SurfaceVariant,
    #[prop(default = Elevation::Raised)] elevation: Elevation,
    #[prop(default = LayoutPadding::Md)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-panel", layout_class)
            data-origin-primitive="panel"
            data-ui-primitive="true"
            data-ui-kind="panel"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            data-ui-padding=padding.token()
            role=role
            aria-label=aria_label
        >
            {children()}
        </section>
    }
}

#[component]
pub fn ListSurface(
    #[prop(default = SurfaceVariant::Muted)] variant: SurfaceVariant,
    #[prop(default = Elevation::Raised)] elevation: Elevation,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-list-surface", layout_class)
            data-origin-primitive="list-surface"
            data-ui-primitive="true"
            data-ui-kind="list-surface"
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn DataTable(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] tabindex: Signal<i32>,
    #[prop(optional, into)] aria_activedescendant: Signal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <table
            class=merge_layout_class("ui-data-table", layout_class)
            data-origin-primitive="data-table"
            data-ui-primitive="true"
            data-ui-kind="data-table"
            role=role
            aria-label=aria_label
            tabindex=move || tabindex.get()
            aria-activedescendant=move || aria_activedescendant.get()
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.run(ev);
                }
            }
        >
            {children()}
        </table>
    }
}

#[component]
pub fn CompletionItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <crate::origin_components::Button
            layout_class=layout_class.unwrap_or("")
            ui_slot="completion-item"
            variant=ButtonVariant::Quiet
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
pub fn CompletionList(
    #[prop(default = Elevation::Transient)] elevation: Elevation,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-completion-list", layout_class)
            data-origin-primitive="completion-list"
            data-ui-primitive="true"
            data-ui-kind="completion-list"
            data-ui-elevation=elevation.token()
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TerminalSurface(
    #[prop(default = SurfaceVariant::Inset)] variant: SurfaceVariant,
    #[prop(default = Elevation::Embedded)] elevation: Elevation,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] node_ref: NodeRef<html::Div>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_live: Option<&'static str>,
    #[prop(optional)] on_scroll: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-surface", layout_class)
            data-origin-primitive="terminal-surface"
            data-ui-primitive="true"
            data-ui-kind="terminal-surface"
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            node_ref=node_ref
            role=role
            aria-live=aria_live
            on:scroll=move |ev| {
                if let Some(on_scroll) = on_scroll.as_ref() {
                    on_scroll.run(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TerminalTranscript(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-transcript", layout_class)
            data-origin-primitive="terminal-transcript"
            data-ui-primitive="true"
            data-ui-kind="terminal-transcript"
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TerminalLine(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = TextTone::Primary)] tone: TextTone,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-line", layout_class)
            data-origin-primitive="terminal-line"
            data-ui-primitive="true"
            data-ui-kind="terminal-line"
            data-ui-tone=tone.token()
        >
            {children()}
        </div>
    }
}

#[component]
pub fn TerminalPrompt(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-prompt", layout_class)
            data-origin-primitive="terminal-prompt"
            data-ui-primitive="true"
            data-ui-kind="terminal-prompt"
            role=role
        >
            {children()}
        </div>
    }
}
