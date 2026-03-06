//! Shared host-bundle and capability models for browser and desktop runtime composition.

use std::rc::Rc;

use crate::{
    AppStateStore, ContentCache, ExplorerFsService, ExternalUrlService, NotificationService,
    PrefsStore, TerminalProcessService, WallpaperAssetService,
};

/// Stable host strategy selected for the current build/runtime composition path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStrategy {
    /// Browser-backed runtime composition.
    Browser,
    /// Tauri-hosted desktop composition.
    DesktopTauri,
    /// Desktop composition with placeholder/no-op native adapters.
    DesktopStub,
}

impl HostStrategy {
    /// Returns a stable string token for diagnostics and runtime inspection.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::DesktopTauri => "desktop-tauri",
            Self::DesktopStub => "desktop-stub",
        }
    }
}

/// Host availability state for one optional capability domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityStatus {
    /// Capability is available and may be granted by runtime policy.
    Available,
    /// Capability is not implemented or not supported on the active host.
    Unavailable,
    /// Capability exists but remains disabled until explicit host/user activation.
    RequiresUserActivation,
}

impl CapabilityStatus {
    /// Returns whether the capability can be used immediately.
    pub const fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }
}

/// Typed error describing capability-level rejection before a host operation executes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    /// Runtime policy denied access for the current app.
    Denied {
        /// Stable capability identifier used in diagnostics.
        capability: &'static str,
    },
    /// The host does not support the requested capability.
    Unavailable {
        /// Stable capability identifier used in diagnostics.
        capability: &'static str,
    },
    /// The capability requires an explicit user activation or permission grant first.
    RequiresUserActivation {
        /// Stable capability identifier used in diagnostics.
        capability: &'static str,
    },
}

impl CapabilityError {
    /// Returns a stable capability label for diagnostics.
    pub const fn capability(&self) -> &'static str {
        match self {
            Self::Denied { capability }
            | Self::Unavailable { capability }
            | Self::RequiresUserActivation { capability } => capability,
        }
    }
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied { capability } => write!(f, "capability denied: {capability}"),
            Self::Unavailable { capability } => write!(f, "capability unavailable: {capability}"),
            Self::RequiresUserActivation { capability } => {
                write!(f, "capability requires user activation: {capability}")
            }
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Host capability snapshot exposed to runtime wiring and mounted apps.
///
/// This snapshot is intentionally coarse-grained and stable across browser and desktop
/// compositions so apps can branch on capability posture without importing host-specific adapter
/// types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostCapabilities {
    /// Structured shell command backend availability.
    pub structured_commands: CapabilityStatus,
    /// Optional host-process/PTY terminal backend availability.
    pub terminal_process: CapabilityStatus,
    /// Explorer native root/picker availability.
    pub native_explorer: CapabilityStatus,
    /// External URL opening availability.
    pub external_urls: CapabilityStatus,
    /// Host notification availability.
    pub notifications: CapabilityStatus,
    /// Wallpaper import/library mutation availability.
    pub wallpaper_library: CapabilityStatus,
}

impl HostCapabilities {
    /// Browser-default capability posture.
    pub const fn browser() -> Self {
        Self {
            structured_commands: CapabilityStatus::Available,
            terminal_process: CapabilityStatus::Unavailable,
            native_explorer: CapabilityStatus::RequiresUserActivation,
            external_urls: CapabilityStatus::Available,
            notifications: CapabilityStatus::RequiresUserActivation,
            wallpaper_library: CapabilityStatus::Available,
        }
    }

    /// Desktop Tauri capability posture.
    pub const fn desktop_tauri() -> Self {
        Self {
            structured_commands: CapabilityStatus::Available,
            terminal_process: CapabilityStatus::Unavailable,
            native_explorer: CapabilityStatus::Available,
            external_urls: CapabilityStatus::Available,
            notifications: CapabilityStatus::Available,
            wallpaper_library: CapabilityStatus::Available,
        }
    }

    /// Stub desktop capability posture.
    pub const fn desktop_stub() -> Self {
        Self {
            structured_commands: CapabilityStatus::Available,
            terminal_process: CapabilityStatus::Unavailable,
            native_explorer: CapabilityStatus::Unavailable,
            external_urls: CapabilityStatus::Unavailable,
            notifications: CapabilityStatus::Unavailable,
            wallpaper_library: CapabilityStatus::Available,
        }
    }
}

/// Runtime-selected host service bundle injected into the shared desktop runtime.
///
/// All environment-specific service selection happens before this bundle crosses into
/// `desktop_runtime`, which keeps the runtime and app crates decoupled from browser/desktop
/// adapter details.
#[derive(Clone)]
pub struct HostServices {
    /// Durable app-state store used across runtime and app crates.
    pub app_state: Rc<dyn AppStateStore>,
    /// Lightweight typed preference store.
    pub prefs: Rc<dyn PrefsStore>,
    /// Explorer/filesystem host service.
    pub explorer: Rc<dyn ExplorerFsService>,
    /// Derived-content cache service.
    pub cache: Rc<dyn ContentCache>,
    /// External URL opening service.
    pub external_urls: Rc<dyn ExternalUrlService>,
    /// Notification delivery service.
    pub notifications: Rc<dyn NotificationService>,
    /// Wallpaper asset/library service.
    pub wallpaper: Rc<dyn WallpaperAssetService>,
    /// Optional host terminal-process backend.
    pub terminal_process: Option<Rc<dyn TerminalProcessService>>,
    /// Host availability snapshot for optional capability domains.
    pub capabilities: HostCapabilities,
    /// Stable strategy identifier for diagnostics and policy.
    pub host_strategy: HostStrategy,
}
