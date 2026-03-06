//! Runtime-agnostic browser-native shell engine with hierarchical command registration.
//!
//! [`ShellEngine`] owns a shared [`CommandRegistry`] while each [`ShellSessionHandle`] maintains
//! its own cwd, event log, active execution slot, and cancellation state. The engine emits typed
//! stream events defined in [`system_shell_contract`] so the desktop runtime and terminal UI can
//! render notices, progress, and structured output consistently.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use futures::future::LocalBoxFuture;
use leptos::{create_rw_signal, ReadSignal, RwSignal, SignalGetUntracked, SignalSet, SignalUpdate};
use system_shell_contract::{
    CommandDataShape, CommandDescriptor, CommandInputShape, CommandNotice, CommandNoticeLevel,
    CommandPath, CommandRegistrationToken, CommandResult, CommandScope, CommandVisibility,
    CompletionItem, CompletionRequest, DisplayPreference, ExecutionId, ParsedCommandLine,
    ParsedInvocation, ParsedLiteral, ParsedOption, ParsedValue, ShellError, ShellErrorCode,
    ShellExecutionSummary, ShellExit, ShellRequest, ShellStreamEvent, StructuredData,
    StructuredRecord, StructuredScalar, StructuredTable, StructuredValue,
};

/// Async completion provider.
pub type CompletionHandler = Rc<
    dyn Fn(CompletionRequest) -> LocalBoxFuture<'static, Result<Vec<CompletionItem>, ShellError>>,
>;

/// Async command handler.
pub type CommandHandler = Rc<
    dyn Fn(CommandExecutionContext) -> LocalBoxFuture<'static, Result<CommandResult, ShellError>>,
>;

/// Shared command execution context for handlers.
#[derive(Clone)]
pub struct CommandExecutionContext {
    /// Parsed execution identifier.
    pub execution_id: ExecutionId,
    /// Canonical descriptor for the resolved command.
    pub descriptor: CommandDescriptor,
    /// Parsed invocation metadata.
    pub invocation: ParsedInvocation,
    /// Full token vector for the invocation.
    pub argv: Vec<String>,
    /// Positional argument tokens after command-path resolution.
    pub args: Vec<String>,
    /// Current logical cwd.
    pub cwd: String,
    /// Structured input from the previous pipeline stage.
    pub input: StructuredData,
    /// Optional source window identifier.
    pub source_window_id: Option<u64>,
    emitter: EventEmitter,
    session_cwd: RwSignal<String>,
    cancelled: Rc<Cell<bool>>,
}

impl CommandExecutionContext {
    /// Emits an informational notice.
    pub fn info(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Info, message);
    }

    /// Emits a warning notice.
    pub fn warn(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Warning, message);
    }

    /// Emits an error notice.
    pub fn error(&self, message: impl Into<String>) {
        self.notice(CommandNoticeLevel::Error, message);
    }

    /// Emits a structured notice.
    pub fn notice(&self, level: CommandNoticeLevel, message: impl Into<String>) {
        self.emitter.notice(
            self.execution_id,
            CommandNotice {
                level,
                message: message.into(),
            },
        );
    }

    /// Emits a progress update.
    pub fn progress(&self, value: Option<f32>, label: Option<String>) {
        self.emitter.progress(self.execution_id, value, label);
    }

    /// Updates the logical cwd for the active session.
    pub fn set_cwd(&self, cwd: impl Into<String>) {
        self.session_cwd.set(cwd.into());
    }

    /// Returns whether the foreground execution has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
}

#[derive(Clone)]
struct EventEmitter {
    events: RwSignal<Vec<ShellStreamEvent>>,
}

impl EventEmitter {
    fn push(&self, event: ShellStreamEvent) {
        self.events.update(|events| events.push(event));
    }

    fn notice(&self, execution_id: ExecutionId, notice: CommandNotice) {
        self.push(ShellStreamEvent::Notice {
            execution_id,
            notice,
        });
    }

    fn progress(&self, execution_id: ExecutionId, value: Option<f32>, label: Option<String>) {
        self.push(ShellStreamEvent::Progress {
            execution_id,
            value,
            label,
        });
    }

    fn data(&self, execution_id: ExecutionId, data: StructuredData, display: DisplayPreference) {
        self.push(ShellStreamEvent::Data {
            execution_id,
            data,
            display,
        });
    }
}

#[derive(Clone)]
struct RegisteredCommand {
    descriptor: CommandDescriptor,
    completion: Option<CompletionHandler>,
    handler: CommandHandler,
}

#[derive(Default)]
struct RegistryState {
    next_token: u64,
    by_token: BTreeMap<CommandRegistrationToken, RegisteredCommand>,
}

/// Shared command registry.
///
/// The registry stores descriptors, completion handlers, and execution handlers. Session objects
/// snapshot visible commands from this registry when resolving completions and command execution.
#[derive(Clone, Default)]
pub struct CommandRegistry {
    state: Rc<RefCell<RegistryState>>,
}

impl CommandRegistry {
    /// Registers one command and returns its registration token.
    pub fn register(
        &self,
        descriptor: CommandDescriptor,
        completion: Option<CompletionHandler>,
        handler: CommandHandler,
    ) -> CommandRegistrationToken {
        let mut state = self.state.borrow_mut();
        state.next_token = state.next_token.saturating_add(1);
        let token = CommandRegistrationToken(state.next_token);
        state.by_token.insert(
            token,
            RegisteredCommand {
                descriptor,
                completion,
                handler,
            },
        );
        token
    }

    /// Removes a previously registered command token.
    pub fn unregister(&self, token: CommandRegistrationToken) {
        self.state.borrow_mut().by_token.remove(&token);
    }

    fn visible_commands(&self) -> Vec<RegisteredCommand> {
        self.state.borrow().by_token.values().cloned().collect()
    }

    /// Returns the currently registered command descriptors.
    pub fn descriptors(&self) -> Vec<CommandDescriptor> {
        let mut descriptors = self
            .visible_commands()
            .into_iter()
            .map(|registered| registered.descriptor)
            .collect::<Vec<_>>();
        descriptors.sort_by(|left, right| left.path.display().cmp(&right.path.display()));
        descriptors
    }
}

/// Drop-based registration handle.
#[derive(Clone)]
pub struct CommandRegistryHandle {
    registry: CommandRegistry,
    token: CommandRegistrationToken,
    active: Rc<Cell<bool>>,
}

impl CommandRegistryHandle {
    /// Unregisters the command if it is still active.
    pub fn unregister(&self) {
        if self.active.replace(false) {
            self.registry.unregister(self.token);
        }
    }
}

impl Drop for CommandRegistryHandle {
    fn drop(&mut self) {
        self.unregister();
    }
}

#[derive(Clone)]
struct SessionState {
    cwd: RwSignal<String>,
    events: RwSignal<Vec<ShellStreamEvent>>,
    active_execution: RwSignal<Option<ExecutionId>>,
    next_execution_id: Rc<Cell<u64>>,
    cancel_flag: Rc<Cell<bool>>,
}

/// A shell session with one foreground execution slot.
#[derive(Clone)]
pub struct ShellSessionHandle {
    state: SessionState,
    registry: CommandRegistry,
}

impl ShellSessionHandle {
    /// Reactive stream event log for this session.
    pub fn events(&self) -> ReadSignal<Vec<ShellStreamEvent>> {
        self.state.events.read_only()
    }

    /// Reactive active execution id for this session.
    pub fn active_execution(&self) -> ReadSignal<Option<ExecutionId>> {
        self.state.active_execution.read_only()
    }

    /// Reactive current cwd for this session.
    pub fn cwd(&self) -> ReadSignal<String> {
        self.state.cwd.read_only()
    }

    /// Cancels the active foreground execution.
    pub fn cancel(&self) {
        if self.state.active_execution.get_untracked().is_some() {
            self.state.cancel_flag.set(true);
        }
    }

    /// Resolves completion candidates for the current input.
    pub async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Vec<CompletionItem>, ShellError> {
        let snapshot = RegistrySnapshot::new(self.registry.visible_commands());
        snapshot.complete(request).await
    }

    /// Parses and executes one command request.
    pub fn submit(&self, request: ShellRequest) {
        if self.state.active_execution.get_untracked().is_some() {
            self.state.events.update(|events| {
                events.push(ShellStreamEvent::Notice {
                    execution_id: ExecutionId(0),
                    notice: CommandNotice {
                        level: CommandNoticeLevel::Warning,
                        message: "another command is already running".to_string(),
                    },
                });
            });
            return;
        }

        let parsed = match parse_command_line(&request.line) {
            Ok(parsed) => parsed,
            Err(err) => {
                let execution_id = self.next_execution_id();
                self.state.events.update(|events| {
                    events.push(ShellStreamEvent::Started { execution_id });
                    events.push(ShellStreamEvent::Notice {
                        execution_id,
                        notice: CommandNotice {
                            level: CommandNoticeLevel::Error,
                            message: err.message.clone(),
                        },
                    });
                    events.push(ShellStreamEvent::Completed {
                        summary: ShellExecutionSummary {
                            execution_id,
                            command_path: None,
                            exit: ShellExit {
                                code: err.exit_code(),
                                message: Some(err.message),
                            },
                        },
                    });
                });
                return;
            }
        };

        if parsed.pipeline.is_empty() {
            return;
        }

        let execution_id = self.next_execution_id();
        self.state.cancel_flag.set(false);
        self.state.active_execution.set(Some(execution_id));
        let state = self.state.clone();
        let registry = self.registry.clone();
        leptos::spawn_local(async move {
            let emitter = EventEmitter {
                events: state.events,
            };
            emitter.push(ShellStreamEvent::Started { execution_id });

            let snapshot = RegistrySnapshot::new(registry.visible_commands());
            let mut piped_input = StructuredData::Empty;
            let mut final_summary = ShellExecutionSummary {
                execution_id,
                command_path: None,
                exit: ShellExit::success(),
            };

            for stage in parsed.pipeline {
                if state.cancel_flag.get() {
                    emitter.push(ShellStreamEvent::Cancelled { execution_id });
                    final_summary.exit = ShellExit::cancelled();
                    break;
                }

                match snapshot.resolve_stage(&stage.tokens) {
                    Ok(ResolvedStage::Namespace { path }) => {
                        let result = snapshot.namespace_result(&path);
                        for notice in &result.notices {
                            emitter.notice(execution_id, notice.clone());
                        }
                        if !matches!(result.output, StructuredData::Empty) {
                            emitter.data(execution_id, result.output.clone(), result.display);
                            piped_input = result.output;
                        }
                        final_summary.command_path = Some(path);
                        final_summary.exit = result.exit;
                    }
                    Ok(ResolvedStage::Leaf {
                        registered,
                        matched_len,
                    }) => {
                        let (options, values, args) =
                            parse_invocation_arguments(&stage.tokens[matched_len..]);
                        let invocation = ParsedInvocation {
                            tokens: stage.tokens.clone(),
                            options,
                            values,
                        };

                        if wants_help(&invocation) {
                            let result = snapshot.command_help_result(&registered.descriptor);
                            emitter.data(execution_id, result.output.clone(), result.display);
                            piped_input = result.output;
                            final_summary.command_path = Some(registered.descriptor.path.clone());
                            final_summary.exit = result.exit;
                            continue;
                        }

                        let input_shape = registered.descriptor.input_shape.clone();
                        if let Err(err) = validate_input_shape(&piped_input, &input_shape) {
                            emitter.notice(
                                execution_id,
                                CommandNotice {
                                    level: CommandNoticeLevel::Error,
                                    message: err.message.clone(),
                                },
                            );
                            final_summary.command_path = Some(registered.descriptor.path.clone());
                            final_summary.exit = ShellExit {
                                code: err.exit_code(),
                                message: Some(err.message),
                            };
                            break;
                        }

                        let context = CommandExecutionContext {
                            execution_id,
                            descriptor: registered.descriptor.clone(),
                            invocation,
                            argv: stage.tokens.clone(),
                            args,
                            cwd: state.cwd.get_untracked(),
                            input: piped_input.clone(),
                            source_window_id: request.source_window_id,
                            emitter: emitter.clone(),
                            session_cwd: state.cwd,
                            cancelled: state.cancel_flag.clone(),
                        };
                        match (registered.handler)(context).await {
                            Ok(result) => {
                                if let Some(cwd) = result.cwd.clone() {
                                    state.cwd.set(cwd);
                                }
                                for notice in &result.notices {
                                    emitter.notice(execution_id, notice.clone());
                                }
                                if !matches!(result.output, StructuredData::Empty) {
                                    emitter.data(
                                        execution_id,
                                        result.output.clone(),
                                        result.display,
                                    );
                                }
                                piped_input = result.output;
                                final_summary.command_path =
                                    Some(registered.descriptor.path.clone());
                                final_summary.exit = result.exit.clone();
                                if final_summary.exit.code != 0 {
                                    break;
                                }
                            }
                            Err(err) => {
                                emitter.notice(
                                    execution_id,
                                    CommandNotice {
                                        level: CommandNoticeLevel::Error,
                                        message: err.message.clone(),
                                    },
                                );
                                final_summary.command_path =
                                    Some(registered.descriptor.path.clone());
                                final_summary.exit = ShellExit {
                                    code: err.exit_code(),
                                    message: Some(err.message),
                                };
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        emitter.notice(
                            execution_id,
                            CommandNotice {
                                level: CommandNoticeLevel::Error,
                                message: err.message.clone(),
                            },
                        );
                        final_summary.exit = ShellExit {
                            code: err.exit_code(),
                            message: Some(err.message),
                        };
                        break;
                    }
                }
            }

            emitter.push(ShellStreamEvent::Completed {
                summary: final_summary,
            });
            state.active_execution.set(None);
        });
    }

    fn next_execution_id(&self) -> ExecutionId {
        let next = self.state.next_execution_id.get().saturating_add(1);
        self.state.next_execution_id.set(next);
        ExecutionId(next)
    }
}

#[derive(Clone)]
struct RegistrySnapshot {
    commands: Vec<RegisteredCommand>,
}

impl RegistrySnapshot {
    fn new(commands: Vec<RegisteredCommand>) -> Self {
        Self { commands }
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Vec<CompletionItem>, ShellError> {
        let parsed = tokenize_line(&request.line)?;
        let stages = split_pipeline_tokens(parsed)?;
        let current_stage = stages.last().cloned().unwrap_or_default();
        let ends_with_space = request
            .line
            .chars()
            .last()
            .map(|ch| ch.is_whitespace())
            .unwrap_or(false);
        let (base_tokens, prefix) = if ends_with_space {
            (current_stage.clone(), String::new())
        } else if let Some(last) = current_stage.last() {
            (
                current_stage[..current_stage.len().saturating_sub(1)].to_vec(),
                last.clone(),
            )
        } else {
            (Vec::new(), String::new())
        };

        if let Ok(ResolvedStage::Leaf {
            registered,
            matched_len,
        }) = self.resolve_stage(&base_tokens)
        {
            if base_tokens.len() >= matched_len {
                if let Some(completion) = registered.completion {
                    return completion(request).await;
                }
            }
        }

        let mut items = Vec::new();
        for (segment, descriptor) in self.child_segments(&base_tokens, &prefix) {
            items.push(CompletionItem {
                value: segment.clone(),
                label: segment,
                detail: descriptor.map(|descriptor| descriptor.help.summary.clone()),
            });
        }
        items.sort_by(|left, right| left.label.cmp(&right.label));
        items.dedup_by(|left, right| left.value == right.value);
        Ok(items)
    }

    fn descriptors(&self) -> Vec<CommandDescriptor> {
        let mut descriptors = self
            .commands
            .iter()
            .filter(|registered| registered.descriptor.visibility == CommandVisibility::Public)
            .map(|registered| registered.descriptor.clone())
            .collect::<Vec<_>>();
        descriptors.sort_by(|left, right| left.path.display().cmp(&right.path.display()));
        descriptors
    }

    fn child_segments(
        &self,
        base_tokens: &[String],
        prefix: &str,
    ) -> Vec<(String, Option<CommandDescriptor>)> {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for descriptor in self.descriptors() {
            let tokens = descriptor_path_tokens(&descriptor);
            if tokens.len() <= base_tokens.len() || !tokens.starts_with(base_tokens) {
                continue;
            }
            let next = tokens[base_tokens.len()].clone();
            if next.starts_with(prefix) && seen.insert(next.clone()) {
                out.push((next, Some(descriptor.clone())));
            }
        }
        out
    }

    fn command_help_result(&self, descriptor: &CommandDescriptor) -> CommandResult {
        let aliases = if descriptor.aliases.is_empty() {
            StructuredValue::List(Vec::new())
        } else {
            StructuredValue::List(
                descriptor
                    .aliases
                    .iter()
                    .cloned()
                    .map(|alias| StructuredValue::Scalar(StructuredScalar::String(alias)))
                    .collect(),
            )
        };
        let examples = StructuredValue::List(
            descriptor
                .help
                .examples
                .iter()
                .map(|example| {
                    StructuredValue::Record(StructuredRecord {
                        fields: vec![
                            field_string("command", example.command.clone()),
                            field_string("summary", example.summary.clone()),
                        ],
                    })
                })
                .collect(),
        );
        CommandResult {
            output: StructuredData::Record(StructuredRecord {
                fields: vec![
                    field_string("path", descriptor.path.display()),
                    field_string("summary", descriptor.help.summary.clone()),
                    field_string("usage", descriptor.help.usage.clone()),
                    StructuredFieldBuilder::new("aliases", aliases).build(),
                    StructuredFieldBuilder::new("examples", examples).build(),
                ],
            }),
            display: DisplayPreference::Help,
            notices: Vec::new(),
            cwd: None,
            exit: ShellExit::success(),
        }
    }

    fn namespace_result(&self, path: &CommandPath) -> CommandResult {
        let prefix = path
            .segments()
            .iter()
            .map(|segment| segment.as_str().to_string())
            .collect::<Vec<_>>();
        let mut rows = Vec::new();
        let mut seen = BTreeSet::new();
        for descriptor in self.descriptors() {
            let tokens = descriptor_path_tokens(&descriptor);
            if tokens.len() <= prefix.len() || !tokens.starts_with(&prefix) {
                continue;
            }
            let name = tokens[prefix.len()].clone();
            if !seen.insert(name.clone()) {
                continue;
            }
            rows.push(StructuredRecord {
                fields: vec![
                    field_string("name", name),
                    field_string("summary", descriptor.help.summary.clone()),
                ],
            });
        }
        let table = StructuredTable {
            columns: vec!["name".to_string(), "summary".to_string()],
            rows,
            schema: None,
            source_command: Some(path.clone()),
            fallback_text: None,
        };
        CommandResult {
            output: StructuredData::Table(table),
            display: DisplayPreference::Help,
            notices: Vec::new(),
            cwd: None,
            exit: ShellExit::success(),
        }
    }

    fn resolve_stage(&self, tokens: &[String]) -> Result<ResolvedStage, ShellError> {
        let mut best_match: Option<(RegisteredCommand, usize, u8)> = None;
        let mut ambiguous = false;

        for registered in &self.commands {
            for candidate in candidate_paths(&registered.descriptor) {
                if tokens.len() < candidate.len() || !tokens.starts_with(&candidate) {
                    continue;
                }
                let score = (candidate.len(), scope_rank(&registered.descriptor.scope));
                match best_match.as_ref() {
                    Some((_, best_len, best_scope))
                        if score.0 < *best_len
                            || (score.0 == *best_len && score.1 < *best_scope) =>
                    {
                        continue;
                    }
                    Some((_, best_len, best_scope))
                        if score.0 == *best_len && score.1 == *best_scope =>
                    {
                        ambiguous = true;
                    }
                    _ => {
                        ambiguous = false;
                        best_match = Some((registered.clone(), candidate.len(), score.1));
                    }
                }
            }
        }

        if ambiguous {
            return Err(ShellError::new(
                ShellErrorCode::Usage,
                format!("ambiguous command `{}`", tokens.join(" ")),
            ));
        }

        if let Some((registered, matched_len, _)) = best_match {
            return Ok(ResolvedStage::Leaf {
                registered: Box::new(registered),
                matched_len,
            });
        }

        if prefix_exists(&self.descriptors(), tokens) {
            return Ok(ResolvedStage::Namespace {
                path: CommandPath::from_segments(
                    tokens
                        .iter()
                        .cloned()
                        .map(system_shell_contract::CommandSegment::new),
                ),
            });
        }

        Err(ShellError::new(
            ShellErrorCode::NotFound,
            format!("command not found: {}", tokens.join(" ")),
        ))
    }
}

#[derive(Clone)]
enum ResolvedStage {
    Namespace {
        path: CommandPath,
    },
    Leaf {
        registered: Box<RegisteredCommand>,
        matched_len: usize,
    },
}

fn scope_rank(scope: &CommandScope) -> u8 {
    match scope {
        CommandScope::Window { .. } => 3,
        CommandScope::App { .. } => 2,
        CommandScope::Global => 1,
    }
}

fn descriptor_path_tokens(descriptor: &CommandDescriptor) -> Vec<String> {
    descriptor
        .path
        .segments()
        .iter()
        .map(|segment| segment.as_str().to_string())
        .collect()
}

fn candidate_paths(descriptor: &CommandDescriptor) -> Vec<Vec<String>> {
    let mut candidates = vec![descriptor_path_tokens(descriptor)];
    candidates.extend(
        descriptor
            .aliases
            .iter()
            .map(|alias| alias.split_whitespace().map(str::to_string).collect()),
    );
    candidates
}

fn prefix_exists(descriptors: &[CommandDescriptor], prefix: &[String]) -> bool {
    descriptors.iter().any(|descriptor| {
        candidate_paths(descriptor)
            .into_iter()
            .any(|candidate| candidate.len() > prefix.len() && candidate.starts_with(prefix))
    })
}

fn wants_help(invocation: &ParsedInvocation) -> bool {
    invocation
        .options
        .iter()
        .any(|option| option.name == "help" || option.short == Some('h'))
}

fn validate_input_shape(
    input: &StructuredData,
    shape: &CommandInputShape,
) -> Result<(), ShellError> {
    if !shape.accepts_pipeline_input {
        if matches!(input, StructuredData::Empty) {
            return Ok(());
        }
        return Err(ShellError::new(
            ShellErrorCode::Usage,
            "command does not accept piped input",
        ));
    }

    if shape.shape == CommandDataShape::Any || matches!(input, StructuredData::Empty) {
        return Ok(());
    }

    if input.shape() == shape.shape {
        return Ok(());
    }

    Err(ShellError::new(
        ShellErrorCode::Usage,
        format!(
            "expected {:?} pipeline input, got {:?}",
            shape.shape,
            input.shape()
        ),
    ))
}

fn tokenize_line(line: &str) -> Result<Vec<Token>, ShellError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut quote = None::<char>;

    while let Some(ch) = chars.next() {
        match quote {
            Some(active) if ch == active => quote = None,
            Some(_) if ch == '\\' => {
                let Some(next) = chars.next() else {
                    return Err(ShellError::new(
                        ShellErrorCode::Usage,
                        "dangling escape sequence",
                    ));
                };
                current.push(next);
            }
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '|' => {
                if !current.is_empty() {
                    tokens.push(Token::Word(std::mem::take(&mut current)));
                }
                tokens.push(Token::Pipe);
            }
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(Token::Word(std::mem::take(&mut current)));
                }
            }
            None if ch == '\\' => {
                let Some(next) = chars.next() else {
                    return Err(ShellError::new(
                        ShellErrorCode::Usage,
                        "dangling escape sequence",
                    ));
                };
                current.push(next);
            }
            None => current.push(ch),
        }
    }

    if quote.is_some() {
        return Err(ShellError::new(
            ShellErrorCode::Usage,
            "unterminated quoted string",
        ));
    }

    if !current.is_empty() {
        tokens.push(Token::Word(current));
    }

    Ok(tokens)
}

fn split_pipeline_tokens(tokens: Vec<Token>) -> Result<Vec<Vec<String>>, ShellError> {
    let mut stages = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        match token {
            Token::Pipe => {
                if current.is_empty() {
                    return Err(ShellError::new(
                        ShellErrorCode::Usage,
                        "empty pipeline stage",
                    ));
                }
                stages.push(std::mem::take(&mut current));
            }
            Token::Word(word) => current.push(word),
        }
    }
    if current.is_empty() && !stages.is_empty() {
        return Err(ShellError::new(
            ShellErrorCode::Usage,
            "pipeline cannot end with `|`",
        ));
    }
    if !current.is_empty() {
        stages.push(current);
    }
    Ok(stages)
}

fn parse_command_line(line: &str) -> Result<ParsedCommandLine, ShellError> {
    let stages = split_pipeline_tokens(tokenize_line(line)?)?;
    Ok(ParsedCommandLine {
        pipeline: stages
            .into_iter()
            .map(|tokens| ParsedInvocation {
                tokens,
                options: Vec::new(),
                values: Vec::new(),
            })
            .collect(),
    })
}

fn parse_invocation_arguments(
    tokens: &[String],
) -> (Vec<ParsedOption>, Vec<ParsedValue>, Vec<String>) {
    let mut options = Vec::new();
    let mut values = Vec::new();
    let mut args = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let token = &tokens[index];
        if let Some(rest) = token.strip_prefix("--") {
            if !rest.is_empty() {
                if let Some((name, raw_value)) = rest.split_once('=') {
                    options.push(ParsedOption {
                        name: name.to_string(),
                        short: None,
                        value: Some(parse_value(raw_value)),
                    });
                } else {
                    let takes_value =
                        index + 1 < tokens.len() && !tokens[index + 1].starts_with('-');
                    let value = takes_value.then(|| {
                        index += 1;
                        parse_value(&tokens[index])
                    });
                    options.push(ParsedOption {
                        name: rest.to_string(),
                        short: None,
                        value,
                    });
                }
                index += 1;
                continue;
            }
        }

        if token.starts_with('-') && token.len() > 1 {
            for short in token.trim_start_matches('-').chars() {
                options.push(ParsedOption {
                    name: short.to_string(),
                    short: Some(short),
                    value: None,
                });
            }
            index += 1;
            continue;
        }

        args.push(token.clone());
        values.push(parse_value(token));
        index += 1;
    }

    (options, values, args)
}

fn parse_value(raw: &str) -> ParsedValue {
    let literal = if raw == "null" {
        ParsedLiteral::Null
    } else if matches!(raw, "true" | "on") {
        ParsedLiteral::Bool(true)
    } else if matches!(raw, "false" | "off") {
        ParsedLiteral::Bool(false)
    } else if let Ok(value) = raw.parse::<i64>() {
        ParsedLiteral::Int(value)
    } else if let Ok(value) = raw.parse::<f64>() {
        ParsedLiteral::Float(value)
    } else {
        ParsedLiteral::String(raw.to_string())
    };

    ParsedValue {
        raw: raw.to_string(),
        literal,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Pipe,
    Word(String),
}

fn field_string(name: &str, value: String) -> system_shell_contract::StructuredField {
    StructuredFieldBuilder::new(
        name,
        StructuredValue::Scalar(StructuredScalar::String(value)),
    )
    .build()
}

struct StructuredFieldBuilder {
    name: String,
    value: StructuredValue,
}

impl StructuredFieldBuilder {
    fn new(name: &str, value: StructuredValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    fn build(self) -> system_shell_contract::StructuredField {
        system_shell_contract::StructuredField {
            name: self.name,
            value: self.value,
        }
    }
}

/// Root shell engine used by the runtime.
///
/// Create one engine for the runtime, register built-in or app-provided commands on its registry,
/// then spawn per-window sessions with [`ShellEngine::new_session`].
#[derive(Clone, Default)]
pub struct ShellEngine {
    registry: CommandRegistry,
}

impl ShellEngine {
    /// Creates a new shared shell engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the shared registry.
    pub fn registry(&self) -> CommandRegistry {
        self.registry.clone()
    }

    /// Returns all currently visible command descriptors.
    pub fn descriptors(&self) -> Vec<CommandDescriptor> {
        self.registry.descriptors()
    }

    /// Registers a command and returns a drop-based handle.
    pub fn register_command(
        &self,
        descriptor: CommandDescriptor,
        completion: Option<CompletionHandler>,
        handler: CommandHandler,
    ) -> CommandRegistryHandle {
        let token = self.registry.register(descriptor, completion, handler);
        CommandRegistryHandle {
            registry: self.registry.clone(),
            token,
            active: Rc::new(Cell::new(true)),
        }
    }

    /// Creates one shell session with its own cwd and event stream.
    pub fn new_session(&self, cwd: impl Into<String>) -> ShellSessionHandle {
        let cwd = cwd.into();
        let state = SessionState {
            cwd: create_rw_signal(cwd),
            events: create_rw_signal(Vec::new()),
            active_execution: create_rw_signal(None),
            next_execution_id: Rc::new(Cell::new(0)),
            cancel_flag: Rc::new(Cell::new(false)),
        };
        ShellSessionHandle {
            state,
            registry: self.registry.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use system_shell_contract::{
        CommandArgSpec, CommandExample, CommandId, CommandInteractionKind, CommandOptionSpec,
        CommandOutputShape, HelpDoc,
    };

    fn descriptor(path: &str, aliases: &[&str], scope: CommandScope) -> CommandDescriptor {
        let path = CommandPath::new(path);
        let display = path.display();
        CommandDescriptor {
            id: CommandId::new(display.clone()),
            parent_path: path.parent(),
            path,
            aliases: aliases.iter().map(|alias| alias.to_string()).collect(),
            scope,
            visibility: CommandVisibility::Public,
            interaction_kind: CommandInteractionKind::Hierarchical,
            discoverable_children: true,
            input_shape: CommandInputShape::none(),
            output_shape: CommandOutputShape::new(CommandDataShape::Table),
            args: vec![CommandArgSpec {
                name: "value".to_string(),
                summary: "value".to_string(),
                required: false,
                repeatable: false,
            }],
            options: vec![CommandOptionSpec {
                name: "help".to_string(),
                short: Some('h'),
                summary: "show help".to_string(),
                takes_value: false,
            }],
            help: HelpDoc {
                summary: "summary".to_string(),
                description: None,
                usage: display.clone(),
                examples: vec![CommandExample {
                    command: display,
                    summary: "example".to_string(),
                }],
            },
        }
    }

    #[test]
    fn registration_handle_unregisters() {
        let _ = leptos::create_runtime();
        let engine = ShellEngine::new();
        let handle = engine.register_command(
            descriptor("apps list", &[], CommandScope::Global),
            None,
            Rc::new(|_| Box::pin(async { Ok(CommandResult::success(StructuredData::Empty)) })),
        );
        assert_eq!(engine.registry.visible_commands().len(), 1);
        handle.unregister();
        assert_eq!(engine.registry.visible_commands().len(), 0);
    }

    #[test]
    fn parser_splits_pipelines() {
        let parsed = parse_command_line("ls | data select name").expect("parse");
        assert_eq!(parsed.pipeline.len(), 2);
        assert_eq!(parsed.pipeline[0].tokens, vec!["ls"]);
        assert_eq!(parsed.pipeline[1].tokens, vec!["data", "select", "name"]);
    }
}
