//! Window manager namespace exposing the reducer-driven desktop model.

pub mod geometry;
pub mod modality;
pub mod reducer;
pub mod stack;
pub mod workspace;

pub use crate::model::*;
pub use crate::reducer::{reduce_desktop, DesktopAction, ReducerError, RuntimeEffect};
pub use crate::window_manager::{
    focus_window_internal, normalize_window_stack, resize_rect, snap_window_to_viewport_edge,
    MIN_WINDOW_HEIGHT, MIN_WINDOW_WIDTH, SNAP_EDGE_THRESHOLD,
};
