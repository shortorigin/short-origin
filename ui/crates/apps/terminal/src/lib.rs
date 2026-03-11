//! Terminal desktop app UI component backed by the browser-native shell session bridge.
//!
//! The app persists cwd, input, transcript, and active-execution metadata through the runtime and
//! renders typed shell notices, progress, and structured output produced by
//! [`system_shell_contract`].

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use desktop_app_contract::{AppServices, WindowRuntimeId, window_primary_input_dom_id};
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::tachys::view::any_view::{AnyView, IntoAny};
use leptos::{logging, task::spawn_local};
use platform_host::CapabilityStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_shell_contract::{
    CommandNotice, CompletionItem as ShellCompletionItem, CompletionRequest, DisplayPreference,
    ExecutionId, ShellRequest, ShellStreamEvent, ShellSubmitError, StructuredData,
    StructuredRecord, StructuredScalar, StructuredTable, StructuredValue,
};
use system_ui::components::AppShell;
use system_ui::primitives::{
    CompletionItem as CompletionOption, CompletionList, DataTable, ListSurface, TerminalLine,
    TerminalPrompt, TerminalSurface, TerminalTranscript, TextField, TextTone,
};

const MAX_TERMINAL_ENTRIES: usize = 200;
const AUTO_FOLLOW_THRESHOLD_PX: i32 = 32;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PersistedExecutionState {
    execution_id: ExecutionId,
    command: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum TerminalTranscriptEntry {
    Prompt {
        cwd: String,
        command: String,
        execution_id: Option<ExecutionId>,
    },
    Notice {
        notice: CommandNotice,
        execution_id: ExecutionId,
    },
    Data {
        data: StructuredData,
        display: DisplayPreference,
        execution_id: ExecutionId,
    },
    Progress {
        execution_id: ExecutionId,
        value: Option<f32>,
        label: Option<String>,
    },
    System {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TerminalPersistedState {
    cwd: String,
    input: String,
    transcript: Vec<TerminalTranscriptEntry>,
    history_cursor: Option<usize>,
    active_execution: Option<PersistedExecutionState>,
}

fn default_terminal_transcript() -> Vec<TerminalTranscriptEntry> {
    vec![TerminalTranscriptEntry::System {
        text: "Use `help list` to inspect commands.".to_string(),
    }]
}

fn terminal_mode_label(services: Option<&AppServices>) -> &'static str {
    match services {
        Some(services) if services.capabilities().supports_terminal_process() => "hybrid",
        _ => "structured",
    }
}

fn terminal_mode_notice(services: Option<&AppServices>) -> &'static str {
    match services {
        Some(services) => match services.capabilities().host().terminal_process {
            CapabilityStatus::Available => "Host terminal-process backend available.",
            CapabilityStatus::RequiresUserActivation => {
                "Host terminal-process backend requires activation."
            }
            CapabilityStatus::Unavailable => {
                "Running in structured shell mode; native host process access is unavailable."
            }
        },
        None => "Running in structured shell mode.",
    }
}

fn normalize_terminal_transcript(transcript: &mut Vec<TerminalTranscriptEntry>) {
    if transcript.is_empty() {
        *transcript = default_terminal_transcript();
        return;
    }

    if transcript.len() > MAX_TERMINAL_ENTRIES {
        let overflow = transcript.len() - MAX_TERMINAL_ENTRIES;
        transcript.drain(0..overflow);
    }
}

fn push_terminal_entry(
    transcript: &mut Vec<TerminalTranscriptEntry>,
    entry: TerminalTranscriptEntry,
) {
    transcript.push(entry);
    normalize_terminal_transcript(transcript);
}

fn restore_terminal_state(
    mut restored: TerminalPersistedState,
    launch_cwd: &str,
) -> TerminalPersistedState {
    if restored.cwd.trim().is_empty() {
        restored.cwd = launch_cwd.to_string();
    }
    if restored.active_execution.is_some() {
        restored.active_execution = None;
        restored.transcript.push(TerminalTranscriptEntry::System {
            text: "Previous command interrupted during restore.".to_string(),
        });
    }
    normalize_terminal_transcript(&mut restored.transcript);
    restored
}

fn apply_shell_session_events(
    transcript: &mut Vec<TerminalTranscriptEntry>,
    active_execution: &mut Option<PersistedExecutionState>,
    pending_command: &mut Option<String>,
    already_processed: u64,
    events: &[system_shell_contract::SequencedShellStreamEvent],
) -> u64 {
    let Some(last_event) = events.last() else {
        return already_processed;
    };
    if already_processed >= last_event.sequence {
        return already_processed;
    }

    if let Some(first_event) = events.first() {
        let expected_next = already_processed.saturating_add(1);
        if expected_next < first_event.sequence {
            push_terminal_entry(
                transcript,
                TerminalTranscriptEntry::System {
                    text: "Older shell session events were evicted from the in-memory log."
                        .to_string(),
                },
            );
        }
    }

    for event in events
        .iter()
        .filter(|event| event.sequence > already_processed)
    {
        match &event.event {
            ShellStreamEvent::Started { execution_id } => {
                if let Some(command) = pending_command.take().filter(|command| !command.is_empty())
                {
                    *active_execution = Some(PersistedExecutionState {
                        execution_id: *execution_id,
                        command,
                    });
                }
            }
            ShellStreamEvent::Notice {
                execution_id,
                notice,
            } => push_terminal_entry(
                transcript,
                TerminalTranscriptEntry::Notice {
                    notice: notice.clone(),
                    execution_id: *execution_id,
                },
            ),
            ShellStreamEvent::Data {
                execution_id,
                data,
                display,
            } => push_terminal_entry(
                transcript,
                TerminalTranscriptEntry::Data {
                    data: data.clone(),
                    display: *display,
                    execution_id: *execution_id,
                },
            ),
            ShellStreamEvent::Progress {
                execution_id,
                value,
                label,
            } => push_terminal_entry(
                transcript,
                TerminalTranscriptEntry::Progress {
                    execution_id: *execution_id,
                    value: *value,
                    label: label.clone(),
                },
            ),
            ShellStreamEvent::Cancelled { .. } | ShellStreamEvent::Completed { .. } => {
                *active_execution = None;
            }
        }
    }

    last_event.sequence
}

fn should_auto_follow(
    scroll_height: i32,
    scroll_top: i32,
    client_height: i32,
    threshold: i32,
) -> bool {
    scroll_height - (scroll_top + client_height) <= threshold
}

fn scroll_terminal_to_bottom(terminal_screen: &NodeRef<html::Div>) {
    if let Some(screen) = terminal_screen.get() {
        screen.set_scroll_top(screen.scroll_height());
    }
}

fn terminal_snapshot(
    cwd: &RwSignal<String>,
    input: &RwSignal<String>,
    transcript: &RwSignal<Vec<TerminalTranscriptEntry>>,
    history_cursor: &RwSignal<Option<usize>>,
    active_execution: &RwSignal<Option<PersistedExecutionState>>,
) -> TerminalPersistedState {
    let mut snapshot = TerminalPersistedState {
        cwd: cwd.get_untracked(),
        input: input.get_untracked(),
        transcript: transcript.get_untracked(),
        history_cursor: history_cursor.get_untracked(),
        active_execution: active_execution.get_untracked(),
    };
    normalize_terminal_transcript(&mut snapshot.transcript);
    snapshot
}

fn completion_request(cwd: &str, line: &str) -> CompletionRequest {
    CompletionRequest {
        cwd: cwd.to_string(),
        line: line.to_string(),
        argv: line
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>(),
        cursor: line.len(),
        source_window_id: None,
    }
}

fn scalar_text(value: &StructuredScalar) -> String {
    match value {
        StructuredScalar::Null => "null".to_string(),
        StructuredScalar::Bool(value) => value.to_string(),
        StructuredScalar::Int(value) => value.to_string(),
        StructuredScalar::Float(value) => value.to_string(),
        StructuredScalar::String(value) => value.clone(),
    }
}

fn value_summary(value: &StructuredValue) -> String {
    match value {
        StructuredValue::Scalar(value) => scalar_text(value),
        StructuredValue::Record(record) => {
            format!("{{{}}}", record.fields.len())
        }
        StructuredValue::List(values) => format!("[{}]", values.len()),
    }
}

fn render_record(record: StructuredRecord) -> impl IntoView {
    view! {
        <ListSurface>
            {record
                .fields
                .into_iter()
                .map(|field| {
                    view! {
                        <div>
                            <span>{field.name}</span>
                            <span>{value_summary(&field.value)}</span>
                        </div>
                    }
                })
                .collect_view()}
        </ListSurface>
    }
}

fn render_list(values: Vec<StructuredValue>) -> impl IntoView {
    view! {
        <ListSurface>
            {values
                .into_iter()
                .map(|value| {
                    view! { <div>{value_summary(&value)}</div> }
                })
                .collect_view()}
        </ListSurface>
    }
}

fn field_text(record: &StructuredRecord, name: &str) -> String {
    record
        .fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| value_summary(&field.value))
        .unwrap_or_default()
}

fn render_table(table: StructuredTable) -> impl IntoView {
    let columns = table.columns.clone();
    let rows = table.rows.clone();
    view! {
        <ListSurface>
            <DataTable role="table">
                <thead>
                    <tr>
                        {columns
                            .iter()
                            .map(|column| view! { <th>{column.clone()}</th> })
                            .collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {rows
                        .iter()
                        .enumerate()
                        .map(|(index, row)| {
                            view! {
                                <tr data-row=index.to_string()>
                                    {columns
                                        .iter()
                                        .map(|column| view! { <td>{field_text(row, column)}</td> })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </DataTable>
        </ListSurface>
    }
}

fn render_data(data: StructuredData, _display: DisplayPreference) -> AnyView {
    match data {
        StructuredData::Empty => ().into_view().into_any(),
        StructuredData::Value(StructuredValue::Scalar(value)) => {
            view! { <TerminalLine>{scalar_text(&value)}</TerminalLine> }
                .into_view()
                .into_any()
        }
        StructuredData::Value(StructuredValue::Record(record)) | StructuredData::Record(record) => {
            render_record(record).into_view().into_any()
        }
        StructuredData::Value(StructuredValue::List(values)) | StructuredData::List(values) => {
            render_list(values).into_view().into_any()
        }
        StructuredData::Table(table) => render_table(table).into_view().into_any(),
    }
}

fn render_entry(entry: TerminalTranscriptEntry) -> AnyView {
    match entry {
        TerminalTranscriptEntry::Prompt { cwd, command, .. } => view! {
            <TerminalLine tone=TextTone::Secondary>{format!("{cwd} \u{203a} {command}")}</TerminalLine>
        }
        .into_view()
        .into_any(),
        TerminalTranscriptEntry::Notice { notice, .. } => view! {
            <TerminalLine tone=TextTone::Accent>{notice.message}</TerminalLine>
        }
        .into_view()
        .into_any(),
        TerminalTranscriptEntry::Data { data, display, .. } => render_data(data, display),
        TerminalTranscriptEntry::Progress { value, label, .. } => {
            let label = label.unwrap_or_else(|| "progress".to_string());
            let suffix = value
                .map(|value| format!(" {:.0}%", value * 100.0))
                .unwrap_or_default();
            view! {
                <TerminalLine tone=TextTone::Accent>{format!("{label}{suffix}")}</TerminalLine>
            }
            .into_view()
            .into_any()
        }
        TerminalTranscriptEntry::System { text } => view! {
            <TerminalLine tone=TextTone::Secondary>{text}</TerminalLine>
        }
        .into_view()
        .into_any(),
    }
}

#[component]
/// Terminal app window contents.
///
/// This component presents a browser-native shell backed by runtime-owned commands and persists
/// transcript state via typed host contracts.
pub fn TerminalApp(
    /// Stable runtime window id used to expose the primary input focus target.
    window_id: WindowRuntimeId,
    /// App launch parameters (for example, the initial working directory).
    launch_params: Value,
    /// Manager-restored app state payload for this window instance.
    restored_state: Option<Value>,
    /// Optional app-host bridge for manager-owned commands.
    services: Option<AppServices>,
) -> impl IntoView {
    let input_id = window_primary_input_dom_id(window_id);
    let launch_cwd = launch_params
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or("~/desktop")
        .to_string();
    let mode_label = terminal_mode_label(services.as_ref());
    let shell_session = services
        .as_ref()
        .and_then(|services| services.commands.create_session(launch_cwd.clone()).ok());
    let services_for_persist = services.clone();
    let cwd = RwSignal::new(launch_cwd.clone());
    let input = RwSignal::new(String::new());
    let transcript = RwSignal::new(default_terminal_transcript());
    let suggestions = RwSignal::new(Vec::<ShellCompletionItem>::new());
    let history_cursor = RwSignal::new(None::<usize>);
    let active_execution = RwSignal::new(None::<PersistedExecutionState>);
    let processed_events = RwSignal::new(0u64);
    let pending_command = RwSignal::new(None::<String>);
    let hydrated = RwSignal::new(false);
    let last_saved = RwSignal::new(None::<String>);
    let should_follow_output = RwSignal::new(true);
    let terminal_screen = NodeRef::<html::Div>::new();
    let prompt_mode = move || {
        if active_execution.get().is_some() {
            "running"
        } else {
            mode_label
        }
    };
    if let Some(restored_state) = restored_state.as_ref()
        && let Ok(restored) =
            serde_json::from_value::<TerminalPersistedState>(restored_state.clone())
    {
        let restored = restore_terminal_state(restored, &launch_cwd);
        let serialized = serde_json::to_string(&restored).ok();
        cwd.set(restored.cwd);
        input.set(restored.input);
        transcript.set(restored.transcript);
        history_cursor.set(restored.history_cursor);
        active_execution.set(restored.active_execution);
        last_saved.set(serialized);
        hydrated.set(true);
    }
    transcript.update(|entries| {
        entries.push(TerminalTranscriptEntry::System {
            text: terminal_mode_notice(services.as_ref()).to_string(),
        });
        normalize_terminal_transcript(entries);
    });
    hydrated.set(true);

    Effect::new(move |_| {
        if !hydrated.get() {
            return;
        }

        let _cwd = cwd.get();
        let _input = input.get();
        let _transcript = transcript.get();
        let _history_cursor = history_cursor.get();
        let _active_execution = active_execution.get();
        let snapshot = terminal_snapshot(
            &cwd,
            &input,
            &transcript,
            &history_cursor,
            &active_execution,
        );

        let serialized = match serde_json::to_string(&snapshot) {
            Ok(raw) => raw,
            Err(err) => {
                logging::warn!("terminal serialize failed: {err}");
                return;
            }
        };

        if last_saved.get().as_deref() == Some(serialized.as_str()) {
            return;
        }
        last_saved.set(Some(serialized));

        if let Some(services) = services_for_persist.clone() {
            let state_service = services.state.clone();
            if let Ok(value) = serde_json::to_value(&snapshot) {
                state_service.persist_window_state(value);
            }
        }
    });

    if let Some(shell_session) = shell_session.clone() {
        Effect::new(move |_| {
            let events = shell_session.events.get();
            let already_processed = processed_events.get();
            let mut transcript_entries = transcript.get_untracked();
            let mut active = active_execution.get_untracked();
            let mut pending = pending_command.get_untracked();
            let next_processed = apply_shell_session_events(
                &mut transcript_entries,
                &mut active,
                &mut pending,
                already_processed,
                &events,
            );
            if next_processed == already_processed {
                return;
            }

            transcript.set(transcript_entries);
            active_execution.set(active);
            pending_command.set(pending);
            processed_events.set(next_processed);
            cwd.set(shell_session.cwd.get());
        });
    }

    Effect::new(move |_| {
        let _transcript_len = transcript.get().len();
        let hydrated = hydrated.get();
        let should_follow_output = should_follow_output.get();
        if !hydrated || !should_follow_output {
            return;
        }

        scroll_terminal_to_bottom(&terminal_screen);
    });

    let shell_session_handle = StoredValue::new_local(shell_session.clone());

    let submit_command = StoredValue::new_local({
        let shell_session = shell_session.clone();
        move |command: String| {
            let command = command.trim().to_string();
            if command.is_empty() {
                return;
            }

            transcript.update(|entries| {
                entries.push(TerminalTranscriptEntry::Prompt {
                    cwd: cwd.get_untracked(),
                    command: command.clone(),
                    execution_id: None,
                });
                normalize_terminal_transcript(entries);
            });

            history_cursor.set(None);
            suggestions.set(Vec::new());
            input.set(String::new());

            if command.eq_ignore_ascii_case("clear")
                || command.eq_ignore_ascii_case("terminal clear")
            {
                transcript.set(default_terminal_transcript());
                active_execution.set(None);
                pending_command.set(None);
                return;
            }

            match shell_session.clone() {
                Some(shell_session) => {
                    let request = ShellRequest {
                        line: command.clone(),
                        cwd: cwd.get_untracked(),
                        source_window_id: None,
                    };
                    match shell_session.submit(request) {
                        Ok(_) => pending_command.set(Some(command.clone())),
                        Err(ShellSubmitError::Busy { active_execution }) => {
                            transcript.update(|entries| {
                                entries.push(TerminalTranscriptEntry::System {
                                    text: format!(
                                        "Another command is already running (execution {}).",
                                        active_execution.0
                                    ),
                                });
                                normalize_terminal_transcript(entries);
                            });
                        }
                        Err(ShellSubmitError::EmptyRequest) => {}
                    }
                }
                None => transcript.update(|entries| {
                    entries.push(TerminalTranscriptEntry::System {
                        text: "Shell session unavailable.".to_string(),
                    });
                    normalize_terminal_transcript(entries);
                }),
            }
        }
    });

    let try_history_navigation = StoredValue::new_local({
        let services = services.clone();
        move |direction: i32| {
            let Some(services) = services.as_ref() else {
                return;
            };
            let history = services.commands.history.get();
            if history.is_empty() {
                return;
            }

            let next_index = match (history_cursor.get_untracked(), direction) {
                (None, -1) => Some(history.len().saturating_sub(1)),
                (Some(index), -1) if index > 0 => Some(index - 1),
                (Some(index), 1) if index + 1 < history.len() => Some(index + 1),
                (Some(_), 1) => None,
                (current, _) => current,
            };

            history_cursor.set(next_index);
            match next_index {
                Some(index) => input.set(history[index].clone()),
                None => input.set(String::new()),
            }
        }
    });

    let trigger_completion = StoredValue::new_local({
        let shell_session = shell_session.clone();
        move || {
            let Some(shell_session) = shell_session.clone() else {
                return;
            };
            let current_input = input.get_untracked();
            spawn_local(async move {
                match shell_session
                    .complete(completion_request(&cwd.get_untracked(), &current_input))
                    .await
                {
                    Ok(items) => {
                        if items.len() == 1 {
                            let value = items[0].value.clone();
                            input.set(format!("{value} "));
                            suggestions.set(Vec::new());
                        } else {
                            suggestions.set(items);
                        }
                    }
                    Err(err) => {
                        transcript.update(|entries| {
                            entries.push(TerminalTranscriptEntry::System { text: err.message });
                            normalize_terminal_transcript(entries);
                        });
                    }
                }
            });
        }
    });

    let indexed_entries = move || transcript.get().into_iter().enumerate().collect::<Vec<_>>();

    view! {
        <AppShell>
            <TerminalSurface
                role="log"
                aria_live="polite"
                node_ref=terminal_screen
                on:scroll=move |_| {
                    if let Some(screen) = terminal_screen.get() {
                        should_follow_output.set(should_auto_follow(
                            screen.scroll_height(),
                            screen.scroll_top(),
                            screen.client_height(),
                            AUTO_FOLLOW_THRESHOLD_PX,
                        ));
                    }
                }
            >
                <Show when=move || !suggestions.get().is_empty() fallback=|| ()>
                    <CompletionList role="listbox" aria_label="Completions">
                        <For each=move || suggestions.get() key=|item| item.value.clone() let:item>
                            <CompletionOption
                                on_click=Callback::new(move |_| {
                                    input.set(format!("{} ", item.value));
                                    suggestions.set(Vec::new());
                                })
                            >
                                {item.label}
                            </CompletionOption>
                        </For>
                    </CompletionList>
                </Show>

                <TerminalTranscript>
                    <For each=indexed_entries key=|(idx, _)| *idx let:entry>
                        {render_entry(entry.1)}
                    </For>

                    <TerminalPrompt>
                        <label hidden for=input_id.clone()>
                            {move || format!("Command input for {} in {} mode", cwd.get(), prompt_mode())}
                        </label>
                        <div aria-hidden="true">
                            <span>{move || cwd.get()}</span>
                            <span>{move || prompt_mode()}</span>
                            <span>"\u{203a}"</span>
                        </div>
                        <TextField
                            id=input_id.clone()
                            input_type="text"
                            value=Signal::derive(move || input.get())
                            autocomplete="off"
                            spellcheck=false
                            aria_label="Terminal command input"
                            on_input=Callback::new(move |ev| {
                                input.set(event_target_value(&ev));
                                suggestions.set(Vec::new());
                            })
                            on_keydown=Callback::new(move |ev: KeyboardEvent| match ev.key().as_str() {
                                "Enter" => {
                                    ev.prevent_default();
                                    ev.stop_propagation();
                                    submit_command
                                        .with_value(|submit_command| submit_command(input.get_untracked()));
                                }
                                "ArrowUp" => {
                                    ev.prevent_default();
                                    try_history_navigation
                                        .with_value(|try_history_navigation| try_history_navigation(-1));
                                }
                                "ArrowDown" => {
                                    ev.prevent_default();
                                    try_history_navigation
                                        .with_value(|try_history_navigation| try_history_navigation(1));
                                }
                                "Tab" => {
                                    ev.prevent_default();
                                    trigger_completion
                                        .with_value(|trigger_completion| trigger_completion());
                                }
                                "Escape" => suggestions.set(Vec::new()),
                                "c" | "C" if ev.ctrl_key() => {
                                    shell_session_handle.with_value(|shell_session| {
                                        if let Some(shell_session) = shell_session.clone() {
                                            ev.prevent_default();
                                            shell_session.cancel();
                                        }
                                    });
                                }
                                "l" | "L" if ev.ctrl_key() => {
                                    ev.prevent_default();
                                    transcript.set(default_terminal_transcript());
                                }
                                _ => {}
                            })
                        />
                    </TerminalPrompt>
                </TerminalTranscript>
            </TerminalSurface>
        </AppShell>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use system_shell_contract::{CommandNoticeLevel, ShellExecutionSummary, ShellExit};

    fn system_texts(transcript: &[TerminalTranscriptEntry]) -> Vec<&str> {
        transcript
            .iter()
            .filter_map(|entry| match entry {
                TerminalTranscriptEntry::System { text } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn restore_terminal_state_marks_interrupted_execution_and_bounds_history() {
        let transcript = (0..MAX_TERMINAL_ENTRIES)
            .map(|index| TerminalTranscriptEntry::System {
                text: format!("entry-{index}"),
            })
            .collect::<Vec<_>>();
        let restored = restore_terminal_state(
            TerminalPersistedState {
                cwd: String::new(),
                input: "ls".to_string(),
                transcript,
                history_cursor: None,
                active_execution: Some(PersistedExecutionState {
                    execution_id: ExecutionId(9),
                    command: "ls".to_string(),
                }),
            },
            "~/desktop",
        );

        assert_eq!(restored.cwd, "~/desktop");
        assert!(restored.active_execution.is_none());
        assert_eq!(restored.transcript.len(), MAX_TERMINAL_ENTRIES);
        assert_eq!(
            restored.transcript.last(),
            Some(&TerminalTranscriptEntry::System {
                text: "Previous command interrupted during restore.".to_string(),
            })
        );
    }

    #[test]
    fn apply_shell_session_events_reports_eviction_and_tracks_sequences() {
        let mut transcript = default_terminal_transcript();
        let mut active_execution = None;
        let mut pending_command = Some("help list".to_string());
        let processed = apply_shell_session_events(
            &mut transcript,
            &mut active_execution,
            &mut pending_command,
            2,
            &[
                system_shell_contract::SequencedShellStreamEvent {
                    sequence: 5,
                    event: ShellStreamEvent::Started {
                        execution_id: ExecutionId(41),
                    },
                },
                system_shell_contract::SequencedShellStreamEvent {
                    sequence: 6,
                    event: ShellStreamEvent::Notice {
                        execution_id: ExecutionId(41),
                        notice: CommandNotice {
                            level: CommandNoticeLevel::Info,
                            message: "listing help".to_string(),
                        },
                    },
                },
                system_shell_contract::SequencedShellStreamEvent {
                    sequence: 7,
                    event: ShellStreamEvent::Completed {
                        summary: ShellExecutionSummary {
                            execution_id: ExecutionId(41),
                            command_path: None,
                            exit: ShellExit::success(),
                        },
                    },
                },
            ],
        );

        assert_eq!(processed, 7);
        assert!(active_execution.is_none());
        assert!(pending_command.is_none());
        assert!(
            system_texts(&transcript)
                .contains(&"Older shell session events were evicted from the in-memory log.")
        );
        assert!(transcript.iter().any(|entry| matches!(
            entry,
            TerminalTranscriptEntry::Notice { execution_id, notice }
                if *execution_id == ExecutionId(41) && notice.message == "listing help"
        )));
    }
}
