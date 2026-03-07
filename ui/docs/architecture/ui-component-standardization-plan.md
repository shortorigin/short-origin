# UI Component Standardization Plan

## Standardization Decisions
- Keep `system_ui` as the shared component vocabulary.
- Use shell CSS to style `data-ui-*` primitives consistently instead of adding app-local control variants.
- Treat window chrome, taskbar, menus, and field surfaces as part of the same design system.

## Structural Fixes Applied
- Window frames now respect viewport bounds.
- Window bodies and content regions scroll instead of clipping.
- Responsive shell layout turns window layers into stacked flow on narrow viewports.
- Taskbar and tray strips can overflow horizontally instead of breaking layout.

## Remaining Standardization Targets
- Further split `desktop_runtime::components` into smaller shell/layout modules.
- Continue replacing app-local layout/styling fragments with `system_ui` primitives.
