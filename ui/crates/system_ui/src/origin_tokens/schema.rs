use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TokenFile {
    pub color: BTreeMap<String, String>,
    pub spacing: BTreeMap<String, String>,
    pub typography: TypographyTokens,
    pub radius: BTreeMap<String, String>,
    pub shadow: BTreeMap<String, String>,
    pub border: BorderTokens,
    pub opacity: BTreeMap<String, String>,
    pub z_index: BTreeMap<String, String>,
    pub motion: MotionTokens,
    pub icon: BTreeMap<String, String>,
    pub shell: ShellTokens,
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
}

#[derive(Debug, Deserialize)]
pub struct MotionTokens {
    pub duration: BTreeMap<String, String>,
    pub easing: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ShellTokens {
    pub taskbar: BTreeMap<String, String>,
    pub titlebar: BTreeMap<String, String>,
    pub window_chrome: BTreeMap<String, String>,
    pub resize_handle: BTreeMap<String, String>,
}
