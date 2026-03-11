use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TokenFile {
    pub raw: RawTokens,
    pub semantic: SemanticTokens,
    pub theme: ThemeTokens,
}

#[derive(Debug, Deserialize)]
pub struct RawTokens {
    pub color: BTreeMap<String, String>,
    pub space: BTreeMap<String, String>,
    #[serde(rename = "type")]
    pub type_tokens: BTreeMap<String, String>,
    pub blur: BTreeMap<String, String>,
    pub radius: BTreeMap<String, String>,
    pub motion: BTreeMap<String, String>,
    pub border: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct SemanticTokens {
    pub surface: BTreeMap<String, String>,
    pub control: BTreeMap<String, String>,
    pub text: BTreeMap<String, String>,
    pub border: BTreeMap<String, String>,
    pub state: BTreeMap<String, String>,
    pub shell: BTreeMap<String, String>,
    pub layer: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeTokens {
    pub default: String,
    pub dark: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::TokenFile;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn semantic_token_schema_parses_current_token_file() {
        let raw = include_str!("../../tokens/tokens.toml");
        let tokens: TokenFile = toml::from_str(raw).expect("token file should parse");

        assert!(tokens.raw.color.contains_key("canvas"));
        assert!(tokens.raw.space.contains_key("16"));
        assert!(tokens.raw.type_tokens.contains_key("size-body"));
        assert!(tokens.semantic.surface.contains_key("taskbar-background"));
        assert!(tokens.semantic.control.contains_key("accent-background"));
        assert!(tokens.semantic.shell.contains_key("taskbar-height"));
        assert!(tokens.semantic.layer.contains_key("modal"));
        assert_eq!(tokens.theme.default, "light");
        assert!(tokens.theme.dark.contains_key("raw-color-canvas"));
    }

    #[test]
    fn generated_tailwind_config_exposes_semantic_shell_tokens() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../site/tailwind.config.js");
        let raw = fs::read_to_string(&path).expect("generated tailwind config should exist");

        assert!(raw.contains("semantic"));
        assert!(raw.contains("taskbar"));
        assert!(raw.contains("windowActive"));
        assert!(raw.contains("focusRing"));
    }
}
