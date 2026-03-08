//! Browser explorer/filesystem service backed by the shared JS bridge.

use platform_host::{
    ExplorerBackendStatus, ExplorerFileReadResult, ExplorerFsFuture, ExplorerFsService,
    ExplorerListResult, ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
    HostResult,
};

#[derive(Debug, Clone, Copy, Default)]
/// Browser explorer service backed by IndexedDB VFS + File System Access API bridge code.
pub struct WebExplorerFsService;

impl ExplorerFsService for WebExplorerFsService {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async move { crate::bridge::explorer_status().await })
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async move { crate::bridge::explorer_pick_native_directory().await })
    }

    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerPermissionState>> {
        Box::pin(async move { crate::bridge::explorer_request_permission(mode).await })
    }

    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerListResult>> {
        Box::pin(async move { crate::bridge::explorer_list_dir(path).await })
    }

    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerFileReadResult>> {
        Box::pin(async move { crate::bridge::explorer_read_text_file(path).await })
    }

    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_write_text_file(path, text).await })
    }

    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_create_dir(path).await })
    }

    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_create_file(path, text).await })
    }

    fn delete<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
    ) -> ExplorerFsFuture<'a, HostResult<()>> {
        Box::pin(async move { crate::bridge::explorer_delete(path, recursive).await })
    }

    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_stat(path).await })
    }
}

#[derive(Debug, Clone, Copy, Default)]
/// Desktop explorer service backed by Tauri command transport through the shared bridge interop.
pub struct TauriExplorerFsService;

impl ExplorerFsService for TauriExplorerFsService {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async move { crate::bridge::explorer_status().await })
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async move { crate::bridge::explorer_pick_native_directory().await })
    }

    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerPermissionState>> {
        Box::pin(async move { crate::bridge::explorer_request_permission(mode).await })
    }

    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerListResult>> {
        Box::pin(async move { crate::bridge::explorer_list_dir(path).await })
    }

    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerFileReadResult>> {
        Box::pin(async move { crate::bridge::explorer_read_text_file(path).await })
    }

    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_write_text_file(path, text).await })
    }

    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_create_dir(path).await })
    }

    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_create_file(path, text).await })
    }

    fn delete<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
    ) -> ExplorerFsFuture<'a, HostResult<()>> {
        Box::pin(async move { crate::bridge::explorer_delete(path, recursive).await })
    }

    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async move { crate::bridge::explorer_stat(path).await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use platform_host::{FsErrorKind, HostError, HostErrorKind};

    use super::*;

    fn assert_non_wasm_fs_error(err: HostError, operation: &str) {
        assert_eq!(err.kind, HostErrorKind::Fs(FsErrorKind::Unsupported));
        assert_eq!(
            err.safe_message,
            "Browser storage APIs are only available when compiled for wasm32"
        );
        assert_eq!(err.metadata.operation.as_deref(), Some(operation));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_explorer_adapter_matches_bridge_fallback_behavior() {
        let fs = WebExplorerFsService;
        let fs_obj: &dyn ExplorerFsService = &fs;
        assert_non_wasm_fs_error(
            block_on(fs_obj.status()).expect_err("status"),
            "explorer.status",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.pick_native_directory()).expect_err("pick native dir"),
            "explorer.pick_native_directory",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.request_permission(ExplorerPermissionMode::Read))
                .expect_err("request permission"),
            "explorer.request_permission",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.list_dir("/")).expect_err("list dir"),
            "explorer.list_dir",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.read_text_file("/demo.txt")).expect_err("read file"),
            "explorer.read_text_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.write_text_file("/demo.txt", "text")).expect_err("write file"),
            "explorer.write_text_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.create_dir("/Demo")).expect_err("create dir"),
            "explorer.create_dir",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.create_file("/Demo/new.txt", "text")).expect_err("create file"),
            "explorer.create_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.delete("/Demo/new.txt", false)).expect_err("delete"),
            "explorer.delete",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.stat("/Demo/new.txt")).expect_err("stat"),
            "explorer.stat",
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_tauri_explorer_adapter_matches_bridge_fallback_behavior() {
        let fs = TauriExplorerFsService;
        let fs_obj: &dyn ExplorerFsService = &fs;
        assert_non_wasm_fs_error(
            block_on(fs_obj.status()).expect_err("status"),
            "explorer.status",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.pick_native_directory()).expect_err("pick native dir"),
            "explorer.pick_native_directory",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.request_permission(ExplorerPermissionMode::Read))
                .expect_err("request permission"),
            "explorer.request_permission",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.list_dir("/")).expect_err("list dir"),
            "explorer.list_dir",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.read_text_file("/demo.txt")).expect_err("read file"),
            "explorer.read_text_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.write_text_file("/demo.txt", "text")).expect_err("write file"),
            "explorer.write_text_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.create_dir("/Demo")).expect_err("create dir"),
            "explorer.create_dir",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.create_file("/Demo/new.txt", "text")).expect_err("create file"),
            "explorer.create_file",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.delete("/Demo/new.txt", false)).expect_err("delete"),
            "explorer.delete",
        );
        assert_non_wasm_fs_error(
            block_on(fs_obj.stat("/Demo/new.txt")).expect_err("stat"),
            "explorer.stat",
        );
    }
}
