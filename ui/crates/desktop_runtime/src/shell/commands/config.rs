#![allow(clippy::clone_on_copy)]

use std::rc::Rc;

use desktop_app_contract::AppCommandRegistration;
use platform_host::{load_pref_with, save_pref_with};
use system_shell_contract::{
    CommandArgSpec, CommandDataShape, CommandOutputShape, StructuredScalar, StructuredValue,
};

use crate::components::DesktopRuntimeContext;

pub(super) fn registrations(runtime: DesktopRuntimeContext) -> Vec<AppCommandRegistration> {
    vec![
        config_get_registration(runtime.clone()),
        config_set_registration(runtime),
    ]
}

fn config_get_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "config get",
            &[],
            "Load one config value from prefs storage.",
            "config get <namespace> <key>",
            vec![
                CommandArgSpec {
                    name: "namespace".to_string(),
                    summary: "Config namespace.".to_string(),
                    required: true,
                    repeatable: false,
                },
                CommandArgSpec {
                    name: "key".to_string(),
                    summary: "Config key.".to_string(),
                    required: true,
                    repeatable: false,
                },
            ],
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Any),
        ),
        completion: None,
        handler: Rc::new(move |context| {
            let runtime = runtime.clone();
            Box::pin(async move {
                let namespace = context.args.first().ok_or_else(|| {
                    super::super::usage_error("usage: config get <namespace> <key>")
                })?;
                let key = context.args.get(1).ok_or_else(|| {
                    super::super::usage_error("usage: config get <namespace> <key>")
                })?;
                let pref_key = format!("{namespace}.{key}");
                let value =
                    load_pref_with(runtime.host.get_value().prefs_store().as_ref(), &pref_key)
                        .await
                        .map_err(super::super::unavailable)?;
                match value {
                    Some(value) => Ok(system_shell_contract::CommandResult {
                        output: super::super::json_to_structured_data(value),
                        display: system_shell_contract::DisplayPreference::Auto,
                        notices: Vec::new(),
                        cwd: None,
                        exit: system_shell_contract::ShellExit::success(),
                    }),
                    None => Ok(super::super::info_result(format!(
                        "no value stored for `{pref_key}`"
                    ))),
                }
            })
        }),
    }
}

fn config_set_registration(runtime: DesktopRuntimeContext) -> AppCommandRegistration {
    AppCommandRegistration {
        descriptor: super::super::namespaced_descriptor(
            "config set",
            &[],
            "Store one config value in prefs storage.",
            "config set <namespace> <key> <value>",
            vec![
                CommandArgSpec {
                    name: "namespace".to_string(),
                    summary: "Config namespace.".to_string(),
                    required: true,
                    repeatable: false,
                },
                CommandArgSpec {
                    name: "key".to_string(),
                    summary: "Config key.".to_string(),
                    required: true,
                    repeatable: false,
                },
                CommandArgSpec {
                    name: "value".to_string(),
                    summary: "Typed literal or string payload.".to_string(),
                    required: true,
                    repeatable: false,
                },
            ],
            Vec::new(),
            system_shell_contract::CommandInputShape::none(),
            CommandOutputShape::new(CommandDataShape::Empty),
        ),
        completion: None,
        handler: Rc::new(move |context| {
            let runtime = runtime.clone();
            Box::pin(async move {
                if context.args.len() < 3 {
                    return Err(super::super::usage_error(
                        "usage: config set <namespace> <key> <value>",
                    ));
                }
                let namespace = &context.args[0];
                let key = &context.args[1];
                let value = context
                    .invocation
                    .values
                    .get(2)
                    .map(super::super::parsed_value_to_structured)
                    .unwrap_or_else(|| {
                        StructuredValue::Scalar(StructuredScalar::String(context.args[2].clone()))
                    });
                let pref_key = format!("{namespace}.{key}");
                save_pref_with(
                    runtime.host.get_value().prefs_store().as_ref(),
                    &pref_key,
                    &super::super::structured_value_to_json(&value),
                )
                .await
                .map_err(super::super::unavailable)?;
                Ok(super::super::info_result(format!("saved `{pref_key}`")))
            })
        }),
    }
}
