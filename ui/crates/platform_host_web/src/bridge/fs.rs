use platform_host::{
    ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState,
};

pub(crate) async fn explorer_status() -> Result<ExplorerBackendStatus, String> {
    super::interop::explorer_status().await
}

pub(crate) async fn explorer_pick_native_directory() -> Result<ExplorerBackendStatus, String> {
    super::interop::explorer_pick_native_directory().await
}

pub(crate) async fn explorer_request_permission(
    mode: ExplorerPermissionMode,
) -> Result<ExplorerPermissionState, String> {
    super::interop::explorer_request_permission(mode).await
}

pub(crate) async fn explorer_list_dir(path: &str) -> Result<ExplorerListResult, String> {
    super::interop::explorer_list_dir(path).await
}

pub(crate) async fn explorer_read_text_file(path: &str) -> Result<ExplorerFileReadResult, String> {
    super::interop::explorer_read_text_file(path).await
}

pub(crate) async fn explorer_write_text_file(
    path: &str,
    text: &str,
) -> Result<ExplorerMetadata, String> {
    super::interop::explorer_write_text_file(path, text).await
}

pub(crate) async fn explorer_create_dir(path: &str) -> Result<ExplorerMetadata, String> {
    super::interop::explorer_create_dir(path).await
}

pub(crate) async fn explorer_create_file(
    path: &str,
    text: &str,
) -> Result<ExplorerMetadata, String> {
    super::interop::explorer_create_file(path, text).await
}

pub(crate) async fn explorer_delete(path: &str, recursive: bool) -> Result<(), String> {
    super::interop::explorer_delete(path, recursive).await
}

pub(crate) async fn explorer_stat(path: &str) -> Result<ExplorerMetadata, String> {
    super::interop::explorer_stat(path).await
}
