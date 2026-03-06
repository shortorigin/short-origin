//! Tauri desktop shell bootstrap for the Short Origin OS runtime/app crates.
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

/// Starts the Tauri desktop host process.
///
/// This registers the current host-domain commands and then transfers control to Tauri's runtime
/// event loop.
pub fn run() {
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
        .expect("desktop_tauri failed to run Tauri application");
}
