// Generated from ui/crates/system_ui/tokens/tokens.toml
module.exports = {
  content: ["./src/**/*.rs", "./src/**/*.html"],
  theme: {
    extend: {
      colors: {
        canvas: "var(--origin-color-canvas)",
        desktop: "var(--origin-color-desktop)",
        surface: "var(--origin-color-surface)",
        accent: "var(--origin-color-accent)",
        text: { primary: "var(--origin-color-text-primary)", secondary: "var(--origin-color-text-secondary)" },
      },
      spacing: {
        1: "var(--origin-space-1)",
        2: "var(--origin-space-2)",
        3: "var(--origin-space-3)",
        4: "var(--origin-space-4)",
        5: "var(--origin-space-5)",
        6: "var(--origin-space-6)",
        8: "var(--origin-space-8)",
      },
      borderRadius: {
        sm: "var(--origin-radius-sm)",
        md: "var(--origin-radius-md)",
        lg: "var(--origin-radius-lg)",
        xl: "var(--origin-radius-xl)",
      },
      boxShadow: {
        panel: "var(--origin-shadow-panel)",
        window: "var(--origin-shadow-window)",
        overlay: "var(--origin-shadow-overlay)",
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
  corePlugins: { preflight: false },
};
