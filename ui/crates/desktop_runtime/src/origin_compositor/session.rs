use crate::model::{PointerPosition, ResizeEdge, WindowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveCompositorSession {
    Move {
        window_id: WindowId,
        pointer_start: PointerPosition,
    },
    Resize {
        window_id: WindowId,
        edge: ResizeEdge,
        pointer_start: PointerPosition,
    },
}
