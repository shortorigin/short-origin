# Dark Translucent Material System

## Summary
- Authority remains in `ui/crates/system_ui`.
- Visual propagation is token file -> generated CSS/custom properties -> generated Tailwind projection -> Rust primitives -> shared components -> app layer.
- Generated `ui/crates/site/src/generated/tailwind.css` is the active browser styling baseline.

## Token Taxonomy
- `color`: canvas, desktop, text, accent, semantic states, focus, selection.
- `material`: tint, translucency, diffusion, reflection, overlay density.
- `surface`: background, border, and highlight tracks for base, raised, overlay, modal, and control glass.
- `blur`: low, medium, high, modal, fallback.
- `elevation`: alpha, border, shadow, and blur scales for `shell`, `embedded`, `raised`, `floating`, `modal`, and `transient`.
- `spacing`, `radius`, `border`, `opacity`, `motion`, `state`, `icon`, `shell`, typography.

## Material Variants
- `standard`: base glass for primary content panes.
- `muted`: raised glass for cards, toolbars, and stacked content.
- `inset`: denser glass for editor wells and terminal/input regions.
- `overlay`: floating glass for taskbar and popovers.
- `modal`: deepest glass for windows and modal layers.
- `control`: compact glass for buttons and interactive chrome.

## Elevation Model
- `flat`: no promoted material treatment.
- `embedded`: low blur, low shadow, primary content depth.
- `raised`: stronger shadow and highlight separation for cards/toolbars.
- `overlay`: taskbar and anchored shell overlays.
- `modal`: windows and highest-focus containers.
- `transient`: menus and ephemeral floating surfaces.
- `inset`: internal wells and depressed controls.
- `pressed`: active control response only.

## Motion Rules
- Default transitions use tokenized duration/easing.
- Hover promotes via border/highlight/background shifts before large depth changes.
- Active state compresses through pressed shadow and background change, not large movement.
- Overlay and modal surfaces own meaningful blur changes; controls do not animate blur.
- Reduced-motion mode zeroes all durations via tokens rather than conditional component logic.

## Contextual Adaptation
- Buttons and fields inherit denser surfaces when placed in inset or overlay containers through token-backed control surfaces.
- Menu, taskbar, and window chrome use overlay/modal surface tracks rather than ad hoc local recipes.
- Focus visibility always routes through tokenized border and focus-ring values.

## Prohibited Patterns
- No hardcoded colors, blur radii, shadows, spacing, or transition literals in app-layer Rust.
- No parallel authored theme system outside `system_ui` token generation.
- No component-local glass recipes in app crates.
- No browser-specific visual fixes that bypass token fallbacks.

## Fallback Behavior
- `@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)))` disables backdrop blur and falls back to denser opaque token surfaces.
- High-contrast and reduced-motion overrides are generated into `tokens.css` and remain active for both `:root` and `.desktop-shell`.
