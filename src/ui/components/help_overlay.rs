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
        Line::from("  j/↓  Next result     k/↑  Previous result"),
        Line::from("  h/←  Previous page   l/→  Next page"),
        Line::from("  Enter  Search   /  Command mode   v  Visual"),
        Line::from("  ?  Help   Ctrl+B  Toggle sidebar   Esc  Quit"),
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
