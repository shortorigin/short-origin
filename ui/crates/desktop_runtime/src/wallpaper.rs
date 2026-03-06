//! Built-in wallpaper catalog helpers and runtime wallpaper-library utilities.

use std::sync::OnceLock;

use platform_host::{
    ResolvedWallpaperSource, WallpaperAssetRecord, WallpaperConfig, WallpaperLibrarySnapshot,
    WallpaperMediaKind, WallpaperSelection, WallpaperSourceKind,
};
use serde::Deserialize;

include!(concat!(env!("OUT_DIR"), "/wallpaper_catalog_generated.rs"));

#[derive(Debug, Clone, Deserialize)]
struct BuiltInWallpaperCatalogEntry {
    wallpaper_id: String,
    display_name: String,
    media_kind: WallpaperMediaKind,
    primary_path: String,
    poster_path: Option<String>,
    featured: bool,
}

fn builtin_catalog_entries() -> &'static [BuiltInWallpaperCatalogEntry] {
    static CATALOG: OnceLock<Vec<BuiltInWallpaperCatalogEntry>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(BUILTIN_WALLPAPER_CATALOG_JSON)
            .expect("generated built-in wallpaper catalog should parse")
    })
}

/// Returns the generated built-in wallpaper catalog JSON payload.
pub fn builtin_wallpaper_catalog_json() -> &'static str {
    BUILTIN_WALLPAPER_CATALOG_JSON
}

/// Returns built-in wallpaper records normalized for the runtime library surface.
pub fn builtin_wallpaper_records() -> Vec<WallpaperAssetRecord> {
    builtin_catalog_entries()
        .iter()
        .map(|entry| WallpaperAssetRecord {
            asset_id: entry.wallpaper_id.clone(),
            display_name: entry.display_name.clone(),
            source_kind: WallpaperSourceKind::BuiltIn,
            media_kind: entry.media_kind,
            mime_type: mime_type_for_path(&entry.primary_path).to_string(),
            byte_len: 0,
            natural_width: None,
            natural_height: None,
            duration_ms: None,
            favorite: false,
            tags: Vec::new(),
            collection_ids: Vec::new(),
            primary_url: format!("/wallpapers/{}", entry.primary_path),
            poster_url: entry
                .poster_path
                .as_ref()
                .map(|path| format!("/wallpapers/{path}")),
            created_at_unix_ms: None,
            last_used_at_unix_ms: None,
        })
        .collect()
}

/// Returns a merged wallpaper library including built-ins and imported assets.
pub fn merged_wallpaper_library(imported: &WallpaperLibrarySnapshot) -> WallpaperLibrarySnapshot {
    let mut assets = builtin_wallpaper_records();
    assets.extend(imported.assets.clone());
    WallpaperLibrarySnapshot {
        assets,
        collections: imported.collections.clone(),
        soft_limit_bytes: imported.soft_limit_bytes,
        used_bytes: imported.used_bytes,
    }
}

/// Upserts one imported wallpaper asset into the merged library view.
pub fn upsert_imported_wallpaper_asset(
    library: &mut WallpaperLibrarySnapshot,
    asset: WallpaperAssetRecord,
) {
    if let Some(existing) = library
        .assets
        .iter_mut()
        .find(|existing| existing.asset_id == asset.asset_id)
    {
        *existing = asset;
        return;
    }
    library.assets.push(asset);
}

/// Upserts one wallpaper collection into the merged library view.
pub fn upsert_wallpaper_collection(
    library: &mut WallpaperLibrarySnapshot,
    collection: platform_host::WallpaperCollection,
) {
    if let Some(existing) = library
        .collections
        .iter_mut()
        .find(|existing| existing.collection_id == collection.collection_id)
    {
        *existing = collection;
        return;
    }
    library.collections.push(collection);
}

/// Removes one imported wallpaper asset from the merged library view.
pub fn remove_imported_wallpaper_asset(
    library: &mut WallpaperLibrarySnapshot,
    asset_id: &str,
    used_bytes: u64,
) {
    library.assets.retain(|asset| asset.asset_id != asset_id);
    library.used_bytes = used_bytes;
}

/// Removes one wallpaper collection from the merged library view and strips asset memberships.
pub fn remove_wallpaper_collection(library: &mut WallpaperLibrarySnapshot, collection_id: &str) {
    library
        .collections
        .retain(|collection| collection.collection_id != collection_id);
    for asset in &mut library.assets {
        asset.collection_ids.retain(|id| id != collection_id);
    }
}

/// Resolves a wallpaper configuration against the merged library.
pub fn resolve_wallpaper_source(
    config: &WallpaperConfig,
    library: &WallpaperLibrarySnapshot,
) -> Option<ResolvedWallpaperSource> {
    match &config.selection {
        WallpaperSelection::BuiltIn { wallpaper_id } => builtin_wallpaper_records()
            .into_iter()
            .find(|asset| asset.asset_id == *wallpaper_id)
            .map(to_resolved_source),
        WallpaperSelection::Imported { asset_id } => library
            .assets
            .iter()
            .find(|asset| asset.asset_id == *asset_id)
            .cloned()
            .map(to_resolved_source),
    }
}

/// Resolves a built-in wallpaper id, including legacy aliases, to a canonical id.
pub fn canonical_wallpaper_id(raw: &str) -> &str {
    match raw {
        "slate-grid" => "teal-grid",
        "" => "cloud-bands",
        other => other,
    }
}

/// Returns a built-in wallpaper record by id.
pub fn builtin_wallpaper_by_id(wallpaper_id: &str) -> Option<WallpaperAssetRecord> {
    let canonical = canonical_wallpaper_id(wallpaper_id);
    builtin_wallpaper_records()
        .into_iter()
        .find(|asset| asset.asset_id == canonical)
}

/// Returns the featured built-in wallpaper records used by quick-access shell menus.
pub fn featured_builtin_wallpapers() -> Vec<WallpaperAssetRecord> {
    let featured_ids: Vec<String> = builtin_catalog_entries()
        .iter()
        .filter(|entry| entry.featured)
        .map(|entry| entry.wallpaper_id.clone())
        .collect();
    builtin_wallpaper_records()
        .into_iter()
        .filter(|asset| featured_ids.contains(&asset.asset_id))
        .collect()
}

fn to_resolved_source(asset: WallpaperAssetRecord) -> ResolvedWallpaperSource {
    ResolvedWallpaperSource {
        primary_url: asset.primary_url,
        poster_url: asset.poster_url,
        media_kind: asset.media_kind,
        natural_width: asset.natural_width,
        natural_height: asset.natural_height,
        duration_ms: asset.duration_ms,
    }
}

fn mime_type_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default() {
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use platform_host::{
        WallpaperAnimationPolicy, WallpaperCollection, WallpaperDisplayMode, WallpaperPosition,
    };

    #[test]
    fn merges_builtins_with_imported_assets() {
        let imported = WallpaperLibrarySnapshot {
            assets: vec![WallpaperAssetRecord {
                asset_id: "wallpaper-1".to_string(),
                display_name: "Imported".to_string(),
                source_kind: WallpaperSourceKind::Imported,
                media_kind: WallpaperMediaKind::StaticImage,
                mime_type: "image/png".to_string(),
                byte_len: 42,
                natural_width: None,
                natural_height: None,
                duration_ms: None,
                favorite: false,
                tags: Vec::new(),
                collection_ids: Vec::new(),
                primary_url: "data:image/png;base64,abc".to_string(),
                poster_url: None,
                created_at_unix_ms: None,
                last_used_at_unix_ms: None,
            }],
            collections: vec![WallpaperCollection {
                collection_id: "collection-1".to_string(),
                display_name: "Favorites".to_string(),
                sort_order: 0,
            }],
            ..WallpaperLibrarySnapshot::default()
        };

        let merged = merged_wallpaper_library(&imported);
        assert!(merged
            .assets
            .iter()
            .any(|asset| asset.asset_id == "cloud-bands"));
        assert!(merged
            .assets
            .iter()
            .any(|asset| asset.asset_id == "wallpaper-1"));
        assert_eq!(merged.collections.len(), 1);
    }

    #[test]
    fn resolves_imported_sources_from_library() {
        let library = WallpaperLibrarySnapshot {
            assets: vec![WallpaperAssetRecord {
                asset_id: "wallpaper-1".to_string(),
                display_name: "Imported".to_string(),
                source_kind: WallpaperSourceKind::Imported,
                media_kind: WallpaperMediaKind::StaticImage,
                mime_type: "image/png".to_string(),
                byte_len: 42,
                natural_width: None,
                natural_height: None,
                duration_ms: None,
                favorite: false,
                tags: Vec::new(),
                collection_ids: Vec::new(),
                primary_url: "data:image/png;base64,abc".to_string(),
                poster_url: None,
                created_at_unix_ms: None,
                last_used_at_unix_ms: None,
            }],
            ..WallpaperLibrarySnapshot::default()
        };
        let config = WallpaperConfig {
            selection: WallpaperSelection::Imported {
                asset_id: "wallpaper-1".to_string(),
            },
            display_mode: WallpaperDisplayMode::Fill,
            position: WallpaperPosition::Center,
            animation: WallpaperAnimationPolicy::None,
        };
        let resolved = resolve_wallpaper_source(&config, &library).expect("resolved");
        assert!(resolved.primary_url.starts_with("data:image/png"));
    }

    #[test]
    fn upserts_imported_asset_without_reloading_library() {
        let mut library = merged_wallpaper_library(&WallpaperLibrarySnapshot::default());
        let original_len = library.assets.len();

        upsert_imported_wallpaper_asset(
            &mut library,
            WallpaperAssetRecord {
                asset_id: "wallpaper-1".to_string(),
                display_name: "Imported".to_string(),
                source_kind: WallpaperSourceKind::Imported,
                media_kind: WallpaperMediaKind::StaticImage,
                mime_type: "image/png".to_string(),
                byte_len: 42,
                natural_width: None,
                natural_height: None,
                duration_ms: None,
                favorite: true,
                tags: vec!["featured".to_string()],
                collection_ids: vec!["collection-1".to_string()],
                primary_url: "data:image/png;base64,abc".to_string(),
                poster_url: None,
                created_at_unix_ms: None,
                last_used_at_unix_ms: None,
            },
        );

        assert_eq!(library.assets.len(), original_len + 1);
        assert!(library.assets.iter().any(|asset| {
            asset.asset_id == "wallpaper-1"
                && asset.favorite
                && asset.collection_ids == vec!["collection-1".to_string()]
        }));
    }

    #[test]
    fn upserts_collection_without_reloading_library() {
        let mut library = WallpaperLibrarySnapshot::default();
        upsert_wallpaper_collection(
            &mut library,
            WallpaperCollection {
                collection_id: "favorites".to_string(),
                display_name: "Favorites".to_string(),
                sort_order: 1,
            },
        );
        upsert_wallpaper_collection(
            &mut library,
            WallpaperCollection {
                collection_id: "favorites".to_string(),
                display_name: "Pinned".to_string(),
                sort_order: 2,
            },
        );

        assert_eq!(library.collections.len(), 1);
        assert_eq!(library.collections[0].display_name, "Pinned");
        assert_eq!(library.collections[0].sort_order, 2);
    }

    #[test]
    fn removes_imported_asset_without_reloading_library() {
        let mut library = WallpaperLibrarySnapshot {
            assets: vec![WallpaperAssetRecord {
                asset_id: "wallpaper-1".to_string(),
                display_name: "Imported".to_string(),
                source_kind: WallpaperSourceKind::Imported,
                media_kind: WallpaperMediaKind::StaticImage,
                mime_type: "image/png".to_string(),
                byte_len: 42,
                natural_width: None,
                natural_height: None,
                duration_ms: None,
                favorite: false,
                tags: Vec::new(),
                collection_ids: vec!["collection-1".to_string()],
                primary_url: "data:image/png;base64,abc".to_string(),
                poster_url: None,
                created_at_unix_ms: None,
                last_used_at_unix_ms: None,
            }],
            collections: Vec::new(),
            soft_limit_bytes: 100,
            used_bytes: 42,
        };

        remove_imported_wallpaper_asset(&mut library, "wallpaper-1", 0);

        assert!(library.assets.is_empty());
        assert_eq!(library.used_bytes, 0);
    }

    #[test]
    fn removes_collection_memberships_without_reloading_library() {
        let mut library = WallpaperLibrarySnapshot {
            assets: vec![WallpaperAssetRecord {
                asset_id: "wallpaper-1".to_string(),
                display_name: "Imported".to_string(),
                source_kind: WallpaperSourceKind::Imported,
                media_kind: WallpaperMediaKind::StaticImage,
                mime_type: "image/png".to_string(),
                byte_len: 42,
                natural_width: None,
                natural_height: None,
                duration_ms: None,
                favorite: false,
                tags: Vec::new(),
                collection_ids: vec!["collection-1".to_string(), "collection-2".to_string()],
                primary_url: "data:image/png;base64,abc".to_string(),
                poster_url: None,
                created_at_unix_ms: None,
                last_used_at_unix_ms: None,
            }],
            collections: vec![WallpaperCollection {
                collection_id: "collection-1".to_string(),
                display_name: "Favorites".to_string(),
                sort_order: 0,
            }],
            soft_limit_bytes: 100,
            used_bytes: 42,
        };

        remove_wallpaper_collection(&mut library, "collection-1");

        assert!(library.collections.is_empty());
        assert_eq!(
            library.assets[0].collection_ids,
            vec!["collection-2".to_string()]
        );
    }
}
