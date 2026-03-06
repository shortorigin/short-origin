use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use dev_console_core::{
    ActionOutcome, ActionPlan, ConfirmationRequirement, DashboardState, DevConsoleService,
    DxDomain, JourneyKind, ReleaseInputs, ShellContext,
};

/// Main application tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Overview dashboard.
    Home,
    /// Prerequisite setup.
    Setup,
    /// Branch, issue, and PR state.
    Work,
    /// Task execution and filtering.
    Tasks,
    /// GitHub Actions runs.
    Workflows,
    /// Release-candidate and production promotion flows.
    Release,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Setup => "Setup",
            Self::Work => "Work",
            Self::Tasks => "Tasks",
            Self::Workflows => "Workflows",
            Self::Release => "Release",
        }
    }

    pub fn all() -> [Self; 6] {
        [
            Self::Home,
            Self::Setup,
            Self::Work,
            Self::Tasks,
            Self::Workflows,
            Self::Release,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseField {
    RcTag,
    TargetSha,
    PromoteRcTag,
    ReleaseTag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    TasksFilter,
    ReleaseField(ReleaseField),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    Confirm(ActionPlan),
    Typed {
        plan: ActionPlan,
        expected: String,
        typed: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionState {
    pub title: String,
    pub logs: Vec<String>,
    pub running: bool,
    pub last_outcome: Option<ActionOutcome>,
}

impl ExecutionState {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            logs: Vec::new(),
            running: true,
            last_outcome: None,
        }
    }
}

#[derive(Debug)]
pub enum WorkerEvent {
    Output(String),
    Finished(ActionOutcome),
}

/// Mutable Ratatui application state.
pub struct App {
    workspace_root: PathBuf,
    pub(crate) screen: Screen,
    input_mode: InputMode,
    pub(crate) dashboard: DashboardState,
    setup_index: usize,
    task_index: usize,
    workflow_index: usize,
    release_index: usize,
    tasks_filter: String,
    release_inputs: ReleaseInputs,
    pub(crate) pending_action: Option<PendingAction>,
    pub(crate) execution: Option<ExecutionState>,
    worker_rx: Option<Receiver<WorkerEvent>>,
}

impl App {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            screen: Screen::Home,
            input_mode: InputMode::Normal,
            dashboard: DashboardState::default(),
            setup_index: 0,
            task_index: 0,
            workflow_index: 0,
            release_index: 0,
            tasks_filter: String::new(),
            release_inputs: ReleaseInputs::default(),
            pending_action: None,
            execution: None,
            worker_rx: None,
        }
    }

    pub fn dashboard(&self) -> &DashboardState {
        &self.dashboard
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn input_mode(&self) -> &InputMode {
        &self.input_mode
    }

    pub fn tasks_filter(&self) -> &str {
        &self.tasks_filter
    }

    pub fn setup_index(&self) -> usize {
        self.setup_index
    }

    pub fn task_index(&self) -> usize {
        self.task_index
    }

    pub fn workflow_index(&self) -> usize {
        self.workflow_index
    }

    pub fn release_index(&self) -> usize {
        self.release_index
    }

    pub fn release_inputs(&self) -> &ReleaseInputs {
        &self.release_inputs
    }

    pub fn pending_action(&self) -> Option<&PendingAction> {
        self.pending_action.as_ref()
    }

    pub fn execution(&self) -> Option<&ExecutionState> {
        self.execution.as_ref()
    }

    pub fn refresh(&mut self) {
        let context = ShellContext::new(self.workspace_root.clone());
        let service =
            DevConsoleService::new(context.clone(), context.clone(), context.clone(), context);
        self.dashboard = service.refresh(self.release_inputs.clone());
        if self.release_inputs.target_sha.is_empty() {
            self.release_inputs.target_sha = self.dashboard.release.rc_request.target_sha.clone();
        }
        self.clamp_selection();
    }

    pub fn drain_events(&mut self) {
        let Some(receiver) = &self.worker_rx else {
            return;
        };
        let mut finished = None;
        while let Ok(event) = receiver.try_recv() {
            match event {
                WorkerEvent::Output(line) => {
                    if let Some(execution) = &mut self.execution {
                        execution.logs.push(line);
                        if execution.logs.len() > 200 {
                            let overflow = execution.logs.len() - 200;
                            execution.logs.drain(0..overflow);
                        }
                    }
                }
                WorkerEvent::Finished(outcome) => {
                    if let Some(execution) = &mut self.execution {
                        execution.running = false;
                        execution.last_outcome = Some(outcome.clone());
                        execution.logs.push(outcome.summary.clone());
                    }
                    finished = Some(outcome);
                }
            }
        }

        if let Some(outcome) = finished {
            self.worker_rx = None;
            if outcome.success {
                self.refresh();
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }

        match self.input_mode.clone() {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::TasksFilter => self.handle_filter_key(key),
            InputMode::ReleaseField(field) => self.handle_release_input_key(key, field),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        if self.pending_action.is_some() {
            return self.handle_pending_action_key(key);
        }

        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Tab => self.next_screen(),
            KeyCode::BackTab => self.previous_screen(),
            KeyCode::Char('1') => self.screen = Screen::Home,
            KeyCode::Char('2') => self.screen = Screen::Setup,
            KeyCode::Char('3') => self.screen = Screen::Work,
            KeyCode::Char('4') => self.screen = Screen::Tasks,
            KeyCode::Char('5') => self.screen = Screen::Workflows,
            KeyCode::Char('6') => self.screen = Screen::Release,
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('/') if self.screen == Screen::Tasks => {
                self.input_mode = InputMode::TasksFilter;
            }
            KeyCode::Char('e') if self.screen == Screen::Release => self.begin_release_edit(),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Enter => self.activate_selected(),
            _ => {}
        }
        false
    }

    fn handle_filter_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => self.input_mode = InputMode::Normal,
            KeyCode::Backspace => {
                self.tasks_filter.pop();
                self.clamp_selection();
            }
            KeyCode::Char(ch) => {
                self.tasks_filter.push(ch);
                self.clamp_selection();
            }
            _ => {}
        }
        false
    }

    fn handle_release_input_key(&mut self, key: KeyEvent, field: ReleaseField) -> bool {
        let target = match field {
            ReleaseField::RcTag => &mut self.release_inputs.rc_tag,
            ReleaseField::TargetSha => &mut self.release_inputs.target_sha,
            ReleaseField::PromoteRcTag => &mut self.release_inputs.promote_rc_tag,
            ReleaseField::ReleaseTag => &mut self.release_inputs.release_tag,
        };
        match key.code {
            KeyCode::Esc => self.input_mode = InputMode::Normal,
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.refresh();
            }
            KeyCode::Backspace => {
                target.pop();
            }
            KeyCode::Char(ch) => target.push(ch),
            _ => {}
        }
        false
    }

    fn handle_pending_action_key(&mut self, key: KeyEvent) -> bool {
        let Some(pending) = self.pending_action.clone() else {
            return false;
        };

        match pending {
            PendingAction::Confirm(plan) => match key.code {
                KeyCode::Esc => self.pending_action = None,
                KeyCode::Enter => self.start_action(plan),
                _ => {}
            },
            PendingAction::Typed {
                plan,
                expected,
                mut typed,
            } => match key.code {
                KeyCode::Esc => self.pending_action = None,
                KeyCode::Backspace => {
                    typed.pop();
                    self.pending_action = Some(PendingAction::Typed {
                        plan,
                        expected,
                        typed,
                    });
                }
                KeyCode::Enter => {
                    if typed == expected {
                        self.start_action(plan);
                    } else {
                        self.pending_action = Some(PendingAction::Typed {
                            plan,
                            expected,
                            typed,
                        });
                    }
                }
                KeyCode::Char(ch) => {
                    typed.push(ch);
                    self.pending_action = Some(PendingAction::Typed {
                        plan,
                        expected,
                        typed,
                    });
                }
                _ => {}
            },
        }
        false
    }

    fn next_screen(&mut self) {
        let all = Screen::all();
        let current = all
            .iter()
            .position(|screen| *screen == self.screen)
            .unwrap_or(0);
        self.screen = all[(current + 1) % all.len()];
    }

    fn previous_screen(&mut self) {
        let all = Screen::all();
        let current = all
            .iter()
            .position(|screen| *screen == self.screen)
            .unwrap_or(0);
        self.screen = all[(current + all.len() - 1) % all.len()];
    }

    fn move_selection(&mut self, delta: isize) {
        match self.screen {
            Screen::Setup => {
                adjust_index(&mut self.setup_index, self.dashboard.doctor.len(), delta);
            }
            Screen::Tasks => {
                let filtered_len = self.filtered_tasks().len();
                adjust_index(&mut self.task_index, filtered_len, delta);
            }
            Screen::Workflows => adjust_index(
                &mut self.workflow_index,
                self.dashboard.workflows.runs.len(),
                delta,
            ),
            Screen::Release => adjust_index(&mut self.release_index, 6, delta),
            Screen::Home | Screen::Work => {}
        }
    }

    fn clamp_selection(&mut self) {
        if self.setup_index >= self.dashboard.doctor.len() {
            self.setup_index = self.dashboard.doctor.len().saturating_sub(1);
        }
        let filtered = self.filtered_tasks();
        if self.task_index >= filtered.len() {
            self.task_index = filtered.len().saturating_sub(1);
        }
        if self.workflow_index >= self.dashboard.workflows.runs.len() {
            self.workflow_index = self.dashboard.workflows.runs.len().saturating_sub(1);
        }
    }

    pub fn filtered_tasks(&self) -> Vec<&dev_console_core::TaskInfo> {
        let needle = self.tasks_filter.to_lowercase();
        self.dashboard
            .tasks
            .tasks
            .iter()
            .filter(|task| {
                needle.is_empty()
                    || task.id.to_lowercase().contains(&needle)
                    || task.description.to_lowercase().contains(&needle)
            })
            .collect()
    }

    fn begin_release_edit(&mut self) {
        self.input_mode = match self.release_index {
            0 => InputMode::ReleaseField(ReleaseField::RcTag),
            1 => InputMode::ReleaseField(ReleaseField::TargetSha),
            3 => InputMode::ReleaseField(ReleaseField::PromoteRcTag),
            4 => InputMode::ReleaseField(ReleaseField::ReleaseTag),
            _ => InputMode::Normal,
        };
    }

    fn activate_selected(&mut self) {
        let action = match self.screen {
            Screen::Home => self.dashboard.recommended_action.clone(),
            Screen::Setup => self
                .dashboard
                .doctor
                .get(self.setup_index)
                .and_then(|snapshot| {
                    self.dashboard
                        .journeys
                        .iter()
                        .find(|journey| journey.kind == JourneyKind::Setup)
                        .and_then(|journey| {
                            journey
                                .steps
                                .iter()
                                .find(|step| step.title.starts_with(snapshot.domain.label()))
                                .and_then(|step| step.action.clone())
                        })
                })
                .or_else(|| {
                    self.dashboard
                        .doctor
                        .get(self.setup_index)
                        .map(|snapshot| recheck_domain_action(snapshot.domain))
                }),
            Screen::Work => self.work_action(),
            Screen::Tasks => self
                .filtered_tasks()
                .get(self.task_index)
                .map(|task| task_action(task)),
            Screen::Workflows => self.workflow_action(),
            Screen::Release => self.release_action(),
        };

        if let Some(action) = action {
            self.queue_action(action);
        }
    }

    fn queue_action(&mut self, action: ActionPlan) {
        self.pending_action = match action.confirmation.clone() {
            ConfirmationRequirement::None => {
                self.start_action(action);
                None
            }
            ConfirmationRequirement::Confirm => Some(PendingAction::Confirm(action)),
            ConfirmationRequirement::TypedMatch { expected } => Some(PendingAction::Typed {
                plan: action,
                expected,
                typed: String::new(),
            }),
        };
    }

    fn start_action(&mut self, action: ActionPlan) {
        self.pending_action = None;
        let (sender, receiver) = mpsc::channel();
        self.execution = Some(ExecutionState::new(action.title.clone()));
        self.worker_rx = Some(receiver);

        let workspace_root = self.workspace_root.clone();
        thread::spawn(move || run_action(action, workspace_root, sender));
    }

    fn work_action(&self) -> Option<ActionPlan> {
        if let Some(pr) = &self.dashboard.work.pull_request {
            return Some(ActionPlan {
                id: "work-pr-view".to_string(),
                title: format!("View PR #{}", pr.number),
                description: "Inspect the current pull request.".to_string(),
                command_preview: format!("gh pr view {}", pr.number),
                command: dev_console_core::ShellCommand {
                    program: "gh".to_string(),
                    args: vec!["pr".to_string(), "view".to_string(), pr.number.to_string()],
                    cwd: None,
                    env: Vec::new(),
                },
                risk: dev_console_core::ActionRisk::Low,
                confirmation: ConfirmationRequirement::Confirm,
                refresh_after: false,
            });
        }
        self.dashboard.work.issue.as_ref().map(|issue| ActionPlan {
            id: "work-issue-view".to_string(),
            title: format!("View issue #{}", issue.number),
            description: "Inspect the linked issue.".to_string(),
            command_preview: format!("gh issue view {}", issue.number),
            command: dev_console_core::ShellCommand {
                program: "gh".to_string(),
                args: vec![
                    "issue".to_string(),
                    "view".to_string(),
                    issue.number.to_string(),
                ],
                cwd: None,
                env: Vec::new(),
            },
            risk: dev_console_core::ActionRisk::Low,
            confirmation: ConfirmationRequirement::Confirm,
            refresh_after: false,
        })
    }

    fn workflow_action(&self) -> Option<ActionPlan> {
        self.dashboard
            .workflows
            .runs
            .get(self.workflow_index)
            .map(|run| {
                let failed = matches!(
                    run.conclusion.as_deref(),
                    Some("failure" | "cancelled" | "timed_out" | "startup_failure")
                );
                if failed {
                    ActionPlan {
                        id: format!("workflow-rerun-{}", run.id),
                        title: format!("Rerun {}", run.workflow_name),
                        description: "Rerun the selected GitHub Actions workflow.".to_string(),
                        command_preview: format!("gh run rerun {}", run.id),
                        command: dev_console_core::ShellCommand {
                            program: "gh".to_string(),
                            args: vec!["run".to_string(), "rerun".to_string(), run.id.to_string()],
                            cwd: None,
                            env: Vec::new(),
                        },
                        risk: dev_console_core::ActionRisk::Medium,
                        confirmation: ConfirmationRequirement::Confirm,
                        refresh_after: true,
                    }
                } else {
                    ActionPlan {
                        id: format!("workflow-view-{}", run.id),
                        title: format!("View {}", run.workflow_name),
                        description: "Inspect the selected GitHub Actions run.".to_string(),
                        command_preview: format!("gh run view {}", run.id),
                        command: dev_console_core::ShellCommand {
                            program: "gh".to_string(),
                            args: vec!["run".to_string(), "view".to_string(), run.id.to_string()],
                            cwd: None,
                            env: Vec::new(),
                        },
                        risk: dev_console_core::ActionRisk::Low,
                        confirmation: ConfirmationRequirement::Confirm,
                        refresh_after: false,
                    }
                }
            })
    }

    fn release_action(&self) -> Option<ActionPlan> {
        match self.release_index {
            2 => self.dashboard.release.rc_preflight.action.clone(),
            5 => self.dashboard.release.promote_preflight.action.clone(),
            _ => None,
        }
    }
}

fn adjust_index(index: &mut usize, len: usize, delta: isize) {
    if len == 0 {
        *index = 0;
        return;
    }
    if delta.is_negative() {
        *index = index.saturating_sub(delta.unsigned_abs());
    } else {
        *index = index.saturating_add(delta.unsigned_abs());
    }
    *index = (*index).min(len.saturating_sub(1));
}

fn recheck_domain_action(domain: DxDomain) -> ActionPlan {
    ActionPlan {
        id: format!("doctor-{}", domain.label()),
        title: format!("Recheck {}", domain.label()),
        description: "Re-run doctor for the selected domain.".to_string(),
        command_preview: format!("cargo xtask doctor --domain {}", domain.label()),
        command: dev_console_core::ShellCommand {
            program: "cargo".to_string(),
            args: vec![
                "xtask".to_string(),
                "doctor".to_string(),
                "--domain".to_string(),
                domain.label().to_string(),
            ],
            cwd: None,
            env: Vec::new(),
        },
        risk: dev_console_core::ActionRisk::Low,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: true,
    }
}

fn task_action(task: &dev_console_core::TaskInfo) -> ActionPlan {
    ActionPlan {
        id: format!("task-{}", task.id),
        title: format!("Run {}", task.id),
        description: task.description.clone(),
        command_preview: format!("cargo xtask run {}", task.id),
        command: dev_console_core::ShellCommand {
            program: "cargo".to_string(),
            args: vec!["xtask".to_string(), "run".to_string(), task.id.clone()],
            cwd: None,
            env: Vec::new(),
        },
        risk: dev_console_core::ActionRisk::Low,
        confirmation: ConfirmationRequirement::Confirm,
        refresh_after: true,
    }
}

fn run_action(action: ActionPlan, workspace_root: PathBuf, sender: Sender<WorkerEvent>) {
    let mut command = Command::new(&action.command.program);
    command.args(action.command.args.iter());
    command.current_dir(
        action
            .command
            .cwd
            .as_deref()
            .map_or(workspace_root.as_path(), Path::new),
    );
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    for (key, value) in &action.command.env {
        command.env(key, value);
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = sender.send(WorkerEvent::Finished(ActionOutcome {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: error.to_string(),
                summary: format!("Failed to start {}", action.title),
            }));
            return;
        }
    };

    if let Some(stdout) = child.stdout.take() {
        spawn_stream_reader(stdout, sender.clone(), "stdout");
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_stream_reader(stderr, sender.clone(), "stderr");
    }

    let status = child.wait();
    let _ = sender.send(WorkerEvent::Finished(match status {
        Ok(status) => ActionOutcome {
            success: status.success(),
            exit_code: status.code(),
            stdout: String::new(),
            stderr: String::new(),
            summary: if status.success() {
                format!("Completed {}", action.title)
            } else {
                format!("{} exited with {}", action.title, status)
            },
        },
        Err(error) => ActionOutcome {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
            summary: format!("Failed {}", action.title),
        },
    }));
}

fn spawn_stream_reader<R>(reader: R, sender: Sender<WorkerEvent>, label: &'static str)
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            let _ = sender.send(WorkerEvent::Output(format!("[{label}] {line}")));
        }
    });
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use dev_console_core::{
        ActionRisk, DoctorSnapshot, RepoSnapshot, TaskCatalogSnapshot, WorkSnapshot,
        WorkflowRunSnapshot,
    };

    use super::{App, ExecutionState, InputMode, PendingAction, ReleaseField, Screen};

    #[test]
    fn app_starts_on_home_screen() {
        let app = App::new(PathBuf::from("."));
        assert_eq!(app.screen(), Screen::Home);
        assert!(matches!(app.input_mode(), InputMode::Normal));
    }

    #[test]
    fn execution_state_tracks_logs() {
        let mut state = ExecutionState::new("Verify");
        state.logs.push("hello".to_string());
        assert_eq!(state.logs.len(), 1);
    }

    #[test]
    fn release_edit_mode_switches_for_target_field() {
        let mut app = App::new(PathBuf::from("."));
        app.screen = Screen::Release;
        app.release_index = 1;
        app.begin_release_edit();
        assert!(matches!(
            app.input_mode(),
            InputMode::ReleaseField(ReleaseField::TargetSha)
        ));
    }

    #[test]
    fn pending_typed_action_preserves_expected_value() {
        let pending = PendingAction::Typed {
            plan: dev_console_core::ActionPlan {
                risk: ActionRisk::High,
                ..Default::default()
            },
            expected: "v1.0.0".to_string(),
            typed: String::new(),
        };
        match pending {
            PendingAction::Typed { expected, .. } => assert_eq!(expected, "v1.0.0"),
            PendingAction::Confirm(_) => panic!("unexpected confirm"),
        }
    }

    #[test]
    fn dashboard_accessors_expose_state() {
        let app = App::new(PathBuf::from("."));
        let _ = (
            app.dashboard(),
            app.execution(),
            app.pending_action(),
            app.release_inputs(),
            app.setup_index(),
            app.task_index(),
            app.workflow_index(),
            app.release_index(),
        );
        let _ = (
            RepoSnapshot::default(),
            TaskCatalogSnapshot::default(),
            WorkSnapshot::default(),
            WorkflowRunSnapshot::default(),
            Vec::<DoctorSnapshot>::new(),
        );
    }
}
