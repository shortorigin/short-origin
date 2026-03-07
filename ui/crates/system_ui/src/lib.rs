//! Shared UI package for the Origin desktop shell.
//!
//! The public architecture is organized as:
//!
//! - [`tokens`] for machine-generated design tokens
//! - [`primitives`] for low-level layout, typography, and shell regions
//! - [`components`] for approved reusable application and shell components

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(missing_docs)]

mod foundation;
mod icon;
mod origin_components;
mod origin_primitives;
mod origin_tokens;

pub mod components {
    pub use crate::origin_components::*;
}

pub mod primitives {
    pub use crate::origin_primitives::*;
}

pub mod tokens {
    pub use crate::origin_tokens::*;
}

/// Convenience imports for application crates consuming the shared primitive set.
///
/// Prefer importing from this module in app crates so primitive usage stays consistent and review
/// diffs do not churn on long individual import lists.
pub mod prelude {
    pub use crate::components::StepStatus;
    pub use crate::components::{
        AppShell, Button, DisclosurePanel, IconButton, StatusBar, StatusBarItem, StepFlow,
        StepFlowActions, StepFlowHeader, StepFlowStep, ToggleRow, WindowControls, WindowFrame,
        WindowTitleBar,
    };
    pub use crate::primitives::{
        ButtonShape, ButtonSize, ButtonVariant, Center, CheckboxField, Cluster, Elevation,
        FieldVariant, Grid, Heading, Icon, IconName, IconSize, Inline, Inset, Layer, LayoutAlign,
        LayoutGap, LayoutJustify, LayoutPadding, ListSurface, Panel, ResizeHandleRegion, Stack,
        Surface, SurfaceVariant, TerminalLine, TerminalPrompt, TerminalSurface, TerminalTranscript,
        Text, TextField, TextRole, TextTone, TitlebarRegion, Viewport, WindowBody,
        WindowControlButton, WindowSurface, WindowTitle,
    };
    pub use crate::tokens::baseline_style_id;
}
