use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TokenFile {
    color: BTreeMap<String, String>,
    material: BTreeMap<String, String>,
    surface: SurfaceTokens,
    blur: BTreeMap<String, String>,
    elevation: ElevationTokens,
    spacing: BTreeMap<String, String>,
    typography: TypographyTokens,
    radius: BTreeMap<String, String>,
    shadow: BTreeMap<String, String>,
    border: BorderTokens,
    opacity: BTreeMap<String, String>,
    z_index: BTreeMap<String, String>,
    motion: MotionTokens,
    state: StateTokens,
    icon: BTreeMap<String, String>,
    shell: ShellTokens,
}

#[derive(Debug, Deserialize)]
struct SurfaceTokens {
    background: BTreeMap<String, String>,
    border: BTreeMap<String, String>,
    highlight: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ElevationTokens {
    alpha: BTreeMap<String, String>,
    border: BTreeMap<String, String>,
    shadow: BTreeMap<String, String>,
    blur: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct TypographyTokens {
    family: BTreeMap<String, String>,
    size: BTreeMap<String, String>,
    weight: BTreeMap<String, String>,
    line_height: BTreeMap<String, String>,
    letter_spacing: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct BorderTokens {
    width: BTreeMap<String, String>,
    opacity: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct MotionTokens {
    duration: BTreeMap<String, String>,
    easing: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct StateTokens {
    hover: BTreeMap<String, String>,
    focus: BTreeMap<String, String>,
    active: BTreeMap<String, String>,
    disabled: BTreeMap<String, String>,
    selected: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ShellTokens {
    taskbar: BTreeMap<String, String>,
    titlebar: BTreeMap<String, String>,
    window_chrome: BTreeMap<String, String>,
    resize_handle: BTreeMap<String, String>,
}

fn sanitize_ident(raw: &str) -> String {
    let mut ident = raw.replace('-', "_").to_ascii_uppercase();
    if ident
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        ident = format!("N_{ident}");
    }
    ident
}

fn push_const_block(buffer: &mut String, prefix: &str, values: &BTreeMap<String, String>) {
    for (key, value) in values {
        let ident = sanitize_ident(key);
        buffer.push_str(&format!("pub const {prefix}_{ident}: &str = {value:?};\n"));
    }
}

fn push_css_vars(buffer: &mut String, section: &str, values: &BTreeMap<String, String>) {
    for (key, value) in values {
        buffer.push_str(&format!("  --origin-{section}-{key}: {value};\n"));
    }
}

fn ensure_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
}

fn write_if_changed(path: &Path, contents: &str) {
    ensure_parent(path);
    let current = fs::read_to_string(path).ok();
    if current.as_deref() != Some(contents) {
        fs::write(path, contents).expect("write generated file");
    }
}

fn tailwind_config() -> &'static str {
    r#"// Generated from ui/crates/system_ui/tokens/tokens.toml
const plugin = require("tailwindcss/plugin");

module.exports = {
  content: ["./src/**/*.rs", "./src/**/*.html"],
  theme: {
    extend: {
      colors: {
        canvas: "var(--origin-color-canvas)",
        desktop: "var(--origin-color-desktop)",
        accent: "var(--origin-color-accent)",
        focus: "var(--origin-color-focus)",
        text: {
          primary: "var(--origin-color-text-primary)",
          secondary: "var(--origin-color-text-secondary)",
          muted: "var(--origin-color-text-muted)",
          inverse: "var(--origin-color-text-inverse)",
        },
        status: {
          success: "var(--origin-color-success)",
          warning: "var(--origin-color-warning)",
          danger: "var(--origin-color-danger)",
        },
        surface: {
          base: "var(--origin-surface-background-base)",
          raised: "var(--origin-surface-background-raised)",
          overlay: "var(--origin-surface-background-overlay)",
          modal: "var(--origin-surface-background-modal)",
          control: "var(--origin-surface-background-control)",
        },
      },
      spacing: {
        "1q": "var(--origin-space-1q)",
        1: "var(--origin-space-1)",
        2: "var(--origin-space-2)",
        3: "var(--origin-space-3)",
        4: "var(--origin-space-4)",
        5: "var(--origin-space-5)",
        6: "var(--origin-space-6)",
        7: "var(--origin-space-7)",
        8: "var(--origin-space-8)",
        9: "var(--origin-space-9)",
        panel: "var(--origin-space-panel)",
        section: "var(--origin-space-section)",
        content: "var(--origin-space-content)",
      },
      borderRadius: {
        sm: "var(--origin-radius-sm)",
        md: "var(--origin-radius-md)",
        lg: "var(--origin-radius-lg)",
        xl: "var(--origin-radius-xl)",
        round: "var(--origin-radius-round)",
      },
      borderWidth: {
        hairline: "var(--origin-border-width-hairline)",
        DEFAULT: "var(--origin-border-width-standard)",
        strong: "var(--origin-border-width-strong)",
      },
      backdropBlur: {
        low: "var(--origin-blur-low)",
        medium: "var(--origin-blur-medium)",
        high: "var(--origin-blur-high)",
        modal: "var(--origin-blur-modal)",
      },
      boxShadow: {
        panel: "var(--origin-shadow-panel)",
        window: "var(--origin-shadow-window)",
        overlay: "var(--origin-shadow-overlay)",
        focus: "var(--origin-shadow-focus-ring)",
        glass: "var(--origin-shadow-glass)",
      },
      zIndex: {
        wallpaper: "var(--origin-z-wallpaper)",
        desktop: "var(--origin-z-desktop)",
        windows: "var(--origin-z-windows)",
        overlay: "var(--origin-z-overlay)",
        menu: "var(--origin-z-menu)",
        modal: "var(--origin-z-modal)",
        taskbar: "var(--origin-z-taskbar)",
      },
      opacity: {
        disabled: "var(--origin-opacity-disabled)",
        veil: "var(--origin-opacity-overlay)",
      },
      fontFamily: {
        sans: ["var(--origin-type-family-sans)"],
        mono: ["var(--origin-type-family-mono)"],
      },
      transitionDuration: {
        fast: "var(--origin-motion-duration-fast)",
        DEFAULT: "var(--origin-motion-duration-standard)",
        slow: "var(--origin-motion-duration-slow)",
      },
      transitionTimingFunction: {
        standard: "var(--origin-motion-easing-standard)",
        emphasized: "var(--origin-motion-easing-emphasized)",
      },
    },
  },
  plugins: [
    plugin(function ({ addComponents, addUtilities }) {
      addComponents({
        ".base-glass": {
          background: "var(--origin-surface-background-base)",
          borderColor: "var(--origin-surface-border-base)",
          boxShadow: "var(--origin-shadow-panel)",
          backdropFilter: "blur(var(--origin-elevation-blur-embedded))",
        },
        ".raised-glass": {
          background: "var(--origin-surface-background-raised)",
          borderColor: "var(--origin-surface-border-raised)",
          boxShadow: "var(--origin-shadow-glass)",
          backdropFilter: "blur(var(--origin-elevation-blur-raised))",
        },
        ".overlay-glass": {
          background: "var(--origin-surface-background-overlay)",
          borderColor: "var(--origin-surface-border-overlay)",
          boxShadow: "var(--origin-shadow-overlay)",
          backdropFilter: "blur(var(--origin-elevation-blur-floating))",
        },
        ".modal-glass": {
          background: "var(--origin-surface-background-modal)",
          borderColor: "var(--origin-surface-border-modal)",
          boxShadow: "var(--origin-shadow-overlay)",
          backdropFilter: "blur(var(--origin-elevation-blur-modal))",
        },
        ".control-glass": {
          background: "var(--origin-surface-background-control)",
          borderColor: "var(--origin-surface-border-control)",
          boxShadow: "var(--origin-shadow-inset)",
          backdropFilter: "blur(var(--origin-elevation-blur-embedded))",
        },
      });
      addUtilities({
        ".text-glow": {
          textShadow: "0 1px 18px rgba(151, 211, 255, 0.16)",
        },
        ".glass-highlight": {
          position: "relative",
          overflow: "hidden",
        },
      });
    }),
  ],
  corePlugins: { preflight: false },
};
"#
}

fn generated_tailwind_css() -> &'static str {
    r#"/* Generated token-driven Tailwind layer */
html,
body,
.site-root,
.site-root * {
  box-sizing: border-box;
}

:root {
  color-scheme: dark;
}

body {
  margin: 0;
  min-height: 100vh;
  background:
    radial-gradient(circle at 14% 12%, color-mix(in srgb, var(--origin-color-accent) 18%, transparent), transparent 26%),
    radial-gradient(circle at 86% 10%, color-mix(in srgb, var(--origin-color-success) 12%, transparent), transparent 28%),
    linear-gradient(180deg, var(--origin-color-canvas), color-mix(in srgb, var(--origin-color-desktop) 88%, #02060d));
  color: var(--origin-color-text-primary);
  font-family: var(--origin-type-family-sans);
  font-size: var(--origin-type-size-body);
  line-height: var(--origin-type-line-height-body);
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
}

button,
input,
textarea,
select {
  font: inherit;
}

a {
  color: inherit;
}

[data-ui-primitive="true"] {
  transition:
    background-color var(--origin-motion-duration-standard) var(--origin-motion-easing-standard),
    border-color var(--origin-motion-duration-standard) var(--origin-motion-easing-standard),
    box-shadow var(--origin-motion-duration-standard) var(--origin-motion-easing-standard),
    color var(--origin-motion-duration-standard) var(--origin-motion-easing-standard),
    opacity var(--origin-motion-duration-standard) var(--origin-motion-easing-standard),
    transform var(--origin-motion-duration-fast) var(--origin-motion-easing-standard),
    filter var(--origin-motion-duration-fast) var(--origin-motion-easing-standard);
}

[data-ui-primitive="true"]:focus-visible {
  outline: none;
  border-color: var(--origin-color-border-focus);
  box-shadow: var(--origin-shadow-focus-ring);
}

[data-ui-kind="stack"] {
  display: grid;
}

[data-ui-kind="inline"],
[data-ui-kind="cluster"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"],
[data-ui-kind="statusbar-item"],
[data-ui-kind="window-controls"],
[data-ui-kind="taskbar-section"],
[data-ui-kind="tray-list"] {
  display: flex;
  align-items: center;
}

[data-ui-kind="cluster"] {
  flex-wrap: wrap;
}

[data-ui-kind="grid"] {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

[data-ui-kind="center"] {
  display: grid;
  place-items: center;
}

[data-ui-gap="none"] { gap: 0; }
[data-ui-gap="sm"] { gap: var(--origin-space-2); }
[data-ui-gap="md"] { gap: var(--origin-space-4); }
[data-ui-gap="lg"] { gap: var(--origin-space-6); }
[data-ui-padding="none"] { padding: 0; }
[data-ui-padding="sm"] { padding: var(--origin-space-2); }
[data-ui-padding="md"] { padding: var(--origin-space-4); }
[data-ui-padding="lg"] { padding: var(--origin-space-6); }
[data-ui-align="start"] { align-items: flex-start; }
[data-ui-align="center"] { align-items: center; }
[data-ui-align="end"] { align-items: flex-end; }
[data-ui-align="stretch"] { align-items: stretch; }
[data-ui-justify="start"] { justify-content: flex-start; }
[data-ui-justify="center"] { justify-content: center; }
[data-ui-justify="between"] { justify-content: space-between; }
[data-ui-justify="end"] { justify-content: flex-end; }

[data-ui-kind="surface"],
[data-ui-kind="panel"],
[data-ui-kind="list-surface"],
[data-ui-kind="layer"],
[data-ui-kind="menu-surface"],
[data-ui-kind="terminal-surface"],
[data-ui-kind="completion-list"],
[data-ui-kind="window-surface"],
[data-ui-kind="window-frame"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"],
[data-ui-kind="toggle-row"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="disclosure"] {
  position: relative;
  color: var(--origin-color-text-primary);
  border: var(--origin-border-width-standard) solid var(--origin-surface-border-base);
  background: var(--origin-surface-background-base);
  box-shadow: var(--origin-shadow-panel);
  backdrop-filter: blur(var(--origin-elevation-blur-embedded)) saturate(150%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-embedded)) saturate(150%);
}

[data-ui-kind="surface"]::before,
[data-ui-kind="panel"]::before,
[data-ui-kind="list-surface"]::before,
[data-ui-kind="menu-surface"]::before,
[data-ui-kind="window-surface"]::before,
[data-ui-kind="window-frame"]::before,
[data-ui-kind="toolbar"]::before,
[data-ui-kind="statusbar"]::before,
[data-ui-kind="toggle-row"]::before,
[data-ui-kind="step-flow-step"]::before,
[data-ui-kind="disclosure"]::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  border-radius: inherit;
  background:
    linear-gradient(180deg, var(--origin-surface-highlight-base), transparent 38%),
    linear-gradient(120deg, color-mix(in srgb, var(--origin-color-text-primary) 6%, transparent), transparent 30%);
  opacity: 0.92;
}

[data-ui-kind="surface"],
[data-ui-kind="panel"],
[data-ui-kind="list-surface"],
[data-ui-kind="terminal-surface"],
[data-ui-kind="completion-list"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"],
[data-ui-kind="toggle-row"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="disclosure"] {
  border-radius: var(--origin-radius-lg);
}

[data-ui-kind="layer"],
[data-ui-kind="menu-surface"],
[data-ui-kind="window-surface"],
[data-ui-kind="window-frame"] {
  border-radius: var(--origin-radius-xl);
}

[data-ui-kind="surface"][data-ui-variant="muted"],
[data-ui-kind="panel"][data-ui-variant="muted"] {
  background: var(--origin-surface-background-raised);
  border-color: var(--origin-surface-border-raised);
  box-shadow: var(--origin-shadow-glass);
  backdrop-filter: blur(var(--origin-elevation-blur-raised)) saturate(155%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-raised)) saturate(155%);
}

[data-ui-kind="surface"][data-ui-variant="inset"],
[data-ui-kind="panel"][data-ui-variant="inset"],
[data-ui-kind="text-field"][data-ui-variant="inset"] {
  background: var(--origin-surface-background-control);
  border-color: var(--origin-surface-border-control);
  box-shadow: var(--origin-shadow-inset);
}

[data-ui-kind="panel"] {
  overflow: hidden;
}

[data-ui-elevation="flat"] {
  box-shadow: none;
}

[data-ui-elevation="embedded"] {
  box-shadow: var(--origin-shadow-panel);
}

[data-ui-elevation="raised"] {
  box-shadow: var(--origin-shadow-glass);
  backdrop-filter: blur(var(--origin-elevation-blur-raised)) saturate(155%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-raised)) saturate(155%);
}

[data-ui-elevation="overlay"] {
  box-shadow: var(--origin-shadow-overlay);
  backdrop-filter: blur(var(--origin-elevation-blur-floating)) saturate(165%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-floating)) saturate(165%);
}

[data-ui-elevation="modal"],
[data-ui-kind="menu-surface"],
[data-ui-kind="window-surface"],
[data-ui-kind="window-frame"] {
  box-shadow: var(--origin-shadow-overlay);
  backdrop-filter: blur(var(--origin-elevation-blur-modal)) saturate(170%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-modal)) saturate(170%);
}

[data-ui-elevation="transient"] {
  box-shadow: var(--origin-shadow-overlay);
  backdrop-filter: blur(var(--origin-elevation-blur-transient)) saturate(165%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-transient)) saturate(165%);
}

[data-ui-elevation="inset"] {
  box-shadow: var(--origin-shadow-inset);
}

[data-ui-elevation="pressed"] {
  box-shadow: var(--origin-shadow-pressed);
}

[data-ui-kind="text"],
[data-ui-kind="heading"],
[data-ui-kind="window-title"] {
  min-width: 0;
  position: relative;
  z-index: 1;
}

[data-ui-kind="text"][data-ui-variant="body"] {
  font-size: var(--origin-type-size-body);
  line-height: var(--origin-type-line-height-body);
}

[data-ui-kind="text"][data-ui-variant="label"] {
  font-size: var(--origin-type-size-label);
  font-weight: var(--origin-type-weight-semibold);
  letter-spacing: var(--origin-type-letter-spacing-caps);
  text-transform: uppercase;
}

[data-ui-kind="text"][data-ui-variant="caption"] {
  color: var(--origin-color-text-muted);
  font-size: var(--origin-type-size-caption);
}

[data-ui-kind="text"][data-ui-variant="title"],
[data-ui-kind="heading"],
[data-ui-kind="heading"][data-ui-variant="title"] {
  font-size: var(--origin-type-size-title);
  line-height: var(--origin-type-line-height-heading);
  font-weight: var(--origin-type-weight-semibold);
  letter-spacing: var(--origin-type-letter-spacing-tight);
}

[data-ui-kind="text"][data-ui-variant="code"] {
  font-family: var(--origin-type-family-mono);
  font-size: var(--origin-type-size-body-sm);
}

[data-ui-tone="secondary"] { color: var(--origin-color-text-secondary); }
[data-ui-tone="accent"] { color: var(--origin-color-accent-strong); }
[data-ui-tone="success"] { color: var(--origin-color-success); }
[data-ui-tone="warning"] { color: var(--origin-color-warning); }
[data-ui-tone="danger"] { color: var(--origin-color-danger); }

[data-ui-kind="button"],
[data-ui-kind="icon-button"],
[data-ui-kind="taskbar-button"],
[data-ui-kind="window-control-button"],
[data-ui-kind="desktop-icon-button"] {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--origin-space-2);
  border: var(--origin-border-width-standard) solid var(--origin-surface-border-control);
  border-radius: var(--origin-radius-md);
  background: var(--origin-surface-background-control);
  color: var(--origin-color-text-primary);
  cursor: pointer;
  overflow: hidden;
}

[data-ui-kind="button"]::before,
[data-ui-kind="icon-button"]::before,
[data-ui-kind="desktop-icon-button"]::before {
  content: "";
  position: absolute;
  inset: 0;
  background:
    linear-gradient(180deg, var(--origin-surface-highlight-control), transparent 45%),
    linear-gradient(120deg, color-mix(in srgb, var(--origin-color-text-primary) 4%, transparent), transparent 35%);
  opacity: 0.95;
  pointer-events: none;
}

[data-ui-kind="button"] > *,
[data-ui-kind="icon-button"] > * {
  position: relative;
  z-index: 1;
}

[data-ui-kind="button"][data-ui-size="sm"] {
  min-height: 34px;
  padding: 0 var(--origin-space-3);
}

[data-ui-kind="button"][data-ui-size="md"] {
  min-height: 40px;
  padding: 0 var(--origin-space-4);
}

[data-ui-kind="button"][data-ui-size="lg"] {
  min-height: 46px;
  padding: 0 var(--origin-space-5);
}

[data-ui-kind="button"][data-ui-shape="pill"] {
  border-radius: var(--origin-radius-round);
}

[data-ui-kind="button"][data-ui-shape="circle"],
[data-ui-kind="icon-button"] {
  border-radius: var(--origin-radius-round);
  width: 40px;
  min-width: 40px;
  min-height: 40px;
  padding: 0;
}

[data-ui-kind="button"][data-ui-variant="primary"] {
  background: linear-gradient(180deg, color-mix(in srgb, var(--origin-color-accent-strong) 34%, transparent), color-mix(in srgb, var(--origin-surface-background-control) 88%, var(--origin-color-accent)));
  border-color: color-mix(in srgb, var(--origin-color-accent) 55%, var(--origin-surface-border-control));
}

[data-ui-kind="button"][data-ui-variant="accent"] {
  background: linear-gradient(180deg, color-mix(in srgb, var(--origin-color-accent) 26%, transparent), color-mix(in srgb, var(--origin-surface-background-raised) 86%, var(--origin-color-accent)));
  border-color: color-mix(in srgb, var(--origin-color-accent) 50%, var(--origin-surface-border-raised));
}

[data-ui-kind="button"][data-ui-variant="danger"] {
  background: linear-gradient(180deg, color-mix(in srgb, var(--origin-color-danger) 28%, transparent), color-mix(in srgb, var(--origin-surface-background-control) 92%, var(--origin-color-danger)));
  border-color: color-mix(in srgb, var(--origin-color-danger) 54%, var(--origin-surface-border-control));
}

[data-ui-kind="button"][data-ui-variant="quiet"],
[data-ui-kind="button"][data-ui-variant="icon"],
[data-ui-kind="button"][data-ui-variant="segmented"] {
  background: color-mix(in srgb, var(--origin-surface-background-control) 92%, transparent);
}

[data-ui-kind="button"]:hover,
[data-ui-kind="icon-button"]:hover,
[data-ui-kind="button"][data-ui-selected="true"],
[data-ui-kind="button"][data-ui-state="pressed"] {
  transform: translateY(-1px);
  border-color: var(--origin-color-border-selected);
  background: var(--origin-state-hover-surface);
}

[data-ui-kind="button"]:active,
[data-ui-kind="icon-button"]:active {
  transform: translateY(0);
  background: var(--origin-state-active-surface);
  box-shadow: var(--origin-shadow-pressed);
}

[data-ui-disabled="true"],
[disabled] {
  opacity: var(--origin-opacity-disabled);
  cursor: not-allowed;
}

[data-ui-kind="text-field"] {
  width: 100%;
  min-height: 42px;
  padding: 0 var(--origin-space-4);
  border: var(--origin-border-width-standard) solid var(--origin-surface-border-control);
  border-radius: var(--origin-radius-md);
  background: var(--origin-surface-background-control);
  color: var(--origin-color-text-primary);
  box-shadow: var(--origin-shadow-inset);
}

[data-ui-kind="text-field"]::placeholder {
  color: var(--origin-color-text-muted);
}

[data-ui-kind="checkbox"] {
  inline-size: 18px;
  block-size: 18px;
  border-radius: 6px;
  accent-color: var(--origin-color-accent);
}

[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"] {
  gap: var(--origin-space-2);
  padding: var(--origin-space-2) var(--origin-space-3);
  background: var(--origin-surface-background-raised);
  border-color: var(--origin-surface-border-raised);
}

[data-ui-kind="statusbar"] {
  justify-content: space-between;
  color: var(--origin-color-text-secondary);
}

[data-ui-kind="statusbar-item"] {
  gap: var(--origin-space-2);
}

[data-ui-kind="list-surface"] {
  display: grid;
  gap: var(--origin-space-3);
  padding: var(--origin-space-4);
}

[data-ui-kind="list-surface"] > div {
  position: relative;
  z-index: 1;
  display: grid;
  gap: var(--origin-space-1);
  padding: var(--origin-space-3);
  border-radius: var(--origin-radius-md);
  background: color-mix(in srgb, var(--origin-surface-background-control) 88%, transparent);
  border: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-control) 88%, transparent);
}

[data-ui-kind="data-table"] {
  width: 100%;
  border-collapse: collapse;
  position: relative;
  z-index: 1;
}

[data-ui-kind="data-table"] th,
[data-ui-kind="data-table"] td {
  padding: var(--origin-space-2) var(--origin-space-3);
  text-align: left;
  border-bottom: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-control) 85%, transparent);
}

[data-ui-kind="data-table"] th {
  color: var(--origin-color-text-secondary);
  font-size: var(--origin-type-size-caption);
  letter-spacing: var(--origin-type-letter-spacing-caps);
  text-transform: uppercase;
}

[data-ui-kind="terminal-surface"] {
  display: grid;
  gap: var(--origin-space-4);
  min-height: 420px;
  padding: var(--origin-space-4);
  background: color-mix(in srgb, var(--origin-surface-background-base) 88%, var(--origin-color-canvas));
}

[data-ui-kind="terminal-transcript"] {
  display: grid;
  gap: var(--origin-space-2);
  min-height: 0;
}

[data-ui-kind="terminal-line"],
[data-ui-kind="terminal-prompt"] {
  position: relative;
  z-index: 1;
  font-family: var(--origin-type-family-mono);
  font-size: var(--origin-type-size-body-sm);
}

[data-ui-kind="completion-list"] {
  display: grid;
  gap: var(--origin-space-1q);
  padding: var(--origin-space-2);
}

[data-ui-slot="completion-item"] {
  width: 100%;
  justify-content: flex-start;
}

[data-ui-kind="menu-surface"] {
  display: grid;
  gap: var(--origin-space-1q);
  min-width: 250px;
  padding: var(--origin-space-2);
  background: var(--origin-surface-background-overlay);
  border-color: var(--origin-surface-border-overlay);
  z-index: var(--origin-z-menu);
}

[data-ui-slot="menu-item"] {
  width: 100%;
  min-height: 38px;
  justify-content: flex-start;
  border-radius: var(--origin-radius-md);
  text-align: left;
}

[data-ui-slot="menu-item"]:hover,
[data-ui-slot="menu-item"]:focus-visible,
[data-ui-slot="menu-item"][data-ui-selected="true"] {
  background: var(--origin-state-selected-surface);
  border-color: var(--origin-color-border-selected);
}

[data-ui-kind="menu-separator"] {
  height: var(--origin-border-width-standard);
  margin: var(--origin-space-1) 0;
  background: color-mix(in srgb, var(--origin-surface-border-overlay) 75%, transparent);
}

[data-ui-kind="disclosure"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="toggle-row"] {
  padding: var(--origin-space-4);
}

[data-ui-kind="disclosure"] [data-ui-slot="body"],
[data-ui-kind="step-flow-step"] [data-ui-slot="body"] {
  position: relative;
  z-index: 1;
  margin-top: var(--origin-space-3);
}

[data-ui-kind="step-flow"] {
  display: grid;
  gap: var(--origin-space-3);
}

[data-ui-kind="step-flow-header"] {
  display: grid;
  gap: var(--origin-space-1);
  margin-bottom: var(--origin-space-3);
}

[data-ui-kind="step-flow-header"] [data-ui-slot="title"] {
  font-size: var(--origin-type-size-title);
  font-weight: var(--origin-type-weight-semibold);
}

[data-ui-kind="step-flow-header"] [data-ui-slot="description"],
[data-ui-kind="toggle-row"] [data-ui-slot="description"],
[data-ui-kind="disclosure"] [data-ui-slot="description"] {
  color: var(--origin-color-text-secondary);
}

[data-ui-kind="step-flow-step"] [data-ui-slot="header"],
[data-ui-kind="toggle-row"] {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: var(--origin-space-3);
}

[data-ui-kind="step-flow-step"] [data-ui-slot="badge"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 74px;
  padding: var(--origin-space-1) var(--origin-space-2);
  border-radius: var(--origin-radius-round);
  background: color-mix(in srgb, var(--origin-color-accent) 16%, transparent);
  color: var(--origin-color-accent-strong);
  font-size: var(--origin-type-size-caption);
  letter-spacing: var(--origin-type-letter-spacing-caps);
  text-transform: uppercase;
}

[data-ui-kind="step-flow-step"][data-ui-state="complete"] [data-ui-slot="badge"] {
  background: color-mix(in srgb, var(--origin-color-success) 18%, transparent);
  color: var(--origin-color-success);
}

[data-ui-kind="step-flow-step"][data-ui-state="error"] [data-ui-slot="badge"] {
  background: color-mix(in srgb, var(--origin-color-danger) 18%, transparent);
  color: var(--origin-color-danger);
}

[data-ui-kind="step-flow-actions"] {
  display: flex;
  gap: var(--origin-space-3);
  margin-top: var(--origin-space-4);
}

[data-ui-kind="app-shell"] {
  display: grid;
  gap: var(--origin-space-section);
  min-height: 100%;
  padding: var(--origin-space-section);
}

.site-root {
  min-height: 100vh;
}

.desktop-shell,
[data-ui-kind="viewport"] {
  min-height: 100vh;
  display: grid;
  grid-template-rows: minmax(0, 1fr) var(--origin-shell-taskbar-height);
  background:
    radial-gradient(circle at 12% 18%, color-mix(in srgb, var(--origin-color-accent) 12%, transparent), transparent 26%),
    radial-gradient(circle at 84% 14%, color-mix(in srgb, var(--origin-color-success) 10%, transparent), transparent 28%),
    linear-gradient(180deg, color-mix(in srgb, var(--origin-color-desktop) 94%, transparent), var(--origin-color-desktop));
}

[data-ui-kind="desktop-backdrop"] {
  position: relative;
  overflow: hidden;
  background:
    radial-gradient(circle at center top, color-mix(in srgb, var(--origin-color-text-primary) 4%, transparent), transparent 42%),
    linear-gradient(180deg, color-mix(in srgb, var(--origin-color-desktop) 88%, var(--origin-color-canvas)), var(--origin-color-desktop));
}

[data-ui-kind="wallpaper-layer"],
[data-ui-slot="dismiss-layer"],
[data-ui-kind="desktop-window-layer"] {
  position: absolute;
  inset: 0;
}

[data-ui-kind="wallpaper-layer"] {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

[data-ui-kind="desktop-backdrop"]::after {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(circle at top center, color-mix(in srgb, var(--origin-color-text-primary) 8%, transparent), transparent 34%),
    linear-gradient(180deg, color-mix(in srgb, var(--origin-material-overlay-density-strong) 70%, transparent), transparent 34%, color-mix(in srgb, var(--origin-color-canvas) 22%, transparent));
}

[data-ui-slot="dismiss-layer"] {
  z-index: var(--origin-z-desktop);
}

[data-ui-kind="desktop-icon-grid"] {
  position: absolute;
  top: var(--origin-shell-window-chrome-desktop-padding);
  left: var(--origin-shell-window-chrome-desktop-padding);
  z-index: calc(var(--origin-z-desktop) + 1);
  display: grid;
  gap: var(--origin-space-3);
  width: calc(var(--origin-shell-window-chrome-icon-tile-size) + var(--origin-space-5));
}

[data-ui-kind="desktop-icon-button"] {
  display: grid;
  justify-items: center;
  gap: var(--origin-space-2);
  min-height: var(--origin-shell-window-chrome-icon-tile-size);
  padding: var(--origin-space-3);
  border-radius: var(--origin-radius-lg);
  background: color-mix(in srgb, var(--origin-surface-background-control) 38%, transparent);
  border-color: color-mix(in srgb, var(--origin-surface-border-control) 58%, transparent);
  text-align: center;
  text-shadow: 0 1px 18px rgba(3, 6, 13, 0.65);
}

[data-ui-kind="desktop-icon-button"]:hover,
[data-ui-kind="desktop-icon-button"]:focus-visible {
  background: color-mix(in srgb, var(--origin-state-hover-surface) 92%, transparent);
  border-color: color-mix(in srgb, var(--origin-color-border-selected) 82%, transparent);
}

[data-ui-kind="desktop-icon-button"] > span:first-child {
  width: var(--origin-icon-desktop);
  height: var(--origin-icon-desktop);
  display: grid;
  place-items: center;
  border-radius: var(--origin-radius-lg);
  background: color-mix(in srgb, var(--origin-surface-background-raised) 88%, transparent);
  border: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-raised) 84%, transparent);
  box-shadow: var(--origin-shadow-glass);
  backdrop-filter: blur(var(--origin-elevation-blur-raised));
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-raised));
}

[data-ui-kind="desktop-window-layer"] {
  pointer-events: none;
  z-index: var(--origin-z-windows);
}

[data-ui-kind="window-surface"],
[data-ui-kind="window-frame"] {
  position: absolute;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-width: var(--origin-shell-window-chrome-min-width);
  min-height: var(--origin-shell-window-chrome-min-height);
  max-width: calc(100vw - (var(--origin-shell-window-chrome-desktop-padding) * 2));
  max-height: calc(100vh - var(--origin-shell-taskbar-height) - (var(--origin-shell-window-chrome-desktop-padding) * 2));
  overflow: hidden;
  pointer-events: auto;
  background: var(--origin-surface-background-modal);
  border-color: var(--origin-surface-border-modal);
}

[data-ui-kind="window-surface"][data-ui-focused="false"],
[data-ui-kind="window-frame"][data-ui-focused="false"] {
  border-color: color-mix(in srgb, var(--origin-surface-border-raised) 82%, transparent);
  box-shadow: var(--origin-shadow-window);
  filter: saturate(0.92);
}

[data-ui-kind="window-surface"][data-ui-focused="true"],
[data-ui-kind="window-frame"][data-ui-focused="true"] {
  border-color: var(--origin-color-border-selected);
  box-shadow: var(--origin-shadow-overlay), var(--origin-shadow-focus-ring);
}

[data-ui-kind="window-surface"][data-ui-maximized="true"],
[data-ui-kind="window-frame"][data-ui-maximized="true"] {
  border-radius: var(--origin-radius-md);
}

[data-ui-kind="titlebar-region"],
[data-ui-kind="window-titlebar"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--origin-space-3);
  min-height: var(--origin-shell-titlebar-height);
  padding: 0 var(--origin-space-3);
  background:
    linear-gradient(180deg, var(--origin-surface-highlight-modal), transparent 70%),
    linear-gradient(180deg, color-mix(in srgb, var(--origin-surface-background-modal) 92%, var(--origin-color-text-primary)), var(--origin-surface-background-overlay));
  border-bottom: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-modal) 88%, transparent);
}

[data-ui-kind="window-title"] {
  display: flex;
  align-items: center;
  gap: var(--origin-space-2);
  min-width: 0;
  font-size: var(--origin-type-size-title-sm);
  font-weight: var(--origin-type-weight-semibold);
}

[data-ui-kind="window-title"] > span:last-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

[data-ui-kind="window-controls"] {
  gap: var(--origin-space-1q);
}

[data-ui-slot="window-control"] {
  width: var(--origin-shell-titlebar-button-size);
  min-width: var(--origin-shell-titlebar-button-size);
  min-height: var(--origin-shell-titlebar-button-size);
}

[data-ui-slot="window-control"]:last-child:hover {
  background: color-mix(in srgb, var(--origin-color-danger) 24%, var(--origin-state-hover-surface));
  border-color: color-mix(in srgb, var(--origin-color-danger) 48%, var(--origin-color-border-subtle));
}

[data-ui-kind="window-body"] {
  min-height: 0;
  padding: var(--origin-shell-window-chrome-content-padding);
  overflow: auto;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--origin-surface-highlight-base) 80%, transparent), transparent 14%),
    color-mix(in srgb, var(--origin-surface-background-base) 96%, transparent);
}

[data-ui-kind="resize-handle-region"],
[data-ui-kind="resize-handle"] {
  position: absolute;
}

[data-ui-slot="launcher-menu"] {
  left: var(--origin-space-3);
  bottom: calc(var(--origin-shell-taskbar-height) + var(--origin-space-2));
}

[data-ui-kind="taskbar"] {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--origin-space-3);
  padding-inline: var(--origin-space-3);
  min-height: var(--origin-shell-taskbar-height);
  border-top: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-overlay) 82%, transparent);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--origin-surface-highlight-overlay) 94%, transparent), transparent 70%),
    color-mix(in srgb, var(--origin-surface-background-overlay) 92%, transparent);
  box-shadow: var(--origin-shadow-overlay);
  backdrop-filter: blur(var(--origin-elevation-blur-floating)) saturate(170%);
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-floating)) saturate(170%);
}

[data-ui-kind="taskbar-section"],
[data-ui-kind="tray-list"] {
  gap: var(--origin-space-2);
  min-width: 0;
}

[data-ui-kind="taskbar-section"][data-ui-slot="running"] {
  padding: var(--origin-space-1q);
  border-radius: var(--origin-radius-round);
  background: color-mix(in srgb, var(--origin-surface-background-control) 55%, transparent);
  border: var(--origin-border-width-standard) solid color-mix(in srgb, var(--origin-surface-border-control) 76%, transparent);
  backdrop-filter: blur(var(--origin-elevation-blur-raised));
  -webkit-backdrop-filter: blur(var(--origin-elevation-blur-raised));
}

[data-ui-kind="taskbar-section"][data-ui-slot="start"],
[data-ui-kind="taskbar-section"][data-ui-slot="tray"] {
  padding: var(--origin-space-1q);
  border-radius: var(--origin-radius-round);
}

[data-ui-slot="taskbar-button"],
[data-ui-slot="tray-button"],
[data-ui-slot="clock-button"],
[data-ui-slot="taskbar-overflow-button"],
[data-ui-slot="start-button"],
[data-ui-slot="window-control"] {
  min-height: var(--origin-shell-taskbar-button-height);
}

[data-ui-slot="taskbar-button"],
[data-ui-slot="tray-button"],
[data-ui-slot="clock-button"],
[data-ui-slot="taskbar-overflow-button"],
[data-ui-slot="start-button"] {
  border-radius: var(--origin-radius-round);
}

[data-ui-slot="taskbar-button"][data-ui-selected="true"],
[data-ui-slot="clock-button"][data-ui-selected="true"],
[data-ui-slot="tray-button"][data-ui-selected="true"],
[data-ui-slot="start-button"][data-ui-selected="true"] {
  background: var(--origin-state-selected-surface);
  border-color: var(--origin-color-border-selected);
}

[data-ui-slot="clock-button"] {
  min-width: var(--origin-shell-taskbar-clock-width);
  padding-inline: var(--origin-space-4);
  justify-content: space-between;
}

[data-ui-slot="tray-button"] {
  width: var(--origin-shell-taskbar-button-height);
  min-width: var(--origin-shell-taskbar-button-height);
  padding: 0;
}

.canonical-content {
  display: grid;
  gap: var(--origin-space-4);
  max-width: 720px;
  margin: 0 auto;
  padding: calc(var(--origin-space-section) * 2) var(--origin-space-section);
}

.canonical-content > * {
  margin: 0;
}

.canonical-content a {
  display: inline-flex;
  align-items: center;
  width: fit-content;
  min-height: 40px;
  padding: 0 var(--origin-space-4);
  border-radius: var(--origin-radius-round);
  border: var(--origin-border-width-standard) solid var(--origin-surface-border-control);
  background: var(--origin-surface-background-control);
  text-decoration: none;
}

@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))) {
  [data-ui-kind="surface"],
  [data-ui-kind="panel"],
  [data-ui-kind="list-surface"],
  [data-ui-kind="layer"],
  [data-ui-kind="menu-surface"],
  [data-ui-kind="terminal-surface"],
  [data-ui-kind="completion-list"],
  [data-ui-kind="window-surface"],
  [data-ui-kind="window-frame"],
  [data-ui-kind="toolbar"],
  [data-ui-kind="statusbar"],
  [data-ui-kind="taskbar"],
  [data-ui-kind="taskbar-section"][data-ui-slot="running"],
  [data-ui-kind="desktop-icon-button"] > span:first-child {
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    background: color-mix(in srgb, var(--origin-surface-background-control) 96%, var(--origin-color-canvas));
  }

  [data-ui-kind="menu-surface"],
  [data-ui-kind="window-surface"],
  [data-ui-kind="window-frame"],
  [data-ui-kind="taskbar"] {
    background: color-mix(in srgb, var(--origin-surface-background-overlay) 98%, var(--origin-color-canvas));
  }
}

@media (max-width: 960px) {
  [data-ui-kind="window-surface"],
  [data-ui-kind="window-frame"] {
    max-width: calc(100vw - (var(--origin-space-3) * 2));
  }
}

@media (max-width: 720px) {
  [data-ui-kind="desktop-icon-grid"] {
    position: static;
    width: auto;
    grid-template-columns: repeat(auto-fit, minmax(var(--origin-shell-window-chrome-icon-tile-size), 1fr));
    padding: var(--origin-space-4);
  }

  [data-ui-kind="taskbar"] {
    grid-template-columns: 1fr;
    padding-block: var(--origin-space-2);
    height: auto;
  }

  [data-ui-kind="desktop-window-layer"] {
    position: relative;
    display: grid;
    gap: var(--origin-space-3);
    padding: 0 var(--origin-space-3) var(--origin-space-3);
    overflow-y: auto;
  }

  [data-ui-kind="window-surface"],
  [data-ui-kind="window-frame"] {
    position: relative;
    inset: auto !important;
    width: 100% !important;
    height: auto !important;
    max-width: 100%;
  }

  [data-ui-kind="resize-handle-region"],
  [data-ui-kind="resize-handle"] {
    display: none;
  }
}
"#
}

fn main() {
    println!("cargo:rerun-if-changed=tokens/tokens.toml");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let tokens_path = manifest_dir.join("tokens/tokens.toml");
    let raw = fs::read_to_string(&tokens_path).expect("read tokens.toml");
    let tokens: TokenFile = toml::from_str(&raw).expect("parse tokens.toml");

    let mut rust = String::from("// Generated by system_ui/build.rs. Do not edit by hand.\n");
    rust.push_str("pub const BASELINE_STYLE_ID: &str = \"origin-baseline\";\n");
    push_const_block(&mut rust, "COLOR", &tokens.color);
    push_const_block(&mut rust, "MATERIAL", &tokens.material);
    push_const_block(&mut rust, "SURFACE_BACKGROUND", &tokens.surface.background);
    push_const_block(&mut rust, "SURFACE_BORDER", &tokens.surface.border);
    push_const_block(&mut rust, "SURFACE_HIGHLIGHT", &tokens.surface.highlight);
    push_const_block(&mut rust, "BLUR", &tokens.blur);
    push_const_block(&mut rust, "ELEVATION_ALPHA", &tokens.elevation.alpha);
    push_const_block(&mut rust, "ELEVATION_BORDER", &tokens.elevation.border);
    push_const_block(&mut rust, "ELEVATION_SHADOW", &tokens.elevation.shadow);
    push_const_block(&mut rust, "ELEVATION_BLUR", &tokens.elevation.blur);
    push_const_block(&mut rust, "SPACE", &tokens.spacing);
    push_const_block(&mut rust, "RADIUS", &tokens.radius);
    push_const_block(&mut rust, "SHADOW", &tokens.shadow);
    push_const_block(&mut rust, "OPACITY", &tokens.opacity);
    push_const_block(&mut rust, "Z_INDEX", &tokens.z_index);
    push_const_block(&mut rust, "ICON", &tokens.icon);
    push_const_block(&mut rust, "TYPE_FAMILY", &tokens.typography.family);
    push_const_block(&mut rust, "TYPE_SIZE", &tokens.typography.size);
    push_const_block(&mut rust, "TYPE_WEIGHT", &tokens.typography.weight);
    push_const_block(
        &mut rust,
        "TYPE_LINE_HEIGHT",
        &tokens.typography.line_height,
    );
    push_const_block(
        &mut rust,
        "TYPE_LETTER_SPACING",
        &tokens.typography.letter_spacing,
    );
    push_const_block(&mut rust, "BORDER_WIDTH", &tokens.border.width);
    push_const_block(&mut rust, "BORDER_OPACITY", &tokens.border.opacity);
    push_const_block(&mut rust, "MOTION_DURATION", &tokens.motion.duration);
    push_const_block(&mut rust, "MOTION_EASING", &tokens.motion.easing);
    push_const_block(&mut rust, "STATE_HOVER", &tokens.state.hover);
    push_const_block(&mut rust, "STATE_FOCUS", &tokens.state.focus);
    push_const_block(&mut rust, "STATE_ACTIVE", &tokens.state.active);
    push_const_block(&mut rust, "STATE_DISABLED", &tokens.state.disabled);
    push_const_block(&mut rust, "STATE_SELECTED", &tokens.state.selected);
    push_const_block(&mut rust, "SHELL_TASKBAR", &tokens.shell.taskbar);
    push_const_block(&mut rust, "SHELL_TITLEBAR", &tokens.shell.titlebar);
    push_const_block(
        &mut rust,
        "SHELL_WINDOW_CHROME",
        &tokens.shell.window_chrome,
    );
    push_const_block(
        &mut rust,
        "SHELL_RESIZE_HANDLE",
        &tokens.shell.resize_handle,
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    write_if_changed(&out_dir.join("origin_tokens_generated.rs"), &rust);

    let mut css =
        String::from("/* Generated from ui/crates/system_ui/tokens/tokens.toml */\n:root {\n");
    push_css_vars(&mut css, "color", &tokens.color);
    push_css_vars(&mut css, "material", &tokens.material);
    push_css_vars(&mut css, "surface-background", &tokens.surface.background);
    push_css_vars(&mut css, "surface-border", &tokens.surface.border);
    push_css_vars(&mut css, "surface-highlight", &tokens.surface.highlight);
    push_css_vars(&mut css, "blur", &tokens.blur);
    push_css_vars(&mut css, "elevation-alpha", &tokens.elevation.alpha);
    push_css_vars(&mut css, "elevation-border", &tokens.elevation.border);
    push_css_vars(&mut css, "elevation-shadow", &tokens.elevation.shadow);
    push_css_vars(&mut css, "elevation-blur", &tokens.elevation.blur);
    push_css_vars(&mut css, "space", &tokens.spacing);
    push_css_vars(&mut css, "radius", &tokens.radius);
    push_css_vars(&mut css, "shadow", &tokens.shadow);
    push_css_vars(&mut css, "opacity", &tokens.opacity);
    push_css_vars(&mut css, "z", &tokens.z_index);
    push_css_vars(&mut css, "icon", &tokens.icon);
    push_css_vars(&mut css, "type-family", &tokens.typography.family);
    push_css_vars(&mut css, "type-size", &tokens.typography.size);
    push_css_vars(&mut css, "type-weight", &tokens.typography.weight);
    push_css_vars(&mut css, "type-line-height", &tokens.typography.line_height);
    push_css_vars(
        &mut css,
        "type-letter-spacing",
        &tokens.typography.letter_spacing,
    );
    push_css_vars(&mut css, "border-width", &tokens.border.width);
    push_css_vars(&mut css, "border-opacity", &tokens.border.opacity);
    push_css_vars(&mut css, "motion-duration", &tokens.motion.duration);
    push_css_vars(&mut css, "motion-easing", &tokens.motion.easing);
    push_css_vars(&mut css, "state-hover", &tokens.state.hover);
    push_css_vars(&mut css, "state-focus", &tokens.state.focus);
    push_css_vars(&mut css, "state-active", &tokens.state.active);
    push_css_vars(&mut css, "state-disabled", &tokens.state.disabled);
    push_css_vars(&mut css, "state-selected", &tokens.state.selected);
    push_css_vars(&mut css, "shell-taskbar", &tokens.shell.taskbar);
    push_css_vars(&mut css, "shell-titlebar", &tokens.shell.titlebar);
    push_css_vars(&mut css, "shell-window-chrome", &tokens.shell.window_chrome);
    push_css_vars(&mut css, "shell-resize-handle", &tokens.shell.resize_handle);
    css.push_str("}\n");
    css.push_str(
        "\n:root[data-high-contrast=\"true\"],\n.desktop-shell[data-high-contrast=\"true\"] {\n  --origin-color-canvas: #010101;\n  --origin-color-desktop: #040608;\n  --origin-color-text-primary: #ffffff;\n  --origin-color-text-secondary: #f2f5f9;\n  --origin-color-text-muted: #dde5ee;\n  --origin-color-text-inverse: #020305;\n  --origin-color-border-focus: #ffffff;\n  --origin-color-border-selected: #9ed1ff;\n  --origin-surface-background-base: rgba(12, 18, 28, 0.96);\n  --origin-surface-background-raised: rgba(17, 24, 36, 0.98);\n  --origin-surface-background-overlay: rgba(18, 24, 35, 0.985);\n  --origin-surface-background-modal: rgba(20, 28, 40, 0.992);\n  --origin-surface-background-control: rgba(16, 23, 34, 0.98);\n  --origin-surface-border-base: rgba(255, 255, 255, 0.48);\n  --origin-surface-border-raised: rgba(255, 255, 255, 0.58);\n  --origin-surface-border-overlay: rgba(255, 255, 255, 0.72);\n  --origin-surface-border-modal: rgba(255, 255, 255, 0.78);\n  --origin-surface-border-control: rgba(255, 255, 255, 0.56);\n  --origin-shadow-panel: none;\n  --origin-shadow-window: none;\n  --origin-shadow-overlay: none;\n  --origin-shadow-glass: none;\n  --origin-shadow-pressed: none;\n  --origin-shadow-focus-ring: 0 0 0 3px rgba(255, 255, 255, 0.34);\n}\n",
    );
    css.push_str(
        "\n:root[data-reduced-motion=\"true\"],\n.desktop-shell[data-reduced-motion=\"true\"] {\n  --origin-motion-duration-fast: 0ms;\n  --origin-motion-duration-standard: 0ms;\n  --origin-motion-duration-slow: 0ms;\n}\n",
    );

    let site_dir = manifest_dir.parent().expect("ui/crates").join("site");
    write_if_changed(&site_dir.join("src/generated/tokens.css"), &css);
    write_if_changed(
        &site_dir.join("src/generated/tailwind.css"),
        generated_tailwind_css(),
    );
    write_if_changed(&site_dir.join("tailwind.config.js"), tailwind_config());
}
