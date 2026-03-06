//! Reducer helpers for desktop theme and wallpaper transitions.

use platform_host::{
    WallpaperConfig, WallpaperDisplayMode, WallpaperMediaKind, WallpaperSelection,
};

use crate::{
    model::{DesktopSkin, DesktopState},
    reducer::{DesktopAction, ReducerError, RuntimeEffect},
    wallpaper,
};

pub(super) fn desktop_skin_from_id(skin_id: &str) -> Option<DesktopSkin> {
    match skin_id.trim() {
        "soft-neumorphic" => Some(DesktopSkin::SoftNeumorphic),
        "modern-adaptive" => Some(DesktopSkin::ModernAdaptive),
        "classic-xp" => Some(DesktopSkin::ClassicXp),
        "classic-95" => Some(DesktopSkin::Classic95),
        _ => None,
    }
}

pub(super) fn reduce_appearance_action(
    state: &mut DesktopState,
    action: &DesktopAction,
    effects: &mut Vec<RuntimeEffect>,
) -> Result<bool, ReducerError> {
    match action {
        DesktopAction::SetSkin { skin } => {
            state.theme.skin = *skin;
            effects.push(RuntimeEffect::PersistTheme);
        }
        DesktopAction::SetCurrentWallpaper { config } => {
            state.wallpaper = validate_wallpaper_config(state, config)?;
            state.wallpaper_preview = None;
            effects.push(RuntimeEffect::PersistWallpaper);
        }
        DesktopAction::PreviewWallpaper { config } => {
            state.wallpaper_preview = Some(validate_wallpaper_config(state, config)?);
        }
        DesktopAction::ApplyWallpaperPreview => {
            if let Some(config) = state.wallpaper_preview.clone() {
                state.wallpaper = validate_wallpaper_config(state, &config)?;
                state.wallpaper_preview = None;
                effects.push(RuntimeEffect::PersistWallpaper);
            }
        }
        DesktopAction::ClearWallpaperPreview => {
            state.wallpaper_preview = None;
        }
        DesktopAction::HydrateTheme { theme } => {
            state.theme = theme.clone();
        }
        DesktopAction::HydrateWallpaper { wallpaper } => {
            state.wallpaper = canonicalize_wallpaper_config(wallpaper.clone());
            state.wallpaper_preview = None;
        }
        DesktopAction::WallpaperLibraryLoaded { snapshot } => {
            state.wallpaper_library = wallpaper::merged_wallpaper_library(snapshot);
            normalize_wallpaper_state(state);
        }
        DesktopAction::WallpaperAssetUpdated { asset } => {
            wallpaper::upsert_imported_wallpaper_asset(&mut state.wallpaper_library, asset.clone());
            normalize_wallpaper_state(state);
        }
        DesktopAction::WallpaperCollectionUpdated { collection } => {
            wallpaper::upsert_wallpaper_collection(
                &mut state.wallpaper_library,
                collection.clone(),
            );
            normalize_wallpaper_state(state);
        }
        DesktopAction::WallpaperCollectionDeleted { collection_id } => {
            wallpaper::remove_wallpaper_collection(&mut state.wallpaper_library, collection_id);
            normalize_wallpaper_state(state);
        }
        DesktopAction::WallpaperAssetDeleted {
            asset_id,
            used_bytes,
        } => {
            wallpaper::remove_imported_wallpaper_asset(
                &mut state.wallpaper_library,
                asset_id,
                *used_bytes,
            );
            normalize_wallpaper_state(state);
        }
        DesktopAction::SetHighContrast { enabled } => {
            state.theme.high_contrast = *enabled;
            effects.push(RuntimeEffect::PersistTheme);
        }
        DesktopAction::SetReducedMotion { enabled } => {
            state.theme.reduced_motion = *enabled;
            effects.push(RuntimeEffect::PersistTheme);
        }
        _ => return Ok(false),
    }

    Ok(true)
}

fn default_builtin_wallpaper() -> WallpaperConfig {
    WallpaperConfig {
        selection: WallpaperSelection::BuiltIn {
            wallpaper_id: "cloud-bands".to_string(),
        },
        display_mode: WallpaperDisplayMode::Fill,
        ..WallpaperConfig::default()
    }
}

fn canonicalize_wallpaper_config(mut config: WallpaperConfig) -> WallpaperConfig {
    if let WallpaperSelection::BuiltIn { wallpaper_id } = &mut config.selection {
        *wallpaper_id = wallpaper::canonical_wallpaper_id(wallpaper_id).to_string();
    }
    config
}

fn wallpaper_media_kind_for_config(
    state: &DesktopState,
    config: &WallpaperConfig,
) -> Option<WallpaperMediaKind> {
    wallpaper::resolve_wallpaper_source(config, &state.wallpaper_library)
        .map(|source| source.media_kind)
}

fn validate_wallpaper_config(
    state: &DesktopState,
    config: &WallpaperConfig,
) -> Result<WallpaperConfig, ReducerError> {
    let config = canonicalize_wallpaper_config(config.clone());
    let media_kind = wallpaper_media_kind_for_config(state, &config)
        .or_else(|| match &config.selection {
            WallpaperSelection::BuiltIn { wallpaper_id } => {
                wallpaper::builtin_wallpaper_by_id(wallpaper_id).map(|asset| asset.media_kind)
            }
            WallpaperSelection::Imported { .. } => None,
        })
        .ok_or_else(|| {
            ReducerError::InvalidWallpaperConfig("wallpaper asset not found".to_string())
        })?;

    if config.display_mode == WallpaperDisplayMode::Tile
        && matches!(
            media_kind,
            WallpaperMediaKind::AnimatedImage | WallpaperMediaKind::Video
        )
    {
        return Err(ReducerError::InvalidWallpaperConfig(
            "tile mode is unsupported for animated wallpapers".to_string(),
        ));
    }

    Ok(config)
}

fn normalize_wallpaper_state(state: &mut DesktopState) {
    let current_missing =
        wallpaper::resolve_wallpaper_source(&state.wallpaper, &state.wallpaper_library).is_none();
    if current_missing {
        state.wallpaper = default_builtin_wallpaper();
    }
    if let Some(preview) = state.wallpaper_preview.clone() {
        if wallpaper::resolve_wallpaper_source(&preview, &state.wallpaper_library).is_none() {
            state.wallpaper_preview = None;
        }
    }
}
