//! Notification command handlers for desktop host integration.

/// Sends a desktop notification through the Tauri notification plugin.
#[tauri::command]
pub fn notify_send(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    tauri_plugin_notification::NotificationExt::notification(&app)
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|err| format!("notification dispatch failed: {err}"))
}
