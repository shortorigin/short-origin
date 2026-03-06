//! Wallpaper asset service contracts and shared update models.

use std::{future::Future, pin::Pin};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
/// Identifies either a built-in wallpaper or an imported managed asset.
pub enum WallpaperSelection {
    /// Built-in wallpaper from the generated runtime catalog.
    BuiltIn {
        /// Stable built-in wallpaper identifier.
        wallpaper_id: String,
    },
    /// Imported managed wallpaper asset.
    Imported {
        /// Stable managed asset identifier.
        asset_id: String,
    },
}

impl Default for WallpaperSelection {
    fn default() -> Self {
        Self::BuiltIn {
            wallpaper_id: "cloud-bands".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Traditional desktop wallpaper display modes.
pub enum WallpaperDisplayMode {
    /// Preserve aspect ratio while covering the viewport.
    #[default]
    Fill,
    /// Preserve aspect ratio while containing the image inside the viewport.
    Fit,
    /// Stretch the wallpaper to match the viewport exactly.
    Stretch,
    /// Repeat the wallpaper at intrinsic size from the top-left origin.
    Tile,
    /// Render the wallpaper once at intrinsic size.
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Anchor position used for non-tiled wallpaper placement.
pub enum WallpaperPosition {
    /// Center the wallpaper.
    #[default]
    Center,
    /// Place the wallpaper in the top-left corner.
    TopLeft,
    /// Align the wallpaper to the top edge.
    Top,
    /// Place the wallpaper in the top-right corner.
    TopRight,
    /// Align the wallpaper to the left edge.
    Left,
    /// Align the wallpaper to the right edge.
    Right,
    /// Place the wallpaper in the bottom-left corner.
    BottomLeft,
    /// Align the wallpaper to the bottom edge.
    Bottom,
    /// Place the wallpaper in the bottom-right corner.
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Persisted animation intent for wallpapers that can move.
pub enum WallpaperAnimationPolicy {
    /// Render the wallpaper in a static form.
    #[default]
    None,
    /// Loop animated media with muted playback.
    LoopMuted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Media kind used by the shell renderer.
pub enum WallpaperMediaKind {
    /// Static bitmap image.
    #[default]
    StaticImage,
    /// Animated image such as GIF or animated SVG.
    AnimatedImage,
    /// Video wallpaper.
    Video,
    /// Static or animated SVG wallpaper.
    Svg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Source origin for wallpaper assets in the user library.
pub enum WallpaperSourceKind {
    /// Shell-provided built-in wallpaper.
    BuiltIn,
    /// User-imported managed asset.
    #[default]
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Current or previewed wallpaper configuration.
pub struct WallpaperConfig {
    /// Wallpaper asset selection.
    pub selection: WallpaperSelection,
    /// Viewport rendering mode.
    #[serde(default)]
    pub display_mode: WallpaperDisplayMode,
    /// Anchor position used by placement modes.
    #[serde(default)]
    pub position: WallpaperPosition,
    /// Animation intent for moving wallpaper media.
    #[serde(default)]
    pub animation: WallpaperAnimationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Managed wallpaper asset metadata known to the runtime library.
pub struct WallpaperAssetRecord {
    /// Stable asset identifier.
    pub asset_id: String,
    /// User-facing label.
    pub display_name: String,
    /// Built-in vs imported source classification.
    pub source_kind: WallpaperSourceKind,
    /// Media kind used by the shell renderer.
    pub media_kind: WallpaperMediaKind,
    /// MIME type for the primary asset payload.
    pub mime_type: String,
    /// Asset size in bytes.
    pub byte_len: u64,
    /// Natural width in pixels when known.
    pub natural_width: Option<u32>,
    /// Natural height in pixels when known.
    pub natural_height: Option<u32>,
    /// Duration in milliseconds for animated media when known.
    pub duration_ms: Option<u64>,
    /// Favorite flag shown in the library UI.
    #[serde(default)]
    pub favorite: bool,
    /// User-defined tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// User-defined collection memberships.
    #[serde(default)]
    pub collection_ids: Vec<String>,
    /// Managed primary URL or data URL.
    pub primary_url: String,
    /// Optional poster URL or data URL.
    pub poster_url: Option<String>,
    /// Creation timestamp when known.
    pub created_at_unix_ms: Option<u64>,
    /// Last-used timestamp when known.
    pub last_used_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// User-defined grouping for wallpaper library browsing.
pub struct WallpaperCollection {
    /// Stable collection identifier.
    pub collection_id: String,
    /// User-facing label.
    pub display_name: String,
    /// Stable ordering key.
    pub sort_order: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Full wallpaper library snapshot exposed to apps and persisted by the runtime.
pub struct WallpaperLibrarySnapshot {
    /// Known built-in and imported assets.
    pub assets: Vec<WallpaperAssetRecord>,
    /// User-defined collections.
    pub collections: Vec<WallpaperCollection>,
    /// Soft storage limit enforced by the host/runtime policy.
    pub soft_limit_bytes: u64,
    /// Current managed library usage in bytes.
    pub used_bytes: u64,
}

impl Default for WallpaperLibrarySnapshot {
    fn default() -> Self {
        Self {
            assets: Vec::new(),
            collections: Vec::new(),
            soft_limit_bytes: 512 * 1024 * 1024,
            used_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Import request describing how a newly selected wallpaper should enter the library.
pub struct WallpaperImportRequest {
    /// Default display name to use when host metadata is missing.
    pub display_name: Option<String>,
    /// Wallpaper configuration to apply after import when provided.
    pub default_config: Option<WallpaperConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Result of importing one wallpaper asset into the managed library.
pub struct WallpaperImportResult {
    /// Imported asset record.
    pub asset: WallpaperAssetRecord,
    /// Soft storage limit enforced by the host/runtime policy.
    pub soft_limit_bytes: u64,
    /// Current managed library usage in bytes after the import.
    pub used_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Result of deleting one wallpaper collection.
pub struct WallpaperCollectionDeleteResult {
    /// Deleted collection identifier.
    pub collection_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Result of deleting one wallpaper asset from the managed library.
pub struct WallpaperAssetDeleteResult {
    /// Deleted asset identifier.
    pub asset_id: String,
    /// Current managed library usage in bytes after the deletion.
    pub used_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Resolved wallpaper source information used by the renderer.
pub struct ResolvedWallpaperSource {
    /// Resolved URL for the primary asset.
    pub primary_url: String,
    /// Optional poster URL for animated media.
    pub poster_url: Option<String>,
    /// Media kind used by the renderer.
    pub media_kind: WallpaperMediaKind,
    /// Natural width in pixels when known.
    pub natural_width: Option<u32>,
    /// Natural height in pixels when known.
    pub natural_height: Option<u32>,
    /// Duration in milliseconds when known.
    pub duration_ms: Option<u64>,
}

/// Object-safe boxed future used by [`WallpaperAssetService`] async methods.
pub type WallpaperAssetFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
/// Patch payload used to update managed wallpaper asset metadata.
pub struct WallpaperAssetMetadataPatch {
    /// Updated display name when present.
    pub display_name: Option<String>,
    /// Updated favorite flag when present.
    pub favorite: Option<bool>,
    /// Replacement tag list when present.
    pub tags: Option<Vec<String>>,
    /// Replacement collection memberships when present.
    pub collection_ids: Option<Vec<String>>,
}

/// Host service for managed wallpaper asset import, metadata updates, and source resolution.
pub trait WallpaperAssetService {
    /// Imports a wallpaper asset through the host picker flow.
    fn import_from_picker<'a>(
        &'a self,
        request: WallpaperImportRequest,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperImportResult, String>>;

    /// Lists the current wallpaper library snapshot.
    fn list_library<'a>(
        &'a self,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperLibrarySnapshot, String>>;

    /// Updates asset metadata by patch.
    fn update_asset_metadata<'a>(
        &'a self,
        asset_id: &'a str,
        patch: WallpaperAssetMetadataPatch,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetRecord, String>>;

    /// Creates a wallpaper collection.
    fn create_collection<'a>(
        &'a self,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>>;

    /// Renames a wallpaper collection.
    fn rename_collection<'a>(
        &'a self,
        collection_id: &'a str,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>>;

    /// Deletes a wallpaper collection and removes its memberships.
    fn delete_collection<'a>(
        &'a self,
        collection_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollectionDeleteResult, String>>;

    /// Deletes a wallpaper asset.
    fn delete_asset<'a>(
        &'a self,
        asset_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetDeleteResult, String>>;

    /// Resolves a wallpaper selection to a renderer-safe source.
    fn resolve_source<'a>(
        &'a self,
        selection: WallpaperSelection,
    ) -> WallpaperAssetFuture<'a, Result<Option<ResolvedWallpaperSource>, String>>;
}

#[derive(Debug, Clone, Copy, Default)]
/// No-op wallpaper host adapter used by unsupported targets and baseline tests.
pub struct NoopWallpaperAssetService;

impl NoopWallpaperAssetService {
    fn unsupported(op: &str) -> String {
        format!("wallpaper asset service unavailable: {op}")
    }
}

impl WallpaperAssetService for NoopWallpaperAssetService {
    fn import_from_picker<'a>(
        &'a self,
        _request: WallpaperImportRequest,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperImportResult, String>> {
        Box::pin(async { Err(Self::unsupported("import_from_picker")) })
    }

    fn list_library<'a>(
        &'a self,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperLibrarySnapshot, String>> {
        Box::pin(async { Ok(WallpaperLibrarySnapshot::default()) })
    }

    fn update_asset_metadata<'a>(
        &'a self,
        _asset_id: &'a str,
        _patch: WallpaperAssetMetadataPatch,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetRecord, String>> {
        Box::pin(async { Err(Self::unsupported("update_asset_metadata")) })
    }

    fn create_collection<'a>(
        &'a self,
        _display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        Box::pin(async { Err(Self::unsupported("create_collection")) })
    }

    fn rename_collection<'a>(
        &'a self,
        _collection_id: &'a str,
        _display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        Box::pin(async { Err(Self::unsupported("rename_collection")) })
    }

    fn delete_collection<'a>(
        &'a self,
        _collection_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollectionDeleteResult, String>> {
        Box::pin(async { Err(Self::unsupported("delete_collection")) })
    }

    fn delete_asset<'a>(
        &'a self,
        _asset_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetDeleteResult, String>> {
        Box::pin(async { Err(Self::unsupported("delete_asset")) })
    }

    fn resolve_source<'a>(
        &'a self,
        _selection: WallpaperSelection,
    ) -> WallpaperAssetFuture<'a, Result<Option<ResolvedWallpaperSource>, String>> {
        Box::pin(async { Ok(None) })
    }
}
