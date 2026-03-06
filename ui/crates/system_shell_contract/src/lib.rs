//! Shared shell command contracts used by the headless shell engine, runtime integration, and
//! terminal UI.
//!
//! This crate is intentionally runtime-agnostic. It defines serializable command metadata,
//! structured execution requests, completion payloads, parser outputs, and stream events without
//! depending on Leptos, browser APIs, or desktop runtime internals.
//!
//! Consumers include the shared shell engine, runtime-side command registration surfaces, and the
//! terminal UI that renders typed notices, progress updates, and structured data.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use serde::{Deserialize, Serialize};

/// Stable command registration identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandId(String);

impl CommandId {
    /// Creates a command identifier from trusted caller input.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Returns the identifier text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One segment in a hierarchical command path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CommandSegment(String);

impl CommandSegment {
    /// Creates a segment from trusted caller input.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Returns the raw segment text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CommandSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable command path such as `theme set skin`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandPath {
    segments: Vec<CommandSegment>,
}

impl CommandPath {
    /// Creates a command path from a display string.
    pub fn new(raw: impl AsRef<str>) -> Self {
        Self::from_segments(
            raw.as_ref()
                .split_whitespace()
                .filter(|segment| !segment.is_empty())
                .map(CommandSegment::new),
        )
    }

    /// Creates a command path from explicit segments.
    pub fn from_segments(segments: impl IntoIterator<Item = CommandSegment>) -> Self {
        Self {
            segments: segments.into_iter().collect(),
        }
    }

    /// Returns the path segments.
    pub fn segments(&self) -> &[CommandSegment] {
        &self.segments
    }

    /// Returns the segment count.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` when the path has no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the parent path when one exists.
    pub fn parent(&self) -> Option<Self> {
        (self.segments.len() > 1).then(|| Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        })
    }

    /// Returns the space-delimited display string.
    pub fn display(&self) -> String {
        self.segments
            .iter()
            .map(CommandSegment::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns `true` when this path starts with `other`.
    pub fn starts_with(&self, other: &Self) -> bool {
        self.segments.starts_with(other.segments())
    }
}

impl std::fmt::Display for CommandPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display())
    }
}

/// Identifies how a command is meant to be presented in help and discovery surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandInteractionKind {
    /// Familiar single-word shell verb such as `ls`.
    RootVerb,
    /// Hierarchical namespaced command such as `theme set skin`.
    Hierarchical,
}

/// Visibility policy for registered commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandVisibility {
    /// Command is listed in help and completion.
    Public,
    /// Command is callable but omitted from normal listings.
    Hidden,
}

/// Registry scope for a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum CommandScope {
    /// Runtime-owned or globally visible command.
    Global,
    /// Commands exposed by an application package.
    App {
        /// Canonical application identifier.
        app_id: String,
    },
    /// Commands exposed only for a specific window instance.
    Window {
        /// Stable runtime window identifier.
        window_id: u64,
    },
}

/// Describes the shape of piped input or command output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandDataShape {
    /// No input or output value.
    Empty,
    /// Any structured value.
    Any,
    /// Scalar values such as strings, numbers, booleans, or null.
    Scalar,
    /// Structured record object.
    Record,
    /// Structured list.
    List,
    /// Structured table with row/column semantics.
    Table,
}

/// Piped input contract for a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandInputShape {
    /// Whether piped input is accepted.
    pub accepts_pipeline_input: bool,
    /// Input data shape when piped input is accepted.
    pub shape: CommandDataShape,
}

impl CommandInputShape {
    /// Returns a command input contract that rejects pipeline input.
    pub fn none() -> Self {
        Self {
            accepts_pipeline_input: false,
            shape: CommandDataShape::Empty,
        }
    }

    /// Returns a command input contract for the provided shape.
    pub fn accepts(shape: CommandDataShape) -> Self {
        Self {
            accepts_pipeline_input: true,
            shape,
        }
    }
}

/// Output contract for a command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOutputShape {
    /// Expected output data shape.
    pub shape: CommandDataShape,
}

impl CommandOutputShape {
    /// Creates a new output shape.
    pub fn new(shape: CommandDataShape) -> Self {
        Self { shape }
    }
}

/// Positional argument specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandArgSpec {
    /// Human-readable argument label.
    pub name: String,
    /// Short description.
    pub summary: String,
    /// Whether this argument is required.
    pub required: bool,
    /// Whether this argument consumes remaining values.
    pub repeatable: bool,
}

/// Named option or flag specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandOptionSpec {
    /// Long option name without leading `--`.
    pub name: String,
    /// Optional short option name without leading `-`.
    pub short: Option<char>,
    /// Short description.
    pub summary: String,
    /// Whether the option consumes a value.
    pub takes_value: bool,
}

/// Example invocation rendered in help output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandExample {
    /// Example command line.
    pub command: String,
    /// Example explanation.
    pub summary: String,
}

/// Complete help metadata for a registered command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpDoc {
    /// Summary sentence.
    pub summary: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Usage string displayed in help output.
    pub usage: String,
    /// Example invocations.
    pub examples: Vec<CommandExample>,
}

/// Full command registration metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandDescriptor {
    /// Stable command identifier.
    pub id: CommandId,
    /// Canonical command path.
    pub path: CommandPath,
    /// Cached parent path for help-tree generation.
    pub parent_path: Option<CommandPath>,
    /// Alternate command strings, such as `clear`.
    pub aliases: Vec<String>,
    /// Registration scope.
    pub scope: CommandScope,
    /// Visibility policy.
    pub visibility: CommandVisibility,
    /// Interaction model for display and help layout.
    pub interaction_kind: CommandInteractionKind,
    /// Whether this node should be treated as a discoverable namespace when invoked directly.
    pub discoverable_children: bool,
    /// Piped input contract.
    pub input_shape: CommandInputShape,
    /// Output contract.
    pub output_shape: CommandOutputShape,
    /// Positional argument metadata.
    pub args: Vec<CommandArgSpec>,
    /// Option metadata.
    pub options: Vec<CommandOptionSpec>,
    /// Help metadata.
    pub help: HelpDoc,
}

/// Completion request payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Current cwd for the shell session.
    pub cwd: String,
    /// Full input line.
    pub line: String,
    /// Parsed argv tokens before the active cursor position.
    pub argv: Vec<String>,
    /// Cursor offset within `line`.
    pub cursor: usize,
    /// Optional source window identifier.
    pub source_window_id: Option<u64>,
}

/// One completion candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Text inserted into the input line.
    pub value: String,
    /// Human-readable label.
    pub label: String,
    /// Optional short description.
    pub detail: Option<String>,
}

/// Shell execution request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellRequest {
    /// Input line to parse and execute.
    pub line: String,
    /// Current logical cwd.
    pub cwd: String,
    /// Optional source window identifier.
    pub source_window_id: Option<u64>,
}

/// Typed literal parsed from shell input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum ParsedLiteral {
    /// Null literal.
    Null,
    /// Boolean literal.
    Bool(bool),
    /// Integer literal.
    Int(i64),
    /// Float literal.
    Float(f64),
    /// String or bareword literal.
    String(String),
}

/// Parsed value token from shell input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedValue {
    /// Original token text.
    pub raw: String,
    /// Parsed literal interpretation.
    pub literal: ParsedLiteral,
}

/// Parsed option from shell input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedOption {
    /// Canonical option name without leading dashes.
    pub name: String,
    /// Optional short flag when parsed from `-x`.
    pub short: Option<char>,
    /// Optional parsed option value.
    pub value: Option<ParsedValue>,
}

/// Parsed invocation from shell input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedInvocation {
    /// Raw tokens making up this invocation.
    pub tokens: Vec<String>,
    /// Parsed options.
    pub options: Vec<ParsedOption>,
    /// Parsed positional values.
    pub values: Vec<ParsedValue>,
}

/// Parsed shell line including all pipeline stages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedCommandLine {
    /// Ordered pipeline stages.
    pub pipeline: Vec<ParsedInvocation>,
}

/// Primitive scalar value stored in terminal data flows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum StructuredScalar {
    /// Null value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// UTF-8 string value.
    String(String),
}

/// Ordered key/value pair in a structured record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredField {
    /// Stable field name.
    pub name: String,
    /// Field value.
    pub value: StructuredValue,
}

/// Ordered record value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StructuredRecord {
    /// Ordered fields.
    pub fields: Vec<StructuredField>,
}

/// Recursively typed value stored in terminal data flows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum StructuredValue {
    /// Scalar value.
    Scalar(StructuredScalar),
    /// Ordered record value.
    Record(StructuredRecord),
    /// Ordered list value.
    List(Vec<StructuredValue>),
}

/// Optional field schema metadata for records and tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredSchemaField {
    /// Field name.
    pub name: String,
    /// Declared data shape.
    pub shape: CommandDataShape,
}

/// Schema metadata for structured results when known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StructuredSchema {
    /// Ordered fields.
    pub fields: Vec<StructuredSchemaField>,
}

/// Table-oriented result payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StructuredTable {
    /// Ordered column names.
    pub columns: Vec<String>,
    /// Ordered row values.
    pub rows: Vec<StructuredRecord>,
    /// Optional schema metadata.
    pub schema: Option<StructuredSchema>,
    /// Optional source command path.
    pub source_command: Option<CommandPath>,
    /// Optional plain-text fallback rendering.
    pub fallback_text: Option<String>,
}

/// Top-level structured data value emitted by commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum StructuredData {
    /// No value.
    Empty,
    /// Scalar or recursive typed value.
    Value(StructuredValue),
    /// Ordered record payload.
    Record(StructuredRecord),
    /// Ordered list payload.
    List(Vec<StructuredValue>),
    /// Table payload.
    Table(StructuredTable),
}

impl StructuredData {
    /// Returns the coarse data shape.
    pub fn shape(&self) -> CommandDataShape {
        match self {
            Self::Empty => CommandDataShape::Empty,
            Self::Value(StructuredValue::Scalar(_)) => CommandDataShape::Scalar,
            Self::Value(StructuredValue::Record(_)) | Self::Record(_) => CommandDataShape::Record,
            Self::Value(StructuredValue::List(_)) | Self::List(_) => CommandDataShape::List,
            Self::Table(_) => CommandDataShape::Table,
        }
    }
}

/// Preferred transcript presentation for a structured command result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayPreference {
    /// Let the terminal choose based on the data shape.
    Auto,
    /// Render as contextual help.
    Help,
    /// Render as a scalar value.
    Value,
    /// Render as a record view.
    Record,
    /// Render as a table view.
    Table,
}

/// Severity for command notices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandNoticeLevel {
    /// Informational notice.
    Info,
    /// Warning notice.
    Warning,
    /// Error notice.
    Error,
}

/// Human-readable notice emitted alongside structured output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandNotice {
    /// Notice severity.
    pub level: CommandNoticeLevel,
    /// User-facing message.
    pub message: String,
}

/// Final command result payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandResult {
    /// Final structured output.
    pub output: StructuredData,
    /// Preferred transcript presentation.
    pub display: DisplayPreference,
    /// Supplemental notices emitted alongside the output.
    pub notices: Vec<CommandNotice>,
    /// Optional cwd update for the owning session.
    pub cwd: Option<String>,
    /// Process-style exit metadata.
    pub exit: ShellExit,
}

impl CommandResult {
    /// Creates a result with the provided output and a successful exit status.
    pub fn success(output: StructuredData) -> Self {
        Self {
            display: DisplayPreference::Auto,
            notices: Vec::new(),
            cwd: None,
            output,
            exit: ShellExit::success(),
        }
    }
}

/// Final execution result metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExecutionSummary {
    /// Execution identifier.
    pub execution_id: ExecutionId,
    /// Canonical command path if one was matched.
    pub command_path: Option<CommandPath>,
    /// Process-style exit metadata.
    pub exit: ShellExit,
}

/// Execution identifier for a terminal command run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExecutionId(pub u64);

/// Shell exit status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellExit {
    /// Numeric exit code.
    pub code: i32,
    /// Optional explanatory message.
    pub message: Option<String>,
}

impl ShellExit {
    /// Successful command completion.
    pub fn success() -> Self {
        Self {
            code: 0,
            message: None,
        }
    }

    /// Cancellation completion.
    pub fn cancelled() -> Self {
        Self {
            code: 130,
            message: Some("command cancelled".to_string()),
        }
    }
}

/// Structured shell error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellErrorCode {
    /// User input violated command usage.
    Usage,
    /// The command was not found.
    NotFound,
    /// The command is unavailable in this host context.
    Unavailable,
    /// The caller lacks permission to perform the action.
    PermissionDenied,
    /// Internal command or runtime failure.
    Internal,
}

/// Error emitted by shell parsing, lookup, or handlers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellError {
    /// Error category.
    pub code: ShellErrorCode,
    /// Human-readable message.
    pub message: String,
}

impl ShellError {
    /// Creates a new shell error.
    pub fn new(code: ShellErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Converts the error into a conventional exit code.
    pub fn exit_code(&self) -> i32 {
        match self.code {
            ShellErrorCode::Usage => 2,
            ShellErrorCode::NotFound => 3,
            ShellErrorCode::Unavailable | ShellErrorCode::PermissionDenied => 4,
            ShellErrorCode::Internal => 5,
        }
    }
}

/// Incremental stream event emitted by the shell runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ShellStreamEvent {
    /// Execution started.
    Started {
        /// Execution identifier.
        execution_id: ExecutionId,
    },
    /// Human-readable notice.
    Notice {
        /// Execution identifier.
        execution_id: ExecutionId,
        /// Notice payload.
        notice: CommandNotice,
    },
    /// Progress update in the `0.0..=1.0` range when known.
    Progress {
        /// Execution identifier.
        execution_id: ExecutionId,
        /// Optional progress ratio.
        value: Option<f32>,
        /// Optional short label.
        label: Option<String>,
    },
    /// Structured result data frame.
    Data {
        /// Execution identifier.
        execution_id: ExecutionId,
        /// Structured payload.
        data: StructuredData,
        /// Preferred presentation.
        display: DisplayPreference,
    },
    /// Execution completed successfully or with a command error.
    Completed {
        /// Summary payload.
        summary: ShellExecutionSummary,
    },
    /// Execution was cancelled.
    Cancelled {
        /// Execution identifier.
        execution_id: ExecutionId,
    },
}

/// Opaque registration token used to unregister commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommandRegistrationToken(pub u64);
