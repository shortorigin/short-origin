//! Runtime-effect dispatch for the desktop host boundary.

use crate::{
    host::{DesktopHostContext, app_bus, host_ui, persistence_effects},
    reducer::RuntimeEffect,
    runtime_context::DesktopRuntimeContext,
};

pub(super) fn run_runtime_effect(
    host: DesktopHostContext,
    runtime: DesktopRuntimeContext,
    effect: RuntimeEffect,
) {
    match effect {
        RuntimeEffect::PersistLayout => persistence_effects::persist_layout(host, runtime),
        RuntimeEffect::PersistTheme => persistence_effects::persist_theme(host, runtime),
        RuntimeEffect::PersistTerminalHistory => {
            persistence_effects::persist_terminal_history(host, runtime)
        }
        RuntimeEffect::OpenExternalUrl(url) => host_ui::open_external_url(host, &url),
        RuntimeEffect::FocusWindowInput(window_id) => host.focus_window_input(window_id),
        RuntimeEffect::PlaySound(_) => {}
        RuntimeEffect::DispatchLifecycle { window_id, event } => {
            app_bus::dispatch_lifecycle(runtime, window_id, event);
        }
        RuntimeEffect::DeliverAppEvent { window_id, event } => {
            app_bus::deliver_app_event(runtime, window_id, event);
        }
        RuntimeEffect::SubscribeWindowTopic { window_id, topic } => {
            app_bus::subscribe_topic(runtime, window_id, topic);
        }
        RuntimeEffect::UnsubscribeWindowTopic { window_id, topic } => {
            app_bus::unsubscribe_topic(runtime, window_id, topic);
        }
        RuntimeEffect::PublishTopicEvent {
            source_window_id,
            topic,
            payload,
            correlation_id,
            reply_to,
        } => app_bus::publish_event(
            runtime,
            source_window_id,
            topic,
            payload,
            correlation_id,
            reply_to,
        ),
        RuntimeEffect::SaveConfig {
            namespace,
            key,
            value,
        } => persistence_effects::save_config(host, namespace, key, value),
        RuntimeEffect::Notify { title, body } => host_ui::notify(host, title, body),
    }
}
