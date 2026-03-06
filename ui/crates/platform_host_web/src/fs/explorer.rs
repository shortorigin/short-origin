//! Browser explorer/filesystem service backed by the shared JS bridge.

use platform_host::{
    ExplorerBackendStatus, ExplorerFileReadResult, ExplorerFsFuture, ExplorerFsService,
    ExplorerListResult, ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
};

#[derive(Debug, Clone, Copy, Default)]
/// Browser explorer service backed by IndexedDB VFS + File System Access API bridge code.
pub struct WebExplorerFsService;

impl ExplorerFsService for WebExplorerFsService {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        Box::pin(async move { crate::bridge::explorer_status().await })
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        Box::pin(async move { crate::bridge::explorer_pick_native_directory().await })
    }

    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, Result<ExplorerPermissionState, String>> {
        Box::pin(async move { crate::bridge::explorer_request_permission(mode).await })
    }

    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerListResult, String>> {
        Box::pin(async move { crate::bridge::explorer_list_dir(path).await })
    }

    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerFileReadResult, String>> {
        Box::pin(async move { crate::bridge::explorer_read_text_file(path).await })
    }

    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_write_text_file(path, text).await })
    }

    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_create_dir(path).await })
    }

    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_create_file(path, text).await })
    }

    fn delete<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
    ) -> ExplorerFsFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::explorer_delete(path, recursive).await })
    }

    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_stat(path).await })
    }
}

#[derive(Debug, Clone, Copy, Default)]
/// Desktop explorer service backed by Tauri command transport through the shared bridge interop.
pub struct TauriExplorerFsService;

impl ExplorerFsService for TauriExplorerFsService {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        Box::pin(async move { crate::bridge::explorer_status().await })
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, Result<ExplorerBackendStatus, String>> {
        Box::pin(async move { crate::bridge::explorer_pick_native_directory().await })
    }

    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, Result<ExplorerPermissionState, String>> {
        Box::pin(async move { crate::bridge::explorer_request_permission(mode).await })
    }

    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerListResult, String>> {
        Box::pin(async move { crate::bridge::explorer_list_dir(path).await })
    }

    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerFileReadResult, String>> {
        Box::pin(async move { crate::bridge::explorer_read_text_file(path).await })
    }

    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_write_text_file(path, text).await })
    }

    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_create_dir(path).await })
    }

    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_create_file(path, text).await })
    }

    fn delete<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
    ) -> ExplorerFsFuture<'a, Result<(), String>> {
        Box::pin(async move { crate::bridge::explorer_delete(path, recursive).await })
    }

    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, Result<ExplorerMetadata, String>> {
        Box::pin(async move { crate::bridge::explorer_stat(path).await })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_explorer_adapter_matches_bridge_fallback_behavior() {
        let fs = WebExplorerFsService;
        let fs_obj: &dyn ExplorerFsService = &fs;
        let expected = "Browser storage APIs are only available when compiled for wasm32";

        assert_eq!(block_on(fs_obj.status()).expect_err("status"), expected);
        assert_eq!(
            block_on(fs_obj.pick_native_directory()).expect_err("pick native dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.request_permission(ExplorerPermissionMode::Read))
                .expect_err("request permission"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.list_dir("/")).expect_err("list dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.read_text_file("/demo.txt")).expect_err("read file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.write_text_file("/demo.txt", "text")).expect_err("write file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.create_dir("/Demo")).expect_err("create dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.create_file("/Demo/new.txt", "text")).expect_err("create file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.delete("/Demo/new.txt", false)).expect_err("delete"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.stat("/Demo/new.txt")).expect_err("stat"),
            expected
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_tauri_explorer_adapter_matches_bridge_fallback_behavior() {
        let fs = TauriExplorerFsService;
        let fs_obj: &dyn ExplorerFsService = &fs;
        let expected = "Browser storage APIs are only available when compiled for wasm32";

        assert_eq!(block_on(fs_obj.status()).expect_err("status"), expected);
        assert_eq!(
            block_on(fs_obj.pick_native_directory()).expect_err("pick native dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.request_permission(ExplorerPermissionMode::Read))
                .expect_err("request permission"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.list_dir("/")).expect_err("list dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.read_text_file("/demo.txt")).expect_err("read file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.write_text_file("/demo.txt", "text")).expect_err("write file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.create_dir("/Demo")).expect_err("create dir"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.create_file("/Demo/new.txt", "text")).expect_err("create file"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.delete("/Demo/new.txt", false)).expect_err("delete"),
            expected
        );
        assert_eq!(
            block_on(fs_obj.stat("/Demo/new.txt")).expect_err("stat"),
            expected
        );
    }
}
