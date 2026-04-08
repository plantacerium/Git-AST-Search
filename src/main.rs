use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, time::Duration};

mod commands;
mod engine;
mod languages;
mod modules;
mod navigation;
pub mod ui;

use ui::app::App;
use ui::render::draw_ui;
use navigation::modes::NavMode;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let repo_path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repo_path);

    loop {
        app.update();
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.handle_key(key) { break; }
                    // Check if command resulted in Quit
                    if app.nav_state.mode == NavMode::Normal && matches!(key.code, KeyCode::Enter) {
                        // Quit is handled via apply_command_result
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
