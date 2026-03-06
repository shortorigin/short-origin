#![allow(clippy::clone_on_copy)]

use std::rc::Rc;

use desktop_app_contract::AppCommandRegistration;
use leptos::SignalGetUntracked;
use system_shell_contract::{CommandArgSpec, CommandDataShape, CommandOutputShape};

use crate::{components::DesktopRuntimeContext, model::DesktopSkin, reducer::DesktopAction};

pub(super) fn registrations(runtime: DesktopRuntimeContext) -> Vec<AppCommandRegistration> {
    vec![
        theme_show_registration(runtime.clone()),
        theme_set_skin_registration(runtime.clone()),
        theme_set_high_contrast_registration(runtime.clone()),
        theme_set_reduced_motion_registration(runtime),
    ]
}

fn theme_show_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "theme show",
            &[],
            "Show current theme state.",
            "theme show",
            Vec::new(),
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Record),
        ),
        completion: None,
        handler: Rc::new(move |_| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let theme = runtime.state.get_untracked().theme;
                Ok(system_shell_contract::CommandResult {
                    output: super::super::record_data(vec![
                        super::super::string_field("skin", theme.skin.css_id()),
                        super::super::bool_field("high_contrast", theme.high_contrast),
                        super::super::bool_field("reduced_motion", theme.reduced_motion),
                        super::super::bool_field("audio_enabled", theme.audio_enabled),
                    ]),
                    display: system_shell_contract::DisplayPreference::Record,
                    notices: Vec::new(),
                    cwd: None,
                    exit: system_shell_contract::ShellExit::success(),
                })
            })
        }),
    }
}

fn theme_set_skin_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "theme set skin",
            &[],
            "Set the desktop skin.",
            "theme set skin <soft-neumorphic|modern-adaptive|classic-xp|classic-95>",
            vec![CommandArgSpec {
                name: "skin".to_string(),
                summary: "Desktop skin id.".to_string(),
                required: true,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Empty),
        ),
        completion: None,
        handler: Rc::new(move |context| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let skin = match context.args.first().map(String::as_str) {
                    Some("soft-neumorphic") => DesktopSkin::SoftNeumorphic,
                    Some("modern-adaptive") => DesktopSkin::ModernAdaptive,
                    Some("classic-xp") => DesktopSkin::ClassicXp,
                    Some("classic-95") => DesktopSkin::Classic95,
                    Some(other) => {
                        return Err(super::super::usage_error(format!("unknown skin `{other}`")))
                    }
                    None => return Err(super::super::usage_error("usage: theme set skin <skin>")),
                };
                runtime.dispatch_action(DesktopAction::SetSkin { skin });
                Ok(super::super::info_result(format!(
                    "skin set to {}",
                    skin.css_id()
                )))
            })
        }),
    }
}

fn theme_flag_registration(
    runtime: DesktopRuntimeContext,
    path: &'static str,
    summary: &'static str,
    builder: fn(bool) -> DesktopAction,
) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            path,
            &[],
            summary,
            &format!("{path} <on|off>"),
            vec![CommandArgSpec {
                name: "value".to_string(),
                summary: "Use on or off.".to_string(),
                required: true,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Empty),
        ),
        completion: None,
        handler: Rc::new(move |context| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let raw = context
                    .args
                    .first()
                    .ok_or_else(|| super::super::usage_error(format!("usage: {path} <on|off>")))?;
                let value = super::super::parse_bool_flag(raw)?;
                runtime.dispatch_action(builder(value));
                Ok(super::super::info_result(format!("{path} {raw}")))
            })
        }),
    }
}

fn theme_set_high_contrast_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    theme_flag_registration(
        runtime,
        "theme set high-contrast",
        "Set high-contrast mode.",
        |enabled| DesktopAction::SetHighContrast { enabled },
    )
}

fn theme_set_reduced_motion_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    theme_flag_registration(
        runtime,
        "theme set reduced-motion",
        "Set reduced-motion mode.",
        |enabled| DesktopAction::SetReducedMotion { enabled },
    )
}
