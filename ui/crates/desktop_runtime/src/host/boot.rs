use leptos::{create_effect, logging, spawn_local, Callable, Callback};

use crate::{
    current_browser_e2e_config, host::DesktopHostContext, persistence, reducer::DesktopAction,
};

pub(super) fn install_boot_hydration(host: DesktopHostContext, dispatch: Callback<DesktopAction>) {
    create_effect(move |_| {
        let dispatch = dispatch;
        let host = host.clone();
        spawn_local(async move {
            let browser_e2e_active = current_browser_e2e_config().is_some();

            if !browser_e2e_active {
                let legacy_snapshot = persistence::load_boot_snapshot(&host).await;
                if let Some(snapshot) = legacy_snapshot.clone() {
                    dispatch.call(DesktopAction::HydrateSnapshot { snapshot });
                }

                if let Some(theme) = persistence::load_theme(&host).await {
                    dispatch.call(DesktopAction::HydrateTheme { theme });
                }

                if let Some(wallpaper) = persistence::load_wallpaper(&host).await {
                    dispatch.call(DesktopAction::HydrateWallpaper { wallpaper });
                }

                if let Some(snapshot) = persistence::load_durable_boot_snapshot(&host).await {
                    dispatch.call(DesktopAction::HydrateSnapshot { snapshot });
                } else if let Some(snapshot) = legacy_snapshot {
                    let migrated_state = crate::model::DesktopState::from_snapshot(snapshot);
                    if let Err(err) =
                        persistence::persist_durable_layout_snapshot(&host, &migrated_state).await
                    {
                        logging::warn!("migrate legacy snapshot to durable store failed: {err}");
                    }
                }
            }

            match host.wallpaper_asset_service().list_library().await {
                Ok(snapshot) => {
                    dispatch.call(DesktopAction::WallpaperLibraryLoaded { snapshot });
                }
                Err(err) => logging::warn!("wallpaper library load failed: {err}"),
            }

            dispatch.call(DesktopAction::BootHydrationComplete);
        });
    });
}
