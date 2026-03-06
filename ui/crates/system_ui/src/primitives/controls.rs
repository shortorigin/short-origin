use super::*;

fn clamp_percent(value: &str, min: Option<&str>, max: Option<&str>) -> f32 {
    let value = value.parse::<f32>().unwrap_or(0.0);
    let min = min.and_then(|raw| raw.parse::<f32>().ok()).unwrap_or(0.0);
    let max = max.and_then(|raw| raw.parse::<f32>().ok()).unwrap_or(100.0);
    let span = (max - min).max(1.0);
    (((value - min) / span) * 100.0).clamp(0.0, 100.0)
}

#[component]
/// Shared button primitive with standardized states, icon slots, and semantic shape tokens.
pub fn Button(
    #[prop(default = ButtonVariant::Standard)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(default = ButtonShape::Standard)] shape: ButtonShape,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] aria_controls: MaybeSignal<String>,
    #[prop(optional, into)] aria_expanded: MaybeSignal<bool>,
    #[prop(optional, into)] aria_haspopup: MaybeSignal<String>,
    #[prop(optional, into)] aria_checked: MaybeSignal<String>,
    #[prop(optional, into)] aria_pressed: MaybeSignal<bool>,
    #[prop(optional, into)] aria_keyshortcuts: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional, into)] data_app: MaybeSignal<String>,
    #[prop(optional)] tabindex: Option<i32>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    #[prop(optional, into)] pressed: MaybeSignal<bool>,
    #[prop(optional)] leading_icon: Option<IconName>,
    #[prop(optional)] trailing_icon: Option<IconName>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
    #[prop(optional)] on_mousedown: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_dblclick: Option<Callback<MouseEvent>>,
    #[prop(optional)] on_contextmenu: Option<Callback<MouseEvent>>,
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
            data-ui-primitive="true"
            data-ui-kind="button"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-size=size.token()
            data-ui-shape=shape.token()
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
                    on_click.call(ev);
                }
            }
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
            on:dblclick=move |ev| {
                if let Some(on_dblclick) = on_dblclick.as_ref() {
                    on_dblclick.call(ev);
                }
            }
            on:contextmenu=move |ev| {
                if let Some(on_contextmenu) = on_contextmenu.as_ref() {
                    on_contextmenu.call(ev);
                }
            }
            on:pointerdown=move |ev| {
                if let Some(on_pointerdown) = on_pointerdown.as_ref() {
                    on_pointerdown.call(ev);
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
/// Shared circular icon button used for transport controls and compact surface actions.
pub fn IconButton(
    icon: IconName,
    #[prop(default = ButtonVariant::Icon)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Md)] size: ButtonSize,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional, into)] title: MaybeSignal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional, into)] pressed: MaybeSignal<bool>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=merge_layout_class("ui-icon-button", layout_class)
            aria-label=move || aria_label.get()
            title=move || title.get()
            disabled=move || disabled.get()
            data-ui-primitive="true"
            data-ui-kind="icon-button"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            data-ui-size=size.token()
            data-ui-shape=ButtonShape::Circle.token()
            data-ui-pressed=move || bool_token(pressed.get())
            data-ui-disabled=move || bool_token(disabled.get())
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
                }
            }
        >
            <Icon icon size=IconSize::Md />
        </button>
    }
}

#[component]
/// Shared pill-style segmented control container.
pub fn SegmentedControl(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-segmented-control", layout_class)
            role="group"
            aria-label=move || aria_label.get()
            data-ui-primitive="true"
            data-ui-kind="segmented-control"
            data-ui-slot=ui_slot
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared segmented control option button.
pub fn SegmentedControlOption(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] selected: MaybeSignal<bool>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional, into)] pressed: MaybeSignal<bool>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class=merge_layout_class("ui-segmented-control-option", layout_class)
            aria-label=move || aria_label.get()
            disabled=move || disabled.get()
            data-ui-primitive="true"
            data-ui-kind="segmented-control-option"
            data-ui-slot=ui_slot
            data-ui-variant=ButtonVariant::Segmented.token()
            data-ui-selected=move || bool_token(selected.get())
            data-ui-pressed=move || bool_token(pressed.get())
            data-ui-disabled=move || bool_token(disabled.get())
            on:click=move |ev| {
                if let Some(on_click) = on_click.as_ref() {
                    on_click.call(ev);
                }
            }
        >
            {children()}
        </button>
    }
}

#[component]
/// Shared labeled field wrapper that keeps copy and control structure on the primitive layer.
pub fn FieldGroup(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <label
            class=merge_layout_class("ui-field-group", layout_class)
            data-ui-primitive="true"
            data-ui-kind="field-group"
        >
            <span data-ui-slot="copy">
                {title.map(|title| view! { <span data-ui-slot="title">{title}</span> })}
                {description.map(|description| view! { <span data-ui-slot="description">{description}</span> })}
            </span>
            <span data-ui-slot="control">{children()}</span>
        </label>
    }
}

#[component]
/// Shared text input primitive.
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
/// Shared multiline text area primitive.
pub fn TextArea(
    #[prop(default = FieldVariant::Inset)] variant: FieldVariant,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] spellcheck: Option<&'static str>,
    #[prop(optional)] autocomplete: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] value: MaybeSignal<String>,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
    #[prop(optional)] on_keydown: Option<Callback<KeyboardEvent>>,
) -> impl IntoView {
    view! {
        <textarea
            class=merge_layout_class("ui-textarea", layout_class)
            id=id
            aria-label=aria_label
            spellcheck=spellcheck.unwrap_or("false")
            autocomplete=autocomplete.unwrap_or("off")
            prop:value=move || value.get()
            data-ui-primitive="true"
            data-ui-kind="text-area"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
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
        ></textarea>
    }
}

#[component]
/// Shared select-field primitive.
pub fn SelectField(
    #[prop(default = FieldVariant::Standard)] variant: FieldVariant,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] value: MaybeSignal<String>,
    #[prop(optional)] on_change: Option<Callback<web_sys::Event>>,
    children: Children,
) -> impl IntoView {
    view! {
        <select
            class=merge_layout_class("ui-field", layout_class)
            aria-label=aria_label
            prop:value=move || value.get()
            data-ui-primitive="true"
            data-ui-kind="select"
            data-ui-slot=ui_slot
            data-ui-variant=variant.token()
            on:change=move |ev| {
                if let Some(on_change) = on_change.as_ref() {
                    on_change.call(ev);
                }
            }
        >
            {children()}
        </select>
    }
}

#[component]
/// Shared range-field primitive with a percent CSS hook for active-track styling.
pub fn RangeField(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] min: Option<&'static str>,
    #[prop(optional)] max: Option<&'static str>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] value: MaybeSignal<String>,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
    let value_signal = Signal::derive(move || value.get());
    let percent = Signal::derive(move || clamp_percent(&value_signal.get(), min, max));

    view! {
        <input
            class=merge_layout_class("ui-field", layout_class)
            type="range"
            min=min
            max=max
            aria-label=aria_label
            prop:value=move || value_signal.get()
            data-ui-primitive="true"
            data-ui-kind="range"
            data-ui-slot=ui_slot
            data-ui-variant="standard"
            data-ui-value=move || value_signal.get()
            data-ui-min=min.unwrap_or("0")
            data-ui-max=max.unwrap_or("100")
            data-ui-percent=move || format!("{:.2}", percent.get())
            on:input=move |ev| {
                if let Some(on_input) = on_input.as_ref() {
                    on_input.call(ev);
                }
            }
        />
    }
}

#[component]
/// Shared color-field primitive.
pub fn ColorField(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] value: MaybeSignal<String>,
    #[prop(optional)] on_input: Option<Callback<web_sys::Event>>,
) -> impl IntoView {
    view! {
        <input
            class=merge_layout_class("ui-field", layout_class)
            type="color"
            aria-label=aria_label
            prop:value=move || value.get()
            data-ui-primitive="true"
            data-ui-kind="color-field"
            data-ui-slot=ui_slot
            data-ui-variant="standard"
            on:input=move |ev| {
                if let Some(on_input) = on_input.as_ref() {
                    on_input.call(ev);
                }
            }
        />
    }
}

#[component]
/// Shared checkbox input for settings and binary preferences.
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

#[component]
/// Shared neumorphic switch with explicit `role="switch"` semantics.
pub fn Switch(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] checked: MaybeSignal<bool>,
    #[prop(optional, into)] disabled: MaybeSignal<bool>,
    #[prop(optional)] on_toggle: Option<Callback<bool>>,
) -> impl IntoView {
    let handle_toggle = move || {
        if disabled.get_untracked() {
            return;
        }
        if let Some(on_toggle) = on_toggle.as_ref() {
            on_toggle.call(!checked.get_untracked());
        }
    };

    view! {
        <button
            type="button"
            class=merge_layout_class("ui-switch", layout_class)
            role="switch"
            aria-label=move || aria_label.get()
            aria-checked=move || checked.get().to_string()
            disabled=move || disabled.get()
            data-ui-primitive="true"
            data-ui-kind="switch"
            data-ui-slot=ui_slot
            data-ui-selected=move || bool_token(checked.get())
            data-ui-disabled=move || bool_token(disabled.get())
            on:click=move |_| handle_toggle()
            on:keydown=move |ev| match ev.key().as_str() {
                " " | "Enter" => {
                    ev.prevent_default();
                    handle_toggle();
                }
                _ => {}
            }
        >
            <span data-ui-slot="track">
                <span data-ui-slot="thumb"></span>
            </span>
        </button>
    }
}

#[component]
/// Shared linear progress indicator.
pub fn ProgressBar(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(default = ProgressVariant::Standard)] _variant: ProgressVariant,
    #[prop(optional)] ui_slot: Option<&'static str>,
    max: u16,
    value: u16,
) -> impl IntoView {
    let capped_value = value.min(max);

    view! {
        <progress
            class=merge_layout_class("ui-progress", layout_class)
            max=max
            value=capped_value
            data-ui-primitive="true"
            data-ui-kind="progress"
            data-ui-slot=ui_slot
            data-ui-variant="linear"
            data-ui-value=capped_value
            data-ui-max=max
        ></progress>
    }
}

#[component]
/// Shared circular progress ring with an optional center label.
pub fn CircularProgress(
    value: u16,
    max: u16,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let radius = 24.0f32;
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let clamped = value.min(max);
    let progress = if max == 0 {
        0.0
    } else {
        clamped as f32 / max as f32
    };
    let dash_offset = circumference * (1.0 - progress);

    view! {
        <div
            class=merge_layout_class("ui-progress-ring", layout_class)
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax=max
            aria-valuenow=clamped
            data-ui-primitive="true"
            data-ui-kind="progress-ring"
            data-ui-slot=ui_slot
            data-ui-value=clamped
            data-ui-max=max
        >
            <svg viewBox="0 0 64 64" aria-hidden="true">
                <circle data-ui-slot="track" cx="32" cy="32" r=radius></circle>
                <circle
                    data-ui-slot="fill"
                    cx="32"
                    cy="32"
                    r=radius
                    stroke-dasharray=circumference
                    stroke-dashoffset=dash_offset
                ></circle>
            </svg>
            {label.map(|label| view! { <span data-ui-slot="label">{label}</span> })}
        </div>
    }
}

#[component]
/// Shared showcase-ready dial primitive with radial ticks and a pointer needle.
pub fn KnobDial(
    value: i32,
    #[prop(optional)] min: Option<i32>,
    #[prop(optional)] max: Option<i32>,
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] ui_slot: Option<&'static str>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] on_change: Option<Callback<i32>>,
) -> impl IntoView {
    let min = min.unwrap_or(0);
    let max = max.unwrap_or(100).max(min + 1);
    let clamped = value.clamp(min, max);
    let progress = (clamped - min) as f32 / (max - min) as f32;
    let rotation = -135.0 + progress * 270.0;

    let ticks = (0..15)
        .map(|index| {
            let tick_rotation = -135.0 + (index as f32 / 14.0) * 270.0;
            let selected = tick_rotation <= rotation + 1.0;
            view! {
                <line
                    data-ui-slot="tick"
                    data-ui-selected=bool_token(selected)
                    x1="50"
                    y1="11"
                    x2="50"
                    y2="21"
                    transform=format!("rotate({tick_rotation} 50 50)")
                ></line>
            }
        })
        .collect_view();

    let emit_delta = move |delta: i32| {
        if let Some(on_change) = on_change.as_ref() {
            on_change.call((clamped + delta).clamp(min, max));
        }
    };

    view! {
        <button
            type="button"
            class=merge_layout_class("ui-knob-dial", layout_class)
            aria-label=aria_label
            data-ui-primitive="true"
            data-ui-kind="knob-dial"
            data-ui-slot=ui_slot
            data-ui-value=clamped
            data-ui-min=min
            data-ui-max=max
            on:keydown=move |ev| match ev.key().as_str() {
                "ArrowLeft" | "ArrowDown" => {
                    ev.prevent_default();
                    emit_delta(-1);
                }
                "ArrowRight" | "ArrowUp" => {
                    ev.prevent_default();
                    emit_delta(1);
                }
                "PageDown" => {
                    ev.prevent_default();
                    emit_delta(-10);
                }
                "PageUp" => {
                    ev.prevent_default();
                    emit_delta(10);
                }
                "Home" => {
                    ev.prevent_default();
                    if let Some(on_change) = on_change.as_ref() {
                        on_change.call(min);
                    }
                }
                "End" => {
                    ev.prevent_default();
                    if let Some(on_change) = on_change.as_ref() {
                        on_change.call(max);
                    }
                }
                _ => {}
            }
        >
            <svg viewBox="0 0 100 100" aria-hidden="true">
                <g data-ui-slot="ticks">{ticks}</g>
                <circle data-ui-slot="dial-face" cx="50" cy="50" r="34"></circle>
                <circle data-ui-slot="inner-ring" cx="50" cy="50" r="24"></circle>
                <line
                    data-ui-slot="needle"
                    x1="50"
                    y1="50"
                    x2="50"
                    y2="24"
                    transform=format!("rotate({rotation} 50 50)")
                ></line>
                <circle data-ui-slot="needle-cap" cx="50" cy="50" r="4"></circle>
            </svg>
            <span data-ui-slot="value">{clamped}</span>
        </button>
    }
}

#[component]
/// Shared completion list item.
pub fn CompletionItem(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional)] on_click: Option<Callback<MouseEvent>>,
    children: Children,
) -> impl IntoView {
    view! {
        <Button
            layout_class=layout_class.unwrap_or("")
            ui_slot="completion-item"
            variant=ButtonVariant::Quiet
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
/// Shared completion list surface.
pub fn CompletionList(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] role: Option<String>,
    #[prop(optional, into)] aria_label: MaybeSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=merge_layout_class("ui-completion-list", layout_class)
            data-ui-primitive="true"
            data-ui-kind="completion-list"
            role=role
            aria-label=move || aria_label.get()
        >
            {children()}
        </div>
    }
}

#[component]
/// Shared labeled toggle row.
pub fn ToggleRow(
    #[prop(optional)] layout_class: Option<&'static str>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] checked: MaybeSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <label
            class=merge_layout_class("ui-toggle-row", layout_class)
            data-ui-primitive="true"
            data-ui-kind="toggle-row"
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
