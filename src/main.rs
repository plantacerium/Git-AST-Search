use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use tui_textarea::TextArea;

mod commands;
mod engine;
mod languages;
mod modules;
mod navigation;

use commands::{Autocomplete, CommandExecutor, CommandParser, CommandResult, CommandRegistry};
use engine::{GitEngine, Message};
use languages::supported_languages_display;
use modules::{ChatEntry, SearchResult};
use navigation::modes::NavMode;
use navigation::NavigationState;

struct App<'a> {
    textarea: TextArea<'a>,
    chat_history: Vec<ChatEntry>,
    results: Vec<SearchResult>,
    is_searching: bool,
    receiver: Option<Receiver<Message>>,
    nav_state: NavigationState,
    command_executor: CommandExecutor,
    autocomplete: Autocomplete,
    status_message: Option<String>,
    engine: GitEngine,
}

impl<'a> App<'a> {
    fn new(repo_path: String) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" AST Pattern (Enter to search) ")
                .border_type(BorderType::Rounded),
        );
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            chat_history: Vec::new(),
            results: Vec::new(),
            is_searching: false,
            receiver: None,
            nav_state: NavigationState::new(),
            command_executor: CommandExecutor::new(),
            autocomplete: Autocomplete::new(),
            status_message: None,
            engine: GitEngine::new(repo_path),
        }
    }

    fn start_search(&mut self, pattern_override: Option<String>) {
        let pattern = pattern_override.unwrap_or_else(|| self.textarea.lines().join("\n"));
        if pattern.trim().is_empty() { return; }

        self.is_searching = true;
        self.results.clear();
        self.nav_state.update_results(0);
        self.chat_history.push(ChatEntry::new(pattern.clone()));

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let path = self.engine.get_repo_path().to_string();
        let clean_pattern = pattern.trim().to_string();

        std::thread::spawn(move || {
            GitEngine::run_search(&path, clean_pattern, tx);
        });
    }

    fn update(&mut self) {
        if let Some(rx) = &self.receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Message::ResultFound(res) => {
                        self.results.push(res);
                        self.nav_state.update_results(self.results.len());
                    }
                    Message::SearchFinished => {
                        self.is_searching = false;
                        let count = self.results.len();
                        self.status_message = Some(format!("✓ Found {} results", count));
                        if let Some(entry) = self.chat_history.last_mut() {
                            entry.set_result_count(count);
                        }
                    }
                    Message::Error(err) => {
                        self.is_searching = false;
                        self.status_message = Some(format!("✗ {}", err));
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        if self.nav_state.mode == NavMode::Command {
            match key.code {
                KeyCode::Enter => {
                    let input = self.nav_state.command_buffer.trim_start_matches('/').to_string();
                    self.nav_state.exit_mode();
                    self.autocomplete.add_to_history(format!("/{}", input));
                    match CommandParser::parse(&input) {
                        Ok(parsed) => {
                            let result = self.command_executor.execute(parsed, &self.results, self.nav_state.global_selected());
                            self.apply_command_result(result);
                        }
                        Err(e) => { self.status_message = Some(format!("✗ {}", e)); }
                    }
                }
                KeyCode::Esc => { self.nav_state.exit_mode(); }
                KeyCode::Tab => {
                    if let Some(s) = self.autocomplete.suggest(&self.nav_state.command_buffer) {
                        self.nav_state.command_buffer = s;
                    }
                }
                KeyCode::Up => {
                    if let Some(p) = self.autocomplete.get_history_up() {
                        self.nav_state.command_buffer = p;
                    }
                }
                KeyCode::Down => {
                    if let Some(n) = self.autocomplete.get_history_down() {
                        self.nav_state.command_buffer = n;
                    }
                }
                KeyCode::Backspace => {
                    if self.nav_state.command_buffer.len() > 1 { self.nav_state.command_buffer.pop(); }
                    else { self.nav_state.exit_mode(); }
                }
                KeyCode::Char(c) => { self.nav_state.command_buffer.push(c); }
                _ => {}
            }
            return false;
        }

        if self.nav_state.mode == NavMode::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                self.nav_state.exit_mode();
            }
            return false;
        }

        if self.nav_state.mode == NavMode::Visual {
            match key.code {
                KeyCode::Esc => { self.nav_state.exit_mode(); }
                KeyCode::Char('j') | KeyCode::Down => { self.nav_state.select_next(); }
                KeyCode::Char('k') | KeyCode::Up => { self.nav_state.select_prev(); }
                _ => {}
            }
            return false;
        }

        // Normal mode
        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Right | KeyCode::Char('l') => self.nav_state.next_page(),
            KeyCode::Left | KeyCode::Char('h') => self.nav_state.prev_page(),
            KeyCode::Enter if !self.is_searching => self.start_search(None),
            KeyCode::Char('/') | KeyCode::Char(':') => { self.nav_state.enter_command_mode(); }
            KeyCode::Char('j') | KeyCode::Down => { self.nav_state.select_next(); }
            KeyCode::Char('k') | KeyCode::Up => { self.nav_state.select_prev(); }
            KeyCode::Char('v') => { self.nav_state.set_mode(NavMode::Visual); }
            KeyCode::Char('?') => { self.nav_state.set_mode(NavMode::Help); }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => { self.nav_state.toggle_sidebar(); }
            _ => { self.textarea.input(key); }
        }
        false
    }

    fn apply_command_result(&mut self, result: CommandResult) {
        match result {
            CommandResult::Search(pattern) => { self.start_search(Some(pattern)); }
            CommandResult::Quit => {} // handled by caller
            CommandResult::Redraw => {}
            CommandResult::Clear => {
                self.results.clear();
                self.chat_history.clear();
                self.nav_state.update_results(0);
                self.status_message = Some("Cleared.".to_string());
            }
            CommandResult::ToggleSidebar => { self.nav_state.toggle_sidebar(); }
            CommandResult::NextPage => { self.nav_state.next_page(); }
            CommandResult::PrevPage => { self.nav_state.prev_page(); }
            CommandResult::ExportDone(msg) | CommandResult::Success(Some(msg)) => {
                self.status_message = Some(msg);
            }
            CommandResult::Success(None) => {}
            CommandResult::Error(msg) => { self.status_message = Some(format!("✗ {}", msg)); }
        }
    }
}

fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(7)])
        .split(f.area());

    let main_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(if app.nav_state.sidebar_visible { 25 } else { 0 }),
            Constraint::Percentage(if app.nav_state.sidebar_visible { 75 } else { 100 }),
        ])
        .split(chunks[0]);

    if app.nav_state.sidebar_visible {
        let history_items: Vec<ListItem> = app.chat_history.iter().rev().take(20).map(|entry| {
            let count_str = if entry.result_count > 0 { format!(" ({})", entry.result_count) } else { String::new() };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(entry.timestamp.format("%H:%M").to_string(), Style::default().fg(Color::DarkGray)),
                    Span::styled(count_str, Style::default().fg(Color::Green)),
                ]),
                Line::from(entry.query.lines().next().unwrap_or("").chars().take(30).collect::<String>()),
            ])
        }).collect();
        let history_list = List::new(history_items).block(
            Block::default().borders(Borders::ALL).title(" 📜 History ").border_type(BorderType::Rounded),
        );
        f.render_widget(history_list, main_area[0]);
    }

    if app.nav_state.mode == NavMode::Help {
        render_help(f, main_area[1]);
    } else if app.results.is_empty() && !app.is_searching {
        render_welcome(f, main_area[1], app);
    } else {
        render_results_grid(f, main_area[1], app);
    }

    render_status_bar(f, chunks[1], app);

    // Input area
    if app.is_searching {
        app.textarea.set_block(Block::default().borders(Borders::ALL)
            .title(format!(" ⟳ Searching... ({} results) ", app.results.len()))
            .border_style(Style::default().fg(Color::Yellow)));
    } else if app.nav_state.mode == NavMode::Command {
        app.textarea.set_block(Block::default().borders(Borders::ALL)
            .title(format!(" {} ", app.nav_state.command_buffer))
            .border_style(Style::default().fg(Color::Green)));
    } else {
        app.textarea.set_block(Block::default().borders(Borders::ALL)
            .title(" AST Pattern ").border_type(BorderType::Rounded));
    }
    f.render_widget(&app.textarea, chunks[2]);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let mode_name = app.nav_state.mode.name();
    let mode_ind = app.nav_state.mode.indicator();
    let page_info = format!("Page {}/{}", app.nav_state.page + 1, app.nav_state.total_pages());
    let result_info = format!("{} results", app.nav_state.total_results);
    let bm_count = app.command_executor.get_bookmark_manager().len();
    let bm_info = if bm_count > 0 { format!(" | ★ {}", bm_count) } else { String::new() };
    let status = app.status_message.as_deref().unwrap_or("Ready");

    let bar = Line::from(vec![
        Span::styled(format!(" {} ", mode_ind), Style::default().fg(Color::Black).bg(match app.nav_state.mode {
            NavMode::Normal => Color::Blue, NavMode::Command => Color::Green,
            NavMode::Visual => Color::Magenta, NavMode::Help => Color::Yellow,
        })),
        Span::raw(" "),
        Span::styled(format!("[{}]", mode_name), Style::default().fg(Color::DarkGray)),
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

fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    let langs = supported_languages_display();
    let config = app.command_executor.get_config();
    let sessions_count = app.command_executor.get_session_manager().list_sessions().len();

    let welcome = Paragraph::new(vec![
        Line::from(Span::styled("🔍 Git AST Search v0.2.0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Repo: ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.engine.get_repo_path(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Theme: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&config.theme.name, Style::default().fg(Color::Magenta)),
            Span::styled(format!(" | {} sessions saved", sessions_count), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Write an AST pattern below, then press Enter to search.", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![Span::styled("Languages: ", Style::default().fg(Color::Green)), Span::raw(langs)]),
        Line::from(""),
        Line::from(Span::styled("Commands: /search, /filter, /export, /bookmark, /patterns, /help", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("Keys: j/k (select), h/l (pages), / (command), v (visual), ? (help)", Style::default().fg(Color::DarkGray))),
    ])
    .alignment(Alignment::Left)
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Welcome "));
    f.render_widget(welcome, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let registry = CommandRegistry::new();
    let lang_registry = languages::LanguageRegistry::new();

    let mut lines = vec![
        Line::from(Span::styled("📖 Git AST Search — Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("⌨ Keyboard Shortcuts", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  j/↓  Next result     k/↑  Previous result"),
        Line::from("  h/←  Previous page   l/→  Next page"),
        Line::from("  Enter  Search   /  Command mode   v  Visual"),
        Line::from("  ?  Help   Ctrl+B  Toggle sidebar   Esc  Quit"),
        Line::from(""),
        Line::from(Span::styled("📝 Commands", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    ];
    for cmd in registry.list_all() {
        let aliases = if cmd.aliases.is_empty() { String::new() } else { format!(" ({})", cmd.aliases.join(", ")) };
        lines.push(Line::from(format!("  /{}{} — {}", cmd.name, aliases, cmd.description)));
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
    lines.push(Line::from(Span::styled("Press Esc or ? to close.", Style::default().fg(Color::DarkGray))));

    let help = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Help "),
    );
    f.render_widget(help, area);
}

fn render_results_grid(f: &mut Frame, area: Rect, app: &App) {
    let total = app.results.len();
    let page = app.nav_state.page;
    let per_page = app.nav_state.results_per_page;
    let start_idx = page * per_page;
    let items: Vec<_> = app.results.iter().skip(start_idx).take(per_page).collect();
    let count = items.len();

    let title = format!(" Results (Page {}/{} — Total: {}) ", page + 1, app.nav_state.total_pages(), total);
    let block = Block::default().borders(Borders::ALL).title(title).border_type(BorderType::Rounded);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if count == 0 { return; }

    let rows = if count > 3 { 2 } else { 1 };
    let v_chunks = Layout::default().direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100 / rows); rows as usize]).split(inner);

    for (i, res) in items.into_iter().enumerate() {
        let row_idx = i / 3;
        let col_idx = i % 3;
        let cols = if row_idx == 0 && count < 3 { count } else if row_idx == 1 { (count - 3).max(1) } else { 3 };
        if cols == 0 { continue; }
        let h_chunks = Layout::default().direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100 / cols as u16); cols]).split(v_chunks[row_idx]);

        let is_selected = (start_idx + i) == app.nav_state.global_selected();
        let border_style = if is_selected { Style::default().fg(Color::Cyan) } else { Style::default() };

        let text = vec![
            Line::from(vec![
                Span::styled("Commit: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(&res.commit_short),
                Span::styled(format!(" [{}]", res.lang), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled("File: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}:{}", res.file_path, res.line_number)),
            ]),
            Line::from("─".repeat((inner.width as usize / cols.max(1)).saturating_sub(2))),
            Line::from(res.content.lines().next().unwrap_or("").to_string()),
        ];

        let card = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL)
                .border_type(if is_selected { BorderType::Double } else { BorderType::Rounded })
                .border_style(border_style))
            .wrap(Wrap { trim: false });

        if col_idx < h_chunks.len() { f.render_widget(card, h_chunks[col_idx]); }
    }
}

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
                if app.handle_key(key) { break; }
                // Check if command resulted in Quit
                if app.nav_state.mode == NavMode::Normal && matches!(key.code, KeyCode::Enter) {
                    // Quit is handled via apply_command_result
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
