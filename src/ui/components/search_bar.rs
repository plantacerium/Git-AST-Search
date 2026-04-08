use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::ui::app::App;
use crate::navigation::modes::NavMode;

pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    if app.is_searching {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" ⟳ Searching... ({} results) ", app.results.len()))
                .border_style(Style::default().fg(Color::Yellow)),
        );
    } else if app.nav_state.mode == NavMode::Command {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", app.nav_state.command_buffer))
                .border_style(Style::default().fg(Color::Green)),
        );
    } else {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" AST Pattern ")
                .border_type(BorderType::Rounded),
        );
    }
    
    f.render_widget(&app.textarea, area);
}
