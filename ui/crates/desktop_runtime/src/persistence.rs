//! Desktop runtime persistence adapters for boot hydration and lightweight local preferences.

use crate::host::DesktopHostContext;
use crate::model::{DesktopPreferences, DesktopSnapshot, DesktopState, DesktopTheme};
use leptos::logging;
use platform_host::build_app_state_envelope;
use platform_host::{
    load_pref_with, migrate_envelope_payload, save_pref_with, AppStateEnvelope, HostResult,
    WallpaperConfig, WallpaperSelection, DESKTOP_STATE_NAMESPACE,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
const SNAPSHOT_KEY: &str = "retrodesk.layout.v1";
const LEGACY_THEME_KEY: &str = "retrodesk.theme.v1";
const THEME_KEY: &str = "system.desktop_theme.v2";
const WALLPAPER_KEY: &str = "system.desktop_wallpaper.v1";
const TERMINAL_HISTORY_KEY: &str = "retrodesk.terminal_history.v1";
/// Persisted runtime policy overlay key for app capability grants.
pub const APP_POLICY_KEY: &str = "system.app_policy.v1";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
/// Persisted capability policy overlay keyed by app id.
pub struct AppPolicyOverlay {
    /// App ids treated as privileged by shell policy.
    pub privileged_app_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Typed durable desktop snapshot with the applied app-state revision.
pub struct DurableDesktopSnapshot {
    /// Decoded desktop snapshot payload.
    pub snapshot: DesktopSnapshot,
    /// Monotonic durable app-state revision.
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LegacyThemePayload {
    wallpaper_id: String,
    high_contrast: bool,
    reduced_motion: bool,
    audio_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct LegacyDesktopSnapshotV1 {
    schema_version: u32,
    theme: LegacyThemePayload,
    preferences: crate::model::DesktopPreferences,
    windows: Vec<crate::model::WindowRecord>,
    terminal_history: Vec<String>,
    #[serde(default)]
    app_shared_state: std::collections::BTreeMap<String, serde_json::Value>,
}

fn migrate_desktop_snapshot(
    schema_version: u32,
    envelope: &AppStateEnvelope,
) -> HostResult<Option<DesktopSnapshot>> {
    match schema_version {
        0 => migrate_envelope_payload(envelope).map(Some),
        1 => {
            let legacy = migrate_envelope_payload::<LegacyDesktopSnapshotV1>(envelope)?;
            Ok(Some(DesktopSnapshot {
                schema_version: crate::model::DESKTOP_LAYOUT_SCHEMA_VERSION,
                preferences: legacy.preferences,
                windows: legacy.windows,
                terminal_history: legacy.terminal_history,
                app_shared_state: legacy.app_shared_state,
            }))
        }
        _ => Ok(None),
    }
}

/// Loads the compatibility boot snapshot and terminal history if present.
///
/// On non-WASM targets this returns `None`.
pub async fn load_boot_snapshot(_host: &DesktopHostContext) -> Option<DesktopSnapshot> {
    #[cfg(target_arch = "wasm32")]
    {
        let host = _host;
        let storage = local_storage()?;
        let snapshot = storage
            .get_item(SNAPSHOT_KEY)
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str::<DesktopSnapshot>(&raw).ok());
        let terminal_history =
            match load_pref_with(host.prefs_store().as_ref(), TERMINAL_HISTORY_KEY).await {
                Ok(history) => history,
                Err(err) => {
                    logging::warn!("terminal history compatibility load failed: {err}");
                    None
                }
            };

        match (snapshot, terminal_history) {
            (None, None) => None,
            (Some(mut snapshot), history) => {
                if let Some(history) = history {
                    snapshot.terminal_history = history;
                }
                Some(snapshot)
            }
            (None, Some(history)) => Some(DesktopSnapshot {
                schema_version: crate::model::DESKTOP_LAYOUT_SCHEMA_VERSION,
                preferences: Default::default(),
                windows: Vec::new(),
                terminal_history: history,
                app_shared_state: Default::default(),
            }),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Loads the durable boot snapshot from the configured [`platform_host::AppStateStore`]
/// implementation (IndexedDB-backed in the browser host).
pub async fn load_durable_boot_snapshot(host: &DesktopHostContext) -> Option<DesktopSnapshot> {
    load_durable_boot_snapshot_record(host)
        .await
        .map(|record| record.snapshot)
}

/// Loads the durable boot snapshot together with its authoritative app-state revision.
pub async fn load_durable_boot_snapshot_record(
    host: &DesktopHostContext,
) -> Option<DurableDesktopSnapshot> {
    let store = host.app_state_store();
    let envelope = match store.load_app_state_envelope(DESKTOP_STATE_NAMESPACE).await {
        Ok(envelope) => envelope,
        Err(err) => {
            logging::warn!("durable boot snapshot load failed: {err}");
            return None;
        }
    }?;

    match decode_desktop_snapshot_envelope(&envelope) {
        Ok(Some(snapshot)) => Some(DurableDesktopSnapshot {
            snapshot,
            revision: envelope.updated_at_unix_ms,
        }),
        Ok(None) => None,
        Err(err) => {
            logging::warn!("durable boot snapshot decode failed: {err}");
            None
        }
    }
}

/// Resolves restore preferences from the most authoritative available snapshot.
pub fn resolve_restore_preferences(
    durable_snapshot: Option<&DesktopSnapshot>,
    legacy_snapshot: Option<&DesktopSnapshot>,
) -> DesktopPreferences {
    durable_snapshot
        .or(legacy_snapshot)
        .map(|snapshot| snapshot.preferences.clone())
        .unwrap_or_default()
}

/// Persists a durable desktop layout snapshot through the configured
/// [`platform_host::AppStateStore`] implementation.
pub async fn persist_durable_layout_snapshot(
    host: &DesktopHostContext,
    state: &DesktopState,
) -> HostResult<()> {
    let envelope = build_durable_layout_envelope(state)?;
    save_durable_layout_envelope(host, &envelope).await
}

/// Builds a durable desktop layout envelope and stamps it with a monotonic revision.
pub fn build_durable_layout_envelope(state: &DesktopState) -> HostResult<AppStateEnvelope> {
    build_app_state_envelope(
        DESKTOP_STATE_NAMESPACE,
        crate::model::DESKTOP_LAYOUT_SCHEMA_VERSION,
        &state.snapshot(),
    )
}

/// Persists a durable desktop layout envelope through the configured app-state store.
pub async fn save_durable_layout_envelope(
    host: &DesktopHostContext,
    envelope: &AppStateEnvelope,
) -> HostResult<()> {
    host.app_state_store()
        .save_app_state_envelope(envelope)
        .await
}

/// Persists compatibility layout state.
///
/// The current implementation keeps full layout persistence in the configured app-state store and
/// reserves localStorage for lightweight compatibility/prefs state.
pub fn persist_layout_snapshot(state: &DesktopState) -> HostResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        // Full desktop layout is durably persisted in IndexedDB via the configured app-state
        // store.
        // Keep localStorage reserved for lightweight compatibility/prefs paths.
        let _ = state;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = state;
    }

    Ok(())
}

/// Persists the desktop theme through typed host prefs storage.
pub async fn persist_theme(host: &DesktopHostContext, theme: &DesktopTheme) -> HostResult<()> {
    save_pref_with(host.prefs_store().as_ref(), THEME_KEY, theme).await
}

/// Loads the current desktop theme from typed prefs, falling back to the legacy combined payload.
pub async fn load_theme(host: &DesktopHostContext) -> Option<DesktopTheme> {
    match load_pref_with(host.prefs_store().as_ref(), THEME_KEY).await {
        Ok(Some(theme)) => Some(theme),
        Ok(None) => load_legacy_theme(host).await.map(|legacy| DesktopTheme {
            high_contrast: legacy.high_contrast,
            reduced_motion: legacy.reduced_motion,
        }),
        Err(err) => {
            logging::warn!("desktop theme load failed: {err}");
            None
        }
    }
}

/// Persists the current wallpaper configuration through typed host prefs storage.
pub async fn persist_wallpaper(
    host: &DesktopHostContext,
    wallpaper: &WallpaperConfig,
) -> HostResult<()> {
    save_pref_with(host.prefs_store().as_ref(), WALLPAPER_KEY, wallpaper).await
}

/// Loads the current wallpaper configuration from typed prefs, falling back to the legacy theme payload.
pub async fn load_wallpaper(host: &DesktopHostContext) -> Option<WallpaperConfig> {
    match load_pref_with(host.prefs_store().as_ref(), WALLPAPER_KEY).await {
        Ok(Some(wallpaper)) => Some(normalize_wallpaper(wallpaper)),
        Ok(None) => load_legacy_theme(host).await.map(|legacy| WallpaperConfig {
            selection: WallpaperSelection::BuiltIn {
                wallpaper_id: normalize_legacy_wallpaper_id(&legacy.wallpaper_id),
            },
            ..WallpaperConfig::default()
        }),
        Err(err) => {
            logging::warn!("desktop wallpaper load failed: {err}");
            None
        }
    }
}

fn normalize_legacy_wallpaper_id(raw: &str) -> String {
    match raw.trim() {
        "slate-grid" => "teal-grid".to_string(),
        "" => "cloud-bands".to_string(),
        other => other.to_string(),
    }
}

fn normalize_wallpaper(mut wallpaper: WallpaperConfig) -> WallpaperConfig {
    if let WallpaperSelection::BuiltIn { wallpaper_id } = &mut wallpaper.selection {
        *wallpaper_id = normalize_legacy_wallpaper_id(wallpaper_id);
    }
    wallpaper
}

async fn load_legacy_theme(host: &DesktopHostContext) -> Option<LegacyThemePayload> {
    match load_pref_with(host.prefs_store().as_ref(), LEGACY_THEME_KEY).await {
        Ok(value) => value,
        Err(err) => {
            logging::warn!("legacy theme compatibility load failed: {err}");
            None
        }
    }
}

/// Persists the terminal history list through typed host prefs storage.
pub async fn persist_terminal_history(
    host: &DesktopHostContext,
    history: &[String],
) -> HostResult<()> {
    save_pref_with(host.prefs_store().as_ref(), TERMINAL_HISTORY_KEY, &history).await
}

/// Loads app capability policy overlay from typed host prefs storage.
pub async fn load_app_policy_overlay(host: &DesktopHostContext) -> Option<AppPolicyOverlay> {
    match load_pref_with(host.prefs_store().as_ref(), APP_POLICY_KEY).await {
        Ok(value) => value,
        Err(err) => {
            logging::warn!("app policy overlay load failed: {err}");
            None
        }
    }
}

/// Persists app capability policy overlay through typed host prefs storage.
pub async fn persist_app_policy_overlay(
    host: &DesktopHostContext,
    policy: &AppPolicyOverlay,
) -> HostResult<()> {
    save_pref_with(host.prefs_store().as_ref(), APP_POLICY_KEY, policy).await
}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn decode_desktop_snapshot_envelope(
    envelope: &AppStateEnvelope,
) -> HostResult<Option<DesktopSnapshot>> {
    if envelope.schema_version == crate::model::DESKTOP_LAYOUT_SCHEMA_VERSION {
        migrate_envelope_payload(envelope).map(Some)
    } else if envelope.schema_version > crate::model::DESKTOP_LAYOUT_SCHEMA_VERSION {
        Ok(None)
    } else {
        migrate_desktop_snapshot(envelope.schema_version, envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use platform_host::{AppStateStore, MemoryAppStateStore};

    #[test]
    fn desktop_namespace_migration_supports_schema_zero() {
        let snapshot = DesktopState::default().snapshot();
        let envelope = build_app_state_envelope(DESKTOP_STATE_NAMESPACE, 0, &snapshot)
            .expect("build envelope");

        let migrated =
            migrate_desktop_snapshot(0, &envelope).expect("schema-zero migration should succeed");
        assert!(migrated.is_some(), "expected migrated desktop snapshot");
    }

    #[test]
    fn restore_preferences_prefer_durable_snapshot_then_legacy() {
        let mut legacy = DesktopState::default().snapshot();
        legacy.preferences.restore_on_boot = false;

        let mut durable = DesktopState::default().snapshot();
        durable.preferences.restore_on_boot = true;
        durable.preferences.max_restore_windows = 2;

        let resolved = resolve_restore_preferences(Some(&durable), Some(&legacy));
        assert!(resolved.restore_on_boot);
        assert_eq!(resolved.max_restore_windows, 2);

        let legacy_only = resolve_restore_preferences(None, Some(&legacy));
        assert!(!legacy_only.restore_on_boot);
    }

    #[test]
    fn durable_snapshot_record_preserves_revision() {
        let store = MemoryAppStateStore::default();
        let state = DesktopState::default();
        let envelope =
            build_durable_layout_envelope(&state).expect("durable desktop envelope should build");
        let revision = envelope.updated_at_unix_ms;
        block_on(store.save_app_state_envelope(&envelope)).expect("save envelope");
        let host = crate::host::DesktopHostContext::new(platform_host::HostServices {
            app_state: std::rc::Rc::new(store),
            prefs: std::rc::Rc::new(platform_host::NoopPrefsStore),
            explorer: std::rc::Rc::new(platform_host::NoopExplorerFsService),
            cache: std::rc::Rc::new(platform_host::NoopContentCache),
            external_urls: std::rc::Rc::new(platform_host::NoopExternalUrlService),
            notifications: std::rc::Rc::new(platform_host::NoopNotificationService),
            wallpaper: std::rc::Rc::new(platform_host::NoopWallpaperAssetService),
            terminal_process: None,
            capabilities: platform_host::HostCapabilities::browser(),
            host_strategy: platform_host::HostStrategy::Browser,
        });

        let loaded = block_on(load_durable_boot_snapshot_record(&host)).expect("durable snapshot");
        assert_eq!(loaded.revision, revision);
        assert_eq!(loaded.snapshot, state.snapshot());
    }
}
