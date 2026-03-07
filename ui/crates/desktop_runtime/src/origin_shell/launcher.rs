use crate::{
    components,
    components::DesktopRuntimeContext,
    model::{DesktopState, WindowId},
};

pub fn focus_or_unminimize_window(
    runtime: DesktopRuntimeContext,
    desktop: &DesktopState,
    window_id: WindowId,
) {
    components::focus_or_unminimize_window(runtime, desktop, window_id);
}
