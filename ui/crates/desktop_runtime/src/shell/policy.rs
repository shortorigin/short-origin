use std::rc::Rc;

use desktop_app_contract::{
    AppCapability, AppCommandContext, AppCommandRegistration, ApplicationId,
    CommandRegistrationHandle as AppCommandRegistrationHandle,
};
use system_shell::CommandExecutionContext;
use system_shell_contract::{
    CommandDescriptor, CommandNoticeLevel, CommandScope, ShellStreamEvent,
};

use crate::{apps, components::DesktopRuntimeContext, model::WindowId};

pub(super) fn register_app_command(
    runtime: DesktopRuntimeContext,
    app_id: ApplicationId,
    window_id: WindowId,
    registration: AppCommandRegistration,
) -> Result<AppCommandRegistrationHandle, String> {
    if !app_can_register_commands(&app_id) {
        return Err(format!(
            "{} is not allowed to register system commands",
            app_id.as_str()
        ));
    }
    validate_scope(&registration.descriptor.scope, &app_id, window_id)?;
    let completion = registration.completion.clone();
    let handler = registration.handler.clone();
    let descriptor = registration.descriptor.clone();
    let system_handle = runtime.shell_engine.get_value().register_command(
        registration.descriptor,
        completion.map(|completion| {
            Rc::new(move |request| completion(request)) as system_shell::CompletionHandler
        }),
        Rc::new(move |context: CommandExecutionContext| {
            let app_context = adapt_context(context, descriptor.clone());
            handler(app_context)
        }),
    );
    Ok(AppCommandRegistrationHandle::new(Rc::new(move || {
        system_handle.unregister();
    })))
}

fn adapt_context(
    context: CommandExecutionContext,
    _descriptor: CommandDescriptor,
) -> AppCommandContext {
    let emit_context = context.clone();
    let set_cwd_context = context.clone();
    let cancel_context = context.clone();
    AppCommandContext::new(
        context.execution_id,
        context.invocation.clone(),
        context.argv.clone(),
        context.args.clone(),
        context.cwd.clone(),
        context.input.clone(),
        context.source_window_id,
        Rc::new(move |event| emit_shell_event(&emit_context, event)),
        Rc::new(move |cwd| set_cwd_context.set_cwd(cwd)),
        Rc::new(move || cancel_context.is_cancelled()),
    )
}

fn emit_shell_event(context: &CommandExecutionContext, event: ShellStreamEvent) {
    match event {
        ShellStreamEvent::Notice { notice, .. } => match notice.level {
            CommandNoticeLevel::Info => context.info(notice.message),
            CommandNoticeLevel::Warning => context.warn(notice.message),
            CommandNoticeLevel::Error => context.error(notice.message),
        },
        ShellStreamEvent::Progress { value, label, .. } => context.progress(value, label),
        _ => {}
    }
}

fn app_can_register_commands(app_id: &ApplicationId) -> bool {
    apps::app_is_privileged_by_id(app_id)
        || apps::app_requested_capabilities_by_id(app_id).contains(&AppCapability::Commands)
}

fn validate_scope(
    scope: &CommandScope,
    app_id: &ApplicationId,
    window_id: WindowId,
) -> Result<(), String> {
    match scope {
        CommandScope::Global if apps::app_is_privileged_by_id(app_id) => Ok(()),
        CommandScope::Global => {
            Err("only privileged apps may register global commands".to_string())
        }
        CommandScope::App { app_id: owner } if owner == app_id.as_str() => Ok(()),
        CommandScope::App { .. } => Err("app-scoped command owner mismatch".to_string()),
        CommandScope::Window { window_id: owner } if *owner == window_id.0 => Ok(()),
        CommandScope::Window { .. } => Err("window-scoped command owner mismatch".to_string()),
    }
}
