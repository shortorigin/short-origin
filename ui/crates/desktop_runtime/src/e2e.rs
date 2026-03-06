//! Browser-only E2E scene configuration shared by the site entrypoint, runtime boot flow, and
//! shell surfaces.

use serde::{Deserialize, Serialize};

use crate::model::DesktopSkin;

/// Canonical browser E2E scenes supported by the deterministic UI validation workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserE2eScene {
    /// Idle shell surface with no transient overlays.
    ShellDefault,
    /// Desktop root context menu opened and stable.
    ShellContextMenuOpen,
    /// Settings window opened to the Appearance section.
    SettingsAppearance,
    /// Settings window opened to the Accessibility section.
    SettingsAccessibility,
    /// Start button rendered in hover state.
    StartButtonHover,
    /// Start button rendered in focus-visible state.
    StartButtonFocus,
    /// Shell with high contrast enabled.
    ShellHighContrast,
    /// Shell with reduced motion enabled.
    ShellReducedMotion,
    /// Shared primitive showcase app opened to its control coverage surface.
    UiShowcaseControls,
    /// Terminal app opened in its default readable state.
    TerminalDefault,
}

impl BrowserE2eScene {
    /// Stable query-string scene id.
    pub const fn id(self) -> &'static str {
        match self {
            Self::ShellDefault => "shell-default",
            Self::ShellContextMenuOpen => "shell-context-menu-open",
            Self::SettingsAppearance => "settings-appearance",
            Self::SettingsAccessibility => "settings-accessibility",
            Self::StartButtonHover => "start-button-hover",
            Self::StartButtonFocus => "start-button-focus",
            Self::ShellHighContrast => "shell-high-contrast",
            Self::ShellReducedMotion => "shell-reduced-motion",
            Self::UiShowcaseControls => "ui-showcase-controls",
            Self::TerminalDefault => "terminal-default",
        }
    }

    #[cfg(any(test, target_arch = "wasm32"))]
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "shell-default" => Some(Self::ShellDefault),
            "shell-context-menu-open" => Some(Self::ShellContextMenuOpen),
            "settings-appearance" => Some(Self::SettingsAppearance),
            "settings-accessibility" => Some(Self::SettingsAccessibility),
            "start-button-hover" => Some(Self::StartButtonHover),
            "start-button-focus" => Some(Self::StartButtonFocus),
            "shell-high-contrast" => Some(Self::ShellHighContrast),
            "shell-reduced-motion" => Some(Self::ShellReducedMotion),
            "ui-showcase-controls" => Some(Self::UiShowcaseControls),
            "terminal-default" => Some(Self::TerminalDefault),
            _ => None,
        }
    }
}

/// Parsed browser E2E query-string configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserE2eConfig {
    /// Requested canonical scene.
    pub scene: BrowserE2eScene,
    /// Optional skin override.
    pub skin: Option<DesktopSkin>,
    /// Optional high-contrast override.
    pub high_contrast: Option<bool>,
    /// Optional reduced-motion override.
    pub reduced_motion: Option<bool>,
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn parse_skin(raw: &str) -> Option<DesktopSkin> {
    match raw.trim() {
        "soft-neumorphic" => Some(DesktopSkin::SoftNeumorphic),
        "modern-adaptive" => Some(DesktopSkin::ModernAdaptive),
        "classic-xp" => Some(DesktopSkin::ClassicXp),
        "classic-95" => Some(DesktopSkin::Classic95),
        _ => None,
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
/// Parses browser E2E configuration from a query string.
pub fn parse_browser_e2e_from_query(query: &str) -> Option<BrowserE2eConfig> {
    let mut scene = None;
    let mut skin = None;
    let mut high_contrast = None;
    let mut reduced_motion = None;

    for pair in query
        .trim_start_matches('?')
        .split('&')
        .filter(|part| !part.is_empty())
    {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        match key {
            "e2e-scene" => {
                scene = BrowserE2eScene::parse(value);
            }
            "e2e-skin" => {
                skin = parse_skin(value);
            }
            "e2e-high-contrast" => {
                high_contrast = parse_bool(value);
            }
            "e2e-reduced-motion" => {
                reduced_motion = parse_bool(value);
            }
            _ => {}
        }
    }

    scene.map(|scene| BrowserE2eConfig {
        scene,
        skin,
        high_contrast,
        reduced_motion,
    })
}

/// Returns the active browser E2E configuration when the current URL requests one.
pub fn current_browser_e2e_config() -> Option<BrowserE2eConfig> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let location = window.location();
        let search = location.search().ok()?;
        parse_browser_e2e_from_query(&search)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_browser_e2e_scene_and_theme_overrides() {
        let parsed = parse_browser_e2e_from_query(
            "?e2e-scene=settings-appearance&e2e-skin=soft-neumorphic&e2e-high-contrast=false&e2e-reduced-motion=true",
        )
        .expect("config");
        assert_eq!(parsed.scene, BrowserE2eScene::SettingsAppearance);
        assert_eq!(parsed.skin, Some(DesktopSkin::SoftNeumorphic));
        assert_eq!(parsed.high_contrast, Some(false));
        assert_eq!(parsed.reduced_motion, Some(true));
    }

    #[test]
    fn ignores_invalid_boolean_overrides() {
        let parsed =
            parse_browser_e2e_from_query("?e2e-scene=shell-default&e2e-high-contrast=maybe")
                .expect("config");
        assert_eq!(parsed.scene, BrowserE2eScene::ShellDefault);
        assert_eq!(parsed.high_contrast, None);
    }
}
