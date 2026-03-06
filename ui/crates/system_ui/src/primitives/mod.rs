//! Shared structural, shell, overlay, data-display, control, and layout primitives.

use leptos::ev::{FocusEvent, KeyboardEvent, MouseEvent};
use leptos::*;

use crate::{Icon, IconName, IconSize};

mod controls;
mod data_display;
mod layout;
mod navigation;
mod overlays;
mod shell;

pub use controls::{
    Button, CheckboxField, CircularProgress, ColorField, CompletionItem, CompletionList,
    FieldGroup, IconButton, KnobDial, ProgressBar, RangeField, SegmentedControl,
    SegmentedControlOption, SelectField, Switch, TextArea, TextField, ToggleRow,
};
pub use data_display::{
    Badge, Card, DataTable, ElevationLayer, EmptyState, Heading, InspectorGrid, ListSurface,
    OptionCard, Pane, PaneHeader, Panel, PreviewFrame, StatusBarItem, Surface, TerminalLine,
    TerminalPrompt, TerminalSurface, TerminalTranscript, Text, Tree, TreeItem,
};
pub use layout::{Cluster, Grid, SplitLayout, Stack};
pub use navigation::{
    DisclosurePanel, LauncherMenu, MenuBar, StatusBar, StepFlow, StepFlowActions, StepFlowHeader,
    StepFlowStep, Tab, TabList, ToolBar,
};
pub use overlays::{MenuItem, MenuSeparator, MenuSurface, Modal};
pub use shell::{
    AppShell, ClockButton, DesktopBackdrop, DesktopIconButton, DesktopIconGrid, DesktopRoot,
    DesktopWindowLayer, ResizeHandle, Taskbar, TaskbarButton, TaskbarOverflowButton,
    TaskbarSection, TrayButton, TrayList, WindowBody, WindowControlButton, WindowControls,
    WindowFrame, WindowTitle, WindowTitleBar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Semantic surface variants for structural primitives.
pub enum SurfaceVariant {
    /// Primary surface.
    #[default]
    Standard,
    /// Secondary or muted surface.
    Muted,
    /// Inset surface.
    Inset,
}

impl SurfaceVariant {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Muted => "muted",
            Self::Inset => "inset",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Semantic elevation levels for shared primitives.
pub enum Elevation {
    /// Flat surface.
    #[default]
    Flat,
    /// Raised surface.
    Raised,
    /// Overlay surface.
    Overlay,
    /// Inset surface.
    Inset,
    /// Pressed control surface.
    Pressed,
}

impl Elevation {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Raised => "raised",
            Self::Overlay => "overlay",
            Self::Inset => "inset",
            Self::Pressed => "pressed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared button variants.
pub enum ButtonVariant {
    /// Standard action button.
    #[default]
    Standard,
    /// Secondary neutral action button.
    Secondary,
    /// Primary emphasized action button.
    Primary,
    /// Segmented control option button.
    Segmented,
    /// Circular or icon-only action button.
    Icon,
    /// Quiet/toggle style button.
    Quiet,
    /// Accent/emphasized button.
    Accent,
    /// Danger/destructive button.
    Danger,
}

impl ButtonVariant {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Secondary => "secondary",
            Self::Primary => "primary",
            Self::Segmented => "segmented",
            Self::Icon => "icon",
            Self::Quiet => "quiet",
            Self::Accent => "accent",
            Self::Danger => "danger",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared button shape tokens.
pub enum ButtonShape {
    /// Standard rounded rectangle.
    #[default]
    Standard,
    /// Pill-shaped control.
    Pill,
    /// Circular control.
    Circle,
}

impl ButtonShape {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Pill => "pill",
            Self::Circle => "circle",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared button sizing tokens.
pub enum ButtonSize {
    /// Dense button.
    Sm,
    /// Default button.
    #[default]
    Md,
    /// Large button.
    Lg,
}

impl ButtonSize {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared input-field variants.
pub enum FieldVariant {
    /// Standard input.
    #[default]
    Standard,
    /// Inset/editor input.
    Inset,
}

impl FieldVariant {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Inset => "inset",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared text roles.
pub enum TextRole {
    /// Body text.
    #[default]
    Body,
    /// Label text.
    Label,
    /// Caption text.
    Caption,
    /// Title text.
    Title,
    /// Monospace/code text.
    Code,
}

impl TextRole {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Label => "label",
            Self::Caption => "caption",
            Self::Title => "title",
            Self::Code => "code",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared text tone.
pub enum TextTone {
    /// Primary text.
    #[default]
    Primary,
    /// Secondary text.
    Secondary,
    /// Accent text.
    Accent,
    /// Success/status tone.
    Success,
    /// Warning tone.
    Warning,
    /// Danger tone.
    Danger,
}

impl TextTone {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Accent => "accent",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared progress variants.
pub enum ProgressVariant {
    /// Standard progress indicator.
    #[default]
    Standard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared layout gap tokens.
pub enum LayoutGap {
    /// No gap.
    None,
    /// Small gap.
    Sm,
    /// Default gap.
    #[default]
    Md,
    /// Large gap.
    Lg,
}

impl LayoutGap {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared layout padding tokens.
pub enum LayoutPadding {
    /// No padding.
    None,
    /// Compact padding.
    Sm,
    /// Default padding.
    #[default]
    Md,
    /// Spacious padding.
    Lg,
}

impl LayoutPadding {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared layout alignment tokens.
pub enum LayoutAlign {
    /// Stretch/fill alignment.
    #[default]
    Stretch,
    /// Start alignment.
    Start,
    /// Center alignment.
    Center,
    /// End alignment.
    End,
}

impl LayoutAlign {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Stretch => "stretch",
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Shared layout justification tokens.
pub enum LayoutJustify {
    /// Start justification.
    #[default]
    Start,
    /// Center justification.
    Center,
    /// Space between items.
    Between,
    /// End justification.
    End,
}

impl LayoutJustify {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::Between => "between",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Shared guided-step status tokens.
pub enum StepStatus {
    /// Current active step.
    Current,
    /// Completed prior step.
    Complete,
    /// Pending future step.
    Pending,
    /// Step has a validation error.
    Error,
}

impl StepStatus {
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Complete => "complete",
            Self::Pending => "pending",
            Self::Error => "error",
        }
    }
}

pub(crate) fn merge_layout_class(base: &'static str, layout_class: Option<&'static str>) -> String {
    match layout_class {
        Some(layout_class) if !layout_class.is_empty() => format!("{base} {layout_class}"),
        _ => base.to_string(),
    }
}

pub(crate) fn bool_token(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
