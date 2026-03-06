//! Browser capability bridge implementations for `platform_host_web` service adapters.
//!
//! This module is organized by host domain (`app_state`, `prefs`, `cache`, `fs`) while preserving a
//! stable internal API for the browser and desktop adapter wiring in `platform_host_web`.

mod app_state;
mod cache;
mod fs;
mod interop;
mod prefs;

use platform_host::{
    AppStateEnvelope, ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult,
    ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
};

pub async fn load_app_state_envelope(namespace: &str) -> Result<Option<AppStateEnvelope>, String> {
    app_state::load_app_state_envelope(namespace).await
}

pub async fn save_app_state_envelope(envelope: &AppStateEnvelope) -> Result<(), String> {
    app_state::save_app_state_envelope(envelope).await
}

pub async fn delete_app_state(namespace: &str) -> Result<(), String> {
    app_state::delete_app_state(namespace).await
}

pub async fn list_app_state_namespaces() -> Result<Vec<String>, String> {
    app_state::list_app_state_namespaces().await
}

pub async fn load_pref(key: &str) -> Result<Option<String>, String> {
    prefs::load_pref(key).await
}

pub async fn save_pref(key: &str, raw_json: &str) -> Result<(), String> {
    prefs::save_pref(key, raw_json).await
}

pub async fn delete_pref(key: &str) -> Result<(), String> {
    prefs::delete_pref(key).await
}

pub async fn cache_put_text(cache_name: &str, key: &str, value: &str) -> Result<(), String> {
    cache::cache_put_text(cache_name, key, value).await
}

pub async fn cache_get_text(cache_name: &str, key: &str) -> Result<Option<String>, String> {
    cache::cache_get_text(cache_name, key).await
}

pub async fn cache_delete(cache_name: &str, key: &str) -> Result<(), String> {
    cache::cache_delete(cache_name, key).await
}

pub async fn explorer_status() -> Result<ExplorerBackendStatus, String> {
    fs::explorer_status().await
}

pub async fn explorer_pick_native_directory() -> Result<ExplorerBackendStatus, String> {
    fs::explorer_pick_native_directory().await
}

pub async fn explorer_request_permission(
    mode: ExplorerPermissionMode,
) -> Result<ExplorerPermissionState, String> {
    fs::explorer_request_permission(mode).await
}

pub async fn explorer_list_dir(path: &str) -> Result<ExplorerListResult, String> {
    fs::explorer_list_dir(path).await
}

pub async fn explorer_read_text_file(path: &str) -> Result<ExplorerFileReadResult, String> {
    fs::explorer_read_text_file(path).await
}

pub async fn explorer_write_text_file(path: &str, text: &str) -> Result<ExplorerMetadata, String> {
    fs::explorer_write_text_file(path, text).await
}

pub async fn explorer_create_dir(path: &str) -> Result<ExplorerMetadata, String> {
    fs::explorer_create_dir(path).await
}

pub async fn explorer_create_file(path: &str, text: &str) -> Result<ExplorerMetadata, String> {
    fs::explorer_create_file(path, text).await
}

pub async fn explorer_delete(path: &str, recursive: bool) -> Result<(), String> {
    fs::explorer_delete(path, recursive).await
}

pub async fn explorer_stat(path: &str) -> Result<ExplorerMetadata, String> {
    fs::explorer_stat(path).await
}

pub async fn open_external_url(url: &str) -> Result<(), String> {
    interop::open_external_url(url).await
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use platform_host::{AppStateEnvelope, ExplorerPermissionMode};
    use serde_json::json;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn app_state_public_api_non_wasm_parity() {
        let envelope = AppStateEnvelope {
            envelope_version: 1,
            namespace: "app.example".to_string(),
            schema_version: 1,
            updated_at_unix_ms: 1,
            payload: json!({"value": 1}),
        };

        assert_eq!(
            block_on(load_app_state_envelope("app.example")).expect("load"),
            None
        );
        block_on(save_app_state_envelope(&envelope)).expect("save");
        block_on(delete_app_state("app.example")).expect("delete");
        assert_eq!(
            block_on(list_app_state_namespaces()).expect("list namespaces"),
            Vec::<String>::new()
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn cache_public_api_non_wasm_parity() {
        block_on(cache_put_text("cache", "k", "v")).expect("put text");
        assert_eq!(
            block_on(cache_get_text("cache", "k")).expect("get text"),
            None
        );
        block_on(cache_delete("cache", "k")).expect("delete text");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn prefs_public_api_non_wasm_parity() {
        assert_eq!(
            block_on(load_pref("retrodesk.explorer.prefs.v1")).expect("load pref"),
            None
        );
        block_on(save_pref("retrodesk.explorer.prefs.v1", "{\"k\":1}")).expect("save pref");
        block_on(delete_pref("retrodesk.explorer.prefs.v1")).expect("delete pref");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn fs_public_api_non_wasm_parity() {
        let expected =
            "Browser storage APIs are only available when compiled for wasm32".to_string();

        assert_eq!(
            block_on(explorer_status()).expect_err("status should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_pick_native_directory()).expect_err("pick should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_request_permission(ExplorerPermissionMode::Read))
                .expect_err("permission should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_list_dir("/")).expect_err("list should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_read_text_file("/readme.txt")).expect_err("read should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_write_text_file("/readme.txt", "text"))
                .expect_err("write should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_create_dir("/Docs")).expect_err("create dir should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_create_file("/Docs/new.txt", "text"))
                .expect_err("create file should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_delete("/Docs/new.txt", false)).expect_err("delete should fail"),
            expected
        );
        assert_eq!(
            block_on(explorer_stat("/Docs")).expect_err("stat should fail"),
            expected
        );
    }
}
