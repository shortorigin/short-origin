use leptos::ev::{FocusEvent, KeyboardEvent};
use leptos::html;
use leptos::*;

use crate::foundation::{bool_token, merge_layout_class, FieldVariant};

#[component]
pub fn TextField(
    #[prop(default = FieldVariant::Standard)] variant: FieldVariant,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] node_ref: NodeRef<html::Input>,
    #[prop(optional)] autocomplete: Option<&'static str>,
    #[prop(optional)] spellcheck: Option<bool>,
    #[prop(optional)] input_type: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] value: MaybeSignal<String>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_focus: Option<Callback<FocusEvent>>,
    #[prop(optional)] on_blur: Option<Callback<FocusEvent>>,
) -> impl IntoView {
    view! {
        <input
            class=merge_layout_class("ui-field", layout_class)
            id=id
            title=title
            placeholder=placeholder
            aria-label=aria_label
            node_ref=node_ref
            autocomplete=autocomplete
            spellcheck=spellcheck
            type=input_type.unwrap_or("text")
            prop:value=move || value.get()
            disabled=move || disabled.get()
            data-origin-primitive="text-field"
            data-ui-primitive="true"
            data-ui-kind="text-field"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-disabled=move || bool_token(disabled.get())
            on:input=move |ev| {
                if let Some(on_input) = on_input.as_ref() {
                    on_input.call(ev);
                }
            }
            on:keydown=move |ev| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown.call(ev);
                }
            }
            on:focus=move |ev| {
                if let Some(on_focus) = on_focus.as_ref() {
                    on_focus.call(ev);
                }
            }
            on:blur=move |ev| {
                if let Some(on_blur) = on_blur.as_ref() {
                    on_blur.call(ev);
                }
            }
        />
    }
}

#[component]
pub fn CheckboxField(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] checked: MaybeSignal<bool>,
    #[prop(optional)] on_change: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
    view! {
        <input
            class=merge_layout_class("ui-checkbox", layout_class)
            type="checkbox"
            aria-label=move || aria_label.get()
            prop:checked=move || checked.get()
            data-origin-primitive="checkbox-field"
            data-ui-primitive="true"
            data-ui-kind="checkbox"
            data-ui-selected=move || bool_token(checked.get())
            on:change=move |ev| {
                if let Some(on_change) = on_change.as_ref() {
                    on_change.call(ev);
                }
            }
        />
    }
}
