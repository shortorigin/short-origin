#![allow(clippy::clone_on_copy)]

use std::rc::Rc;

use desktop_app_contract::AppCommandRegistration;
use system_shell_contract::{
    CommandArgSpec, CommandDataShape, CommandOutputShape, CompletionRequest,
};

use crate::{apps, components::DesktopRuntimeContext, reducer::DesktopAction};

pub(super) fn registrations(runtime: DesktopRuntimeContext) -> Vec<AppCommandRegistration> {
    vec![apps_list_registration(), apps_open_registration(runtime)]
}

fn apps_list_registration() -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "apps list",
            &[],
            "List registered apps.",
            "apps list",
            Vec::new(),
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Table),
        ),
        completion: None,
        handler: Rc::new(|_| {
            Box::pin(async move {
                Ok(system_shell_contract::CommandResult {
                    output: super::super::table_data(
                        vec![
                            "app_id".to_string(),
                            "label".to_string(),
                            "single_instance".to_string(),
                        ],
                        apps::app_registry()
                            .iter()
                            .cloned()
                            .map(super::super::app_row)
                            .collect(),
                        Some(system_shell_contract::CommandPath::new("apps list")),
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

fn apps_open_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "apps open",
            &[],
            "Open an app by canonical id.",
            "apps open <app-id>",
            vec![CommandArgSpec {
                name: "app-id".to_string(),
                summary: "Canonical app id.".to_string(),
                required: true,
                repeatable: false,
            }],
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Empty),
        ),
        completion: Some(Rc::new(|request: CompletionRequest| {
            Box::pin(async move { Ok(super::super::open_completion(request)) })
        })),
        handler: Rc::new(move |context| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let target = context
                    .args
                    .first()
                    .ok_or_else(|| super::super::usage_error("usage: apps open <app-id>"))?;
                let Some(mut action) = super::super::resolve_open_target(target) else {
                    return Err(system_shell_contract::ShellError::new(
                        system_shell_contract::ShellErrorCode::NotFound,
                        format!("unknown app `{target}`"),
                    ));
                };
                if let DesktopAction::ActivateApp {
                    ref mut viewport, ..
                } = action
                {
                    *viewport = Some(
                        runtime
                            .host
                            .get_value()
                            .desktop_viewport_rect(super::super::TASKBAR_HEIGHT_PX),
                    );
                }
                runtime.dispatch_action(action);
                Ok(super::super::info_result(format!("opened `{target}`")))
            })
        }),
    }
}
