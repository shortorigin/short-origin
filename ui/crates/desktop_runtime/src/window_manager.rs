//! Shared window-manager transition helpers used by the desktop reducer.

use crate::model::{DesktopState, ResizeEdge, WindowId, WindowRect};

/// Minimum allowed managed window width.
pub const MIN_WINDOW_WIDTH: i32 = 320;
/// Minimum allowed managed window height.
pub const MIN_WINDOW_HEIGHT: i32 = 220;
/// Pointer threshold (in px) for snap-edge behavior.
pub const SNAP_EDGE_THRESHOLD: i32 = 24;

/// Focuses and raises `window_id`, ensuring it is the top/focused non-minimized window.
///
/// Returns `true` when stack/focus state changed.
pub fn focus_window_internal(state: &mut DesktopState, window_id: WindowId) -> bool {
    let Some(index) = state.windows.iter().position(|w| w.id == window_id) else {
        return false;
    };

    let already_focused_top = index + 1 == state.windows.len()
        && state
            .windows
            .get(index)
            .map(|w| w.is_focused && !w.minimized)
            .unwrap_or(false);
    if already_focused_top {
        return true;
    }

    for window in &mut state.windows {
        window.is_focused = false;
    }
    let mut window = state.windows.remove(index);
    window.is_focused = true;
    window.minimized = false;
    window.suspended = false;
    state.windows.push(window);
    normalize_window_stack(state);
    true
}

/// Normalizes z-index ordering and focus invariants for all managed windows.
pub fn normalize_window_stack(state: &mut DesktopState) {
    let mut has_focused = false;
    for (idx, window) in state.windows.iter_mut().enumerate() {
        window.z_index = (idx + 1) as u32;
        if window.minimized {
            window.is_focused = false;
        }
        if window.is_focused {
            if has_focused {
                window.is_focused = false;
            } else {
                has_focused = true;
            }
        }
    }

    if !has_focused {
        if let Some(last_non_minimized) = state.windows.iter_mut().rev().find(|w| !w.minimized) {
            last_non_minimized.is_focused = true;
        }
    }
}

/// Applies resize deltas for a given edge/corner drag.
pub fn resize_rect(start: WindowRect, edge: ResizeEdge, dx: i32, dy: i32) -> WindowRect {
    match edge {
        ResizeEdge::East => WindowRect {
            w: start.w + dx,
            ..start
        },
        ResizeEdge::West => WindowRect {
            x: start.x + dx,
            w: start.w - dx,
            ..start
        },
        ResizeEdge::South => WindowRect {
            h: start.h + dy,
            ..start
        },
        ResizeEdge::North => WindowRect {
            y: start.y + dy,
            h: start.h - dy,
            ..start
        },
        ResizeEdge::NorthEast => WindowRect {
            y: start.y + dy,
            h: start.h - dy,
            w: start.w + dx,
            ..start
        },
        ResizeEdge::NorthWest => WindowRect {
            x: start.x + dx,
            y: start.y + dy,
            w: start.w - dx,
            h: start.h - dy,
        },
        ResizeEdge::SouthEast => WindowRect {
            w: start.w + dx,
            h: start.h + dy,
            ..start
        },
        ResizeEdge::SouthWest => WindowRect {
            x: start.x + dx,
            w: start.w - dx,
            h: start.h + dy,
            ..start
        },
    }
}

/// Applies edge snap/maximize behavior for a dragged window and returns whether a snap was applied.
pub fn snap_window_to_viewport_edge(
    state: &mut DesktopState,
    window_id: WindowId,
    viewport: WindowRect,
) -> bool {
    let Some(window) = state.windows.iter_mut().find(|w| w.id == window_id) else {
        return false;
    };

    if window.minimized {
        return false;
    }

    let near_left = window.rect.x <= viewport.x + SNAP_EDGE_THRESHOLD;
    let near_right = window.rect.x + window.rect.w >= viewport.x + viewport.w - SNAP_EDGE_THRESHOLD;
    let near_top = window.rect.y <= viewport.y + SNAP_EDGE_THRESHOLD;

    if near_top && window.flags.maximizable {
        if !window.maximized {
            window.restore_rect = Some(window.rect);
        }
        window.rect = viewport.clamped_min(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
        window.maximized = true;
        window.minimized = false;
        window.suspended = false;
        return true;
    }

    if !(near_left || near_right) || !window.flags.resizable {
        return false;
    }

    let half_width = (viewport.w / 2).max(MIN_WINDOW_WIDTH);
    let snapped = WindowRect {
        x: if near_right {
            viewport.x + viewport.w - half_width
        } else {
            viewport.x
        },
        y: viewport.y,
        w: half_width,
        h: viewport.h.max(MIN_WINDOW_HEIGHT),
    };

    window.restore_rect = Some(window.rect);
    window.rect = snapped;
    window.maximized = false;
    window.minimized = false;
    window.suspended = false;
    true
}
