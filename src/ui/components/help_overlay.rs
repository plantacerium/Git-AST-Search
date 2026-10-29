use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::ui::app::App;
use crate::commands::CommandRegistry;
use crate::languages;

pub fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    let langs = languages::supported_languages_display();
    let config = app.command_executor.get_config();
    let sessions_count = app.command_executor.get_session_manager().list_sessions().len();

    let welcome = Paragraph::new(vec![
        Line::from(Span::styled(
            "🔍 Git AST Search v0.2.0",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Repo: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.engine.get_repo_path(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Theme: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&config.theme.name, Style::default().fg(Color::Magenta)),
            Span::styled(
                format!(" | {} sessions saved", sessions_count),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Write an AST pattern below, then press Enter to search.",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Languages: ", Style::default().fg(Color::Green)),
            Span::raw(langs),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Commands: /search, /filter, /export, /bookmark, /patterns, /help",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Keys: j/k (select), h/l (pages), / (command), v (visual), ? (help)",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Welcome "),
    );
    
    f.render_widget(welcome, area);
}

pub fn render_help(f: &mut Frame, area: Rect) {
    let registry = CommandRegistry::new();
    let lang_registry = languages::LanguageRegistry::new();

    let mut lines = vec![
        Line::from(Span::styled(
            "📖 Git AST Search — Help",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "⌨ Keyboard Shortcuts",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab  Toggle active pane (Results ⇆ History)"),
        Line::from("  j/↓  Next item (result or history)   k/↑  Previous item"),
        Line::from("  gg   First result                    G    Last result"),
        Line::from("  d / Space  Preview full historical Git blob with syntax highlight (syntect)"),
        Line::from("  o    Open matched file at line in external $EDITOR / VS Code"),
        Line::from("  y / Ctrl+C  Copy selected match to system clipboard"),
        Line::from("  Y    Copy full surrounding context to system clipboard"),
        Line::from("  Ctrl+V  Paste from clipboard into pattern input or command line"),
        Line::from("  Ctrl+E  Toggle Multi-line YAML Rule Editor (Enter: newline, Ctrl+Enter: search)"),
        Line::from("  Enter / r  Search / Re-run historical query"),
        Line::from("  e / i  Edit historical pattern in input box"),
        Line::from("  h/←  Previous page   l/→  Next page / Focus results"),
        Line::from("  Esc  Cancel running search / Close overlay / Quit"),
        Line::from("  /  Command mode   v  Visual   ?  Help   Ctrl+B  Toggle sidebar"),
        Line::from(""),
        Line::from(Span::styled(
            "📝 Commands",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
    ];
    
    for cmd in registry.list_all() {
        let aliases = if cmd.aliases.is_empty() {
            String::new()
        } else {
            format!(" ({})", cmd.aliases.join(", "))
        };
        lines.push(Line::from(format!(
            "  /{}{} — {}",
            cmd.name, aliases, cmd.description
        )));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("🌐 Languages ({} enabled)", lang_registry.enabled_count()),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    
    for lang in lang_registry.enabled_languages() {
        lines.push(Line::from(format!("  {} {}", lang.icon(), lang.name())));
    }
    
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Esc or ? to close.",
        Style::default().fg(Color::DarkGray),
    )));

    let help = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Help "),
    );
    
    f.render_widget(help, area);
}

pub fn render_custom_overlay(f: &mut Frame, area: Rect, title: &str, content: &str, scroll: u16) {
    let mut lines: Vec<Line> = content
        .lines()
        .map(|l| {
            if l.starts_with(">>") {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            } else if l.trim_start().starts_with('+') {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Green)))
            } else if l.trim_start().starts_with('-') {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Red)))
            } else if l.contains("[INTRODUCED]") {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
            } else if l.contains("[MUTATED]") {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            } else if l.contains("[REMOVED]") {
                Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)))
            } else {
                Line::from(l.to_string())
            }
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Use j/k (or ↓/↑) to scroll, Esc or q to close.",
        Style::default().fg(Color::DarkGray),
    )));

    let overlay = Paragraph::new(lines)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Rounded)
                .title(format!("{} [j/k scroll, Esc close] ", title)),
        );

    f.render_widget(overlay, area);
}

pub fn render_styled_overlay(f: &mut Frame, area: Rect, title: &str, lines: &[Line], scroll: u16) {
    let mut all_lines: Vec<Line> = lines.to_vec();
    all_lines.push(Line::from(""));
    all_lines.push(Line::from(Span::styled(
        "Use j/k (or ↓/↑) to scroll, Esc or q to close.",
        Style::default().fg(Color::DarkGray),
    )));

    let overlay = Paragraph::new(all_lines)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .border_type(BorderType::Rounded)
                .title(format!("{} [j/k scroll, Esc close] ", title)),
        );

    f.render_widget(overlay, area);
}
