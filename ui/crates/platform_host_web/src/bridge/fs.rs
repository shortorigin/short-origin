use platform_host::{
    ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState, HostResult,
};

pub(crate) async fn explorer_status() -> HostResult<ExplorerBackendStatus> {
    super::interop::explorer_status().await
}

pub(crate) async fn explorer_pick_native_directory() -> HostResult<ExplorerBackendStatus> {
    super::interop::explorer_pick_native_directory().await
}

pub(crate) async fn explorer_request_permission(
    mode: ExplorerPermissionMode,
) -> HostResult<ExplorerPermissionState> {
    super::interop::explorer_request_permission(mode).await
}

pub(crate) async fn explorer_list_dir(path: &str) -> HostResult<ExplorerListResult> {
    super::interop::explorer_list_dir(path).await
}

pub(crate) async fn explorer_read_text_file(path: &str) -> HostResult<ExplorerFileReadResult> {
    super::interop::explorer_read_text_file(path).await
}

pub(crate) async fn explorer_write_text_file(
    path: &str,
    text: &str,
) -> HostResult<ExplorerMetadata> {
    super::interop::explorer_write_text_file(path, text).await
}

pub(crate) async fn explorer_create_dir(path: &str) -> HostResult<ExplorerMetadata> {
    super::interop::explorer_create_dir(path).await
}

pub(crate) async fn explorer_create_file(path: &str, text: &str) -> HostResult<ExplorerMetadata> {
    super::interop::explorer_create_file(path, text).await
}

pub(crate) async fn explorer_delete(path: &str, recursive: bool) -> HostResult<()> {
    super::interop::explorer_delete(path, recursive).await
}

pub(crate) async fn explorer_stat(path: &str) -> HostResult<ExplorerMetadata> {
    super::interop::explorer_stat(path).await
}
