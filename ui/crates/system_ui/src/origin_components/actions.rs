use leptos::callback::Callable;
use leptos::ev::KeyboardEvent;
use leptos::prelude::*;
use leptos::web_sys;

use crate::foundation::{
    ButtonShape, ButtonSize, ButtonVariant, ControlTone, bool_token, merge_layout_class,
};
use crate::origin_primitives::{Icon, IconName, IconSize};

#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Standard)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(default = ButtonShape::Standard)] shape: ButtonShape,
    #[prop(default = ControlTone::Neutral)] control_tone: ControlTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] aria_controls: Signal<String>,
    #[prop(optional, into)] aria_expanded: Signal<bool>,
    #[prop(optional, into)] aria_haspopup: Signal<String>,
    #[prop(optional, into)] aria_checked: Signal<String>,
    #[prop(optional, into)] aria_pressed: Signal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional, into)] data_app: Signal<String>,
    #[prop(optional)] tabindex: Option<i32>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] selected: Signal<bool>,
    #[prop(optional, into)] pressed: Signal<bool>,
    #[prop(optional)] leading_icon: Option<IconName>,
    #[prop(optional)] trailing_icon: Option<IconName>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_dblclick: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<leptos::ev::MouseEvent>>,
    #[prop(optional)] on_pointerdown: Option<Callback<web_sys::PointerEvent>>,
    children: Children,
) -> impl IntoView {
    let class = merge_layout_class("ui-button", layout_class);
    view! {
        <button
            type="button"
            class=class
            id=id
            role=role
            aria-label=move || aria_label.get()
            aria-controls=move || aria_controls.get()
            aria-expanded=move || aria_expanded.get()
            aria-haspopup=move || aria_haspopup.get()
            aria-checked=move || aria_checked.get()
            aria-pressed=move || aria_pressed.get()
            aria-keyshortcuts=move || aria_keyshortcuts.get()
            title=move || title.get()
            data-app=move || data_app.get()
            tabindex=tabindex
            disabled=move || disabled.get()
            data-origin-component="button"
            data-ui-primitive="true"
            data-ui-kind="button"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-size=size.token()
            data-ui-shape=shape.token()
            data-ui-control-tone=control_tone.token()
            data-ui-state=move || {
                if pressed.get() {
                    "pressed"
                } else if selected.get() {
                    "selected"
                } else {
                    "idle"
                }
            }
            data-ui-selected=move || bool_token(selected.get())
            data-ui-pressed=move || bool_token(pressed.get())
            data-ui-disabled=move || bool_token(disabled.get())
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            }
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
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.run(ev);
                }
            }
            on:contextmenu=move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.run(ev);
                }
            }
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.run(ev);
                }
            }
        >
            {leading_icon.map(|icon| view! { <Icon icon size=IconSize::Sm /> })}
            {children()}
            {trailing_icon.map(|icon| view! { <Icon icon size=IconSize::Sm /> })}
        </button>
    }
}

#[component]
pub fn IconButton(
    icon: IconName,
    #[prop(default = ButtonVariant::Icon)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(default = ControlTone::Neutral)] control_tone: ControlTone,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Signal<String>,
    #[prop(optional, into)] title: Signal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] disabled: Signal<bool>,
    #[prop(optional, into)] pressed: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<leptos::ev::MouseEvent>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=merge_layout_class("ui-icon-button", layout_class)
            aria-label=move || aria_label.get()
            title=move || title.get()
            disabled=move || disabled.get()
            data-origin-component="icon-button"
            data-ui-primitive="true"
            data-ui-kind="icon-button"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-size=size.token()
            data-ui-shape=ButtonShape::Circle.token()
            data-ui-control-tone=control_tone.token()
            data-ui-pressed=move || bool_token(pressed.get())
            data-ui-disabled=move || bool_token(disabled.get())
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.run(ev);
                }
            }
        >
            <Icon icon size=IconSize::Md />
        </button>
    }
}
