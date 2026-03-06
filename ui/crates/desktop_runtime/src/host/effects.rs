//! Runtime-effect dispatch for the desktop host boundary.

use crate::{
    host::{app_bus, host_ui, persistence_effects, wallpaper_effects, DesktopHostContext},
    reducer::RuntimeEffect,
    runtime_context::DesktopRuntimeContext,
};

pub(super) fn run_runtime_effect(
    host: DesktopHostContext,
    runtime: DesktopRuntimeContext,
    effect: RuntimeEffect,
) {
    match effect {
        RuntimeEffect::ParseAndOpenDeepLink(deep_link) => {
            host_ui::open_deep_link(runtime, deep_link)
        }
        RuntimeEffect::PersistLayout => persistence_effects::persist_layout(host, runtime),
        RuntimeEffect::PersistTheme => persistence_effects::persist_theme(host, runtime),
        RuntimeEffect::PersistWallpaper => persistence_effects::persist_wallpaper(host, runtime),
        RuntimeEffect::PersistTerminalHistory => {
            persistence_effects::persist_terminal_history(host, runtime)
        }
        RuntimeEffect::OpenExternalUrl(url) => host_ui::open_external_url(host, &url),
        RuntimeEffect::FocusWindowInput(window_id) => host.focus_window_input(window_id),
        RuntimeEffect::PlaySound(_) => {}
        RuntimeEffect::DispatchLifecycle { window_id, event } => {
            app_bus::dispatch_lifecycle(runtime, window_id, event);
        }
        RuntimeEffect::DeliverAppEvent { window_id, event } => {
            app_bus::deliver_app_event(runtime, window_id, event);
        }
        RuntimeEffect::SubscribeWindowTopic { window_id, topic } => {
            app_bus::subscribe_topic(runtime, window_id, topic);
        }
        RuntimeEffect::UnsubscribeWindowTopic { window_id, topic } => {
            app_bus::unsubscribe_topic(runtime, window_id, topic);
        }
        RuntimeEffect::PublishTopicEvent {
            source_window_id,
            topic,
            payload,
            correlation_id,
            reply_to,
        } => app_bus::publish_event(
            runtime,
            source_window_id,
            topic,
            payload,
            correlation_id,
            reply_to,
        ),
        RuntimeEffect::SaveConfig {
            namespace,
            key,
            value,
        } => persistence_effects::save_config(host, namespace, key, value),
        RuntimeEffect::LoadWallpaperLibrary => wallpaper_effects::load_library(host, runtime),
        RuntimeEffect::ImportWallpaperFromPicker { request } => {
            wallpaper_effects::import_from_picker(host, runtime, request);
        }
        RuntimeEffect::UpdateWallpaperAssetMetadata { asset_id, patch } => {
            wallpaper_effects::update_asset_metadata(host, runtime, asset_id, patch);
        }
        RuntimeEffect::CreateWallpaperCollection { display_name } => {
            wallpaper_effects::create_collection(host, runtime, display_name);
        }
        RuntimeEffect::RenameWallpaperCollection {
            collection_id,
            display_name,
        } => wallpaper_effects::rename_collection(host, runtime, collection_id, display_name),
        RuntimeEffect::DeleteWallpaperCollection { collection_id } => {
            wallpaper_effects::delete_collection(host, runtime, collection_id);
        }
        RuntimeEffect::DeleteWallpaperAsset { asset_id } => {
            wallpaper_effects::delete_asset(host, runtime, asset_id);
        }
        RuntimeEffect::Notify { title, body } => host_ui::notify(host, title, body),
    }
}
