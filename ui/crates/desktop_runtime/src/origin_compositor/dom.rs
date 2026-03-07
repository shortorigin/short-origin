use crate::model::WindowId;

pub fn window_dom_id(window_id: WindowId) -> String {
    format!("origin-window-{}", window_id.0)
}
