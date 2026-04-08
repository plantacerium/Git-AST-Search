use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::ui::app::App;
use crate::navigation::modes::NavMode;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mode_name = app.nav_state.mode.name();
    let mode_ind = app.nav_state.mode.indicator();
    let page_info = format!("Page {}/{}", app.nav_state.page + 1, app.nav_state.total_pages());
    let result_info = format!("{} results", app.nav_state.total_results);
    let bm_count = app.command_executor.get_bookmark_manager().len();
    let bm_info = if bm_count > 0 {
        format!(" | ★ {}", bm_count)
    } else {
        String::new()
    };
    let status = app.status_message.as_deref().unwrap_or("Ready");

    let bar = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_ind),
            Style::default().fg(Color::Black).bg(match app.nav_state.mode {
                NavMode::Normal => Color::Blue,
                NavMode::Insert => Color::Cyan,
                NavMode::Command => Color::Green,
                NavMode::Visual => Color::Magenta,
                NavMode::Help => Color::Yellow,
            }),
        ),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", mode_name),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw(" "),
        Span::styled(page_info, Style::default().fg(Color::Cyan)),
        Span::raw(" │ "),
        Span::styled(result_info, Style::default().fg(Color::Yellow)),
        Span::styled(bm_info, Style::default().fg(Color::LightMagenta)),
        Span::raw(" │ "),
        Span::styled(status, Style::default().fg(Color::DarkGray)),
    ]);
    
    f.render_widget(Paragraph::new(bar), area);
}
