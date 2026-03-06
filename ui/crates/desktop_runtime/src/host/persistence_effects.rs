use leptos::{logging, spawn_local, SignalGetUntracked};
use platform_host::save_pref_with;

use crate::{components::DesktopRuntimeContext, host::DesktopHostContext, persistence};

pub(super) fn persist_layout(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let snapshot_state = runtime.state.get_untracked();
    if let Err(err) = persistence::persist_layout_snapshot(&snapshot_state) {
        logging::warn!("persist layout failed: {err}");
    }
    host.persist_durable_snapshot(snapshot_state, "layout");
}

pub(super) fn persist_theme(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let theme = runtime.state.get_untracked().theme;
    let async_host = host.clone();
    spawn_local(async move {
        if let Err(err) = persistence::persist_theme(&async_host, &theme).await {
            logging::warn!("persist theme failed: {err}");
        }
    });
    host.persist_durable_snapshot(runtime.state.get_untracked(), "theme");
}

pub(super) fn persist_wallpaper(host: DesktopHostContext, runtime: DesktopRuntimeContext) {
    let wallpaper = runtime.state.get_untracked().wallpaper;
    spawn_local(async move {
        if let Err(err) = persistence::persist_wallpaper(&host, &wallpaper).await {
            logging::warn!("persist wallpaper failed: {err}");
        }
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
