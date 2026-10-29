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
    } else if app.is_rule_editor {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📜 AST YAML Rule Editor (Enter: newline | Ctrl+Enter: search | Ctrl+E: toggle) ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta)),
        );
        app.textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    } else if app.nav_state.mode == NavMode::Insert {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ✎ AST Pattern (Enter: search | Shift+Enter: newline | Ctrl+E: Rule Editor) ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        app.textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
    } else {
        app.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🔒 AST Pattern (Press 'i' to type) ")
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        // Hide cursor in normal mode or style it differently
        app.textarea.set_cursor_line_style(Style::default());
    }
    
    f.render_widget(&app.textarea, area);
}
