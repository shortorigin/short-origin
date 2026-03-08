//! Shared transport interop for browser bridge domains.
//!
//! This module routes calls to target-specific implementations while preserving a uniform API
//! for higher-level bridge domain modules.

use platform_host::{
    AppStateEnvelope, ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult,
    ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState, HostResult,
};

#[cfg(not(target_arch = "wasm32"))]
mod non_wasm;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use non_wasm as imp;
#[cfg(target_arch = "wasm32")]
use wasm as imp;

pub async fn load_app_state_envelope(namespace: &str) -> HostResult<Option<AppStateEnvelope>> {
    imp::load_app_state_envelope(namespace).await
}

pub async fn save_app_state_envelope(envelope: &AppStateEnvelope) -> HostResult<()> {
    imp::save_app_state_envelope(envelope).await
}

pub async fn delete_app_state(namespace: &str) -> HostResult<()> {
    imp::delete_app_state(namespace).await
}

pub async fn list_app_state_namespaces() -> HostResult<Vec<String>> {
    imp::list_app_state_namespaces().await
}

pub async fn load_pref(key: &str) -> HostResult<Option<String>> {
    imp::load_pref(key).await
}

pub async fn save_pref(key: &str, raw_json: &str) -> HostResult<()> {
    imp::save_pref(key, raw_json).await
}

pub async fn delete_pref(key: &str) -> HostResult<()> {
    imp::delete_pref(key).await
}

pub async fn cache_put_text(cache_name: &str, key: &str, value: &str) -> HostResult<()> {
    imp::cache_put_text(cache_name, key, value).await
}

pub async fn cache_get_text(cache_name: &str, key: &str) -> HostResult<Option<String>> {
    imp::cache_get_text(cache_name, key).await
}

pub async fn cache_delete(cache_name: &str, key: &str) -> HostResult<()> {
    imp::cache_delete(cache_name, key).await
}

pub async fn explorer_status() -> HostResult<ExplorerBackendStatus> {
    imp::explorer_status().await
}

pub async fn explorer_pick_native_directory() -> HostResult<ExplorerBackendStatus> {
    imp::explorer_pick_native_directory().await
}

pub async fn explorer_request_permission(
    mode: ExplorerPermissionMode,
) -> HostResult<ExplorerPermissionState> {
    imp::explorer_request_permission(mode).await
}

pub async fn explorer_list_dir(path: &str) -> HostResult<ExplorerListResult> {
    imp::explorer_list_dir(path).await
}

pub async fn explorer_read_text_file(path: &str) -> HostResult<ExplorerFileReadResult> {
    imp::explorer_read_text_file(path).await
}

pub async fn explorer_write_text_file(path: &str, text: &str) -> HostResult<ExplorerMetadata> {
    imp::explorer_write_text_file(path, text).await
}

pub async fn explorer_create_dir(path: &str) -> HostResult<ExplorerMetadata> {
    imp::explorer_create_dir(path).await
}

pub async fn explorer_create_file(path: &str, text: &str) -> HostResult<ExplorerMetadata> {
    imp::explorer_create_file(path, text).await
}

pub async fn explorer_delete(path: &str, recursive: bool) -> HostResult<()> {
    imp::explorer_delete(path, recursive).await
}

pub async fn explorer_stat(path: &str) -> HostResult<ExplorerMetadata> {
    imp::explorer_stat(path).await
}

pub async fn open_external_url(url: &str) -> HostResult<()> {
    imp::open_external_url(url).await
}

pub async fn send_notification(title: &str, body: &str) -> HostResult<()> {
    imp::send_notification(title, body).await
}
