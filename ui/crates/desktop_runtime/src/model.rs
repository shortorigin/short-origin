//! Core runtime data model, window geometry, persistence snapshots, and deep-link types.

use std::collections::BTreeMap;

use desktop_app_contract::ApplicationId;
use platform_host::{WallpaperConfig, WallpaperLibrarySnapshot};
use serde::de::Error as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{apps, wallpaper};

/// Schema version for serialized [`DesktopSnapshot`] layout payloads.
pub const DESKTOP_LAYOUT_SCHEMA_VERSION: u32 = 2;
/// Default window width used when no explicit geometry is provided.
pub const DEFAULT_WINDOW_WIDTH: i32 = 720;
/// Default window height used when no explicit geometry is provided.
pub const DEFAULT_WINDOW_HEIGHT: i32 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Stable runtime identifier for an open desktop window.
pub struct WindowId(
    /// Monotonic numeric window id value.
    pub u64,
);

fn parse_application_id_compat(raw: &str) -> Option<ApplicationId> {
    apps::parse_application_id_compat(raw)
}

fn deserialize_application_id_compat<'de, D>(deserializer: D) -> Result<ApplicationId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    parse_application_id_compat(&raw)
        .ok_or_else(|| D::Error::custom(format!("unknown application id `{raw}`")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Window rectangle in desktop viewport coordinates.
pub struct WindowRect {
    /// Left position in pixels.
    pub x: i32,
    /// Top position in pixels.
    pub y: i32,
    /// Width in pixels.
    pub w: i32,
    /// Height in pixels.
    pub h: i32,
}

impl WindowRect {
    /// Returns a copy of the rectangle offset by `dx`/`dy`.
    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }

    /// Returns a copy of the rectangle with minimum width/height enforced.
    pub fn clamped_min(self, min_w: i32, min_h: i32) -> Self {
        Self {
            w: self.w.max(min_w),
            h: self.h.max(min_h),
            ..self
        }
    }
}

impl Default for WindowRect {
    fn default() -> Self {
        Self {
            x: 48,
            y: 48,
            w: DEFAULT_WINDOW_WIDTH,
            h: DEFAULT_WINDOW_HEIGHT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Window behavior flags for shell interactions and layout logic.
pub struct WindowFlags {
    /// Whether resize handles should be available.
    pub resizable: bool,
    /// Whether the window can be minimized.
    pub minimizable: bool,
    /// Whether the window can be maximized or snap-maximized.
    pub maximizable: bool,
    /// Optional modal parent window id for modal child windows.
    pub modal_parent: Option<WindowId>,
}

impl Default for WindowFlags {
    fn default() -> Self {
        Self {
            resizable: true,
            minimizable: true,
            maximizable: true,
            modal_parent: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Runtime record for an open desktop window instance.
pub struct WindowRecord {
    /// Unique runtime id for this window.
    pub id: WindowId,
    /// Application id rendered by the window.
    #[serde(deserialize_with = "deserialize_application_id_compat")]
    pub app_id: ApplicationId,
    /// Current window title shown in the chrome.
    pub title: String,
    /// Current icon id used in window chrome/taskbar.
    pub icon_id: String,
    /// Current window geometry.
    pub rect: WindowRect,
    /// Geometry to restore when leaving a maximized/snapped state.
    pub restore_rect: Option<WindowRect>,
    /// Window stack order used by rendering/taskbar logic.
    pub z_index: u32,
    /// Whether the window is the focused window.
    pub is_focused: bool,
    /// Whether the window is minimized.
    pub minimized: bool,
    /// Whether the window is maximized.
    pub maximized: bool,
    /// Whether the window is currently suspended by the manager.
    #[serde(default)]
    pub suspended: bool,
    /// Window behavior flags.
    pub flags: WindowFlags,
    /// Optional persistence key for app-specific state reuse.
    pub persist_key: Option<String>,
    /// App-specific serialized state payload.
    pub app_state: Value,
    /// Launch parameters provided to the app component.
    pub launch_params: Value,
    /// Last lifecycle token observed for this window.
    #[serde(default)]
    pub last_lifecycle_event: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Typed desktop skin variants rendered by the shell root `data-skin` attribute.
pub enum DesktopSkin {
    /// Soft neumorphic skin with restrained depth cues and adaptive light/dark parity.
    #[serde(rename = "soft-neumorphic")]
    #[default]
    SoftNeumorphic,
    /// Modern adaptive skin with dark-first token language and light/dark remapping.
    #[serde(rename = "modern-adaptive")]
    ModernAdaptive,
    /// Classic XP-inspired nostalgic skin.
    #[serde(rename = "classic-xp")]
    ClassicXp,
    /// Classic Windows 95-inspired nostalgic skin.
    #[serde(rename = "classic-95")]
    Classic95,
}

impl DesktopSkin {
    /// Stable CSS skin id exposed on the shell root `data-skin` attribute.
    pub const fn css_id(&self) -> &'static str {
        match self {
            Self::SoftNeumorphic => "soft-neumorphic",
            Self::ModernAdaptive => "modern-adaptive",
            Self::ClassicXp => "classic-xp",
            Self::Classic95 => "classic-95",
        }
    }

    /// Human-readable label used by UI skin pickers.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::SoftNeumorphic => "Soft Neumorphic",
            Self::ModernAdaptive => "Modern Adaptive",
            Self::ClassicXp => "Classic XP",
            Self::Classic95 => "Classic 95",
        }
    }

    /// Stable ordered list of selectable shell skins.
    pub const ALL: [Self; 4] = [
        Self::SoftNeumorphic,
        Self::ModernAdaptive,
        Self::ClassicXp,
        Self::Classic95,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// User-configurable desktop theme preferences.
pub struct DesktopTheme {
    /// Typed skin preset rendered as the shell root `data-skin`.
    ///
    /// This defaults to [`DesktopSkin::SoftNeumorphic`] for fresh state and legacy persisted
    /// payloads that omitted the typed `skin` field.
    #[serde(default)]
    pub skin: DesktopSkin,
    /// Whether high contrast rendering is enabled.
    pub high_contrast: bool,
    /// Whether reduced motion rendering is enabled.
    pub reduced_motion: bool,
    /// Whether desktop sound effects are enabled.
    pub audio_enabled: bool,
}

/// Current committed desktop wallpaper configuration.
pub type DesktopWallpaperConfig = WallpaperConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Desktop runtime preferences that affect restore behavior and feature toggles.
pub struct DesktopPreferences {
    /// Whether prior layout state should be restored during boot.
    pub restore_on_boot: bool,
    /// Maximum number of windows to restore from persisted snapshots.
    pub max_restore_windows: usize,
    /// Whether terminal command history should be retained across sessions.
    pub terminal_history_enabled: bool,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            restore_on_boot: true,
            max_restore_windows: 5,
            terminal_history_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Root desktop runtime state used by the reducer and shell components.
pub struct DesktopState {
    /// Next window id to assign when opening a window.
    pub next_window_id: u64,
    /// Open windows ordered by stacking position.
    pub windows: Vec<WindowRecord>,
    /// Whether the start menu is currently open.
    pub start_menu_open: bool,
    /// Optional active modal window id.
    pub active_modal: Option<WindowId>,
    /// Current desktop theme.
    pub theme: DesktopTheme,
    /// Current committed desktop wallpaper configuration.
    pub wallpaper: DesktopWallpaperConfig,
    /// Active wallpaper preview, if any.
    pub wallpaper_preview: Option<DesktopWallpaperConfig>,
    /// Wallpaper library snapshot for built-in and imported assets.
    pub wallpaper_library: WallpaperLibrarySnapshot,
    /// Runtime/user preferences.
    pub preferences: DesktopPreferences,
    /// Last explorer path used by shell shortcuts/workflows.
    pub last_explorer_path: Option<String>,
    /// Last notepad slug used by shell shortcuts/workflows.
    pub last_notepad_slug: Option<String>,
    /// Recent terminal commands captured for history.
    pub terminal_history: Vec<String>,
    /// App-shared state payloads keyed by `<app_id>:<key>`.
    #[serde(default)]
    pub app_shared_state: BTreeMap<String, Value>,
    /// Whether asynchronous boot hydration has completed for the current runtime session.
    #[serde(skip)]
    pub boot_hydrated: bool,
}

impl Default for DesktopState {
    fn default() -> Self {
        Self {
            next_window_id: 1,
            windows: Vec::new(),
            start_menu_open: false,
            active_modal: None,
            theme: DesktopTheme::default(),
            wallpaper: DesktopWallpaperConfig::default(),
            wallpaper_preview: None,
            wallpaper_library: wallpaper::merged_wallpaper_library(
                &WallpaperLibrarySnapshot::default(),
            ),
            preferences: DesktopPreferences::default(),
            last_explorer_path: None,
            last_notepad_slug: None,
            terminal_history: Vec::new(),
            app_shared_state: BTreeMap::new(),
            boot_hydrated: false,
        }
    }
}

impl DesktopState {
    /// Returns the focused window id, if any.
    pub fn focused_window_id(&self) -> Option<WindowId> {
        self.windows.iter().find(|w| w.is_focused).map(|w| w.id)
    }

    /// Creates a serializable snapshot of the current desktop state.
    pub fn snapshot(&self) -> DesktopSnapshot {
        DesktopSnapshot {
            schema_version: DESKTOP_LAYOUT_SCHEMA_VERSION,
            preferences: self.preferences.clone(),
            windows: self.windows.clone(),
            last_explorer_path: self.last_explorer_path.clone(),
            last_notepad_slug: self.last_notepad_slug.clone(),
            terminal_history: self.terminal_history.clone(),
            app_shared_state: self.app_shared_state.clone(),
        }
    }

    /// Rebuilds runtime state from a persisted snapshot.
    ///
    /// The next window id is recomputed from the restored window list.
    pub fn from_snapshot(snapshot: DesktopSnapshot) -> Self {
        let mut state = Self::default();
        state.preferences = snapshot.preferences;
        state.windows = snapshot.windows;
        state.last_explorer_path = snapshot.last_explorer_path;
        state.last_notepad_slug = snapshot.last_notepad_slug;
        state.terminal_history = snapshot.terminal_history;
        state.app_shared_state = snapshot.app_shared_state;
        state.boot_hydrated = false;
        state.next_window_id = state
            .windows
            .iter()
            .map(|w| w.id.0)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        state
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Serializable snapshot persisted for desktop layout restore.
pub struct DesktopSnapshot {
    /// Layout schema version for migration logic.
    pub schema_version: u32,
    /// Persisted desktop preferences.
    pub preferences: DesktopPreferences,
    /// Persisted open window records.
    pub windows: Vec<WindowRecord>,
    /// Persisted explorer path hint.
    pub last_explorer_path: Option<String>,
    /// Persisted notepad slug hint.
    pub last_notepad_slug: Option<String>,
    /// Persisted terminal history lines.
    pub terminal_history: Vec<String>,
    /// Persisted app-shared state payloads.
    #[serde(default)]
    pub app_shared_state: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Request payload used by the reducer to open a new window.
pub struct OpenWindowRequest {
    /// Target application id.
    #[serde(deserialize_with = "deserialize_application_id_compat")]
    pub app_id: ApplicationId,
    /// Optional window title override.
    pub title: Option<String>,
    /// Optional icon id override.
    pub icon_id: Option<String>,
    /// Optional initial geometry override.
    pub rect: Option<WindowRect>,
    /// Optional viewport hint used for adaptive sizing/clamping when opening.
    pub viewport: Option<WindowRect>,
    /// Optional persistence key for app instance reuse.
    pub persist_key: Option<String>,
    /// App-specific launch parameters.
    pub launch_params: Value,
    /// Initial app state payload.
    pub app_state: Value,
    /// Window behavior flags.
    pub flags: WindowFlags,
}

impl OpenWindowRequest {
    /// Creates a request with defaults for `app_id`.
    ///
    /// Additional fields can be customized before dispatching [`crate::reducer::DesktopAction::OpenWindow`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use desktop_runtime::{ApplicationId, OpenWindowRequest};
    ///
    /// let request = OpenWindowRequest::new(ApplicationId::trusted("system.explorer"));
    /// assert_eq!(request.app_id, ApplicationId::trusted("system.explorer"));
    /// assert!(request.rect.is_none());
    /// ```
    pub fn new(app_id: impl Into<ApplicationId>) -> Self {
        Self {
            app_id: app_id.into(),
            title: None,
            icon_id: None,
            rect: None,
            viewport: None,
            persist_key: None,
            launch_params: Value::Null,
            app_state: Value::Null,
            flags: WindowFlags::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Pointer coordinates in desktop viewport space.
pub struct PointerPosition {
    /// Horizontal position in pixels.
    pub x: i32,
    /// Vertical position in pixels.
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Edge or corner used for window resize interactions.
pub enum ResizeEdge {
    /// Top edge.
    North,
    /// Bottom edge.
    South,
    /// Right edge.
    East,
    /// Left edge.
    West,
    /// Top-right corner.
    NorthEast,
    /// Top-left corner.
    NorthWest,
    /// Bottom-right corner.
    SouthEast,
    /// Bottom-left corner.
    SouthWest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Active window drag interaction state.
pub struct DragSession {
    /// Window being dragged.
    pub window_id: WindowId,
    /// Pointer position where dragging began.
    pub pointer_start: PointerPosition,
    /// Window rectangle when dragging began.
    pub rect_start: WindowRect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Active window resize interaction state.
pub struct ResizeSession {
    /// Window being resized.
    pub window_id: WindowId,
    /// Edge/corner being dragged.
    pub edge: ResizeEdge,
    /// Pointer position where resizing began.
    pub pointer_start: PointerPosition,
    /// Window rectangle when resizing began.
    pub rect_start: WindowRect,
    /// Desktop viewport available for resize boundary clamping.
    pub viewport: WindowRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Non-persisted UI interaction state tracked alongside [`DesktopState`].
pub struct InteractionState {
    /// Active drag session, if any.
    pub dragging: Option<DragSession>,
    /// Active resize session, if any.
    pub resizing: Option<ResizeSession>,
    /// Origin of a desktop selection gesture, if any.
    pub desktop_selection_origin: Option<PointerPosition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Deep-link targets that can be translated into window-open requests.
pub enum DeepLinkOpenTarget {
    /// Open an app by id.
    App(ApplicationId),
    /// Open a notepad window for a note slug.
    NotesSlug(String),
    /// Open an explorer window scoped to a project slug.
    ProjectSlug(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Parsed deep-link payload extracted from URL query/hash components.
pub struct DeepLinkState {
    /// Ordered list of targets to open.
    pub open: Vec<DeepLinkOpenTarget>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_v1_window_defaults_new_lifecycle_fields() {
        let payload = serde_json::json!({
            "schema_version": 1,
            "preferences": {
                "restore_on_boot": true,
                "max_restore_windows": 5,
                "terminal_history_enabled": true
            },
            "windows": [{
                "id": 7,
                "app_id": "Explorer",
                "title": "Explorer",
                "icon_id": "folder",
                "rect": { "x": 10, "y": 20, "w": 640, "h": 480 },
                "restore_rect": null,
                "z_index": 1,
                "is_focused": true,
                "minimized": false,
                "maximized": false,
                "flags": {
                    "resizable": true,
                    "minimizable": true,
                    "maximizable": true,
                    "modal_parent": null
                },
                "persist_key": null,
                "app_state": null,
                "launch_params": null
            }],
            "last_explorer_path": null,
            "last_notepad_slug": null,
            "terminal_history": []
        });

        let snapshot: DesktopSnapshot =
            serde_json::from_value(payload).expect("snapshot should deserialize");
        let window = snapshot.windows.first().expect("window");
        assert!(!window.suspended);
        assert!(window.last_lifecycle_event.is_none());
    }

    #[test]
    fn legacy_theme_name_defaults_skin_to_soft_neumorphic() {
        let payload = serde_json::json!({
            "high_contrast": true,
            "reduced_motion": true,
            "audio_enabled": false,
            "name": "Fluent Modern"
        });

        let theme: DesktopTheme =
            serde_json::from_value(payload).expect("legacy theme payload should deserialize");
        assert_eq!(theme.skin, DesktopSkin::SoftNeumorphic);
        assert!(theme.high_contrast);
        assert!(theme.reduced_motion);
    }

    #[test]
    fn desktop_theme_roundtrip_preserves_skin() {
        let theme = DesktopTheme {
            skin: DesktopSkin::ClassicXp,
            high_contrast: false,
            reduced_motion: true,
            audio_enabled: true,
        };
        let encoded = serde_json::to_value(&theme).expect("serialize theme");
        let decoded: DesktopTheme = serde_json::from_value(encoded).expect("deserialize theme");
        assert_eq!(decoded.skin, DesktopSkin::ClassicXp);
        assert!(decoded.reduced_motion);
        assert!(decoded.audio_enabled);
    }

    #[test]
    fn from_snapshot_recomputes_next_window_id_from_restored_windows() {
        let state = DesktopState::from_snapshot(DesktopSnapshot {
            schema_version: DESKTOP_LAYOUT_SCHEMA_VERSION,
            preferences: DesktopPreferences::default(),
            windows: vec![
                WindowRecord {
                    id: WindowId(4),
                    app_id: ApplicationId::trusted("system.explorer"),
                    title: "Explorer".to_string(),
                    icon_id: "folder".to_string(),
                    rect: WindowRect::default(),
                    restore_rect: None,
                    z_index: 1,
                    is_focused: false,
                    minimized: false,
                    maximized: false,
                    suspended: false,
                    flags: WindowFlags::default(),
                    persist_key: None,
                    app_state: Value::Null,
                    launch_params: Value::Null,
                    last_lifecycle_event: None,
                },
                WindowRecord {
                    id: WindowId(11),
                    app_id: ApplicationId::trusted("system.terminal"),
                    title: "Terminal".to_string(),
                    icon_id: "terminal".to_string(),
                    rect: WindowRect::default(),
                    restore_rect: None,
                    z_index: 2,
                    is_focused: true,
                    minimized: false,
                    maximized: false,
                    suspended: false,
                    flags: WindowFlags::default(),
                    persist_key: None,
                    app_state: Value::Null,
                    launch_params: Value::Null,
                    last_lifecycle_event: Some("focused".to_string()),
                },
            ],
            last_explorer_path: None,
            last_notepad_slug: None,
            terminal_history: Vec::new(),
            app_shared_state: BTreeMap::new(),
        });

        assert_eq!(state.next_window_id, 12);
        assert_eq!(state.focused_window_id(), Some(WindowId(11)));
    }
}
