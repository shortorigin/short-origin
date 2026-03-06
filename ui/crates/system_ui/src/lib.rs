//! Shared UI primitive library for shell and built-in system applications.
//!
//! The crate owns reusable Leptos primitives, a centralized icon API, and the
//! stable `data-ui-*` DOM contract consumed by the desktop shell CSS layers.
//! Apps should compose these primitives instead of emitting ad hoc control
//! markup or reusing legacy `.app-*` class contracts directly.
//!
//! Theme CSS remaps shared `--sys-*` tokens around these primitives, while docs validation checks
//! that app/runtime crates consume the shared API instead of recreating raw primitive markup.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod icon;
mod primitives;

pub use icon::{Icon, IconName, IconSize};
pub use primitives::{
    AppShell, Badge, Button, ButtonShape, ButtonSize, ButtonVariant, Card, CheckboxField,
    CircularProgress, ClockButton, Cluster, ColorField, CompletionItem, CompletionList, DataTable,
    DesktopBackdrop, DesktopIconButton, DesktopIconGrid, DesktopRoot, DesktopWindowLayer,
    DisclosurePanel, Elevation, ElevationLayer, EmptyState, FieldGroup, FieldVariant, Grid,
    Heading, IconButton, InspectorGrid, KnobDial, LauncherMenu, LayoutAlign, LayoutGap,
    LayoutJustify, LayoutPadding, ListSurface, MenuBar, MenuItem, MenuSeparator, MenuSurface,
    Modal, OptionCard, Pane, PaneHeader, Panel, PreviewFrame, ProgressBar, ProgressVariant,
    RangeField, ResizeHandle, SegmentedControl, SegmentedControlOption, SelectField, SplitLayout,
    Stack, StatusBar, StatusBarItem, StepFlow, StepFlowActions, StepFlowHeader, StepFlowStep,
    StepStatus, Surface, SurfaceVariant, Switch, Tab, TabList, Taskbar, TaskbarButton,
    TaskbarOverflowButton, TaskbarSection, TerminalLine, TerminalPrompt, TerminalSurface,
    TerminalTranscript, Text, TextArea, TextField, TextRole, TextTone, ToggleRow, ToolBar,
    TrayButton, TrayList, Tree, TreeItem, WindowBody, WindowControlButton, WindowControls,
    WindowFrame, WindowTitle, WindowTitleBar,
};

/// Convenience imports for application crates consuming the shared primitive set.
///
/// Prefer importing from this module in app crates so primitive usage stays consistent and review
/// diffs do not churn on long individual import lists.
pub mod prelude {
    pub use crate::{
        AppShell, Badge, Button, ButtonShape, ButtonSize, ButtonVariant, Card, CheckboxField,
        CircularProgress, ClockButton, Cluster, ColorField, CompletionItem, CompletionList,
        DataTable, DesktopBackdrop, DesktopIconButton, DesktopIconGrid, DesktopRoot,
        DesktopWindowLayer, DisclosurePanel, Elevation, ElevationLayer, EmptyState, FieldGroup,
        FieldVariant, Grid, Heading, Icon, IconButton, IconName, IconSize, InspectorGrid, KnobDial,
        LauncherMenu, LayoutAlign, LayoutGap, LayoutJustify, LayoutPadding, ListSurface, MenuBar,
        MenuItem, MenuSeparator, MenuSurface, Modal, OptionCard, Pane, PaneHeader, Panel,
        PreviewFrame, ProgressBar, ProgressVariant, RangeField, ResizeHandle, SegmentedControl,
        SegmentedControlOption, SelectField, SplitLayout, Stack, StatusBar, StatusBarItem,
        StepFlow, StepFlowActions, StepFlowHeader, StepFlowStep, StepStatus, Surface,
        SurfaceVariant, Switch, Tab, TabList, Taskbar, TaskbarButton, TaskbarOverflowButton,
        TaskbarSection, TerminalLine, TerminalPrompt, TerminalSurface, TerminalTranscript, Text,
        TextArea, TextField, TextRole, TextTone, ToggleRow, ToolBar, TrayButton, TrayList, Tree,
        TreeItem, WindowBody, WindowControlButton, WindowControls, WindowFrame, WindowTitle,
        WindowTitleBar,
    };
}
