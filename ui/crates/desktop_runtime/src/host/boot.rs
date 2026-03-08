use leptos::{create_effect, logging, spawn_local, Callable, Callback};

use crate::{
    current_browser_e2e_config,
    host::DesktopHostContext,
    model::{DeepLinkState, DesktopSnapshot, DesktopTheme},
    persistence::{self, AppPolicyOverlay, DurableDesktopSnapshot},
    reducer::DesktopAction,
    WallpaperConfig,
};

#[derive(Debug, Clone, PartialEq)]
enum AuthoritativeBootSnapshot {
    Durable(DurableDesktopSnapshot),
    Legacy(DesktopSnapshot),
}

#[derive(Debug, Clone, PartialEq)]
struct BootHydrationPlan {
    snapshot: Option<AuthoritativeBootSnapshot>,
    migrate_legacy_snapshot: Option<DesktopSnapshot>,
    theme: Option<DesktopTheme>,
    wallpaper: Option<WallpaperConfig>,
    policy_overlay: Option<AppPolicyOverlay>,
    deep_link: Option<DeepLinkState>,
}

impl BootHydrationPlan {
    fn empty(initial_deep_link: Option<DeepLinkState>) -> Self {
        Self {
            snapshot: None,
            migrate_legacy_snapshot: None,
            theme: None,
            wallpaper: None,
            policy_overlay: None,
            deep_link: initial_deep_link.filter(|deep_link| !deep_link.open.is_empty()),
        }
    }
}

pub(super) fn install_boot_hydration(
    host: DesktopHostContext,
    dispatch: Callback<DesktopAction>,
    initial_deep_link: Option<DeepLinkState>,
) {
    create_effect(move |_| {
        let dispatch = dispatch;
        let boot_host = host.clone();
        let boot_deep_link = initial_deep_link.clone();
        spawn_local(async move {
            let browser_e2e_active = current_browser_e2e_config().is_some();
            let plan = if browser_e2e_active {
                BootHydrationPlan::empty(None)
            } else {
                let legacy_snapshot = persistence::load_boot_snapshot(&boot_host).await;
                let durable_snapshot =
                    persistence::load_durable_boot_snapshot_record(&boot_host).await;
                let theme = persistence::load_theme(&boot_host).await;
                let wallpaper = persistence::load_wallpaper(&boot_host).await;
                let policy_overlay = persistence::load_app_policy_overlay(&boot_host).await;
                resolve_boot_hydration_plan(
                    durable_snapshot,
                    legacy_snapshot,
                    theme,
                    wallpaper,
                    policy_overlay,
                    boot_deep_link,
                )
            };

            let (snapshot, snapshot_revision) =
                resolve_authoritative_snapshot(&boot_host, &plan).await;

            dispatch.call(DesktopAction::CompleteBootHydration {
                snapshot,
                snapshot_revision,
                theme: plan.theme,
                wallpaper: plan.wallpaper,
                privileged_app_ids: plan
                    .policy_overlay
                    .map(|policy| policy.privileged_app_ids)
                    .unwrap_or_default(),
                deep_link: plan.deep_link,
            });
        });

        let host = host.clone();
        spawn_local(async move {
            match host.wallpaper_asset_service().list_library().await {
                Ok(snapshot) => {
                    dispatch.call(DesktopAction::WallpaperLibraryLoaded { snapshot });
                }
                Err(err) => logging::warn!("wallpaper library load failed: {err}"),
            }
        });
    });
}

fn resolve_boot_hydration_plan(
    durable_snapshot: Option<DurableDesktopSnapshot>,
    legacy_snapshot: Option<DesktopSnapshot>,
    theme: Option<DesktopTheme>,
    wallpaper: Option<WallpaperConfig>,
    policy_overlay: Option<AppPolicyOverlay>,
    deep_link: Option<DeepLinkState>,
) -> BootHydrationPlan {
    let durable_present = durable_snapshot.is_some();
    let restore_preferences = persistence::resolve_restore_preferences(
        durable_snapshot.as_ref().map(|snapshot| &snapshot.snapshot),
        legacy_snapshot.as_ref(),
    );
    let restore_snapshot = if restore_preferences.restore_on_boot {
        durable_snapshot
            .map(AuthoritativeBootSnapshot::Durable)
            .or_else(|| {
                legacy_snapshot
                    .clone()
                    .map(AuthoritativeBootSnapshot::Legacy)
            })
    } else {
        None
    };
    let migrate_legacy_snapshot = (!durable_present).then_some(legacy_snapshot).flatten();

    BootHydrationPlan {
        snapshot: restore_snapshot,
        migrate_legacy_snapshot,
        theme,
        wallpaper,
        policy_overlay,
        deep_link: deep_link.filter(|parsed| !parsed.open.is_empty()),
    }
}

async fn resolve_authoritative_snapshot(
    host: &DesktopHostContext,
    plan: &BootHydrationPlan,
) -> (Option<DesktopSnapshot>, Option<u64>) {
    let Some(migration_snapshot) = plan.migrate_legacy_snapshot.clone() else {
        return match &plan.snapshot {
            Some(AuthoritativeBootSnapshot::Durable(snapshot)) => {
                (Some(snapshot.snapshot.clone()), Some(snapshot.revision))
            }
            Some(AuthoritativeBootSnapshot::Legacy(snapshot)) => (Some(snapshot.clone()), None),
            None => (None, None),
        };
    };

    let migrated_state = crate::model::DesktopState::from_snapshot(migration_snapshot.clone());
    match persistence::build_durable_layout_envelope(&migrated_state) {
        Ok(envelope) => {
            let revision = envelope.updated_at_unix_ms;
            if let Err(err) = persistence::save_durable_layout_envelope(host, &envelope).await {
                logging::warn!("migrate legacy snapshot to durable store failed: {err}");
            }

            match &plan.snapshot {
                Some(AuthoritativeBootSnapshot::Durable(snapshot)) => {
                    (Some(snapshot.snapshot.clone()), Some(snapshot.revision))
                }
                Some(AuthoritativeBootSnapshot::Legacy(_)) => {
                    (Some(migration_snapshot), Some(revision))
                }
                None => (None, Some(revision)),
            }
        }
        Err(err) => {
            logging::warn!("build durable snapshot envelope for legacy migration failed: {err}");
            match &plan.snapshot {
                Some(AuthoritativeBootSnapshot::Durable(snapshot)) => {
                    (Some(snapshot.snapshot.clone()), Some(snapshot.revision))
                }
                Some(AuthoritativeBootSnapshot::Legacy(snapshot)) => (Some(snapshot.clone()), None),
                None => (None, None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DesktopPreferences;

    fn snapshot_with_restore(restore_on_boot: bool) -> DesktopSnapshot {
        let mut snapshot = crate::model::DesktopState::default().snapshot();
        snapshot.preferences = DesktopPreferences {
            restore_on_boot,
            ..DesktopPreferences::default()
        };
        snapshot
    }

    #[test]
    fn durable_snapshot_is_authoritative_for_boot_restore() {
        let durable = DurableDesktopSnapshot {
            snapshot: snapshot_with_restore(true),
            revision: 44,
        };
        let legacy = snapshot_with_restore(true);
        let plan = resolve_boot_hydration_plan(
            Some(durable.clone()),
            Some(legacy),
            None,
            None,
            None,
            None,
        );

        assert_eq!(
            plan.snapshot,
            Some(AuthoritativeBootSnapshot::Durable(durable))
        );
        assert!(plan.migrate_legacy_snapshot.is_none());
    }

    #[test]
    fn legacy_snapshot_is_migrated_even_when_restore_is_disabled() {
        let legacy = snapshot_with_restore(false);
        let plan = resolve_boot_hydration_plan(None, Some(legacy.clone()), None, None, None, None);

        assert!(plan.snapshot.is_none());
        assert_eq!(plan.migrate_legacy_snapshot, Some(legacy));
    }

    #[test]
    fn legacy_snapshot_restores_once_and_migrates_once() {
        let legacy = snapshot_with_restore(true);
        let plan = resolve_boot_hydration_plan(None, Some(legacy.clone()), None, None, None, None);

        assert_eq!(
            plan.snapshot,
            Some(AuthoritativeBootSnapshot::Legacy(legacy.clone()))
        );
        assert_eq!(plan.migrate_legacy_snapshot, Some(legacy));
    }
}
