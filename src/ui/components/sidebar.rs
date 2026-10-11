use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::ui::app::App;
use crate::navigation::ActivePane;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.nav_state.active_pane == ActivePane::History;
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let title = if is_focused {
        " 📜 History [ACTIVE: j/k, Enter/r, e, Tab] "
    } else {
        " 📜 History (Tab to focus) "
    };

    let history_items: Vec<ListItem> = if app.chat_history.is_empty() {
        vec![
            ListItem::new(vec![
                Line::from(Span::styled("  (No queries yet)", Style::default().fg(Color::DarkGray))),
                Line::from(Span::styled("  Search a pattern below", Style::default().fg(Color::DarkGray))),
            ])
        ]
    } else {
        app.chat_history
            .iter()
            .rev()
            .take(50)
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = is_focused && app.nav_state.history_selected_idx == idx;
                let count_str = if entry.result_count > 0 {
                    format!(" ({} matches)", entry.result_count)
                } else {
                    " (0)".to_string()
                };

                let prefix = if is_selected { "▶ " } else { "  " };

                let time_style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                let count_style = if is_selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                let query_style = if is_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let line1 = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(entry.timestamp.format("%H:%M").to_string(), time_style),
                    Span::styled(count_str, count_style),
                ]);

                let query_preview = entry
                    .query
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(28)
                    .collect::<String>();

                let line2 = Line::from(vec![
                    Span::raw("    "),
                    Span::styled(query_preview, query_style),
                ]);

                let mut item = ListItem::new(vec![line1, line2]);
                if is_selected {
                    item = item.style(Style::default().bg(Color::Rgb(25, 38, 55)));
                }
                item
            })
            .collect()
    };

    let history_list = List::new(history_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title)
            .border_type(BorderType::Rounded),
    );

    let mut state = ListState::default();
    if is_focused && !app.chat_history.is_empty() {
        state.select(Some(app.nav_state.history_selected_idx));
    }

    f.render_stateful_widget(history_list, area, &mut state);
}

