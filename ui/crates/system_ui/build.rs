use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TokenFile {
    raw: RawTokens,
    semantic: SemanticTokens,
    theme: ThemeTokens,
}

#[derive(Debug, Deserialize)]
struct RawTokens {
    color: BTreeMap<String, String>,
    space: BTreeMap<String, String>,
    #[serde(rename = "type")]
    type_tokens: BTreeMap<String, String>,
    blur: BTreeMap<String, String>,
    radius: BTreeMap<String, String>,
    motion: BTreeMap<String, String>,
    border: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct SemanticTokens {
    surface: BTreeMap<String, String>,
    control: BTreeMap<String, String>,
    text: BTreeMap<String, String>,
    border: BTreeMap<String, String>,
    state: BTreeMap<String, String>,
    shell: BTreeMap<String, String>,
    layer: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ThemeTokens {
    default: String,
    dark: BTreeMap<String, String>,
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
        semantic: {
          text: {
            primary: "var(--origin-semantic-text-primary)",
            secondary: "var(--origin-semantic-text-secondary)",
            muted: "var(--origin-semantic-text-muted)",
            inverse: "var(--origin-semantic-text-inverse)",
          },
          border: {
            standard: "var(--origin-semantic-border-standard)",
            focus: "var(--origin-semantic-border-focus)",
            selected: "var(--origin-semantic-border-selected)",
          },
          surface: {
            taskbar: "var(--origin-semantic-surface-taskbar-background)",
            window: "var(--origin-semantic-surface-window-background)",
            windowActive: "var(--origin-semantic-surface-window-active-background)",
            menu: "var(--origin-semantic-surface-menu-background)",
            modal: "var(--origin-semantic-surface-modal-background)",
          },
          control: {
            neutral: "var(--origin-semantic-control-neutral-background)",
            accent: "var(--origin-semantic-control-accent-background)",
            danger: "var(--origin-semantic-control-danger-background)",
          },
          state: {
            hover: "var(--origin-semantic-state-hover-surface)",
            active: "var(--origin-semantic-state-active-surface)",
            selected: "var(--origin-semantic-state-selected-surface)",
            focusRing: "var(--origin-semantic-state-focus-ring)",
          },
        },
      },
      spacing: {
        0: "var(--origin-raw-space-0)",
        2: "var(--origin-raw-space-2)",
        4: "var(--origin-raw-space-4)",
        8: "var(--origin-raw-space-8)",
        12: "var(--origin-raw-space-12)",
        16: "var(--origin-raw-space-16)",
        20: "var(--origin-raw-space-20)",
        24: "var(--origin-raw-space-24)",
        28: "var(--origin-raw-space-28)",
        32: "var(--origin-raw-space-32)",
        40: "var(--origin-raw-space-40)",
        48: "var(--origin-raw-space-48)",
      },
      borderRadius: {
        shellSm: "var(--origin-raw-radius-8)",
        shellMd: "var(--origin-raw-radius-12)",
        shellLg: "var(--origin-raw-radius-16)",
        round: "var(--origin-raw-radius-round)",
      },
      boxShadow: {
        embedded: "var(--origin-semantic-layer-embedded-shadow)",
        raised: "var(--origin-semantic-layer-raised-shadow)",
        floating: "var(--origin-semantic-layer-floating-shadow)",
        modal: "var(--origin-semantic-layer-modal-shadow)",
      },
      zIndex: {
        wallpaper: "var(--origin-semantic-layer-wallpaper)",
        desktopBackdrop: "var(--origin-semantic-layer-desktop-backdrop)",
        taskbar: "var(--origin-semantic-layer-taskbar)",
        windows: "var(--origin-semantic-layer-windows)",
        menus: "var(--origin-semantic-layer-menus)",
        modal: "var(--origin-semantic-layer-modal)",
      },
      transitionDuration: {
        fast: "var(--origin-raw-motion-duration-fast)",
        DEFAULT: "var(--origin-raw-motion-duration-standard)",
        slow: "var(--origin-raw-motion-duration-slow)",
      },
      transitionTimingFunction: {
        standard: "var(--origin-raw-motion-easing-standard)",
        emphasized: "var(--origin-raw-motion-easing-emphasized)",
      },
    },
  },
  plugins: [
    plugin(function ({ addUtilities }) {
      addUtilities({
        ".shell-focus-ring": {
          boxShadow: "0 0 0 var(--origin-raw-border-focus-ring-width) var(--origin-semantic-state-focus-ring)",
        },
      });
    }),
  ],
  corePlugins: { preflight: false },
};
"#
}

fn generated_tailwind_css() -> &'static str {
    r#"/* Generated from ui/crates/system_ui/tokens/tokens.toml */
*,
*::before,
*::after {
  box-sizing: border-box;
}

:root,
html,
body,
.site-root,
.desktop-shell,
[data-ui-kind="app-shell"],
[data-ui-kind="viewport"] {
  min-height: 100%;
}

html,
body {
  margin: 0;
  padding: 0;
  background:
    radial-gradient(circle at top left, color-mix(in srgb, var(--origin-raw-color-accent) 12%, transparent), transparent 32%),
    linear-gradient(180deg, var(--origin-raw-color-canvas), var(--origin-raw-color-desktop));
  color: var(--origin-semantic-text-primary);
  font-family: var(--origin-raw-type-family-sans);
  font-size: var(--origin-raw-type-size-body);
  line-height: var(--origin-raw-type-line-body);
}

body {
  overflow: hidden;
}

button,
input,
textarea,
select {
  font: inherit;
}

a {
  color: var(--origin-raw-color-accent-strong);
}

.site-root {
  min-height: 100vh;
}

.canonical-content {
  display: grid;
  gap: var(--origin-raw-space-12);
  max-width: 760px;
  margin: 0 auto;
  padding: var(--origin-raw-space-32);
}

[data-ui-kind="viewport"] {
  position: relative;
  min-height: 100vh;
  width: 100%;
}

[data-ui-kind="app-shell"] {
  position: relative;
  min-height: 100vh;
  isolation: isolate;
}

[data-ui-kind="desktop-backdrop"] {
  position: absolute;
  inset: 0;
  z-index: var(--origin-semantic-layer-desktop-backdrop);
}

[data-ui-kind="desktop-window-layer"] {
  position: absolute;
  inset: 0;
  z-index: var(--origin-semantic-layer-windows);
  pointer-events: none;
}

[data-ui-kind="desktop-window-layer"] > * {
  pointer-events: auto;
}

[data-ui-slot="wallpaper-layer"] {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  z-index: var(--origin-semantic-layer-wallpaper);
}

[data-ui-primitive="true"],
[data-ui-kind] {
  transition:
    background-color var(--origin-raw-motion-duration-standard) var(--origin-raw-motion-easing-standard),
    border-color var(--origin-raw-motion-duration-standard) var(--origin-raw-motion-easing-standard),
    box-shadow var(--origin-raw-motion-duration-standard) var(--origin-raw-motion-easing-standard),
    color var(--origin-raw-motion-duration-standard) var(--origin-raw-motion-easing-standard),
    opacity var(--origin-raw-motion-duration-standard) var(--origin-raw-motion-easing-standard),
    transform var(--origin-raw-motion-duration-fast) var(--origin-raw-motion-easing-standard);
}

[data-ui-kind="stack"] {
  display: flex;
  flex-direction: column;
}

[data-ui-kind="cluster"],
[data-ui-kind="inline"],
[data-ui-kind="taskbar-section"],
[data-ui-kind="window-controls"],
[data-ui-kind="window-title"],
[data-ui-kind="statusbar"],
[data-ui-kind="toolbar"] {
  display: flex;
}

[data-ui-kind="grid"],
[data-ui-kind="desktop-icon-grid"] {
  display: grid;
}

[data-ui-kind="center"] {
  display: grid;
  place-items: center;
}

[data-ui-kind="inset"] {
  display: block;
}

[data-ui-kind="layer"] {
  position: relative;
}

[data-ui-gap="none"] { gap: var(--origin-raw-space-0); }
[data-ui-gap="sm"] { gap: var(--origin-raw-space-8); }
[data-ui-gap="md"] { gap: var(--origin-raw-space-12); }
[data-ui-gap="lg"] { gap: var(--origin-raw-space-16); }

[data-ui-padding="none"] { padding: var(--origin-raw-space-0); }
[data-ui-padding="sm"] { padding: var(--origin-raw-space-8); }
[data-ui-padding="md"] { padding: var(--origin-raw-space-12); }
[data-ui-padding="lg"] { padding: var(--origin-raw-space-16); }

[data-ui-align="stretch"] { align-items: stretch; }
[data-ui-align="start"] { align-items: flex-start; }
[data-ui-align="center"] { align-items: center; }
[data-ui-align="end"] { align-items: flex-end; }

[data-ui-justify="start"] { justify-content: flex-start; }
[data-ui-justify="center"] { justify-content: center; }
[data-ui-justify="between"] { justify-content: space-between; }
[data-ui-justify="end"] { justify-content: flex-end; }

[data-ui-kind="text"],
[data-ui-kind="heading"],
[data-ui-kind="window-title"],
[data-ui-kind="statusbar-item"] {
  min-width: 0;
}

[data-ui-variant="body"] {
  font-size: var(--origin-raw-type-size-body);
  line-height: var(--origin-raw-type-line-body);
}

[data-ui-variant="label"] {
  font-size: var(--origin-raw-type-size-label);
  line-height: var(--origin-raw-type-line-tight);
  font-weight: var(--origin-raw-type-weight-medium);
}

[data-ui-variant="caption"] {
  font-size: var(--origin-raw-type-size-caption);
  line-height: var(--origin-raw-type-line-tight);
  letter-spacing: var(--origin-raw-type-tracking-wide);
}

[data-ui-variant="title"] {
  font-size: var(--origin-raw-type-size-title);
  line-height: var(--origin-raw-type-line-title);
  font-weight: var(--origin-raw-type-weight-semibold);
  letter-spacing: var(--origin-raw-type-tracking-tight);
}

[data-ui-variant="code"] {
  font-family: var(--origin-raw-type-family-mono);
  font-size: var(--origin-raw-type-size-code);
  line-height: var(--origin-raw-type-line-body);
}

[data-ui-tone="primary"] { color: var(--origin-semantic-text-primary); }
[data-ui-tone="secondary"] { color: var(--origin-semantic-text-secondary); }
[data-ui-tone="accent"] { color: var(--origin-raw-color-accent-strong); }
[data-ui-tone="success"] { color: var(--origin-raw-color-success); }
[data-ui-tone="warning"] { color: var(--origin-raw-color-warning); }
[data-ui-tone="danger"] { color: var(--origin-raw-color-danger); }

[data-ui-surface-role="shell"] {
  --ui-surface-background: var(--origin-semantic-surface-shell-background);
  --ui-surface-border: var(--origin-semantic-surface-shell-border);
  --ui-surface-highlight: var(--origin-semantic-surface-shell-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-shell-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-shell-blur);
}

[data-ui-surface-role="taskbar"] {
  --ui-surface-background: var(--origin-semantic-surface-taskbar-background);
  --ui-surface-border: var(--origin-semantic-surface-taskbar-border);
  --ui-surface-highlight: var(--origin-semantic-surface-taskbar-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-taskbar-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-taskbar-blur);
}

[data-ui-surface-role="window-active"] {
  --ui-surface-background: var(--origin-semantic-surface-window-active-background);
  --ui-surface-border: var(--origin-semantic-surface-window-active-border);
  --ui-surface-highlight: var(--origin-semantic-surface-window-active-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-window-active-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-window-active-blur);
}

[data-ui-surface-role="window-inactive"] {
  --ui-surface-background: var(--origin-semantic-surface-window-inactive-background);
  --ui-surface-border: var(--origin-semantic-surface-window-inactive-border);
  --ui-surface-highlight: var(--origin-semantic-surface-window-inactive-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-window-inactive-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-window-inactive-blur);
}

[data-ui-surface-role="menu"] {
  --ui-surface-background: var(--origin-semantic-surface-menu-background);
  --ui-surface-border: var(--origin-semantic-surface-menu-border);
  --ui-surface-highlight: var(--origin-semantic-surface-menu-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-menu-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-menu-blur);
}

[data-ui-surface-role="modal"] {
  --ui-surface-background: var(--origin-semantic-surface-modal-background);
  --ui-surface-border: var(--origin-semantic-surface-modal-border);
  --ui-surface-highlight: var(--origin-semantic-surface-modal-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-modal-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-modal-blur);
}

[data-ui-kind="surface"],
[data-ui-kind="panel"],
[data-ui-kind="list-surface"],
[data-ui-kind="completion-list"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"],
[data-ui-kind="menu-surface"],
[data-ui-kind="disclosure"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="toggle-row"],
[data-ui-kind="taskbar"],
[data-ui-kind="window-frame"],
[data-ui-kind="window-surface"],
[data-ui-kind="launcher-panel"],
[data-ui-kind="side-panel"],
[data-ui-kind="notification-center"] {
  position: relative;
  border: var(--origin-raw-border-width-1) solid var(--ui-surface-border, var(--origin-semantic-border-standard));
  background: var(--ui-surface-background, var(--origin-semantic-surface-raised-background));
  color: var(--origin-semantic-text-primary);
  box-shadow: var(--ui-surface-shadow, var(--origin-semantic-layer-raised-shadow));
  backdrop-filter: blur(var(--ui-surface-blur, var(--origin-semantic-surface-raised-blur))) saturate(150%);
  -webkit-backdrop-filter: blur(var(--ui-surface-blur, var(--origin-semantic-surface-raised-blur))) saturate(150%);
}

[data-ui-kind="surface"]::before,
[data-ui-kind="panel"]::before,
[data-ui-kind="toolbar"]::before,
[data-ui-kind="statusbar"]::before,
[data-ui-kind="taskbar"]::before,
[data-ui-kind="window-frame"]::before,
[data-ui-kind="window-surface"]::before,
[data-ui-kind="menu-surface"]::before,
[data-ui-kind="launcher-panel"]::before,
[data-ui-kind="side-panel"]::before,
[data-ui-kind="notification-center"]::before {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: linear-gradient(180deg, var(--ui-surface-highlight, transparent), transparent 48%);
  pointer-events: none;
}

[data-ui-kind="surface"],
[data-ui-kind="panel"],
[data-ui-kind="list-surface"],
[data-ui-kind="completion-list"],
[data-ui-kind="disclosure"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="toggle-row"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"] {
  border-radius: var(--origin-raw-radius-12);
}

[data-ui-kind="surface"],
[data-ui-kind="panel"],
[data-ui-kind="list-surface"] {
  --ui-surface-background: var(--origin-semantic-surface-embedded-background);
  --ui-surface-border: var(--origin-semantic-surface-embedded-border);
  --ui-surface-highlight: var(--origin-semantic-surface-embedded-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-embedded-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-embedded-blur);
}

[data-ui-kind="panel"],
[data-ui-kind="list-surface"],
[data-ui-kind="completion-list"],
[data-ui-kind="toolbar"],
[data-ui-kind="statusbar"],
[data-ui-kind="disclosure"],
[data-ui-kind="step-flow-step"],
[data-ui-kind="toggle-row"] {
  --ui-surface-background: var(--origin-semantic-surface-raised-background);
  --ui-surface-border: var(--origin-semantic-surface-raised-border);
  --ui-surface-highlight: var(--origin-semantic-surface-raised-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-raised-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-raised-blur);
}

[data-ui-kind="button"],
[data-ui-kind="icon-button"],
[data-ui-kind="desktop-icon-button"] {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--origin-raw-space-8);
  border: var(--origin-raw-border-width-1) solid var(--origin-semantic-border-standard);
  border-radius: var(--origin-raw-radius-12);
  padding: 0 var(--origin-raw-space-12);
  background: var(--origin-semantic-control-neutral-background);
  color: var(--origin-semantic-text-primary);
  min-height: 36px;
  cursor: pointer;
  backdrop-filter: blur(var(--origin-raw-blur-embedded)) saturate(145%);
  -webkit-backdrop-filter: blur(var(--origin-raw-blur-embedded)) saturate(145%);
}

[data-ui-control-tone="accent"],
[data-ui-variant="primary"],
[data-ui-variant="accent"] {
  background: var(--origin-semantic-control-accent-background);
  border-color: var(--origin-semantic-control-accent-border);
}

[data-ui-control-tone="danger"],
[data-ui-variant="danger"] {
  background: var(--origin-semantic-control-danger-background);
  border-color: var(--origin-semantic-control-danger-border);
}

[data-ui-variant="quiet"] {
  background: transparent;
}

[data-ui-size="sm"] { min-height: 32px; padding: 0 var(--origin-raw-space-8); }
[data-ui-size="md"] { min-height: 36px; padding: 0 var(--origin-raw-space-12); }
[data-ui-size="lg"] { min-height: 40px; padding: 0 var(--origin-raw-space-16); }

[data-ui-shape="pill"] { border-radius: var(--origin-raw-radius-round); }
[data-ui-shape="circle"] {
  border-radius: var(--origin-raw-radius-round);
  width: 36px;
  min-width: 36px;
  padding: 0;
}

[data-ui-kind="button"]:hover,
[data-ui-kind="icon-button"]:hover,
[data-ui-kind="desktop-icon-button"]:hover,
[data-ui-kind="menu-item"]:hover,
[data-ui-kind="tray-button"]:hover,
[data-ui-kind="taskbar-button"]:hover {
  background: var(--origin-semantic-state-hover-surface);
}

[data-ui-kind="button"]:active,
[data-ui-kind="icon-button"]:active,
[data-ui-kind="desktop-icon-button"]:active {
  background: var(--origin-semantic-state-active-surface);
  transform: translateY(1px);
}

[data-ui-kind="button"][data-ui-selected="true"],
[data-ui-kind="button"][data-ui-pressed="true"],
[data-ui-kind="desktop-icon-button"][data-ui-selected="true"] {
  background: var(--origin-semantic-state-selected-surface);
  border-color: var(--origin-semantic-border-selected);
}

[data-ui-kind="button"][disabled],
[data-ui-kind="button"][data-ui-disabled="true"],
[data-ui-kind="icon-button"][disabled],
[data-ui-kind="text-field"][disabled] {
  cursor: default;
  opacity: var(--origin-semantic-state-disabled-content);
}

[data-ui-kind="button"]:focus-visible,
[data-ui-kind="icon-button"]:focus-visible,
[data-ui-kind="text-field"]:focus-visible,
[data-ui-kind="desktop-icon-button"]:focus-visible {
  outline: none;
  box-shadow: 0 0 0 var(--origin-raw-border-focus-ring-width) var(--origin-semantic-state-focus-ring);
  border-color: var(--origin-semantic-border-focus);
}

[data-ui-kind="text-field"] {
  width: 100%;
  min-height: 36px;
  border: var(--origin-raw-border-width-1) solid var(--origin-semantic-border-standard);
  border-radius: var(--origin-raw-radius-12);
  background: var(--origin-semantic-control-neutral-background);
  color: var(--origin-semantic-text-primary);
  padding: 0 var(--origin-raw-space-12);
}

[data-ui-kind="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--origin-raw-color-accent);
}

[data-ui-kind="desktop-icon-grid"] {
  position: absolute;
  top: var(--origin-semantic-shell-desktop-padding);
  left: var(--origin-semantic-shell-desktop-padding);
  grid-template-columns: repeat(auto-fill, minmax(var(--origin-semantic-shell-desktop-icon-tile-size), var(--origin-semantic-shell-desktop-icon-tile-size)));
  gap: var(--origin-raw-space-12);
  align-content: start;
}

[data-ui-kind="desktop-icon-button"] {
  min-height: var(--origin-semantic-shell-desktop-icon-tile-size);
  width: var(--origin-semantic-shell-desktop-icon-tile-size);
  flex-direction: column;
  justify-content: flex-start;
  padding: var(--origin-raw-space-8);
  background: rgba(255, 255, 255, 0.14);
  border-radius: var(--origin-raw-radius-16);
}

[data-ui-kind="taskbar"] {
  position: absolute;
  left: var(--origin-semantic-shell-desktop-padding);
  right: var(--origin-semantic-shell-desktop-padding);
  bottom: var(--origin-semantic-shell-desktop-padding);
  min-height: var(--origin-semantic-shell-taskbar-height);
  padding: var(--origin-semantic-shell-taskbar-padding-block) var(--origin-semantic-shell-taskbar-padding-inline);
  border-radius: var(--origin-raw-radius-round);
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--origin-semantic-shell-taskbar-section-gap);
  z-index: var(--origin-semantic-layer-taskbar);
}

[data-ui-slot="taskbar-overlay"] {
  position: absolute;
  left: 50%;
  bottom: var(--origin-semantic-shell-dock-floating-offset);
  z-index: var(--origin-semantic-layer-taskbar);
  transform: translateX(-50%);
  pointer-events: none;
}

[data-ui-kind="dock"] {
  display: inline-flex;
  align-items: center;
  gap: var(--origin-semantic-shell-dock-spacing);
  margin: 0;
  padding: var(--origin-semantic-shell-dock-padding);
  min-height: var(--origin-semantic-shell-dock-height);
  width: max-content;
  max-width: calc(100vw - 20px);
  border: var(--origin-raw-border-width-1) solid color-mix(in srgb, var(--origin-semantic-surface-taskbar-border) 72%, transparent);
  border-radius: var(--origin-raw-radius-round);
  background: color-mix(in srgb, var(--origin-semantic-surface-taskbar-background) 88%, transparent);
  box-shadow: var(--origin-semantic-layer-floating-shadow);
  backdrop-filter: blur(var(--origin-semantic-surface-taskbar-blur)) saturate(130%);
  -webkit-backdrop-filter: blur(var(--origin-semantic-surface-taskbar-blur)) saturate(130%);
  pointer-events: auto;
}

[data-ui-kind="taskbar-section"] {
  min-width: 0;
  align-items: center;
  gap: var(--origin-semantic-shell-taskbar-item-gap);
}

[data-ui-kind="dock-section"] {
  min-width: 0;
  align-items: center;
  gap: var(--origin-semantic-shell-dock-spacing);
}

[data-ui-kind="taskbar-section"][data-ui-slot="center"] {
  justify-content: center;
}

[data-ui-kind="taskbar-section"][data-ui-slot="right"] {
  justify-content: flex-end;
}

[data-ui-kind="dock-section"][data-ui-slot="running"] {
  flex: 1 1 auto;
  justify-content: flex-start;
}

[data-ui-kind="dock-section"][data-ui-slot="left"],
[data-ui-kind="dock-section"][data-ui-slot="right"] {
  flex: 0 0 auto;
}

[data-ui-kind="dock-section"][data-ui-slot="right"] {
  justify-content: flex-end;
}

[data-ui-slot="taskbar-button"],
[data-ui-slot="taskbar-overflow-button"],
[data-ui-slot="tray-button"],
[data-ui-slot="clock-button"] {
  min-height: var(--origin-semantic-shell-taskbar-button-height);
  border-radius: var(--origin-raw-radius-round);
  padding: 0 var(--origin-raw-space-8);
}

[data-ui-slot="taskbar-button"] {
  min-width: var(--origin-semantic-shell-taskbar-button-height);
}

[data-ui-slot="clock-button"] {
  min-width: var(--origin-semantic-shell-taskbar-clock-min-width);
  justify-content: flex-end;
}

[data-ui-slot="dock-button"],
[data-ui-slot="dock-launcher-button"],
[data-ui-slot="dock-overflow-button"] {
  width: var(--origin-semantic-shell-dock-button-size);
  min-width: var(--origin-semantic-shell-dock-button-size);
  min-height: var(--origin-semantic-shell-dock-button-size);
  padding: 0;
  border-radius: var(--origin-raw-radius-round);
}

[data-ui-slot="dock-clock"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: var(--origin-semantic-shell-taskbar-clock-min-width);
  padding: 0 var(--origin-raw-space-8);
  font-size: var(--origin-raw-type-size-label);
  font-weight: var(--origin-raw-type-weight-semibold);
  letter-spacing: var(--origin-raw-type-tracking-wide);
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

[data-ui-kind="tray-list"] {
  display: inline-flex;
  align-items: center;
  gap: var(--origin-raw-space-4);
}

[data-ui-kind="window-frame"],
[data-ui-kind="window-surface"] {
  position: absolute;
  min-width: var(--origin-semantic-shell-window-min-width);
  min-height: var(--origin-semantic-shell-window-min-height);
  border-radius: var(--origin-raw-radius-16);
  overflow: hidden;
  z-index: var(--origin-semantic-layer-windows);
}

[data-ui-kind="window-frame"][data-ui-focused="true"],
[data-ui-kind="window-surface"][data-ui-focused="true"] {
  --ui-surface-background: var(--origin-semantic-surface-window-active-background);
  --ui-surface-border: var(--origin-semantic-surface-window-active-border);
  --ui-surface-highlight: var(--origin-semantic-surface-window-active-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-window-active-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-window-active-blur);
}

[data-ui-kind="window-frame"][data-ui-focused="false"],
[data-ui-kind="window-surface"][data-ui-focused="false"] {
  --ui-surface-background: var(--origin-semantic-surface-window-inactive-background);
  --ui-surface-border: var(--origin-semantic-surface-window-inactive-border);
  --ui-surface-highlight: var(--origin-semantic-surface-window-inactive-highlight);
  --ui-surface-shadow: var(--origin-semantic-surface-window-inactive-shadow);
  --ui-surface-blur: var(--origin-semantic-surface-window-inactive-blur);
}

[data-ui-kind="window-frame"][data-ui-maximized="true"] {
  border-radius: var(--origin-raw-radius-8);
}

[data-ui-kind="window-titlebar"],
[data-ui-kind="titlebar-region"] {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--origin-raw-space-12);
  min-height: var(--origin-semantic-shell-titlebar-height);
  padding: 0 var(--origin-raw-space-12);
  border-bottom: var(--origin-raw-border-width-1) solid color-mix(in srgb, var(--ui-surface-border, var(--origin-semantic-border-standard)) 80%, transparent);
}

[data-ui-kind="window-title"] {
  align-items: center;
  gap: var(--origin-raw-space-8);
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  font-size: var(--origin-raw-type-size-label);
  font-weight: var(--origin-raw-type-weight-medium);
}

[data-ui-kind="window-controls"] {
  align-items: center;
  gap: var(--origin-raw-space-4);
}

[data-ui-slot="window-control"] {
  width: var(--origin-semantic-shell-titlebar-control-size);
  min-width: var(--origin-semantic-shell-titlebar-control-size);
  min-height: var(--origin-semantic-shell-titlebar-control-size);
  padding: 0;
  border-radius: var(--origin-raw-radius-round);
}

[data-ui-kind="window-body"] {
  height: calc(100% - var(--origin-semantic-shell-titlebar-height));
  padding: var(--origin-semantic-shell-window-content-padding);
  overflow: auto;
}

[data-ui-kind="resize-handle"],
[data-ui-kind="resize-handle-region"] {
  position: absolute;
  z-index: 3;
}

[data-ui-slot="edge-n"] {
  top: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  left: var(--origin-semantic-shell-resize-handle-corner);
  right: var(--origin-semantic-shell-resize-handle-corner);
  height: calc(var(--origin-semantic-shell-resize-handle-edge) + var(--origin-semantic-shell-resize-handle-hit-outset));
  cursor: ns-resize;
}

[data-ui-slot="edge-s"] {
  bottom: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  left: var(--origin-semantic-shell-resize-handle-corner);
  right: var(--origin-semantic-shell-resize-handle-corner);
  height: calc(var(--origin-semantic-shell-resize-handle-edge) + var(--origin-semantic-shell-resize-handle-hit-outset));
  cursor: ns-resize;
}

[data-ui-slot="edge-e"] {
  top: var(--origin-semantic-shell-resize-handle-corner);
  bottom: var(--origin-semantic-shell-resize-handle-corner);
  right: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  width: calc(var(--origin-semantic-shell-resize-handle-edge) + var(--origin-semantic-shell-resize-handle-hit-outset));
  cursor: ew-resize;
}

[data-ui-slot="edge-w"] {
  top: var(--origin-semantic-shell-resize-handle-corner);
  bottom: var(--origin-semantic-shell-resize-handle-corner);
  left: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  width: calc(var(--origin-semantic-shell-resize-handle-edge) + var(--origin-semantic-shell-resize-handle-hit-outset));
  cursor: ew-resize;
}

[data-ui-slot="edge-ne"],
[data-ui-slot="edge-nw"],
[data-ui-slot="edge-se"],
[data-ui-slot="edge-sw"] {
  width: calc(var(--origin-semantic-shell-resize-handle-corner) + var(--origin-semantic-shell-resize-handle-hit-outset));
  height: calc(var(--origin-semantic-shell-resize-handle-corner) + var(--origin-semantic-shell-resize-handle-hit-outset));
}

[data-ui-slot="edge-ne"] {
  top: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  right: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  cursor: nesw-resize;
}

[data-ui-slot="edge-nw"] {
  top: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  left: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  cursor: nwse-resize;
}

[data-ui-slot="edge-se"] {
  bottom: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  right: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  cursor: nwse-resize;
}

[data-ui-slot="edge-sw"] {
  bottom: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  left: calc(var(--origin-semantic-shell-resize-handle-hit-outset) * -1);
  cursor: nesw-resize;
}

[data-ui-kind="menu-surface"],
[data-ui-kind="launcher-panel"],
[data-ui-kind="side-panel"],
[data-ui-kind="notification-center"] {
  border-radius: var(--origin-raw-radius-16);
}

[data-ui-kind="menu-surface"] {
  width: min(var(--origin-semantic-shell-menu-width), calc(100vw - 24px));
  padding: var(--origin-raw-space-8);
  display: grid;
  gap: var(--origin-raw-space-4);
  z-index: var(--origin-semantic-layer-menus);
}

[data-ui-slot="menu-item"] {
  width: 100%;
  justify-content: flex-start;
  min-height: 34px;
}

[data-ui-kind="menu-separator"] {
  height: var(--origin-raw-border-width-1);
  margin: var(--origin-raw-space-4) 0;
  background: color-mix(in srgb, var(--origin-semantic-border-standard) 78%, transparent);
}

[data-ui-kind="launcher-panel"],
[data-ui-kind="side-panel"],
[data-ui-kind="notification-center"] {
  width: min(var(--origin-semantic-shell-panel-width), calc(100vw - 24px));
  padding: var(--origin-raw-space-16);
  display: grid;
  gap: var(--origin-raw-space-12);
  z-index: var(--origin-semantic-layer-menus);
}

[data-ui-kind="notification-center"] {
  width: min(var(--origin-semantic-shell-notification-width), calc(100vw - 24px));
}

[data-ui-kind="step-flow"] {
  display: grid;
  gap: var(--origin-raw-space-12);
}

[data-ui-kind="step-flow-header"] {
  display: grid;
  gap: var(--origin-raw-space-4);
}

[data-ui-kind="step-flow-step"],
[data-ui-kind="disclosure"],
[data-ui-kind="toggle-row"] {
  padding: var(--origin-raw-space-16);
}

[data-ui-kind="step-flow-actions"] {
  display: flex;
  gap: var(--origin-raw-space-8);
  margin-top: var(--origin-raw-space-12);
}

[data-ui-kind="toggle-row"] {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--origin-raw-space-12);
}

[data-ui-kind="statusbar"] {
  justify-content: space-between;
  gap: var(--origin-raw-space-12);
  padding: var(--origin-raw-space-8) var(--origin-raw-space-12);
}

[data-ui-kind="statusbar-item"] {
  font-size: var(--origin-raw-type-size-caption);
  color: var(--origin-semantic-text-secondary);
}

[data-ui-kind="toolbar"] {
  align-items: center;
  flex-wrap: wrap;
  gap: var(--origin-raw-space-8);
  padding: var(--origin-raw-space-8);
}

@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px))) {
  [data-ui-kind="surface"],
  [data-ui-kind="panel"],
  [data-ui-kind="list-surface"],
  [data-ui-kind="completion-list"],
  [data-ui-kind="toolbar"],
  [data-ui-kind="statusbar"],
  [data-ui-kind="menu-surface"],
  [data-ui-kind="taskbar"],
  [data-ui-kind="dock"],
  [data-ui-kind="window-frame"],
  [data-ui-kind="window-surface"],
  [data-ui-kind="launcher-panel"],
  [data-ui-kind="side-panel"],
  [data-ui-kind="notification-center"] {
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }
}

@media (max-width: 960px) {
  [data-ui-kind="taskbar"] {
    grid-template-columns: auto minmax(0, 1fr);
  }

  [data-ui-kind="taskbar-section"][data-ui-slot="center"] {
    justify-content: flex-start;
  }
}

@media (max-width: 720px) {
  [data-ui-kind="taskbar"] {
    left: var(--origin-raw-space-8);
    right: var(--origin-raw-space-8);
    bottom: var(--origin-raw-space-8);
  }

  [data-ui-kind="window-frame"],
  [data-ui-kind="window-surface"] {
    min-width: min(var(--origin-semantic-shell-window-min-width), calc(100vw - 16px));
  }
}
"#
}

fn main() {
    println!("cargo:rerun-if-env-changed=ORIGIN_FORCE_SYSTEM_UI_GENERATION");
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let token_path = manifest_dir.join("tokens/tokens.toml");
    println!("cargo:rerun-if-changed={}", token_path.display());

    let raw = fs::read_to_string(&token_path).expect("read tokens.toml");
    let tokens: TokenFile = toml::from_str(&raw).expect("parse tokens.toml");
    let _theme_default = tokens.theme.default.as_str();

    let mut rust = String::from("// Generated by system_ui/build.rs. Do not edit by hand.\n");
    rust.push_str("pub const BASELINE_STYLE_ID: &str = \"origin-baseline\";\n");
    push_const_block(&mut rust, "RAW_COLOR", &tokens.raw.color);
    push_const_block(&mut rust, "RAW_SPACE", &tokens.raw.space);
    push_const_block(&mut rust, "RAW_TYPE", &tokens.raw.type_tokens);
    push_const_block(&mut rust, "RAW_BLUR", &tokens.raw.blur);
    push_const_block(&mut rust, "RAW_RADIUS", &tokens.raw.radius);
    push_const_block(&mut rust, "RAW_MOTION", &tokens.raw.motion);
    push_const_block(&mut rust, "RAW_BORDER", &tokens.raw.border);
    push_const_block(&mut rust, "SEMANTIC_SURFACE", &tokens.semantic.surface);
    push_const_block(&mut rust, "SEMANTIC_CONTROL", &tokens.semantic.control);
    push_const_block(&mut rust, "SEMANTIC_TEXT", &tokens.semantic.text);
    push_const_block(&mut rust, "SEMANTIC_BORDER", &tokens.semantic.border);
    push_const_block(&mut rust, "SEMANTIC_STATE", &tokens.semantic.state);
    push_const_block(&mut rust, "SEMANTIC_SHELL", &tokens.semantic.shell);
    push_const_block(&mut rust, "SEMANTIC_LAYER", &tokens.semantic.layer);

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    write_if_changed(&out_dir.join("origin_tokens_generated.rs"), &rust);

    let mut css =
        String::from("/* Generated from ui/crates/system_ui/tokens/tokens.toml */\n:root {\n");
    push_css_vars(&mut css, "raw-color", &tokens.raw.color);
    push_css_vars(&mut css, "raw-space", &tokens.raw.space);
    push_css_vars(&mut css, "raw-type", &tokens.raw.type_tokens);
    push_css_vars(&mut css, "raw-blur", &tokens.raw.blur);
    push_css_vars(&mut css, "raw-radius", &tokens.raw.radius);
    push_css_vars(&mut css, "raw-motion", &tokens.raw.motion);
    push_css_vars(&mut css, "raw-border", &tokens.raw.border);
    push_css_vars(&mut css, "semantic-surface", &tokens.semantic.surface);
    push_css_vars(&mut css, "semantic-control", &tokens.semantic.control);
    push_css_vars(&mut css, "semantic-text", &tokens.semantic.text);
    push_css_vars(&mut css, "semantic-border", &tokens.semantic.border);
    push_css_vars(&mut css, "semantic-state", &tokens.semantic.state);
    push_css_vars(&mut css, "semantic-shell", &tokens.semantic.shell);
    push_css_vars(&mut css, "semantic-layer", &tokens.semantic.layer);
    css.push_str("}\n");
    css.push_str(&format!(
        "\n:root[data-theme=\"dark\"],\n.desktop-shell[data-theme=\"dark\"] {{\n{}\n}}\n",
        tokens
            .theme
            .dark
            .iter()
            .map(|(key, value)| format!("  --origin-{key}: {value};"))
            .collect::<Vec<_>>()
            .join("\n")
    ));
    css.push_str(
        "\n:root[data-high-contrast=\"true\"],\n.desktop-shell[data-high-contrast=\"true\"] {\n  --origin-raw-color-canvas: #010101;\n  --origin-raw-color-desktop: #040608;\n  --origin-raw-color-text-primary: #ffffff;\n  --origin-raw-color-text-secondary: #f2f5f9;\n  --origin-raw-color-text-muted: #dde5ee;\n  --origin-raw-color-text-inverse: #020305;\n  --origin-semantic-border-standard: rgba(255, 255, 255, 0.72);\n  --origin-semantic-border-focus: #ffffff;\n  --origin-semantic-border-selected: #9ed1ff;\n  --origin-semantic-surface-taskbar-background: rgba(12, 18, 28, 0.96);\n  --origin-semantic-surface-window-active-background: rgba(16, 23, 34, 0.98);\n  --origin-semantic-surface-window-inactive-background: rgba(12, 18, 28, 0.94);\n  --origin-semantic-surface-menu-background: rgba(18, 24, 35, 0.985);\n  --origin-semantic-surface-modal-background: rgba(20, 28, 40, 0.992);\n  --origin-semantic-layer-embedded-shadow: none;\n  --origin-semantic-layer-raised-shadow: none;\n  --origin-semantic-layer-floating-shadow: none;\n  --origin-semantic-layer-modal-shadow: none;\n}\n",
    );
    css.push_str(
        "\n:root[data-reduced-motion=\"true\"],\n.desktop-shell[data-reduced-motion=\"true\"] {\n  --origin-raw-motion-duration-fast: 0ms;\n  --origin-raw-motion-duration-standard: 0ms;\n  --origin-raw-motion-duration-slow: 0ms;\n}\n",
    );

    let site_dir = manifest_dir.parent().expect("ui/crates").join("site");
    write_if_changed(&site_dir.join("src/generated/tokens.css"), &css);
    write_if_changed(
        &site_dir.join("src/generated/tailwind.css"),
        generated_tailwind_css(),
    );
    write_if_changed(&site_dir.join("tailwind.config.js"), tailwind_config());
}
