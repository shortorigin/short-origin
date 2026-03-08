use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TokenFile {
    pub color: BTreeMap<String, String>,
    pub material: BTreeMap<String, String>,
    pub surface: SurfaceTokens,
    pub blur: BTreeMap<String, String>,
    pub elevation: ElevationTokens,
    pub spacing: BTreeMap<String, String>,
    pub typography: TypographyTokens,
    pub radius: BTreeMap<String, String>,
    pub shadow: BTreeMap<String, String>,
    pub border: BorderTokens,
    pub opacity: BTreeMap<String, String>,
    pub z_index: BTreeMap<String, String>,
    pub motion: MotionTokens,
    pub state: StateTokens,
    pub icon: BTreeMap<String, String>,
    pub shell: ShellTokens,
}

#[derive(Debug, Deserialize)]
pub struct SurfaceTokens {
    pub background: BTreeMap<String, String>,
    pub border: BTreeMap<String, String>,
    pub highlight: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ElevationTokens {
    pub alpha: BTreeMap<String, String>,
    pub border: BTreeMap<String, String>,
    pub shadow: BTreeMap<String, String>,
    pub blur: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct TypographyTokens {
    pub family: BTreeMap<String, String>,
    pub size: BTreeMap<String, String>,
    pub weight: BTreeMap<String, String>,
    pub line_height: BTreeMap<String, String>,
    pub letter_spacing: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct BorderTokens {
    pub width: BTreeMap<String, String>,
    pub opacity: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct MotionTokens {
    pub duration: BTreeMap<String, String>,
    pub easing: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct StateTokens {
    pub hover: BTreeMap<String, String>,
    pub focus: BTreeMap<String, String>,
    pub active: BTreeMap<String, String>,
    pub disabled: BTreeMap<String, String>,
    pub selected: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ShellTokens {
    pub taskbar: BTreeMap<String, String>,
    pub titlebar: BTreeMap<String, String>,
    pub window_chrome: BTreeMap<String, String>,
    pub resize_handle: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::TokenFile;

    #[test]
    fn material_token_schema_parses_current_token_file() {
        let raw = include_str!("../../tokens/tokens.toml");
        let tokens: TokenFile = toml::from_str(raw).expect("token file should parse");

        assert!(tokens.material.contains_key("tint-base"));
        assert!(tokens.surface.background.contains_key("modal"));
        assert!(tokens.elevation.blur.contains_key("floating"));
        assert!(tokens.state.selected.contains_key("surface"));
    }

    #[test]
    fn generated_tailwind_config_exposes_material_utilities() {
        let raw = include_str!("../../../site/tailwind.config.js");

        assert!(raw.contains("base-glass"));
        assert!(raw.contains("raised-glass"));
        assert!(raw.contains("overlay-glass"));
        assert!(raw.contains("modal-glass"));
        assert!(raw.contains("control-glass"));
        assert!(raw.contains("backdropBlur"));
    }
}
