use leptos::{create_effect, spawn_local, Callable, Callback};

use crate::{
    current_browser_e2e_config, host::DesktopHostContext, persistence, reducer::DesktopAction,
};

pub(super) fn install_boot_hydration(host: DesktopHostContext, dispatch: Callback<DesktopAction>) {
    create_effect(move |_| {
        let dispatch = dispatch;
        let boot_host = host.clone();
        spawn_local(async move {
            let browser_e2e_active = current_browser_e2e_config().is_some();

            if !browser_e2e_active {
                let legacy_snapshot = persistence::load_boot_snapshot(&boot_host).await;
                let durable_snapshot = persistence::load_durable_boot_snapshot(&boot_host).await;
                let restore_preferences = persistence::resolve_restore_preferences(
                    durable_snapshot.as_ref(),
                    legacy_snapshot.as_ref(),
                );

                if let Some(policy) = persistence::load_app_policy_overlay(&boot_host).await {
                    dispatch.call(DesktopAction::HydratePolicyOverlay {
                        privileged_app_ids: policy.privileged_app_ids,
                    });
                }
                if restore_preferences.restore_on_boot {
                    if let Some(snapshot) = legacy_snapshot.clone() {
                        dispatch.call(DesktopAction::HydrateSnapshot {
                            snapshot,
                            mode: crate::reducer::HydrationMode::BootRestore,
                        });
                    }
                }

                if let Some(theme) = persistence::load_theme(&boot_host).await {
                    dispatch.call(DesktopAction::HydrateTheme { theme });
                }

                if let Some(wallpaper) = persistence::load_wallpaper(&boot_host).await {
                    dispatch.call(DesktopAction::HydrateWallpaper { wallpaper });
                }

                if restore_preferences.restore_on_boot {
                    if let Some(snapshot) = durable_snapshot.clone() {
                        dispatch.call(DesktopAction::HydrateSnapshot {
                            snapshot,
                            mode: crate::reducer::HydrationMode::BootRestore,
                        });
                    }
                } else if let Some(snapshot) = legacy_snapshot.clone() {
                    let migrated_state = crate::model::DesktopState::from_snapshot(snapshot);
                    if let Err(err) =
                        persistence::persist_durable_layout_snapshot(&boot_host, &migrated_state)
                            .await
                    {
                        tracing::warn!("migrate legacy snapshot to durable store failed: {err}");
                    }
                }
            }

            dispatch.call(DesktopAction::BootHydrationComplete);
        });

        let host = host.clone();
        spawn_local(async move {
            match host.wallpaper_asset_service().list_library().await {
                Ok(snapshot) => {
                    dispatch.call(DesktopAction::WallpaperLibraryLoaded { snapshot });
                }
                Err(err) => tracing::warn!("wallpaper library load failed: {err}"),
            }
        });
    });
}
