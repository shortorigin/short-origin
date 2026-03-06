use super::*;

#[component]
/// Shared overlay surface for menus and popups.
pub fn MenuSurface(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] style: MaybeSignal<String>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-menu-surface", layout_class)
            id=id
            role=role
            aria-label=move || aria_label.get()
            style=move || style.get()
            data-ui-primitive="true"
            data-ui-kind="menu-surface"
            data-ui-slot=ui_slot
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
                }
            }
            on:mousedown=move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.call(ev);
                }
            }
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
                }
            }
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared overlay menu item primitive.
pub fn MenuItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(default = ButtonVariant::Quiet)] variant: ButtonVariant,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] aria_checked: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    #[prop(optional)] on_mousedown: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
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
            on_mousedown=Callback::new(move |ev| {
                if let Some(on_mousedown) = on_mousedown.as_ref() {
                    on_mousedown.call(ev);
                }
            })
            on_contextmenu=Callback::new(move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.call(ev);
                }
            })
            on_click=Callback::new(move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
                }
            })
        >
            {children()}
        </Button>
    }
}

#[component]
/// Shared modal overlay surface.
pub fn Modal(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] style: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-modal", layout_class)
            id=id
            role=role.unwrap_or_else(|| "dialog".to_string())
            aria-label=move || aria_label.get()
            style=move || style.get()
            data-ui-primitive="true"
            data-ui-kind="modal"
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared overlay menu separator.
pub fn MenuSeparator(#[prop(optional)] layout_class: Option<&'static str>) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-menu-separator", layout_class)
            role="separator"
            aria-hidden="true"
            data-ui-primitive="true"
            data-ui-kind="menu-separator"
        ></div>
    }
}
