//! Tauri desktop shell bootstrap for the Origin OS runtime/app crates.
//!
//! This crate owns the native desktop bootstrap and keeps Tauri command registration localized so
//! host-domain IPC handlers can evolve without coupling the shared runtime layer directly to Tauri
//! internals.
//!
//! The browser/WASM shell remains available for parity checks. This crate is the authoritative
//! desktop host boundary for app-state, preferences, notifications, external URL opening, cache,
//! and scoped explorer access.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod app_state;
mod cache;
#[doc(hidden)]
pub mod explorer;
mod external_url;
mod notifications;
mod prefs;

use std::sync::Once;

fn init_tracing() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_target(true)
            .with_current_span(false)
            .with_span_list(false)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// Starts the Tauri desktop host process.
///
/// This registers the current host-domain commands and then transfers control to Tauri's runtime
/// event loop.
pub fn run() {
    init_tracing();
    tracing::info!(
        event = "desktop_tauri.start",
        operation = "desktop_tauri.run",
        component = "desktop_tauri",
        runtime_target = %telemetry::RuntimeTarget::Desktop.as_str(),
        environment = %if cfg!(debug_assertions) {
            telemetry::EnvironmentProfile::Development.as_str()
        } else {
            telemetry::EnvironmentProfile::Production.as_str()
        }
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            app_state::app_state_load,
            app_state::app_state_save,
            app_state::app_state_delete,
            app_state::app_state_namespaces,
            cache::cache_put_text,
            cache::cache_get_text,
            cache::cache_delete,
            explorer::explorer_status,
            explorer::explorer_pick_root,
            explorer::explorer_request_permission,
            explorer::explorer_list_dir,
            explorer::explorer_read_text_file,
            explorer::explorer_write_text_file,
            explorer::explorer_create_dir,
            explorer::explorer_create_file,
            explorer::explorer_delete,
            explorer::explorer_stat,
            external_url::external_open_url,
            notifications::notify_send,
            prefs::prefs_load,
            prefs::prefs_save,
            prefs::prefs_delete
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| {
            tracing::error!(
                event = "desktop_tauri.run_failed",
                operation = "desktop_tauri.run",
                component = "desktop_tauri",
                runtime_target = %telemetry::RuntimeTarget::Desktop.as_str(),
                environment = %if cfg!(debug_assertions) {
                    telemetry::EnvironmentProfile::Development.as_str()
                } else {
                    telemetry::EnvironmentProfile::Production.as_str()
                },
                error = %err
            );
            std::process::exit(1);
        });
}
