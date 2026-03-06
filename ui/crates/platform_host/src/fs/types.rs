//! Explorer/filesystem data types shared across host contracts and implementations.

use serde::{Deserialize, Serialize};

/// Cache API cache name used for explorer text previews.
pub const EXPLORER_CACHE_NAME: &str = "retrodesk-explorer-cache-v1";
/// localStorage key used for explorer UI preferences.
pub const EXPLORER_PREFS_KEY: &str = "retrodesk.explorer.prefs.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Explorer backend implementation currently serving requests.
pub enum ExplorerBackend {
    /// Browser native File System Access API.
    NativeFsAccess,
    /// IndexedDB-backed virtual filesystem implementation.
    #[default]
    IndexedDbVirtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Explorer directory entry kind.
pub enum ExplorerEntryKind {
    /// File entry.
    File,
    /// Directory entry.
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Effective explorer permission state for a backend/path.
pub enum ExplorerPermissionState {
    /// Access is granted.
    Granted,
    /// Browser will prompt for permission.
    Prompt,
    /// Access is denied.
    Denied,
    /// Capability is unsupported in this browser context.
    Unsupported,
    /// Virtual filesystem backend (permission concept is synthetic and allowed).
    Virtual,
}

impl ExplorerPermissionState {
    /// Returns `true` when reads are allowed.
    pub fn can_read(self) -> bool {
        matches!(self, Self::Granted | Self::Virtual)
    }

    /// Returns `true` when writes are allowed.
    pub fn can_write(self) -> bool {
        matches!(self, Self::Granted | Self::Virtual)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Permission mode requested from the explorer backend.
pub enum ExplorerPermissionMode {
    /// Read-only access.
    Read,
    /// Read/write access.
    Readwrite,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Current backend capability and permission status for the explorer app.
pub struct ExplorerBackendStatus {
    /// Active backend.
    pub backend: ExplorerBackend,
    /// Whether native File System Access is supported.
    pub native_supported: bool,
    /// Whether a native directory root is already connected.
    pub has_native_root: bool,
    /// Effective permission state.
    pub permission: ExplorerPermissionState,
    /// Optional user-facing root path hint.
    pub root_path_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Directory entry returned by explorer listing operations.
pub struct ExplorerEntry {
    /// Base name of the entry.
    pub name: String,
    /// Full normalized path.
    pub path: String,
    /// File or directory kind.
    pub kind: ExplorerEntryKind,
    /// File size in bytes (files only).
    pub size: Option<u64>,
    /// Last-modified time in unix milliseconds when available.
    pub modified_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Metadata describing a single explorer path.
pub struct ExplorerMetadata {
    /// Base name of the path.
    pub name: String,
    /// Full normalized path.
    pub path: String,
    /// File or directory kind.
    pub kind: ExplorerEntryKind,
    /// Backend that produced the metadata.
    pub backend: ExplorerBackend,
    /// File size in bytes (files only).
    pub size: Option<u64>,
    /// Last-modified time in unix milliseconds when available.
    pub modified_at_unix_ms: Option<u64>,
    /// Effective permission state.
    pub permission: ExplorerPermissionState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Result payload for directory listing operations.
pub struct ExplorerListResult {
    /// Normalized directory path that was listed.
    pub cwd: String,
    /// Backend that served the list request.
    pub backend: ExplorerBackend,
    /// Effective permission state for the listing.
    pub permission: ExplorerPermissionState,
    /// Child entries in the directory.
    pub entries: Vec<ExplorerEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Result payload for reading a text file in the explorer.
pub struct ExplorerFileReadResult {
    /// Backend that served the read request.
    pub backend: ExplorerBackend,
    /// Normalized file path.
    pub path: String,
    /// UTF-8 text content returned by the backend.
    pub text: String,
    /// File metadata snapshot captured at read time.
    pub metadata: ExplorerMetadata,
    /// Cache key suitable for storing/retrieving a preview copy.
    pub cached_preview_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// User preferences for the explorer app UI.
pub struct ExplorerPrefs {
    /// Preferred backend selection.
    pub preferred_backend: ExplorerBackend,
    /// Whether details columns/panels should be shown.
    pub details_visible: bool,
    /// Whether hidden files should be shown.
    pub show_hidden: bool,
}

impl Default for ExplorerPrefs {
    fn default() -> Self {
        Self {
            preferred_backend: ExplorerBackend::IndexedDbVirtual,
            details_visible: true,
            show_hidden: true,
        }
    }
}

/// Builds the Cache API key used for explorer file previews.
pub fn explorer_preview_cache_key(path: &str) -> String {
    let normalized = if path.is_empty() { "/" } else { path };
    format!("file-preview:{}", normalized)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn explorer_enum_serde_values_match_existing_strings() {
        assert_eq!(
            serde_json::to_string(&ExplorerBackend::IndexedDbVirtual).expect("serialize"),
            "\"indexed-db-virtual\""
        );
        assert_eq!(
            serde_json::to_string(&ExplorerBackend::NativeFsAccess).expect("serialize"),
            "\"native-fs-access\""
        );
        assert_eq!(
            serde_json::to_string(&ExplorerPermissionMode::Readwrite).expect("serialize"),
            "\"readwrite\""
        );
        assert_eq!(
            serde_json::to_string(&ExplorerPermissionState::Unsupported).expect("serialize"),
            "\"unsupported\""
        );

        let mode: ExplorerPermissionMode =
            serde_json::from_str("\"readwrite\"").expect("deserialize");
        assert_eq!(mode, ExplorerPermissionMode::Readwrite);
    }

    #[test]
    fn explorer_preview_cache_key_preserves_format() {
        assert_eq!(
            explorer_preview_cache_key("/Documents/readme.txt"),
            "file-preview:/Documents/readme.txt"
        );
        assert_eq!(explorer_preview_cache_key(""), "file-preview:/");
    }

    #[test]
    fn explorer_result_types_round_trip_with_serde() {
        let metadata = ExplorerMetadata {
            name: "file.txt".to_string(),
            path: "/file.txt".to_string(),
            kind: ExplorerEntryKind::File,
            backend: ExplorerBackend::IndexedDbVirtual,
            size: Some(12),
            modified_at_unix_ms: Some(10),
            permission: ExplorerPermissionState::Virtual,
        };
        let result = ExplorerFileReadResult {
            backend: ExplorerBackend::IndexedDbVirtual,
            path: "/file.txt".to_string(),
            text: "hello".to_string(),
            metadata,
            cached_preview_key: explorer_preview_cache_key("/file.txt"),
        };

        let value = serde_json::to_value(&result).expect("serialize");
        assert_eq!(value["path"], json!("/file.txt"));
        let round_trip: ExplorerFileReadResult =
            serde_json::from_value(value).expect("deserialize");
        assert_eq!(round_trip.text, "hello");
    }
}
