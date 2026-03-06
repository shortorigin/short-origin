//! Browser-backed wallpaper asset service implementation.

use platform_host::{
    build_app_state_envelope, migrate_envelope_payload, next_monotonic_timestamp_ms,
    ResolvedWallpaperSource, WallpaperAssetDeleteResult, WallpaperAssetFuture,
    WallpaperAssetMetadataPatch, WallpaperAssetRecord, WallpaperAssetService, WallpaperCollection,
    WallpaperCollectionDeleteResult, WallpaperImportRequest, WallpaperImportResult,
    WallpaperLibrarySnapshot, WallpaperMediaKind, WallpaperSelection, WallpaperSourceKind,
};

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

const WALLPAPER_LIBRARY_NAMESPACE: &str = "system.wallpaper_library.v1";
const WALLPAPER_LIBRARY_SCHEMA_VERSION: u32 = 1;
const STILL_IMAGE_LIMIT_BYTES: u64 = 25 * 1024 * 1024;
const ANIMATED_LIMIT_BYTES: u64 = 150 * 1024 * 1024;
const VIDEO_PLACEHOLDER_POSTER: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 480 320'%3E%3Cdefs%3E%3ClinearGradient id='g' x1='0' y1='0' x2='1' y2='1'%3E%3Cstop stop-color='%23121827'/%3E%3Cstop offset='1' stop-color='%23203a5b'/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect width='480' height='320' fill='url(%23g)'/%3E%3Crect x='144' y='90' width='192' height='140' rx='18' fill='rgba(255,255,255,0.12)' stroke='rgba(255,255,255,0.35)'/%3E%3Cpath d='M214 132l58 29-58 29z' fill='white'/%3E%3Ctext x='240' y='268' text-anchor='middle' font-size='22' font-family='ui-sans-serif,system-ui,sans-serif' fill='rgba(255,255,255,0.82)'%3EImported video%3C/text%3E%3C/svg%3E";

#[derive(Debug, Clone, Copy, Default)]
/// Browser wallpaper asset service backed by IndexedDB app-state plus file-picker imports.
pub struct WebWallpaperAssetService;

impl WallpaperAssetService for WebWallpaperAssetService {
    fn import_from_picker<'a>(
        &'a self,
        request: WallpaperImportRequest,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperImportResult, String>> {
        Box::pin(async move {
            let picked = pick_file().await?;
            let library = load_library_snapshot().await?;
            let record = build_import_record(&picked, request)?;
            let mut next = library;
            next.used_bytes = next.used_bytes.saturating_add(record.byte_len);
            if next.used_bytes > next.soft_limit_bytes {
                return Err(format!(
                    "wallpaper library soft limit exceeded ({} > {})",
                    next.used_bytes, next.soft_limit_bytes
                ));
            }
            next.assets.push(record.clone());
            save_library_snapshot(&next).await?;
            Ok(WallpaperImportResult {
                asset: record,
                soft_limit_bytes: next.soft_limit_bytes,
                used_bytes: next.used_bytes,
            })
        })
    }

    fn list_library<'a>(
        &'a self,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperLibrarySnapshot, String>> {
        Box::pin(async move { load_library_snapshot().await })
    }

    fn update_asset_metadata<'a>(
        &'a self,
        asset_id: &'a str,
        patch: WallpaperAssetMetadataPatch,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetRecord, String>> {
        Box::pin(async move {
            let mut library = load_library_snapshot().await?;
            let asset = library
                .assets
                .iter_mut()
                .find(|asset| asset.asset_id == asset_id)
                .ok_or_else(|| format!("wallpaper asset not found: {asset_id}"))?;
            if let Some(display_name) = patch.display_name {
                asset.display_name = display_name;
            }
            if let Some(favorite) = patch.favorite {
                asset.favorite = favorite;
            }
            if let Some(tags) = patch.tags {
                asset.tags = tags;
            }
            if let Some(collection_ids) = patch.collection_ids {
                asset.collection_ids = collection_ids;
            }
            asset.last_used_at_unix_ms = Some(platform_host::unix_time_ms_now());
            let updated = asset.clone();
            save_library_snapshot(&library).await?;
            Ok(updated)
        })
    }

    fn create_collection<'a>(
        &'a self,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        Box::pin(async move {
            let mut library = load_library_snapshot().await?;
            let collection = WallpaperCollection {
                collection_id: format!("collection-{}", next_monotonic_timestamp_ms()),
                display_name: display_name.trim().to_string(),
                sort_order: library.collections.len() as i32,
            };
            library.collections.push(collection.clone());
            save_library_snapshot(&library).await?;
            Ok(collection)
        })
    }

    fn rename_collection<'a>(
        &'a self,
        collection_id: &'a str,
        display_name: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollection, String>> {
        Box::pin(async move {
            let mut library = load_library_snapshot().await?;
            let collection = library
                .collections
                .iter_mut()
                .find(|collection| collection.collection_id == collection_id)
                .ok_or_else(|| format!("wallpaper collection not found: {collection_id}"))?;
            collection.display_name = display_name.trim().to_string();
            let updated = collection.clone();
            save_library_snapshot(&library).await?;
            Ok(updated)
        })
    }

    fn delete_collection<'a>(
        &'a self,
        collection_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperCollectionDeleteResult, String>> {
        Box::pin(async move {
            let mut library = load_library_snapshot().await?;
            library
                .collections
                .retain(|collection| collection.collection_id != collection_id);
            for asset in &mut library.assets {
                asset.collection_ids.retain(|id| id != collection_id);
            }
            save_library_snapshot(&library).await?;
            Ok(WallpaperCollectionDeleteResult {
                collection_id: collection_id.to_string(),
            })
        })
    }

    fn delete_asset<'a>(
        &'a self,
        asset_id: &'a str,
    ) -> WallpaperAssetFuture<'a, Result<WallpaperAssetDeleteResult, String>> {
        Box::pin(async move {
            let mut library = load_library_snapshot().await?;
            let before = library.assets.len();
            library.assets.retain(|asset| asset.asset_id != asset_id);
            if library.assets.len() == before {
                return Err(format!("wallpaper asset not found: {asset_id}"));
            }
            library.used_bytes = library.assets.iter().map(|asset| asset.byte_len).sum();
            save_library_snapshot(&library).await?;
            Ok(WallpaperAssetDeleteResult {
                asset_id: asset_id.to_string(),
                used_bytes: library.used_bytes,
            })
        })
    }

    fn resolve_source<'a>(
        &'a self,
        selection: WallpaperSelection,
    ) -> WallpaperAssetFuture<'a, Result<Option<ResolvedWallpaperSource>, String>> {
        Box::pin(async move {
            match selection {
                WallpaperSelection::BuiltIn { .. } => Ok(None),
                WallpaperSelection::Imported { asset_id } => {
                    let library = load_library_snapshot().await?;
                    Ok(library
                        .assets
                        .into_iter()
                        .find(|asset| asset.asset_id == asset_id)
                        .map(|asset| ResolvedWallpaperSource {
                            primary_url: asset.primary_url,
                            poster_url: asset.poster_url,
                            media_kind: asset.media_kind,
                            natural_width: asset.natural_width,
                            natural_height: asset.natural_height,
                            duration_ms: asset.duration_ms,
                        }))
                }
            }
        })
    }
}

#[derive(Debug)]
struct PickedFile {
    name: String,
    mime_type: String,
    size: u64,
    data_url: String,
}

fn build_import_record(
    picked: &PickedFile,
    request: WallpaperImportRequest,
) -> Result<WallpaperAssetRecord, String> {
    let media_kind = classify_media_kind(&picked.name, &picked.mime_type)?;
    let limit = match media_kind {
        WallpaperMediaKind::StaticImage | WallpaperMediaKind::Svg => STILL_IMAGE_LIMIT_BYTES,
        WallpaperMediaKind::AnimatedImage | WallpaperMediaKind::Video => ANIMATED_LIMIT_BYTES,
    };
    if picked.size > limit {
        return Err(format!(
            "selected wallpaper exceeds size limit ({} > {})",
            picked.size, limit
        ));
    }

    let stem = picked
        .name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(&picked.name)
        .trim();
    let display_name = request
        .display_name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| stem.to_string());
    let asset_id = format!("wallpaper-{}", next_monotonic_timestamp_ms());
    let poster_url = match media_kind {
        WallpaperMediaKind::Video => Some(VIDEO_PLACEHOLDER_POSTER.to_string()),
        _ => None,
    };

    Ok(WallpaperAssetRecord {
        asset_id,
        display_name,
        source_kind: WallpaperSourceKind::Imported,
        media_kind,
        mime_type: picked.mime_type.clone(),
        byte_len: picked.size,
        natural_width: None,
        natural_height: None,
        duration_ms: None,
        favorite: false,
        tags: Vec::new(),
        collection_ids: Vec::new(),
        primary_url: picked.data_url.clone(),
        poster_url,
        created_at_unix_ms: Some(platform_host::unix_time_ms_now()),
        last_used_at_unix_ms: None,
    })
}

fn classify_media_kind(name: &str, mime_type: &str) -> Result<WallpaperMediaKind, String> {
    let extension = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" => Ok(WallpaperMediaKind::StaticImage),
        "svg" => Ok(WallpaperMediaKind::Svg),
        "gif" => Ok(WallpaperMediaKind::AnimatedImage),
        "mp4" | "webm" => Ok(WallpaperMediaKind::Video),
        _ if mime_type == "image/svg+xml" => Ok(WallpaperMediaKind::Svg),
        _ if mime_type.starts_with("image/") => Ok(WallpaperMediaKind::StaticImage),
        _ if mime_type.starts_with("video/") => Ok(WallpaperMediaKind::Video),
        _ => Err(format!("unsupported wallpaper format: {name}")),
    }
}

async fn load_library_snapshot() -> Result<WallpaperLibrarySnapshot, String> {
    let Some(envelope) =
        crate::bridge::load_app_state_envelope(WALLPAPER_LIBRARY_NAMESPACE).await?
    else {
        return Ok(WallpaperLibrarySnapshot::default());
    };
    if envelope.schema_version != WALLPAPER_LIBRARY_SCHEMA_VERSION {
        return Ok(WallpaperLibrarySnapshot::default());
    }
    migrate_envelope_payload::<WallpaperLibrarySnapshot>(&envelope)
}

async fn save_library_snapshot(snapshot: &WallpaperLibrarySnapshot) -> Result<(), String> {
    let envelope = build_app_state_envelope(
        WALLPAPER_LIBRARY_NAMESPACE,
        WALLPAPER_LIBRARY_SCHEMA_VERSION,
        snapshot,
    )?;
    crate::bridge::save_app_state_envelope(&envelope).await
}

async fn pick_file() -> Result<PickedFile, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err("wallpaper import is only available when compiled for wasm32".to_string())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
        let document = window
            .document()
            .ok_or_else(|| "document unavailable".to_string())?;
        let input = document
            .create_element("input")
            .map_err(|err| format!("failed to create file input: {err:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|_| "failed to cast file input".to_string())?;
        input.set_type("file");
        input.set_accept(
            "image/png,image/jpeg,image/webp,image/svg+xml,image/gif,video/mp4,video/webm",
        );
        input.set_hidden(true);

        if let Some(body) = document.body() {
            let _ = body.append_child(&input);
        }

        let (tx, rx) = oneshot::channel::<Result<web_sys::File, String>>();
        let sender = Rc::new(RefCell::new(Some(tx)));
        let input_for_change = input.clone();
        let change_sender = sender.clone();
        let on_change = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_| {
            let result = input_for_change
                .files()
                .and_then(|files| files.get(0))
                .ok_or_else(|| "no wallpaper file selected".to_string());
            if let Some(tx) = change_sender.borrow_mut().take() {
                let _ = tx.send(result);
            }
        }));
        input.set_onchange(Some(on_change.as_ref().unchecked_ref()));
        input.click();

        let file = rx
            .await
            .map_err(|_| "wallpaper picker was cancelled".to_string())??;
        input.remove();
        on_change.forget();

        let data_url = read_file_as_data_url(&file).await?;
        Ok(PickedFile {
            name: file.name(),
            mime_type: file.type_(),
            size: file.size() as u64,
            data_url,
        })
    }
}

#[cfg(target_arch = "wasm32")]
async fn read_file_as_data_url(file: &web_sys::File) -> Result<String, String> {
    let reader = web_sys::FileReader::new().map_err(|err| format!("{err:?}"))?;
    let (tx, rx) = oneshot::channel::<Result<String, String>>();
    let sender = Rc::new(RefCell::new(Some(tx)));

    let reader_for_load = reader.clone();
    let load_sender = sender.clone();
    let on_load = Closure::<dyn FnMut(web_sys::ProgressEvent)>::wrap(Box::new(move |_| {
        let result = reader_for_load
            .result()
            .map_err(|err| format!("failed to read wallpaper file: {err:?}"))
            .and_then(|value| {
                value
                    .as_string()
                    .ok_or_else(|| "file reader returned non-string result".to_string())
            });
        if let Some(tx) = load_sender.borrow_mut().take() {
            let _ = tx.send(result);
        }
    }));
    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));

    let error_sender = sender.clone();
    let on_error = Closure::<dyn FnMut(web_sys::ProgressEvent)>::wrap(Box::new(move |_| {
        if let Some(tx) = error_sender.borrow_mut().take() {
            let _ = tx.send(Err("failed to load wallpaper file".to_string()));
        }
    }));
    reader.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    reader
        .read_as_data_url(file)
        .map_err(|err| format!("failed to start file read: {err:?}"))?;

    let result = rx
        .await
        .map_err(|_| "wallpaper file read was interrupted".to_string())?;
    on_load.forget();
    on_error.forget();
    result
}
