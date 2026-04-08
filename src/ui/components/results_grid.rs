use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::ui::app::App;
use crate::ui::layout::{calculate_grid_rows, calculate_grid_cols};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let total = app.results.len();
    let page = app.nav_state.page;
    let per_page = app.nav_state.results_per_page;
    let start_idx = page * per_page;
    let items: Vec<_> = app.results.iter().skip(start_idx).take(per_page).collect();
    let count = items.len();

    let title = format!(
        " Results (Page {}/{} — Total: {}) ",
        page + 1,
        app.nav_state.total_pages(),
        total
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_type(BorderType::Rounded);
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    if count == 0 {
        return;
    }

    let rows = if count > 3 { 2 } else { 1 };
    let v_chunks = calculate_grid_rows(inner, rows);

    for (i, res) in items.into_iter().enumerate() {
        let row_idx = i / 3;
        let col_idx = i % 3;
        let cols = if row_idx == 0 && count < 3 {
            count
        } else if row_idx == 1 {
            (count - 3).max(1)
        } else {
            3
        };
        
        if cols == 0 {
            continue;
        }
        
        let h_chunks = calculate_grid_cols(v_chunks[row_idx], cols as u16);

        let is_selected = (start_idx + i) == app.nav_state.global_selected();
        let border_style = if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let text = vec![
            Line::from(vec![
                Span::styled(
                    "Commit: ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(&res.commit_short),
                Span::styled(
                    format!(" [{}]", res.lang),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "File: ",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}:{}", res.file_path, res.line_number)),
            ]),
            Line::from("─".repeat((inner.width as usize / cols.max(1)).saturating_sub(2))),
            Line::from(res.content.lines().next().unwrap_or("").to_string()),
        ];

        let card = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if is_selected {
                        BorderType::Double
                    } else {
                        BorderType::Rounded
                    })
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false });

        if col_idx < h_chunks.len() {
            f.render_widget(card, h_chunks[col_idx]);
        }
    }
}
