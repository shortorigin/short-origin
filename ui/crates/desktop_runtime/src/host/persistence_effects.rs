use leptos::prelude::GetUntracked;
use leptos::{logging, task::spawn_local};
use platform_host::{next_monotonic_timestamp_ms, save_pref_with};
use platform_host_web::{ShellSyncEvent, ShellSyncKind, publish_shell_sync_event};

use crate::{
    components::DesktopRuntimeContext,
    host::DesktopHostContext,
    persistence,
    reducer::{DesktopAction, SyncDomain},
};

pub(super) fn persist_layout(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let snapshot_state = runtime.state.get_untracked();
    if let Err(err) = persistence::persist_layout_snapshot(&snapshot_state) {
        logging::warn!("persist layout failed: {err}");
    }
    match persistence::build_durable_layout_envelope(&snapshot_state) {
        Ok(envelope) => {
            let revision = envelope.updated_at_unix_ms;
            runtime.dispatch_action(DesktopAction::RecordAppliedRevision {
                domain: SyncDomain::Layout,
                revision,
            });
            let async_host = host.clone();
            spawn_local(async move {
                if let Err(err) =
                    persistence::save_durable_layout_envelope(&async_host, &envelope).await
                {
                    logging::warn!("persist durable layout failed: {err}");
                    return;
                }
                publish_shell_sync_event(&ShellSyncEvent::new(ShellSyncKind::Layout, revision));
            });
        }
        Err(err) => logging::warn!("build durable layout envelope failed: {err}"),
    }
}

pub(super) fn persist_theme(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let theme = runtime.state.get_untracked().theme;
    let snapshot_state = runtime.state.get_untracked();
    let revision = next_monotonic_timestamp_ms();
    runtime.dispatch_action(DesktopAction::RecordAppliedRevision {
        domain: SyncDomain::Theme,
        revision,
    });
    let async_host = host.clone();
    spawn_local(async move {
        if let Err(err) = persistence::persist_theme(&async_host, &theme).await {
            logging::warn!("persist theme failed: {err}");
        }
        if let Ok(envelope) = persistence::build_durable_layout_envelope(&snapshot_state)
            && let Err(err) =
                persistence::save_durable_layout_envelope(&async_host, &envelope).await
        {
            logging::warn!("persist theme durable snapshot failed: {err}");
        }
        publish_shell_sync_event(&ShellSyncEvent::new(ShellSyncKind::Theme, revision));
    });
}

pub(super) fn persist_terminal_history(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let history = runtime.state.get_untracked().terminal_history;
    let async_host = host.clone();
    spawn_local(async move {
        if let Err(err) = persistence::persist_terminal_history(&async_host, &history).await {
            logging::warn!("persist terminal history failed: {err}");
        }
    });
    host.persist_durable_snapshot(runtime.state.get_untracked(), "terminal");
}

pub(super) fn save_config(
    host: DesktopHostContext,
    namespace: String,
    key: String,
    value: serde_json::Value,
) {
    let pref_key = format!("{}.{}", namespace, key);
    spawn_local(async move {
        if let Err(err) = save_pref_with(host.prefs_store().as_ref(), &pref_key, &value).await {
            logging::warn!("persist config preference failed: {err}");
        }
    });
}
