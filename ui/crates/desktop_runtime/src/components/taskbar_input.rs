//! Internal taskbar keyboard shortcut helpers shared by global and local handlers.

use leptos::*;

use super::{
    activate_taskbar_shortcut_target, build_taskbar_shortcut_targets, DesktopAction,
    DesktopRuntimeContext, TaskbarWindowContextMenuState,
};

fn shortcut_digit_index(ev: &web_sys::KeyboardEvent) -> Option<usize> {
    match ev.key().as_str() {
        "1" => Some(0),
        "2" => Some(1),
        "3" => Some(2),
        "4" => Some(3),
        "5" => Some(4),
        "6" => Some(5),
        "7" => Some(6),
        "8" => Some(7),
        "9" => Some(8),
        _ => None,
    }
}

/// Returns whether the keyboard event should open a window context menu.
pub(super) fn is_context_menu_shortcut(ev: &web_sys::KeyboardEvent) -> bool {
    ev.key() == "ContextMenu" || (ev.shift_key() && ev.key() == "F10")
}

/// Returns whether the keyboard event should activate the selected taskbar window.
pub(super) fn is_activation_key(ev: &web_sys::KeyboardEvent) -> bool {
    matches!(ev.key().as_str(), "Enter" | " " | "Spacebar")
}

fn dismiss_taskbar_overlay_menus(
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    overflow_menu_open: RwSignal<bool>,
    clock_menu_open: RwSignal<bool>,
) {
    window_context_menu.set(None);
    overflow_menu_open.set(false);
    clock_menu_open.set(false);
}

/// Handles taskbar-global shortcuts shared by window-level and taskbar-local key handlers.
pub(super) fn try_handle_taskbar_shortcuts(
    runtime: DesktopRuntimeContext,
    window_context_menu: RwSignal<Option<TaskbarWindowContextMenuState>>,
    overflow_menu_open: RwSignal<bool>,
    clock_menu_open: RwSignal<bool>,
    ev: &web_sys::KeyboardEvent,
) -> bool {
    if ev.ctrl_key() && !ev.alt_key() && !ev.meta_key() && ev.key() == "Escape" {
        ev.prevent_default();
        ev.stop_propagation();
        dismiss_taskbar_overlay_menus(window_context_menu, overflow_menu_open, clock_menu_open);
        runtime.dispatch_action(DesktopAction::ToggleStartMenu);
        return true;
    }

    if ev.alt_key() && !ev.ctrl_key() && !ev.meta_key() {
        if let Some(index) = shortcut_digit_index(ev) {
            let desktop = runtime.state.get_untracked();
            if let Some(target) = build_taskbar_shortcut_targets(&desktop)
                .into_iter()
                .nth(index)
            {
                ev.prevent_default();
                ev.stop_propagation();
                dismiss_taskbar_overlay_menus(
                    window_context_menu,
                    overflow_menu_open,
                    clock_menu_open,
                );
                runtime.dispatch_action(DesktopAction::CloseStartMenu);
                activate_taskbar_shortcut_target(runtime, target);
            }
            return true;
        }
    }

    if ev.key() == "Escape"
        && (runtime.state.get_untracked().start_menu_open
            || window_context_menu.get_untracked().is_some()
            || overflow_menu_open.get_untracked()
            || clock_menu_open.get_untracked())
    {
        ev.prevent_default();
        ev.stop_propagation();
        dismiss_taskbar_overlay_menus(window_context_menu, overflow_menu_open, clock_menu_open);
        runtime.dispatch_action(DesktopAction::CloseStartMenu);
        return true;
    }

    false
}
