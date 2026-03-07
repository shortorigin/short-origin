# UI Design System Plan

## Design Principles
- One authoritative shell language: modern adaptive.
- Shared primitives own control semantics and states.
- Browser layouts must degrade into responsive panels rather than broken floating-desktop composition.

## Visual Token Model
- Dark system-oriented canvas and desktop surfaces.
- High-contrast text and accent tokens.
- Shared spacing, radius, elevation, and focus tokens remain centralized in `ui/crates/system_ui/tokens/tokens.toml` and its generated outputs.

## Component Vocabulary
- `system_ui` primitives remain the source of truth for buttons, menus, fields, taskbar, tray, window frame, panels, and layout primitives.
- Shell CSS maps those primitives to one visual language instead of several competing theme packs.

## Shell / App Consistency Rules
- Shell chrome and app surfaces must use the same base surface/border/elevation language.
- Window bodies and panel content must support scrolling instead of clipping.
- Responsive rules can change layout, but not component semantics.

## Responsive Behavior Rules
- Wide viewports may retain multiple shell surfaces and floating windows.
- Narrower viewports collapse windows into stacked flow and disable resize handles.
- Taskbar strips and tray regions may scroll horizontally instead of overflowing.

## Interaction State Rules
- Focus-visible remains explicit.
- Selected, pressed, disabled, and hover states remain token-driven through shared primitives.
- Cross-tab state rehydration must not trigger destructive resets.

## Accessibility Rules
- Maintain explicit focus indicators.
- Keep text/surface contrast above the prior baseline.
- Prefer scrollable content over clipped content.

## Anti-Patterns To Remove
- Multiple active theme packs controlling the same surfaces.
- Browser-only behavior hidden inside desktop-shaped boot logic.
- Local styling patches that bypass shared tokens for shell primitives.
