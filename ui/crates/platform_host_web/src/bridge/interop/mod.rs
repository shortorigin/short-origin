//! Shared transport interop for browser bridge domains.
//!
//! This module routes calls to target-specific implementations while preserving a uniform API
//! for higher-level bridge domain modules.

use platform_host::{
    AppStateEnvelope, ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult,
    ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
};

#[cfg(not(target_arch = "wasm32"))]
mod non_wasm;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use non_wasm as imp;
#[cfg(target_arch = "wasm32")]
use wasm as imp;

pub async fn load_app_state_envelope(namespace: &str) -> Result<Option<AppStateEnvelope>, String> {
    imp::load_app_state_envelope(namespace).await
}

pub async fn save_app_state_envelope(envelope: &AppStateEnvelope) -> Result<(), String> {
    imp::save_app_state_envelope(envelope).await
}

pub async fn delete_app_state(namespace: &str) -> Result<(), String> {
    imp::delete_app_state(namespace).await
}

pub async fn list_app_state_namespaces() -> Result<Vec<String>, String> {
    imp::list_app_state_namespaces().await
}

pub async fn load_pref(key: &str) -> Result<Option<String>, String> {
    imp::load_pref(key).await
}

pub async fn save_pref(key: &str, raw_json: &str) -> Result<(), String> {
    imp::save_pref(key, raw_json).await
}

pub async fn delete_pref(key: &str) -> Result<(), String> {
    imp::delete_pref(key).await
}

pub async fn cache_put_text(cache_name: &str, key: &str, value: &str) -> Result<(), String> {
    imp::cache_put_text(cache_name, key, value).await
}

pub async fn cache_get_text(cache_name: &str, key: &str) -> Result<Option<String>, String> {
    imp::cache_get_text(cache_name, key).await
}

pub async fn cache_delete(cache_name: &str, key: &str) -> Result<(), String> {
    imp::cache_delete(cache_name, key).await
}

pub async fn explorer_status() -> Result<ExplorerBackendStatus, String> {
    imp::explorer_status().await
}

pub async fn explorer_pick_native_directory() -> Result<ExplorerBackendStatus, String> {
    imp::explorer_pick_native_directory().await
}

pub async fn explorer_request_permission(
    mode: ExplorerPermissionMode,
) -> Result<ExplorerPermissionState, String> {
    imp::explorer_request_permission(mode).await
}

pub async fn explorer_list_dir(path: &str) -> Result<ExplorerListResult, String> {
    imp::explorer_list_dir(path).await
}

pub async fn explorer_read_text_file(path: &str) -> Result<ExplorerFileReadResult, String> {
    imp::explorer_read_text_file(path).await
}

pub async fn explorer_write_text_file(path: &str, text: &str) -> Result<ExplorerMetadata, String> {
    imp::explorer_write_text_file(path, text).await
}

pub async fn explorer_create_dir(path: &str) -> Result<ExplorerMetadata, String> {
    imp::explorer_create_dir(path).await
}

pub async fn explorer_create_file(path: &str, text: &str) -> Result<ExplorerMetadata, String> {
    imp::explorer_create_file(path, text).await
}

pub async fn explorer_delete(path: &str, recursive: bool) -> Result<(), String> {
    imp::explorer_delete(path, recursive).await
}

pub async fn explorer_stat(path: &str) -> Result<ExplorerMetadata, String> {
    imp::explorer_stat(path).await
}

pub async fn open_external_url(url: &str) -> Result<(), String> {
    imp::open_external_url(url).await
}
