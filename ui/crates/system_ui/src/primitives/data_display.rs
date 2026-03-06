use super::*;

#[component]
/// Generic surface primitive.
pub fn Surface(
    #[prop(default = SurfaceVariant::Standard)] variant: SurfaceVariant,
    #[prop(default = Elevation::Flat)] elevation: Elevation,
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
/// Generic panel primitive.
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
/// Shared card surface for option tiles, summaries, and document-like regions.
pub fn Card(
    #[prop(default = SurfaceVariant::Standard)] variant: SurfaceVariant,
    #[prop(default = Elevation::Raised)] elevation: Elevation,
    #[prop(default = LayoutPadding::Md)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <article
            class=merge_layout_class("ui-card", layout_class)
            data-ui-primitive="true"
            data-ui-kind="card"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-elevation=elevation.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </article>
    }
}

#[component]
/// Visual elevation layer wrapper.
pub fn ElevationLayer(
    #[prop(default = Elevation::Raised)] elevation: Elevation,
    #[prop(default = LayoutPadding::Sm)] padding: LayoutPadding,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-layer", layout_class)
            data-ui-primitive="true"
            data-ui-kind="layer"
            data-ui-slot=ui_slot
            data-ui-elevation=elevation.token()
            data-ui-padding=padding.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared text primitive.
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
/// Shared heading primitive.
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

#[component]
/// Compact status badge primitive.
pub fn Badge(
    #[prop(default = TextTone::Secondary)] tone: TextTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=merge_layout_class("ui-badge", layout_class)
            data-ui-primitive="true"
            data-ui-kind="badge"
            data-ui-tone=tone.token()
        >
            {children()}
        </span>
    }
}

#[component]
/// Empty state content block.
pub fn EmptyState(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-empty-state", layout_class)
            data-ui-primitive="true"
            data-ui-kind="empty-state"
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared pane surface.
pub fn Pane(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = SurfaceVariant::Standard)] variant: SurfaceVariant,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class=merge_layout_class("ui-pane", layout_class)
            data-ui-primitive="true"
            data-ui-kind="pane"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </section>
    }
}

#[component]
/// Shared pane header with title and optional supporting copy/actions.
pub fn PaneHeader(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] meta: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    let title_signal = Signal::derive(move || title.get());
    let meta_signal = Signal::derive(move || meta.get());
    view! {
        <header
            class=merge_layout_class("ui-pane-header", layout_class)
            data-ui-primitive="true"
            data-ui-kind="pane-header"
        >
            <div data-ui-slot="copy">
                <Show when=move || !title_signal.get().is_empty() fallback=|| ()>
                    <div data-ui-slot="title">{move || title_signal.get()}</div>
                </Show>
                <Show when=move || !meta_signal.get().is_empty() fallback=|| ()>
                    <div data-ui-slot="meta">{move || meta_signal.get()}</div>
                </Show>
            </div>
            <div data-ui-slot="actions">{children()}</div>
        </header>
    }
}

#[component]
/// Shared inline statusbar item wrapper.
pub fn StatusBarItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <span
            class=merge_layout_class("ui-statusbar-item", layout_class)
            data-ui-primitive="true"
            data-ui-kind="statusbar-item"
        >
            {children()}
        </span>
    }
}

#[component]
/// Shared list surface.
pub fn ListSurface(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-list-surface", layout_class)
            data-ui-primitive="true"
            data-ui-kind="list-surface"
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared table primitive.
pub fn DataTable(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] tabindex: MaybeSignal<i32>,
    #[prop(optional, into)] aria_activedescendant: MaybeSignal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <table
            class=merge_layout_class("ui-data-table", layout_class)
            data-ui-primitive="true"
            data-ui-kind="data-table"
            role=role
            aria-label=aria_label
            tabindex=move || tabindex.get()
            aria-activedescendant=move || aria_activedescendant.get()
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
                }
            }
        >
            {children()}
        </table>
    }
}

#[component]
/// Shared tree container.
pub fn Tree(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <ul
            class=merge_layout_class("ui-tree", layout_class)
            data-ui-primitive="true"
            data-ui-kind="tree"
        >
            {children()}
        </ul>
    }
}

#[component]
/// Shared tree item surface.
pub fn TreeItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <li
            class=merge_layout_class("ui-tree-item", layout_class)
            data-ui-primitive="true"
            data-ui-kind="tree-item"
            data-ui-selected=move || bool_token(selected.get())
        >
            {children()}
        </li>
    }
}

#[component]
/// Shared key/value inspector grid.
pub fn InspectorGrid(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-inspector-grid", layout_class)
            data-ui-primitive="true"
            data-ui-kind="inspector-grid"
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared option card for selections and presets.
pub fn OptionCard(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-option-card", layout_class)
            data-ui-primitive="true"
            data-ui-kind="option-card"
            data-ui-selected=move || bool_token(selected.get())
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared preview frame.
pub fn PreviewFrame(
    #[prop(optional)] layout_class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-preview-frame", layout_class)
            data-ui-primitive="true"
            data-ui-kind="preview-frame"
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared terminal surface root.
pub fn TerminalSurface(
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
            data-ui-primitive="true"
            data-ui-kind="terminal-surface"
            node_ref=node_ref
            role=role
            aria-live=aria_live
            on:scroll=move |ev| {
                if let Some(on_scroll) = on_scroll.as_ref() {
                    on_scroll.call(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared terminal transcript container.
pub fn TerminalTranscript(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-transcript", layout_class)
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
/// Shared terminal line surface.
pub fn TerminalLine(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = TextTone::Primary)] tone: TextTone,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-line", layout_class)
            data-ui-primitive="true"
            data-ui-kind="terminal-line"
            data-ui-tone=tone.token()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared terminal prompt row.
pub fn TerminalPrompt(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-terminal-prompt", layout_class)
            data-ui-primitive="true"
            data-ui-kind="terminal-prompt"
            role=role
        >
            {children()}
        </div>
    }
}
