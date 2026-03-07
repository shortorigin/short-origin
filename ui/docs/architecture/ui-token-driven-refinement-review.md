# UI Token-Driven Refinement Review

## Scope
- Workspace: `ui/`
- Primary surfaces reviewed and updated:
  - `ui/crates/system_ui/tokens/tokens.toml`
  - `ui/crates/system_ui/src/origin_tokens/schema.rs`
  - `ui/crates/system_ui/build.rs`
  - `ui/crates/site/src/styles/primitives.css`
  - `ui/crates/site/src/styles/components.css`
  - `ui/crates/site/src/styles/shell.css`
  - `ui/crates/desktop_runtime/src/components.rs`
  - `ui/crates/desktop_runtime/src/shell.rs`
  - `ui/crates/desktop_runtime/src/host/host_ui.rs`

## Findings
### Strengths
- The workspace already had the correct ownership hierarchy: token generation in `system_ui`, semantic primitives and components in Rust, and browser/desktop shell composition through shared UI contracts.
- The active site path in `ui/crates/site/src/web_app.rs` already loaded a centralized generated token stylesheet ahead of the authored CSS layers.
- `ui/crates/system_ui/src/foundation.rs` already exposed stable semantic enums for surfaces, elevation, text tone, text role, field variants, button variants, sizes, and shapes.

### Deficiencies
#### Layout
- Shell layout rhythm was partially tokenized but still relied on local spacing and width decisions in authored CSS, especially around menu widths, icon tile sizing, and taskbar group spacing.
- Window body layout did not consistently apply content padding or overflow rules from a shared shell metric contract.

#### Typography
- `tokens.toml` lacked label, section, display, and letter-spacing roles even though the shell needed clearer hierarchy for captions, labels, headings, menus, and titlebars.
- Primitive CSS implemented only a subset of the text role surface exposed by Rust.

#### Color and Surface Hierarchy
- The token set did not define semantic state surfaces such as inset, interactive, hover, pressed, and selected surfaces.
- Active CSS still used local RGBA literals for focus, selected, hover, and shell emphasis instead of token-backed semantic colors.
- Borders for selected/focused states were not first-class tokens.

#### Spacing and Rhythm
- The current spacing scale lacked a tighter step for compact menu/taskbar grouping and a broader section rhythm token for shell composition.
- Shell-specific content padding and icon tile sizing were not tokenized.

#### Interaction States
- `ui/crates/system_ui/src/foundation.rs` exposed `ButtonVariant::{Secondary, Segmented, Icon, Quiet, Accent, Danger}` and multiple size/shape states, but `ui/crates/site/src/styles/primitives.css` only implemented a narrow subset of those semantics.
- Hover, pressed, selected, and disabled states varied between buttons, menus, desktop icons, and shell controls.

#### Focus and Accessibility
- Focus styles existed, but the active authored CSS still treated focus as a mostly generic outline instead of a system-level semantic ring.
- Generated high-contrast and reduced-motion overrides were emitted only for `:root`, while runtime state attributes are applied on `.desktop-shell` in `ui/crates/desktop_runtime/src/components.rs`.

#### Component Consistency
- Toolbars, menus, taskbar sections, tray controls, and clock controls did not share a fully consistent container/group treatment.
- Menu grouping and item selection affordance were under-defined compared to taskbar and window controls.

#### Shell Chrome and Desktop Metaphors
- Window focus treatment and titlebar hierarchy were serviceable but visually under-expressed.
- Desktop icon buttons lacked robust selected/focus/readability treatment.
- Taskbar sections were structurally correct but looked like adjacent controls instead of one coordinated shell surface.

#### Architectural Debt and Legacy Drift
- `ui/crates/site/src/theme_shell/*` remained in the repository as a second styling authority even though the active runtime path now uses the generated token CSS plus `styles/*.css`.
- Desktop runtime geometry still used `38px` taskbar constants in `ui/crates/desktop_runtime/src/components.rs`, `ui/crates/desktop_runtime/src/shell.rs`, and `ui/crates/desktop_runtime/src/host/host_ui.rs`, while token CSS used `52px`.

## Strategy
- Expand `system_ui` tokens first so every new color, state surface, shell metric, and typography role has a centralized source of truth.
- Extend the token schema and generator to support letter-spacing and to emit accessibility overrides that match the runtime’s actual DOM attribute placement.
- Rebuild primitive CSS around the semantic API already exported by Rust rather than introducing new ad hoc variants.
- Keep component CSS composition-only: grouping, menu rhythm, taskbar sections, and shell-specific sizing should layer on top of primitive semantics.
- Refine shell CSS using token-backed shell metrics so browser preview and desktop runtime stay visually aligned.
- Remove legacy `theme_shell` files to avoid silent drift and leave one visual authority in the active code path.
- Replace duplicated runtime taskbar constants with shared `system_ui::tokens` constants so placement logic and CSS dimensions match.

## Implemented Changes
- Expanded token coverage for:
  - semantic surfaces: muted, inset, interactive, hover, pressed, selected
  - text tones and text-muted
  - focus and selected borders
  - shell content padding and desktop icon tile sizing
  - broader typography hierarchy including label, section, display, and letter spacing
  - shadow tokens for pressed and focus-ring states
- Updated the token build pipeline to generate new letter-spacing CSS/Rust exports and to scope accessibility overrides to both `:root` and `.desktop-shell`.
- Rewrote the active primitive CSS so the exported Rust semantic API is visually implemented for surfaces, text roles, field variants, and button variants/states.
- Rewrote component CSS to standardize menu surfaces, menu items, toolbars, status bars, taskbar sections, tray controls, and clock treatment through token-backed styling.
- Rewrote shell CSS to improve desktop backdrop composition, window focus hierarchy, titlebar clarity, icon readability, responsive shell behavior, and token-driven shell padding.
- Removed `ui/crates/site/src/theme_shell/*` so the active visual contract is now the generated token stylesheet plus the authored `styles/*.css` layers.
- Replaced hardcoded `38px` taskbar runtime constants with `system_ui::tokens::SHELL_TASKBAR_HEIGHT_PX`.

## Rationale
- The existing architecture was already correct; the primary problem was incomplete token coverage and drift between semantic Rust APIs, active CSS, and runtime geometry.
- Centralizing visual values in tokens makes future shell changes cheaper because button states, shell rhythm, and surface hierarchy now flow through one pipeline.
- Removing dormant theme files is a maintainability decision, not a visual one: it prevents a second authority from reintroducing inconsistent semantics later.
- Aligning runtime constants with generated tokens eliminates layout defects that appear when window placement, menu anchoring, and viewport calculations assume obsolete chrome dimensions.

## Validation Notes
- Token outputs should be regenerated through the `system_ui` build pipeline before final verification.
- Validation should include:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-targets`
  - focused UI crate validation if full workspace checks are expensive
- Visual validation targets:
  - default shell in browser/WASM preview
  - desktop runtime shell
  - high contrast
  - reduced motion
  - taskbar/menu/window selected and focus states
  - narrow viewport responsive shell behavior
