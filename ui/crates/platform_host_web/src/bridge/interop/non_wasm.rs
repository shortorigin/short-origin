use super::*;
use platform_host::{
    ExternalUrlErrorKind, FsErrorKind, HostError, HostResult, NotificationErrorKind,
};

fn unsupported_fs(operation: &str) -> HostError {
    HostError::fs(
        FsErrorKind::Unsupported,
        "Browser storage APIs are only available when compiled for wasm32",
    )
    .with_operation(operation)
}

pub async fn load_app_state_envelope(_namespace: &str) -> HostResult<Option<AppStateEnvelope>> {
    Ok(None)
}

pub async fn save_app_state_envelope(_envelope: &AppStateEnvelope) -> HostResult<()> {
    Ok(())
}

pub async fn delete_app_state(_namespace: &str) -> HostResult<()> {
    Ok(())
}

pub async fn list_app_state_namespaces() -> HostResult<Vec<String>> {
    Ok(Vec::new())
}

pub async fn load_pref(_key: &str) -> HostResult<Option<String>> {
    Ok(None)
}

pub async fn save_pref(_key: &str, _raw_json: &str) -> HostResult<()> {
    Ok(())
}

pub async fn delete_pref(_key: &str) -> HostResult<()> {
    Ok(())
}

pub async fn cache_put_text(_cache_name: &str, _key: &str, _value: &str) -> HostResult<()> {
    Ok(())
}

pub async fn cache_get_text(_cache_name: &str, _key: &str) -> HostResult<Option<String>> {
    Ok(None)
}

pub async fn cache_delete(_cache_name: &str, _key: &str) -> HostResult<()> {
    Ok(())
}

pub async fn explorer_status() -> HostResult<ExplorerBackendStatus> {
    Err(unsupported_fs("explorer.status"))
}

pub async fn explorer_pick_native_directory() -> HostResult<ExplorerBackendStatus> {
    Err(unsupported_fs("explorer.pick_native_directory"))
}

pub async fn explorer_request_permission(
    _mode: ExplorerPermissionMode,
) -> HostResult<ExplorerPermissionState> {
    Err(unsupported_fs("explorer.request_permission"))
}

pub async fn explorer_list_dir(_path: &str) -> HostResult<ExplorerListResult> {
    Err(unsupported_fs("explorer.list_dir"))
}

pub async fn explorer_read_text_file(_path: &str) -> HostResult<ExplorerFileReadResult> {
    Err(unsupported_fs("explorer.read_text_file"))
}

pub async fn explorer_write_text_file(_path: &str, _text: &str) -> HostResult<ExplorerMetadata> {
    Err(unsupported_fs("explorer.write_text_file"))
}

pub async fn explorer_create_dir(_path: &str) -> HostResult<ExplorerMetadata> {
    Err(unsupported_fs("explorer.create_dir"))
}

pub async fn explorer_create_file(_path: &str, _text: &str) -> HostResult<ExplorerMetadata> {
    Err(unsupported_fs("explorer.create_file"))
}

pub async fn explorer_delete(_path: &str, _recursive: bool) -> HostResult<()> {
    Err(unsupported_fs("explorer.delete"))
}

pub async fn explorer_stat(_path: &str) -> HostResult<ExplorerMetadata> {
    Err(unsupported_fs("explorer.stat"))
}

pub async fn open_external_url(_url: &str) -> HostResult<()> {
    Err(HostError::external_url(
        ExternalUrlErrorKind::Unsupported,
        "External URL opening is unavailable in non-wasm preview mode",
    )
    .with_operation("external_url.open"))
}

pub async fn send_notification(_title: &str, _body: &str) -> HostResult<()> {
    Err(HostError::notification(
        NotificationErrorKind::Unsupported,
        "Notifications are unavailable in non-wasm preview mode",
    )
    .with_operation("notification.notify"))
}
