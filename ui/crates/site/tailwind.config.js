// Generated from ui/crates/system_ui/tokens/tokens.toml
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
