//! Terminal desktop app UI component backed by the browser-native shell session bridge.
//!
//! The app persists cwd, input, transcript, and active-execution metadata through the runtime and
//! renders typed shell notices, progress, and structured output produced by
//! [`system_shell_contract`].

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

use std::rc::Rc;

use desktop_app_contract::{window_primary_input_dom_id, AppServices, WindowRuntimeId};
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::*;
use platform_host::CapabilityStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use system_shell_contract::{
    CommandNotice, CompletionItem, CompletionRequest, DisplayPreference, ExecutionId, ShellRequest,
    ShellStreamEvent, StructuredData, StructuredRecord, StructuredScalar, StructuredTable,
    StructuredValue,
};
use system_ui::prelude::*;

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

fn render_data(data: StructuredData, _display: DisplayPreference) -> View {
    match data {
        StructuredData::Empty => ().into_view(),
        StructuredData::Value(StructuredValue::Scalar(value)) => {
            view! { <TerminalLine>{scalar_text(&value)}</TerminalLine> }.into_view()
        }
        StructuredData::Value(StructuredValue::Record(record)) | StructuredData::Record(record) => {
            render_record(record).into_view()
        }
        StructuredData::Value(StructuredValue::List(values)) | StructuredData::List(values) => {
            render_list(values).into_view()
        }
        StructuredData::Table(table) => render_table(table).into_view(),
    }
}

fn render_entry(entry: TerminalTranscriptEntry) -> View {
    match entry {
        TerminalTranscriptEntry::Prompt { cwd, command, .. } => view! {
            <TerminalLine tone=TextTone::Secondary>{format!("{cwd} \u{203a} {command}")}</TerminalLine>
        }
        .into_view(),
        TerminalTranscriptEntry::Notice { notice, .. } => view! {
            <TerminalLine tone=TextTone::Accent>{notice.message}</TerminalLine>
        }
        .into_view(),
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
        }
        TerminalTranscriptEntry::System { text } => view! {
            <TerminalLine tone=TextTone::Secondary>{text}</TerminalLine>
        }
        .into_view(),
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
    let cwd = create_rw_signal(launch_cwd.clone());
    let input = create_rw_signal(String::new());
    let transcript = create_rw_signal(default_terminal_transcript());
    let suggestions = create_rw_signal(Vec::<CompletionItem>::new());
    let history_cursor = create_rw_signal::<Option<usize>>(None);
    let active_execution = create_rw_signal::<Option<PersistedExecutionState>>(None);
    let processed_events = create_rw_signal(0usize);
    let pending_command = create_rw_signal::<Option<String>>(None);
    let hydrated = create_rw_signal(false);
    let last_saved = create_rw_signal::<Option<String>>(None);
    let should_follow_output = create_rw_signal(true);
    let terminal_screen = create_node_ref::<html::Div>();
    let prompt_mode = move || {
        if active_execution.get().is_some() {
            "running"
        } else {
            mode_label
        }
    };
    if let Some(restored_state) = restored_state.as_ref() {
        if let Ok(restored) =
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
    }
    transcript.update(|entries| {
        entries.push(TerminalTranscriptEntry::System {
            text: terminal_mode_notice(services.as_ref()).to_string(),
        });
        normalize_terminal_transcript(entries);
    });
    hydrated.set(true);

    create_effect(move |_| {
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
            if let Ok(value) = serde_json::to_value(&snapshot) {
                services.state.persist_window_state(value);
            }
        }
    });

    if let Some(shell_session) = shell_session.clone() {
        create_effect(move |_| {
            let events = shell_session.events.get();
            let already_processed = processed_events.get();
            if already_processed >= events.len() {
                return;
            }

            for event in events.iter().skip(already_processed) {
                match event {
                    ShellStreamEvent::Started { execution_id } => {
                        let command = pending_command.get_untracked().unwrap_or_default();
                        if !command.is_empty() {
                            active_execution.set(Some(PersistedExecutionState {
                                execution_id: *execution_id,
                                command,
                            }));
                            pending_command.set(None);
                        }
                    }
                    ShellStreamEvent::Notice {
                        execution_id,
                        notice,
                    } => transcript.update(|entries| {
                        entries.push(TerminalTranscriptEntry::Notice {
                            notice: notice.clone(),
                            execution_id: *execution_id,
                        });
                        normalize_terminal_transcript(entries);
                    }),
                    ShellStreamEvent::Data {
                        execution_id,
                        data,
                        display,
                    } => transcript.update(|entries| {
                        entries.push(TerminalTranscriptEntry::Data {
                            data: data.clone(),
                            display: *display,
                            execution_id: *execution_id,
                        });
                        normalize_terminal_transcript(entries);
                    }),
                    ShellStreamEvent::Progress {
                        execution_id,
                        value,
                        label,
                    } => transcript.update(|entries| {
                        entries.push(TerminalTranscriptEntry::Progress {
                            execution_id: *execution_id,
                            value: *value,
                            label: label.clone(),
                        });
                        normalize_terminal_transcript(entries);
                    }),
                    ShellStreamEvent::Cancelled { .. } => {
                        active_execution.set(None);
                    }
                    ShellStreamEvent::Completed { .. } => {
                        active_execution.set(None);
                    }
                }
            }

            processed_events.set(events.len());
            cwd.set(shell_session.cwd.get());
        });
    }

    create_effect(move |_| {
        let _transcript_len = transcript.get().len();
        let hydrated = hydrated.get();
        let should_follow_output = should_follow_output.get();
        if !hydrated || !should_follow_output {
            return;
        }

        scroll_terminal_to_bottom(&terminal_screen);
    });

    let submit_command: Rc<dyn Fn(String)> = Rc::new({
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
                    pending_command.set(Some(command.clone()));
                    shell_session.submit(ShellRequest {
                        line: command,
                        cwd: cwd.get_untracked(),
                        source_window_id: None,
                    });
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

    let try_history_navigation: Rc<dyn Fn(i32)> = Rc::new({
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

    let trigger_completion: Rc<dyn Fn()> = Rc::new({
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
                            <CompletionItem
                                on_click=Callback::new(move |_| {
                                    input.set(format!("{} ", item.value));
                                    suggestions.set(Vec::new());
                                })
                            >
                                {item.label}
                            </CompletionItem>
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
                                    submit_command(input.get_untracked());
                                }
                                "ArrowUp" => {
                                    ev.prevent_default();
                                    try_history_navigation(-1);
                                }
                                "ArrowDown" => {
                                    ev.prevent_default();
                                    try_history_navigation(1);
                                }
                                "Tab" => {
                                    ev.prevent_default();
                                    trigger_completion();
                                }
                                "Escape" => suggestions.set(Vec::new()),
                                "c" | "C" if ev.ctrl_key() => {
                                    if let Some(shell_session) = shell_session.clone() {
                                        ev.prevent_default();
                                        shell_session.cancel();
                                    }
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
