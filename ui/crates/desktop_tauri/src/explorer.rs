//! Typed Tauri command handlers for explorer filesystem operations.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use platform_host::{
    explorer_preview_cache_key, normalize_virtual_path, ExplorerBackend, ExplorerBackendStatus,
    ExplorerEntry, ExplorerEntryKind, ExplorerFileReadResult, ExplorerListResult, ExplorerMetadata,
    ExplorerPermissionMode, ExplorerPermissionState,
};
use tauri::Manager;

fn explorer_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data dir: {err}"))?
        .join("explorer_root");
    fs::create_dir_all(&root).map_err(|err| format!("failed to create explorer root: {err}"))?;
    Ok(root)
}

fn canonical_root(root: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(root)
        .map_err(|err| format!("failed to canonicalize {}: {err}", root.display()))
}

fn resolve_virtual_path(root: &Path, path: &str) -> (String, PathBuf) {
    let normalized = normalize_virtual_path(path);
    if normalized == "/" {
        return (normalized, root.to_path_buf());
    }

    let mut native = root.to_path_buf();
    for segment in normalized.trim_start_matches('/').split('/') {
        if !segment.is_empty() {
            native.push(segment);
        }
    }
    (normalized, native)
}

fn ensure_existing_within_root(root: &Path, native: &Path) -> Result<(), String> {
    let canonical = fs::canonicalize(native)
        .map_err(|err| format!("failed to canonicalize {}: {err}", native.display()))?;
    if canonical.starts_with(root) {
        Ok(())
    } else {
        Err(format!(
            "path `{}` resolves outside scoped explorer root",
            native.display()
        ))
    }
}

fn ensure_parent_within_root(root: &Path, native: &Path) -> Result<(), String> {
    let parent = native
        .parent()
        .ok_or_else(|| format!("path `{}` has no parent", native.display()))?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|err| format!("failed to canonicalize {}: {err}", parent.display()))?;
    if canonical_parent.starts_with(root) {
        Ok(())
    } else {
        Err(format!(
            "path `{}` parent resolves outside scoped explorer root",
            native.display()
        ))
    }
}

fn modified_at_unix_ms(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as u64)
}

fn entry_kind(metadata: &fs::Metadata) -> ExplorerEntryKind {
    if metadata.is_dir() {
        ExplorerEntryKind::Directory
    } else {
        ExplorerEntryKind::File
    }
}

fn metadata_for_path(
    root: &Path,
    normalized_path: &str,
    native_path: &Path,
) -> Result<ExplorerMetadata, String> {
    ensure_existing_within_root(root, native_path)?;
    let metadata = fs::metadata(native_path)
        .map_err(|err| format!("failed to read metadata {}: {err}", native_path.display()))?;

    let name = if normalized_path == "/" {
        "/".to_string()
    } else {
        native_path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .ok_or_else(|| format!("path `{}` has no file name", native_path.display()))?
    };

    Ok(ExplorerMetadata {
        name,
        path: normalized_path.to_string(),
        kind: entry_kind(&metadata),
        backend: ExplorerBackend::NativeFsAccess,
        size: metadata.is_file().then_some(metadata.len()),
        modified_at_unix_ms: modified_at_unix_ms(&metadata),
        permission: ExplorerPermissionState::Granted,
    })
}

fn current_status() -> ExplorerBackendStatus {
    ExplorerBackendStatus {
        backend: ExplorerBackend::NativeFsAccess,
        native_supported: true,
        has_native_root: true,
        permission: ExplorerPermissionState::Granted,
        root_path_hint: Some("/".to_string()),
    }
}

#[derive(Debug, Clone)]
/// Scoped explorer filesystem service rooted at a canonical native directory.
///
/// This helper powers desktop explorer Tauri commands and is also used by integration tests to
/// validate traversal/symlink safety guarantees.
pub struct ScopedExplorerFs {
    root: PathBuf,
}

impl ScopedExplorerFs {
    /// Creates a scoped explorer service rooted at `root`.
    ///
    /// The root directory is created if needed and canonicalized before use.
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        fs::create_dir_all(root)
            .map_err(|err| format!("failed to create explorer root {}: {err}", root.display()))?;
        Ok(Self {
            root: canonical_root(root)?,
        })
    }

    fn from_app(app: &tauri::AppHandle) -> Result<Self, String> {
        Self::from_root(explorer_root(app)?)
    }

    /// Returns current explorer backend status for desktop native filesystem mode.
    pub fn status(&self) -> ExplorerBackendStatus {
        let _ = self;
        current_status()
    }

    /// Returns current backend status; desktop mode always uses a scoped native root.
    pub fn pick_root(&self) -> ExplorerBackendStatus {
        let _ = self;
        current_status()
    }

    /// Returns granted permission for desktop scoped-root explorer operations.
    pub fn request_permission(&self, _mode: ExplorerPermissionMode) -> ExplorerPermissionState {
        ExplorerPermissionState::Granted
    }

    /// Lists a directory under the scoped explorer root.
    pub fn list_dir(&self, path: &str) -> Result<ExplorerListResult, String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        ensure_existing_within_root(&self.root, &native)?;

        let meta = fs::metadata(&native)
            .map_err(|err| format!("failed to read {}: {err}", native.display()))?;
        if !meta.is_dir() {
            return Err(format!("path `{normalized}` is not a directory"));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&native)
            .map_err(|err| format!("failed to read directory {}: {err}", native.display()))?
        {
            let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
            let child_native = entry.path();
            ensure_existing_within_root(&self.root, &child_native)?;
            let child_meta = fs::metadata(&child_native).map_err(|err| {
                format!("failed to read metadata {}: {err}", child_native.display())
            })?;
            let child_name = entry.file_name().to_string_lossy().to_string();
            let child_path = if normalized == "/" {
                format!("/{}", child_name)
            } else {
                format!("{}/{}", normalized, child_name)
            };
            entries.push(ExplorerEntry {
                name: child_name,
                path: child_path,
                kind: entry_kind(&child_meta),
                size: child_meta.is_file().then_some(child_meta.len()),
                modified_at_unix_ms: modified_at_unix_ms(&child_meta),
            });
        }

        entries.sort_by(|left, right| match (left.kind, right.kind) {
            (ExplorerEntryKind::Directory, ExplorerEntryKind::File) => std::cmp::Ordering::Less,
            (ExplorerEntryKind::File, ExplorerEntryKind::Directory) => std::cmp::Ordering::Greater,
            _ => left.name.cmp(&right.name),
        });

        Ok(ExplorerListResult {
            cwd: normalized,
            backend: ExplorerBackend::NativeFsAccess,
            permission: ExplorerPermissionState::Granted,
            entries,
        })
    }

    /// Reads UTF-8 text content for a file path under the scoped explorer root.
    pub fn read_text_file(&self, path: &str) -> Result<ExplorerFileReadResult, String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        let metadata = metadata_for_path(&self.root, &normalized, &native)?;
        if metadata.kind != ExplorerEntryKind::File {
            return Err(format!("path `{normalized}` is not a file"));
        }

        let text = fs::read_to_string(&native)
            .map_err(|err| format!("failed to read {}: {err}", native.display()))?;
        Ok(ExplorerFileReadResult {
            backend: ExplorerBackend::NativeFsAccess,
            path: normalized.clone(),
            text,
            metadata,
            cached_preview_key: explorer_preview_cache_key(&normalized),
        })
    }

    /// Writes UTF-8 text content to a file path under the scoped explorer root.
    pub fn write_text_file(&self, path: &str, text: &str) -> Result<ExplorerMetadata, String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        if normalized == "/" {
            return Err("cannot write to explorer root".to_string());
        }
        ensure_parent_within_root(&self.root, &native)?;
        fs::write(&native, text)
            .map_err(|err| format!("failed to write {}: {err}", native.display()))?;
        metadata_for_path(&self.root, &normalized, &native)
    }

    /// Creates a directory path under the scoped explorer root.
    pub fn create_dir(&self, path: &str) -> Result<ExplorerMetadata, String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        if normalized == "/" {
            return metadata_for_path(&self.root, &normalized, &native);
        }
        ensure_parent_within_root(&self.root, &native)?;
        fs::create_dir_all(&native)
            .map_err(|err| format!("failed to create directory {}: {err}", native.display()))?;
        metadata_for_path(&self.root, &normalized, &native)
    }

    /// Creates a file and writes initial text under the scoped explorer root.
    pub fn create_file(&self, path: &str, text: &str) -> Result<ExplorerMetadata, String> {
        self.write_text_file(path, text)
    }

    /// Deletes a file or directory path under the scoped explorer root.
    pub fn delete(&self, path: &str, recursive: bool) -> Result<(), String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        if normalized == "/" {
            return Err("cannot delete explorer root".to_string());
        }
        ensure_existing_within_root(&self.root, &native)?;

        let metadata = fs::metadata(&native)
            .map_err(|err| format!("failed to read metadata {}: {err}", native.display()))?;
        if metadata.is_dir() {
            if recursive {
                fs::remove_dir_all(&native).map_err(|err| {
                    format!("failed to remove directory {}: {err}", native.display())
                })?;
            } else {
                fs::remove_dir(&native).map_err(|err| {
                    format!("failed to remove directory {}: {err}", native.display())
                })?;
            }
        } else {
            fs::remove_file(&native)
                .map_err(|err| format!("failed to remove file {}: {err}", native.display()))?;
        }
        Ok(())
    }

    /// Returns metadata for a path under the scoped explorer root.
    pub fn stat(&self, path: &str) -> Result<ExplorerMetadata, String> {
        let (normalized, native) = resolve_virtual_path(&self.root, path);
        metadata_for_path(&self.root, &normalized, &native)
    }
}

/// Returns current explorer backend status for desktop native filesystem mode.
#[tauri::command]
pub fn explorer_status(_app: tauri::AppHandle) -> Result<ExplorerBackendStatus, String> {
    let fs = ScopedExplorerFs::from_app(&_app)?;
    Ok(fs.status())
}

/// Returns current backend status; desktop mode always uses a scoped native root.
#[tauri::command]
pub fn explorer_pick_root(_app: tauri::AppHandle) -> Result<ExplorerBackendStatus, String> {
    let fs = ScopedExplorerFs::from_app(&_app)?;
    Ok(fs.pick_root())
}

/// Returns granted permission for desktop scoped-root explorer operations.
#[tauri::command]
pub fn explorer_request_permission(
    _app: tauri::AppHandle,
    _mode: ExplorerPermissionMode,
) -> Result<ExplorerPermissionState, String> {
    let fs = ScopedExplorerFs::from_app(&_app)?;
    Ok(fs.request_permission(_mode))
}

/// Lists a directory under the scoped explorer root.
#[tauri::command]
pub fn explorer_list_dir(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExplorerListResult, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.list_dir(&path)
}

/// Reads UTF-8 text content for a file path under the scoped explorer root.
#[tauri::command]
pub fn explorer_read_text_file(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExplorerFileReadResult, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.read_text_file(&path)
}

/// Writes UTF-8 text content to a file path under the scoped explorer root.
#[tauri::command]
pub fn explorer_write_text_file(
    app: tauri::AppHandle,
    path: String,
    text: String,
) -> Result<ExplorerMetadata, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.write_text_file(&path, &text)
}

/// Creates a directory path under the scoped explorer root.
#[tauri::command]
pub fn explorer_create_dir(
    app: tauri::AppHandle,
    path: String,
) -> Result<ExplorerMetadata, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.create_dir(&path)
}

/// Creates a file and writes initial text under the scoped explorer root.
#[tauri::command]
pub fn explorer_create_file(
    app: tauri::AppHandle,
    path: String,
    text: String,
) -> Result<ExplorerMetadata, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.create_file(&path, &text)
}

/// Deletes a file or directory path under the scoped explorer root.
#[tauri::command]
pub fn explorer_delete(app: tauri::AppHandle, path: String, recursive: bool) -> Result<(), String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.delete(&path, recursive)
}

/// Returns metadata for a path under the scoped explorer root.
#[tauri::command]
pub fn explorer_stat(app: tauri::AppHandle, path: String) -> Result<ExplorerMetadata, String> {
    let fs = ScopedExplorerFs::from_app(&app)?;
    fs.stat(&path)
}

#[cfg(test)]
mod tests {
    use super::resolve_virtual_path;
    use std::path::Path;

    #[test]
    fn resolves_virtual_path_with_normalization() {
        let root = Path::new("/tmp/explorer-root");
        let (normalized, native) = resolve_virtual_path(root, "/docs/../notes/readme.txt");
        assert_eq!(normalized, "/notes/readme.txt");
        assert!(native.ends_with("notes/readme.txt"));
    }
}
