//! Ratatui frontend for the developer DX console.

#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod app;
mod render;

use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;

/// Runs the interactive Ratatui application.
pub fn run() -> Result<(), Box<dyn Error>> {
    let workspace_root = std::env::current_dir()?;
    let mut app = App::new(workspace_root);
    app.refresh();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(125);
    let mut last_tick = Instant::now();

    let result = loop {
        terminal.draw(|frame| render::render(frame, &app))?;
        app.drain_events();

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.handle_key(key) {
                    break Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
