use super::*;

fn unsupported() -> String {
    "Browser storage APIs are only available when compiled for wasm32".to_string()
}

pub async fn load_app_state_envelope(_namespace: &str) -> Result<Option<AppStateEnvelope>, String> {
    Ok(None)
}

pub async fn save_app_state_envelope(_envelope: &AppStateEnvelope) -> Result<(), String> {
    Ok(())
}

pub async fn delete_app_state(_namespace: &str) -> Result<(), String> {
    Ok(())
}

pub async fn list_app_state_namespaces() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

pub async fn load_pref(_key: &str) -> Result<Option<String>, String> {
    Ok(None)
}

pub async fn save_pref(_key: &str, _raw_json: &str) -> Result<(), String> {
    Ok(())
}

pub async fn delete_pref(_key: &str) -> Result<(), String> {
    Ok(())
}

pub async fn cache_put_text(_cache_name: &str, _key: &str, _value: &str) -> Result<(), String> {
    Ok(())
}

pub async fn cache_get_text(_cache_name: &str, _key: &str) -> Result<Option<String>, String> {
    Ok(None)
}

pub async fn cache_delete(_cache_name: &str, _key: &str) -> Result<(), String> {
    Ok(())
}

pub async fn explorer_status() -> Result<ExplorerBackendStatus, String> {
    Err(unsupported())
}

pub async fn explorer_pick_native_directory() -> Result<ExplorerBackendStatus, String> {
    Err(unsupported())
}

pub async fn explorer_request_permission(
    _mode: ExplorerPermissionMode,
) -> Result<ExplorerPermissionState, String> {
    Err(unsupported())
}

pub async fn explorer_list_dir(_path: &str) -> Result<ExplorerListResult, String> {
    Err(unsupported())
}

pub async fn explorer_read_text_file(_path: &str) -> Result<ExplorerFileReadResult, String> {
    Err(unsupported())
}

pub async fn explorer_write_text_file(
    _path: &str,
    _text: &str,
) -> Result<ExplorerMetadata, String> {
    Err(unsupported())
}

pub async fn explorer_create_dir(_path: &str) -> Result<ExplorerMetadata, String> {
    Err(unsupported())
}

pub async fn explorer_create_file(_path: &str, _text: &str) -> Result<ExplorerMetadata, String> {
    Err(unsupported())
}

pub async fn explorer_delete(_path: &str, _recursive: bool) -> Result<(), String> {
    Err(unsupported())
}

pub async fn explorer_stat(_path: &str) -> Result<ExplorerMetadata, String> {
    Err(unsupported())
}

pub async fn open_external_url(_url: &str) -> Result<(), String> {
    Err(unsupported())
}
