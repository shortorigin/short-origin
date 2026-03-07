//! Shared non-legacy UI style enums and helper utilities.

/// Semantic surface variants for structural primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Semantic elevation levels for shared primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared button variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared button shape tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared button sizing tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared input-field variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared text roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared text tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared layout gap tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared layout padding tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared layout alignment tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// Shared layout justification tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
