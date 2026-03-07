use crate::model::{DesktopState, WindowId};

pub fn modal_blocking_parent(state: &DesktopState, child: WindowId) -> Option<WindowId> {
    state
        .windows
        .iter()
        .find(|window| window.id == child)
        .and_then(|window| window.flags.modal_parent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use desktop_app_contract::ApplicationId;

    use crate::model::{DesktopTheme, WindowFlags, WindowRecord, WindowRect};

    #[test]
    fn returns_modal_parent_when_child_is_modal() {
        let parent_id = WindowId(1);
        let child_id = WindowId(2);
        let state = DesktopState {
            next_window_id: 3,
            windows: vec![
                WindowRecord {
                    id: parent_id,
                    app_id: ApplicationId::trusted("system.settings"),
                    title: "Settings".to_string(),
                    icon_id: "settings".to_string(),
                    rect: WindowRect::default(),
                    restore_rect: None,
                    z_index: 1,
                    is_focused: false,
                    minimized: false,
                    maximized: false,
                    suspended: false,
                    flags: WindowFlags::default(),
                    persist_key: None,
                    app_state: serde_json::Value::Null,
                    launch_params: serde_json::Value::Null,
                    last_lifecycle_event: None,
                },
                WindowRecord {
                    id: child_id,
                    app_id: ApplicationId::trusted("system.settings"),
                    title: "Confirm".to_string(),
                    icon_id: "settings".to_string(),
                    rect: WindowRect::default(),
                    restore_rect: None,
                    z_index: 2,
                    is_focused: true,
                    minimized: false,
                    maximized: false,
                    suspended: false,
                    flags: WindowFlags {
                        modal_parent: Some(parent_id),
                        ..WindowFlags::default()
                    },
                    persist_key: None,
                    app_state: serde_json::Value::Null,
                    launch_params: serde_json::Value::Null,
                    last_lifecycle_event: None,
                },
            ],
            start_menu_open: false,
            active_modal: Some(child_id),
            theme: DesktopTheme::default(),
            wallpaper: Default::default(),
            wallpaper_preview: None,
            wallpaper_library: Default::default(),
            preferences: Default::default(),
            last_explorer_path: None,
            last_notepad_slug: None,
            terminal_history: Vec::new(),
            app_shared_state: Default::default(),
            boot_hydrated: false,
        };

        assert_eq!(modal_blocking_parent(&state, child_id), Some(parent_id));
        assert_eq!(modal_blocking_parent(&state, parent_id), None);
    }
}
