use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::ui::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let history_items: Vec<ListItem> = app
        .chat_history
        .iter()
        .rev()
        .take(20)
        .map(|entry| {
            let count_str = if entry.result_count > 0 {
                format!(" ({})", entry.result_count)
            } else {
                String::new()
            };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        entry.timestamp.format("%H:%M").to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(count_str, Style::default().fg(Color::Green)),
                ]),
                Line::from(
                    entry
                        .query
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(30)
                        .collect::<String>(),
                ),
            ])
        })
        .collect();

    let history_list = List::new(history_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 📜 History ")
            .border_type(BorderType::Rounded),
    );

    f.render_widget(history_list, area);
}
