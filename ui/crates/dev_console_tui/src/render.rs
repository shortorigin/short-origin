use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
};
use ratatui::Frame;

use crate::app::{App, InputMode, PendingAction, Screen};
use dev_console_core::CheckStatus;

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let vertical = if app.execution().is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(12),
                Constraint::Length(10),
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(12)])
            .split(frame.area())
    };

    render_tabs(frame, vertical[0], app);
    match app.screen() {
        Screen::Home => render_home(frame, vertical[1], app),
        Screen::Setup => render_setup(frame, vertical[1], app),
        Screen::Work => render_work(frame, vertical[1], app),
        Screen::Tasks => render_tasks(frame, vertical[1], app),
        Screen::Workflows => render_workflows(frame, vertical[1], app),
        Screen::Release => render_release(frame, vertical[1], app),
    }

    if vertical.len() > 2 {
        render_logs(frame, vertical[2], app);
    }
    render_overlays(frame, app);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let titles = Screen::all()
        .iter()
        .map(|screen| Line::from(Span::raw(screen.title())))
        .collect::<Vec<_>>();
    let selected = Screen::all()
        .iter()
        .position(|screen| *screen == app.screen())
        .unwrap_or(0);
    let block = Block::default()
        .title("Short Origin DX Console")
        .borders(Borders::ALL);
    let tabs = Tabs::new(titles)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(selected)
        .divider(symbols::DOT);
    frame.render_widget(tabs, area);
}

fn render_home(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(columns[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(8)])
        .split(columns[1]);

    let repo = app.dashboard().repo.clone();
    let work = &app.dashboard().work;
    let repo_lines = vec![
        Line::from(format!("Branch: {}", repo.branch)),
        Line::from(format!(
            "Upstream: {}",
            repo.upstream_branch.unwrap_or_else(|| "-".to_string())
        )),
        Line::from(format!(
            "Changes: staged={} unstaged={} untracked={}",
            repo.staged_files, repo.unstaged_files, repo.untracked_files
        )),
        Line::from(format!("Ahead/behind: +{} / -{}", repo.ahead, repo.behind)),
        Line::from(format!(
            "Issue: {}",
            repo.inferred_issue_id
                .map_or_else(|| "unlinked".to_string(), |value| format!("#{value}"))
        )),
    ];
    frame.render_widget(
        Paragraph::new(repo_lines)
            .block(Block::default().title("Repo Health").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        left[0],
    );

    let journey_rows = app
        .dashboard()
        .journeys
        .iter()
        .map(|journey| {
            Row::new(vec![
                Cell::from(journey.kind.label()),
                Cell::from(format!("{}%", journey.score)),
                Cell::from(
                    journey
                        .next_action
                        .as_ref()
                        .map_or_else(|| "Stable".to_string(), |action| action.title.clone()),
                ),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Table::new(
            journey_rows,
            [
                Constraint::Length(12),
                Constraint::Length(8),
                Constraint::Min(24),
            ],
        )
        .header(
            Row::new(vec!["Journey", "Score", "Next action"])
                .style(Style::default().fg(Color::Cyan)),
        )
        .block(
            Block::default()
                .title("Guided Journeys")
                .borders(Borders::ALL),
        ),
        left[1],
    );

    let work_lines = vec![
        Line::from(format!(
            "GitHub: {}",
            match work.auth_status {
                dev_console_core::GithubAuthStatus::Authenticated => "authenticated",
                dev_console_core::GithubAuthStatus::Unavailable => "unavailable",
            }
        )),
        Line::from(work.issue.as_ref().map_or_else(
            || "Issue: none".to_string(),
            |issue| format!("Issue: #{} {}", issue.number, issue.title),
        )),
        Line::from(work.pull_request.as_ref().map_or_else(
            || "PR: none".to_string(),
            |pr| format!("PR: #{} {}", pr.number, pr.title),
        )),
        Line::from(app.dashboard().recommended_action.as_ref().map_or_else(
            || "Next best action: review dashboard".to_string(),
            |action| format!("Next best action: {}", action.title),
        )),
    ];
    frame.render_widget(
        Paragraph::new(work_lines)
            .block(Block::default().title("Current Work").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        right[0],
    );

    let alerts = if app.dashboard().alerts.is_empty() {
        vec![ListItem::new("No refresh alerts.")]
    } else {
        app.dashboard()
            .alerts
            .iter()
            .map(|alert| ListItem::new(alert.clone()))
            .collect::<Vec<_>>()
    };
    frame.render_widget(
        List::new(alerts).block(Block::default().title("Alerts").borders(Borders::ALL)),
        right[1],
    );
}

fn render_setup(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    let items = app
        .dashboard()
        .doctor
        .iter()
        .enumerate()
        .map(|(index, snapshot)| {
            let prefix = if index == app.setup_index() {
                "> "
            } else {
                "  "
            };
            let label = if snapshot.missing_required {
                format!(
                    "{prefix}{} [{} missing]",
                    snapshot.domain.label(),
                    snapshot.missing_count()
                )
            } else {
                format!("{prefix}{} [ready]", snapshot.domain.label())
            };
            ListItem::new(label)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::default().title("Domains").borders(Borders::ALL)),
        columns[0],
    );

    if let Some(snapshot) = app.dashboard().doctor.get(app.setup_index()) {
        let rows = snapshot
            .entries
            .iter()
            .map(|entry| {
                Row::new(vec![
                    Cell::from(entry.tool.clone()),
                    Cell::from(match entry.status {
                        CheckStatus::Ok => "ok",
                        CheckStatus::Warn => "warn",
                        CheckStatus::Missing => "missing",
                    }),
                    Cell::from(entry.guidance.clone()),
                ])
            })
            .collect::<Vec<_>>();
        frame.render_widget(
            Table::new(
                rows,
                [
                    Constraint::Length(18),
                    Constraint::Length(10),
                    Constraint::Min(28),
                ],
            )
            .header(
                Row::new(vec!["Tool", "Status", "Guidance"])
                    .style(Style::default().fg(Color::Cyan)),
            )
            .block(
                Block::default()
                    .title(format!("{} doctor", snapshot.domain.label()))
                    .borders(Borders::ALL),
            ),
            columns[1],
        );
    }
}

fn render_work(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);
    let repo = &app.dashboard().repo;
    let work = &app.dashboard().work;

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("Branch head: {}", repo.pr_head_branch())),
            Line::from(format!("Changed paths: {}", repo.changed_paths.len())),
            Line::from(format!(
                "Working tree: {} local change(s)",
                repo.dirty_files()
            )),
            Line::from(format!(
                "Issue hint: {}",
                repo.inferred_issue_id
                    .map_or_else(|| "none".to_string(), |value| format!("#{value}"))
            )),
        ])
        .block(
            Block::default()
                .title("Branch Context")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true }),
        columns[0],
    );

    let lines = vec![
        Line::from(work.issue.as_ref().map_or_else(
            || "Issue: none".to_string(),
            |issue| format!("Issue #{} [{}] {}", issue.number, issue.state, issue.title),
        )),
        Line::from(
            work.issue
                .as_ref()
                .and_then(|issue| issue.project_status.clone())
                .map_or_else(
                    || "Project status: unavailable".to_string(),
                    |status| format!("Project status: {status}"),
                ),
        ),
        Line::from(work.pull_request.as_ref().map_or_else(
            || "PR: none".to_string(),
            |pr| {
                format!(
                    "PR #{} [{}] {}",
                    pr.number,
                    pr.review_decision
                        .clone()
                        .unwrap_or_else(|| pr.state.clone()),
                    pr.title
                )
            },
        )),
        Line::from(format!(
            "Recommended tasks: {}",
            app.dashboard()
                .recommended_tasks
                .iter()
                .map(|task| task.id.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title("GitHub Work State")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        columns[1],
    );
}

fn render_tasks(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let filter_title = match app.input_mode() {
        InputMode::TasksFilter => format!("Tasks Filter (editing: {})", app.tasks_filter()),
        InputMode::Normal | InputMode::ReleaseField(_) => {
            format!("Tasks Filter (/ to edit): {}", app.tasks_filter())
        }
    };
    let tasks = app.filtered_tasks();
    let items = tasks
        .iter()
        .enumerate()
        .map(|(index, task)| {
            let marker = if index == app.task_index() {
                "> "
            } else {
                "  "
            };
            let label = format!("{marker}{} [{}]", task.id, task.domains.join(","));
            ListItem::new(label)
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(items).block(Block::default().title(filter_title).borders(Borders::ALL)),
        columns[0],
    );

    if let Some(task) = tasks.get(app.task_index()) {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(task.description.clone()),
                Line::from(format!("Prereqs: {}", task.prerequisites.join(", "))),
                Line::from(format!("Deps: {}", task.dependencies.join(", "))),
                Line::from(format!(
                    "In CI: {}",
                    if task.ci_included { "yes" } else { "no" }
                )),
            ])
            .block(Block::default().title("Task Detail").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
            columns[1],
        );
    }
}

fn render_workflows(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let rows = app
        .dashboard()
        .workflows
        .runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let selected_style = if index == app.workflow_index() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(run.workflow_name.clone()),
                Cell::from(run.status.clone()),
                Cell::from(run.conclusion.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(run.head_branch.clone()),
            ])
            .style(selected_style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(12),
                Constraint::Length(18),
                Constraint::Min(22),
            ],
        )
        .header(
            Row::new(vec!["Workflow", "Status", "Conclusion", "Branch"])
                .style(Style::default().fg(Color::Cyan)),
        )
        .block(
            Block::default()
                .title("Recent Workflow Runs (Enter to rerun/view)")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_release(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Min(6),
        ])
        .split(area);

    render_release_card(
        frame,
        vertical[0],
        "Release Candidate",
        &[
            (
                "rc_tag",
                &app.release_inputs().rc_tag,
                app.release_index() == 0,
            ),
            (
                "target_sha",
                &app.release_inputs().target_sha,
                app.release_index() == 1,
            ),
        ],
        &app.dashboard().release.rc_preflight.blockers,
        &app.dashboard().release.rc_preflight.warnings,
        app.dashboard().release.rc_preflight.ready,
        app.release_index() == 2,
        "Dispatch Release Candidate",
    );
    render_release_card(
        frame,
        vertical[1],
        "Promote Release",
        &[
            (
                "rc_tag",
                &app.release_inputs().promote_rc_tag,
                app.release_index() == 3,
            ),
            (
                "release_tag",
                &app.release_inputs().release_tag,
                app.release_index() == 4,
            ),
        ],
        &app.dashboard().release.promote_preflight.blockers,
        &app.dashboard().release.promote_preflight.warnings,
        app.dashboard().release.promote_preflight.ready,
        app.release_index() == 5,
        "Dispatch Promote Release",
    );

    let ready_count = u8::from(app.dashboard().release.rc_preflight.ready)
        + u8::from(app.dashboard().release.promote_preflight.ready);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title("Release Readiness")
                .borders(Borders::ALL),
        )
        .gauge_style(Style::default().fg(Color::Green))
        .label(format!("{ready_count} / 2 ready"))
        .ratio(f64::from(ready_count) / 2.0);
    frame.render_widget(gauge, vertical[2]);
}

fn render_release_card(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    fields: &[(&str, &str, bool)],
    blockers: &[String],
    warnings: &[String],
    ready: bool,
    selected_action: bool,
    action_label: &str,
) {
    let mut lines = fields
        .iter()
        .map(|(label, value, selected)| {
            let prefix = if *selected { "> " } else { "  " };
            Line::from(format!("{prefix}{label}: {value}"))
        })
        .collect::<Vec<_>>();
    lines.push(Line::from(""));
    lines.push(Line::from(format!(
        "{}{}",
        if selected_action { "> " } else { "  " },
        action_label
    )));
    if ready {
        lines.push(Line::from("Ready to dispatch."));
    }
    for blocker in blockers {
        lines.push(Line::from(Span::styled(
            format!("blocker: {blocker}"),
            Style::default().fg(Color::Red),
        )));
    }
    for warning in warnings {
        lines.push(Line::from(Span::styled(
            format!("warn: {warning}"),
            Style::default().fg(Color::Yellow),
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_logs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let lines = app.execution().map_or_else(
        || vec![Line::from("No active execution.")],
        |execution| {
            execution
                .logs
                .iter()
                .rev()
                .take(8)
                .rev()
                .map(|line| Line::from(line.clone()))
                .collect::<Vec<_>>()
        },
    );
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title("Activity Log").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_overlays(frame: &mut Frame<'_>, app: &App) {
    if let Some(pending) = app.pending_action() {
        let area = centered_rect(60, 30, frame.area());
        frame.render_widget(Clear, area);
        let lines = match pending {
            PendingAction::Confirm(plan) => vec![
                Line::from(format!("Confirm action: {}", plan.title)),
                Line::from(plan.command_preview.clone()),
                Line::from("Enter to run, Esc to cancel."),
            ],
            PendingAction::Typed {
                plan,
                expected,
                typed,
            } => vec![
                Line::from(format!("Type `{expected}` to run {}", plan.title)),
                Line::from(plan.command_preview.clone()),
                Line::from(format!("Current input: {typed}")),
            ],
        };
        frame.render_widget(
            Paragraph::new(lines)
                .block(Block::default().title("Confirmation").borders(Borders::ALL))
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::app::{App, ExecutionState, PendingAction, Screen};
    use dev_console_core::{
        ActionPlan, ActionRisk, CheckStatus, ConfirmationRequirement, DashboardState, DoctorEntry,
        DoctorSnapshot, DxDomain, JourneyKind, JourneyState, JourneyStep, JourneyStepStatus,
        ReleasePreflight, ReleaseWizardState, RepoSnapshot, TaskCatalogSnapshot, TaskInfo,
        WorkSnapshot, WorkflowRunSnapshot, WorkflowRunSummary,
    };

    use super::render;

    #[test]
    fn home_dashboard_renders_repo_and_journeys() {
        let mut app = seeded_app();
        app.screen = Screen::Home;
        let output = draw_to_string(&app);
        assert!(output.contains("Repo Health"));
        assert!(output.contains("Guided Journeys"));
    }

    #[test]
    fn setup_screen_renders_missing_tool_blocker() {
        let mut app = seeded_app();
        app.screen = Screen::Setup;
        let output = draw_to_string(&app);
        assert!(output.contains("Domains"));
        assert!(output.contains("missing"));
    }

    #[test]
    fn workflows_screen_renders_failure_list() {
        let mut app = seeded_app();
        app.screen = Screen::Workflows;
        let output = draw_to_string(&app);
        assert!(output.contains("Recent Workflow Runs"));
        assert!(output.contains("startup_failure"));
    }

    #[test]
    fn release_screen_renders_confirmation_modal() {
        let mut app = seeded_app();
        app.screen = Screen::Release;
        app.pending_action = Some(PendingAction::Typed {
            plan: ActionPlan {
                title: "Dispatch Promote Release".to_string(),
                command_preview: "gh workflow run Promote Release".to_string(),
                risk: ActionRisk::High,
                confirmation: ConfirmationRequirement::TypedMatch {
                    expected: "v1.0.0".to_string(),
                },
                ..Default::default()
            },
            expected: "v1.0.0".to_string(),
            typed: "v1".to_string(),
        });
        let output = draw_to_string(&app);
        assert!(output.contains("Confirmation"));
        assert!(output.contains("v1.0.0"));
    }

    fn seeded_app() -> App {
        let mut app = App::new(std::path::PathBuf::from("."));
        app.execution = Some(ExecutionState::new("verify"));
        app.dashboard = DashboardState {
            repo: RepoSnapshot {
                branch: "feature/13-runtime-security".to_string(),
                upstream_branch: Some("origin/feature/13-runtime-security".to_string()),
                head_sha: Some("0123456789abcdef0123456789abcdef01234567".to_string()),
                staged_files: 1,
                unstaged_files: 1,
                untracked_files: 0,
                ahead: 0,
                behind: 0,
                changed_paths: vec!["ui/crates/site/src/web_app.rs".to_string()],
                inferred_issue_id: Some(13),
            },
            doctor: vec![DoctorSnapshot {
                domain: DxDomain::Ui,
                missing_required: true,
                entries: vec![DoctorEntry {
                    tool: "trunk".to_string(),
                    required_by: vec!["ui-verify".to_string()],
                    optional: false,
                    status: CheckStatus::Missing,
                    guidance: "Install trunk".to_string(),
                }],
                notes: vec!["note".to_string()],
            }],
            tasks: TaskCatalogSnapshot {
                tasks: vec![TaskInfo {
                    id: "ui-verify".to_string(),
                    description: "Run UI checks".to_string(),
                    domains: vec!["ui".to_string()],
                    prerequisites: vec!["trunk".to_string()],
                    dependencies: Vec::new(),
                    ci_included: true,
                    listed: true,
                }],
            },
            work: WorkSnapshot::default(),
            workflows: WorkflowRunSnapshot {
                workflows: Vec::new(),
                runs: vec![WorkflowRunSummary {
                    id: 1,
                    workflow_name: "CI".to_string(),
                    display_title: "CI".to_string(),
                    status: "completed".to_string(),
                    conclusion: Some("startup_failure".to_string()),
                    head_branch: "feature/13-runtime-security".to_string(),
                    event: "pull_request".to_string(),
                    created_at: "2026-03-06T15:50:43Z".to_string(),
                    url: None,
                }],
            },
            journeys: vec![JourneyState {
                kind: JourneyKind::Setup,
                score: 10,
                steps: vec![JourneyStep {
                    title: "ui toolchain".to_string(),
                    status: JourneyStepStatus::Blocked,
                    detail: "1 missing".to_string(),
                    action: None,
                }],
                next_action: None,
            }],
            recommended_tasks: Vec::new(),
            recommended_action: None,
            release: ReleaseWizardState {
                rc_preflight: ReleasePreflight {
                    workflow_name: "Release Candidate".to_string(),
                    ready: false,
                    blockers: vec!["Version must match vX.Y.Z-rc.N.".to_string()],
                    warnings: Vec::new(),
                    action: None,
                },
                promote_preflight: ReleasePreflight {
                    workflow_name: "Promote Release".to_string(),
                    ready: false,
                    blockers: vec!["Release tag must match vX.Y.Z.".to_string()],
                    warnings: Vec::new(),
                    action: None,
                },
                ..Default::default()
            },
            alerts: vec!["example alert".to_string()],
        };
        app
    }

    fn draw_to_string(app: &App) -> String {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("terminal");
        terminal
            .draw(|frame| render(frame, app))
            .expect("draw application");
        let buffer = terminal.backend().buffer().clone();
        buffer
            .content
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<Vec<_>>()
            .chunks(120)
            .map(<[&str]>::concat)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
