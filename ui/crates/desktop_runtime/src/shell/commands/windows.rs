#![allow(clippy::clone_on_copy)]

use std::rc::Rc;

use desktop_app_contract::AppCommandRegistration;
use leptos::SignalGetUntracked;
use system_shell_contract::{CommandArgSpec, CommandDataShape, CommandOutputShape};

use crate::{components::DesktopRuntimeContext, model::WindowId, reducer::DesktopAction};

pub(super) fn registrations(runtime: DesktopRuntimeContext) -> Vec<AppCommandRegistration> {
    vec![
        windows_list_registration(runtime.clone(), "windows list", "List open windows."),
        windows_focus_registration(runtime.clone()),
        windows_close_registration(runtime.clone()),
        windows_minimize_registration(runtime.clone()),
        windows_restore_registration(runtime),
    ]
}

fn windows_list_registration(
    runtime: DesktopRuntimeContext,
    path: &'static str,
    summary: &'static str,
) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            path,
            &[],
            summary,
            path,
            Vec::new(),
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Table),
        ),
        completion: None,
        handler: Rc::new(move |_| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let windows = runtime.state.get_untracked().windows;
                Ok(system_shell_contract::CommandResult {
                    output: super::super::table_data(
                        vec![
                            "id".to_string(),
                            "app_id".to_string(),
                            "title".to_string(),
                            "focused".to_string(),
                            "minimized".to_string(),
                            "maximized".to_string(),
                        ],
                        windows.iter().map(super::super::window_row).collect(),
                        Some(system_shell_contract::CommandPath::new(path)),
                    ),
                    display: system_shell_contract::DisplayPreference::Table,
                    notices: Vec::new(),
                    cwd: None,
                    exit: system_shell_contract::ShellExit::success(),
                })
            })
        }),
    }
}

fn simple_window_registration(
    runtime: DesktopRuntimeContext,
    path: &'static str,
    summary: &'static str,
    builder: fn(WindowId) -> DesktopAction,
) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            path,
            &[],
            summary,
            &format!("{path} <window-id>"),
            vec![CommandArgSpec {
                name: "window-id".to_string(),
                summary: "Runtime window identifier.".to_string(),
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
                let raw = context.args.first().ok_or_else(|| {
                    super::super::usage_error(format!("usage: {path} <window-id>"))
                })?;
                let window_id = super::super::parse_window_id(raw)?;
                runtime.dispatch_action(builder(window_id));
                Ok(super::super::info_result(format!("{path} {}", window_id.0)))
            })
        }),
    }
}

fn windows_focus_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    simple_window_registration(runtime, "windows focus", "Focus a window.", |window_id| {
        DesktopAction::FocusWindow { window_id }
    })
}

fn windows_close_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    simple_window_registration(runtime, "windows close", "Close a window.", |window_id| {
        DesktopAction::CloseWindow { window_id }
    })
}

fn windows_minimize_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    simple_window_registration(
        runtime,
        "windows minimize",
        "Minimize a window.",
        |window_id| DesktopAction::MinimizeWindow { window_id },
    )
}

fn windows_restore_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    simple_window_registration(
        runtime,
        "windows restore",
        "Restore a window.",
        |window_id| DesktopAction::RestoreWindow { window_id },
    )
}
