# Dark Translucent Remediation Report

## Retained
- `system_ui` remains the sole design-system owner.
- Token generation continues to emit Rust constants plus browser-consumed generated assets.
- Shared Rust primitives/components remain the consumption path for built-in apps and shell runtime.

## Refactored
- `ui/crates/system_ui/tokens/tokens.toml` now uses a material taxonomy instead of the prior flat surface set.
- `ui/crates/system_ui/build.rs` now emits richer token constants, CSS custom properties, a broader Tailwind config projection, and the generated Tailwind styling baseline.
- Shared primitives/components now carry explicit material and depth attributes into the DOM contract.
- Browser bundle styling authority moved into `ui/crates/site/src/generated/tailwind.css`.

## Removed Or Replaced
- Replaced the placeholder Tailwind output with a generated material styling layer.
- Replaced selector-heavy authored primitive/component/shell CSS with thin placeholder files so they no longer act as competing theme authorities.
- Replaced the flatter panel/window/taskbar recipes with token-driven translucent glass variants.

## Normalization Notes
- App-layer crates continue to consume shared components and no longer need local visual recipes for the shell baseline.
- Backdrop-filter fallback behavior is now token-governed instead of component-specific.
- Accessibility overrides remain centralized in generated token CSS.
