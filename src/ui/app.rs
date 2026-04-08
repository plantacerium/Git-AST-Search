use ratatui::widgets::{Block, Borders, BorderType};
use ratatui::style::Style;
use std::sync::mpsc::{self, Receiver};
use tui_textarea::TextArea;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::commands::{Autocomplete, CommandExecutor, CommandParser, CommandResult};
use crate::engine::{GitEngine, Message};
use crate::modules::{ChatEntry, SearchResult};
use crate::navigation::modes::NavMode;
use crate::navigation::NavigationState;

pub struct App<'a> {
    pub textarea: TextArea<'a>,
    pub chat_history: Vec<ChatEntry>,
    pub results: Vec<SearchResult>,
    pub is_searching: bool,
    pub receiver: Option<Receiver<Message>>,
    pub nav_state: NavigationState,
    pub command_executor: CommandExecutor,
    pub autocomplete: Autocomplete,
    pub status_message: Option<String>,
    pub engine: GitEngine,
}

impl<'a> App<'a> {
    pub fn new(repo_path: String) -> Self {
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

    pub fn start_search(&mut self, pattern_override: Option<String>) {
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

    pub fn update(&mut self) {
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

    pub fn handle_key(&mut self, key: event::KeyEvent) -> bool {
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

    pub fn apply_command_result(&mut self, result: CommandResult) {
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
