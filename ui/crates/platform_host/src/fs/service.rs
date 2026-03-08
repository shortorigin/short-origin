//! Explorer/filesystem service contracts.

use std::{future::Future, pin::Pin};

use super::types::{
    ExplorerBackend, ExplorerBackendStatus, ExplorerFileReadResult, ExplorerListResult,
    ExplorerMetadata, ExplorerPermissionMode, ExplorerPermissionState,
};
use crate::{FsErrorKind, HostError, HostResult};

/// Object-safe boxed future used by [`ExplorerFsService`] async methods.
pub type ExplorerFsFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Host service for explorer filesystem operations and backend capability state.
pub trait ExplorerFsService {
    /// Returns the current explorer backend status and capability information.
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>>;

    /// Opens the native-directory picker and returns updated backend status.
    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>>;

    /// Requests explorer permissions for the active backend.
    fn request_permission<'a>(
        &'a self,
        mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerPermissionState>>;

    /// Lists a directory using the active explorer backend.
    fn list_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerListResult>>;

    /// Reads a text file using the active explorer backend.
    fn read_text_file<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerFileReadResult>>;

    /// Writes a text file using the active explorer backend.
    fn write_text_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>>;

    /// Creates a directory using the active explorer backend.
    fn create_dir<'a>(
        &'a self,
        path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>>;

    /// Creates a text file using the active explorer backend.
    fn create_file<'a>(
        &'a self,
        path: &'a str,
        text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>>;

    /// Deletes a file or directory using the active explorer backend.
    fn delete<'a>(&'a self, path: &'a str, recursive: bool)
        -> ExplorerFsFuture<'a, HostResult<()>>;

    /// Retrieves metadata for a path using the active explorer backend.
    fn stat<'a>(&'a self, path: &'a str) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op explorer service adapter for unsupported targets and baseline tests.
pub struct NoopExplorerFsService;

impl NoopExplorerFsService {
    fn unsupported_status() -> ExplorerBackendStatus {
        ExplorerBackendStatus {
            backend: ExplorerBackend::IndexedDbVirtual,
            native_supported: false,
            has_native_root: false,
            permission: ExplorerPermissionState::Unsupported,
            root_path_hint: None,
        }
    }

    fn unsupported_error(op: &str) -> HostError {
        HostError::fs(
            FsErrorKind::Unsupported,
            "Explorer filesystem support is unavailable",
        )
        .with_operation(op)
    }
}

impl ExplorerFsService for NoopExplorerFsService {
    fn status<'a>(&'a self) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async { Ok(Self::unsupported_status()) })
    }

    fn pick_native_directory<'a>(
        &'a self,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerBackendStatus>> {
        Box::pin(async { Err(Self::unsupported_error("pick_native_directory")) })
    }

    fn request_permission<'a>(
        &'a self,
        _mode: ExplorerPermissionMode,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerPermissionState>> {
        Box::pin(async { Ok(ExplorerPermissionState::Unsupported) })
    }

    fn list_dir<'a>(
        &'a self,
        _path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerListResult>> {
        Box::pin(async { Err(Self::unsupported_error("list_dir")) })
    }

    fn read_text_file<'a>(
        &'a self,
        _path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerFileReadResult>> {
        Box::pin(async { Err(Self::unsupported_error("read_text_file")) })
    }

    fn write_text_file<'a>(
        &'a self,
        _path: &'a str,
        _text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async { Err(Self::unsupported_error("write_text_file")) })
    }

    fn create_dir<'a>(
        &'a self,
        _path: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async { Err(Self::unsupported_error("create_dir")) })
    }

    fn create_file<'a>(
        &'a self,
        _path: &'a str,
        _text: &'a str,
    ) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async { Err(Self::unsupported_error("create_file")) })
    }

    fn delete<'a>(
        &'a self,
        _path: &'a str,
        _recursive: bool,
    ) -> ExplorerFsFuture<'a, HostResult<()>> {
        Box::pin(async { Err(Self::unsupported_error("delete")) })
    }

    fn stat<'a>(&'a self, _path: &'a str) -> ExplorerFsFuture<'a, HostResult<ExplorerMetadata>> {
        Box::pin(async { Err(Self::unsupported_error("stat")) })
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::*;
    use crate::HostErrorKind;

    #[test]
    fn noop_explorer_fs_service_reports_unsupported() {
        let fs = NoopExplorerFsService;
        let fs_obj: &dyn ExplorerFsService = &fs;

        let status = block_on(fs_obj.status()).expect("status");
        assert_eq!(status.permission, ExplorerPermissionState::Unsupported);
        assert!(!status.native_supported);

        assert_eq!(
            block_on(fs_obj.request_permission(ExplorerPermissionMode::Read)).expect("perm"),
            ExplorerPermissionState::Unsupported
        );
        let err = block_on(fs_obj.list_dir("/")).expect_err("list should fail");
        assert_eq!(err.kind, HostErrorKind::Fs(FsErrorKind::Unsupported));
        assert_eq!(err.metadata.operation.as_deref(), Some("list_dir"));
    }
}
