//! External URL command handlers for desktop host integration.

use tauri_plugin_opener::OpenerExt;

/// Opens a URL with the system default external handler through the Tauri opener plugin.
#[tauri::command]
pub fn external_open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<String>)
        .map_err(|err| format!("external URL open failed: {err}"))
}
