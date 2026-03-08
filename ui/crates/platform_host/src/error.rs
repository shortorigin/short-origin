//! Typed error contracts shared by UI host-facing services.

use error_model::{ErrorCategory, ErrorMetadata, ErrorVisibility};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
/// Stable error families exposed by UI host-facing contracts.
pub enum HostErrorKind {
    /// Persistent app state, preference, or storage-backed failures.
    Storage(StorageErrorKind),
    /// Filesystem capability failures surfaced through the host adapter.
    Fs(FsErrorKind),
    /// Content cache operation failures.
    Cache(CacheErrorKind),
    /// Notification capability failures.
    Notification(NotificationErrorKind),
    /// External URL launch failures.
    ExternalUrl(ExternalUrlErrorKind),
    /// Wallpaper asset and collection failures.
    Wallpaper(WallpaperErrorKind),
    /// Terminal process lifecycle failures.
    TerminalProcess(TerminalProcessErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Storage operation classes for persisted UI data.
pub enum StorageErrorKind {
    /// Failed to load previously persisted data.
    Load,
    /// Failed to save data.
    Save,
    /// Failed to remove persisted data.
    Delete,
    /// Failed to enumerate persisted records.
    List,
    /// Failed to serialize a value before persistence or transport.
    Serialize,
    /// Failed to deserialize a previously persisted value.
    Deserialize,
    /// The stored data shape did not match the expected contract.
    Schema,
    /// The storage capability is not currently available.
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Filesystem operation classes surfaced to UI code.
pub enum FsErrorKind {
    /// Failed to determine service or capability status.
    Status,
    /// Access was denied or not granted.
    Permission,
    /// Failed to read data.
    Read,
    /// Failed to write data.
    Write,
    /// Failed to create a resource.
    Create,
    /// Failed to delete a resource.
    Delete,
    /// Failed to stat or inspect a resource.
    Stat,
    /// The requested operation is unsupported on the current target.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Content cache operation classes.
pub enum CacheErrorKind {
    /// Failed to store a cache entry.
    Put,
    /// Failed to read a cache entry.
    Get,
    /// Failed to remove a cache entry.
    Delete,
    /// Failed to serialize a cache payload.
    Serialize,
    /// Failed to deserialize a cache payload.
    Deserialize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Notification delivery failures.
pub enum NotificationErrorKind {
    /// Failed to dispatch a notification request.
    Dispatch,
    /// Notifications are unsupported on the current target.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// External URL capability failures.
pub enum ExternalUrlErrorKind {
    /// Failed to open the requested URL.
    Open,
    /// External URL handling is unsupported on the current target.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Wallpaper capability failures.
pub enum WallpaperErrorKind {
    /// Failed to import a wallpaper asset.
    Import,
    /// Failed to list wallpaper assets or collections.
    List,
    /// Failed to update wallpaper metadata or associations.
    Update,
    /// Failed to create a wallpaper collection.
    CreateCollection,
    /// Failed to rename a wallpaper collection.
    RenameCollection,
    /// Failed to delete a wallpaper collection.
    DeleteCollection,
    /// Failed to delete a wallpaper asset.
    DeleteAsset,
    /// Failed to resolve a wallpaper identifier to an asset.
    Resolve,
    /// Wallpaper operations are unsupported on the current target.
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// Terminal process capability failures.
pub enum TerminalProcessErrorKind {
    /// Failed to spawn the process.
    Spawn,
    /// Failed to write to the process input.
    Write,
    /// Failed to resize the process terminal.
    Resize,
    /// Failed to cancel or terminate the process.
    Cancel,
    /// Terminal process support is unavailable on the current target.
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("{safe_message}")]
/// Typed host capability error shared across UI host-facing contracts.
pub struct HostError {
    /// Stable classification metadata used for logs and presentation boundaries.
    pub metadata: ErrorMetadata,
    /// High-level host subsystem where the failure originated.
    pub kind: HostErrorKind,
    /// Redacted, user-safe message suitable for UI presentation.
    pub safe_message: String,
    /// Optional internal-only detail for diagnostics and structured logs.
    pub internal_message: Option<String>,
}

/// Typed result alias for UI host-facing contracts.
pub type HostResult<T> = Result<T, HostError>;

impl HostError {
    /// Creates a host error with explicit classification metadata.
    #[must_use]
    pub fn new(
        kind: HostErrorKind,
        category: ErrorCategory,
        visibility: ErrorVisibility,
        code: impl Into<String>,
        safe_message: impl Into<String>,
    ) -> Self {
        Self {
            metadata: ErrorMetadata::new(category, visibility, code),
            kind,
            safe_message: safe_message.into(),
            internal_message: None,
        }
    }

    /// Attaches a stable operation identifier to the error metadata.
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_operation(operation);
        self
    }

    /// Attaches internal-only detail for diagnostics.
    #[must_use]
    pub fn with_internal(mut self, internal_message: impl Into<String>) -> Self {
        self.internal_message = Some(internal_message.into());
        self
    }

    /// Builds a storage-scoped host error.
    #[must_use]
    pub fn storage(kind: StorageErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Storage(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("storage.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a validation-scoped storage error.
    #[must_use]
    pub fn validation(kind: StorageErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Storage(kind),
            ErrorCategory::Validation,
            ErrorVisibility::UserSafe,
            format!("storage.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a filesystem-scoped host error.
    #[must_use]
    pub fn fs(kind: FsErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Fs(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("fs.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a cache-scoped host error.
    #[must_use]
    pub fn cache(kind: CacheErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Cache(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("cache.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a notification-scoped host error.
    #[must_use]
    pub fn notification(kind: NotificationErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Notification(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("notification.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds an external-URL-scoped host error.
    #[must_use]
    pub fn external_url(kind: ExternalUrlErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::ExternalUrl(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("external_url.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a wallpaper-scoped host error.
    #[must_use]
    pub fn wallpaper(kind: WallpaperErrorKind, safe_message: impl Into<String>) -> Self {
        Self::new(
            HostErrorKind::Wallpaper(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("wallpaper.{kind:?}").to_lowercase(),
            safe_message,
        )
    }

    /// Builds a terminal-process-scoped host error.
    #[must_use]
    pub fn terminal_process(
        kind: TerminalProcessErrorKind,
        safe_message: impl Into<String>,
    ) -> Self {
        Self::new(
            HostErrorKind::TerminalProcess(kind),
            ErrorCategory::Host,
            ErrorVisibility::UserSafe,
            format!("terminal_process.{kind:?}").to_lowercase(),
            safe_message,
        )
    }
}

impl From<HostError> for String {
    fn from(value: HostError) -> Self {
        value.safe_message
    }
}
