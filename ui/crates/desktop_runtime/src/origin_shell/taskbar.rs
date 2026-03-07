use desktop_app_contract::ApplicationId;

use crate::{
    components,
    model::{DesktopState, WindowId},
};

pub fn preferred_window_for_app(state: &DesktopState, app_id: &ApplicationId) -> Option<WindowId> {
    components::preferred_window_for_app(state, app_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DesktopTheme, WindowFlags, WindowRecord, WindowRect};

    #[test]
    fn prefers_focused_non_minimized_window_for_app() {
        let app_id = ApplicationId::trusted("system.terminal");
        let state = DesktopState {
            next_window_id: 3,
            windows: vec![
                WindowRecord {
                    id: WindowId(1),
                    app_id: app_id.clone(),
                    title: "Terminal A".to_string(),
                    icon_id: "terminal".to_string(),
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
                    id: WindowId(2),
                    app_id: app_id.clone(),
                    title: "Terminal B".to_string(),
                    icon_id: "terminal".to_string(),
                    rect: WindowRect::default(),
                    restore_rect: None,
                    z_index: 2,
                    is_focused: true,
                    minimized: false,
                    maximized: false,
                    suspended: false,
                    flags: WindowFlags::default(),
                    persist_key: None,
                    app_state: serde_json::Value::Null,
                    launch_params: serde_json::Value::Null,
                    last_lifecycle_event: None,
                },
            ],
            start_menu_open: false,
            active_modal: None,
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

        assert_eq!(preferred_window_for_app(&state, &app_id), Some(WindowId(2)));
    }
}
