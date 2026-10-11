use ratatui::widgets::{Block, Borders, BorderType};
use ratatui::style::Style;
use std::sync::mpsc::{self, Receiver};
use tui_textarea::TextArea;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::ai::OllamaClient;
use crate::commands::{Autocomplete, CommandExecutor, CommandParser, CommandResult};
use crate::engine::{GitEngine, Message};
use crate::modules::{ChatEntry, SearchResult};
use crate::navigation::modes::NavMode;
use crate::navigation::{NavigationState, ActivePane};

pub enum AiEvent {
    QuerySuccess { pattern: String, explanation: String },
    ReportSuccess { report_path: String },
    Error(String),
}

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
    pub ai_receiver: Option<Receiver<AiEvent>>,
    pub is_ai_processing: bool,
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
            command_executor: CommandExecutor::new(repo_path.clone()),
            autocomplete: Autocomplete::new(),
            status_message: None,
            engine: GitEngine::new(repo_path),
            ai_receiver: None,
            is_ai_processing: false,
        }
    }

    pub fn start_search(&mut self, pattern_override: Option<String>) {
        let pattern = pattern_override.unwrap_or_else(|| self.textarea.lines().join("\n"));
        if pattern.trim().is_empty() { return; }

        self.is_searching = true;
        self.results.clear();
        self.nav_state.update_results(0);
        self.nav_state.active_pane = ActivePane::Results;
        self.nav_state.history_selected_idx = 0;
        self.chat_history.push(ChatEntry::new(pattern.clone()));

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let path = self.engine.get_repo_path().to_string();
        let clean_pattern = pattern.trim().to_string();

        std::thread::spawn(move || {
            GitEngine::run_search(&path, clean_pattern, tx);
        });
    }

    pub fn selected_history_entry(&self) -> Option<&ChatEntry> {
        let len = self.chat_history.len();
        if len == 0 {
            return None;
        }
        let visual_idx = self.nav_state.history_selected_idx;
        if visual_idx < len {
            let actual_idx = len - 1 - visual_idx;
            self.chat_history.get(actual_idx)
        } else {
            None
        }
    }

    pub fn preview_selected_history(&mut self) {
        if let Some(entry) = self.selected_history_entry() {
            let count = entry.results.len();
            let query = entry.query.clone();
            let time = entry.timestamp.format("%H:%M").to_string();
            self.results = entry.results.clone();
            self.nav_state.update_results(self.results.len());
            if count > 0 {
                self.status_message = Some(format!("📜 History [{}]: '{}' ({} cached matches | Enter/r: re-run, e: edit)", time, query, count));
            } else {
                self.status_message = Some(format!("📜 History [{}]: '{}' (0 matches | Enter/r: re-run)", time, query));
            }
        }
    }

    pub fn replay_selected_history(&mut self) {
        if let Some(entry) = self.selected_history_entry() {
            let query = entry.query.clone();
            self.textarea = TextArea::new(query.lines().map(|s| s.to_string()).collect());
            self.nav_state.active_pane = ActivePane::Results;
            self.status_message = Some(format!("⟳ Re-running historical query: {}", query));
            self.start_search(Some(query));
        }
    }

    pub fn edit_selected_history(&mut self) {
        if let Some(entry) = self.selected_history_entry() {
            let query = entry.query.clone();
            self.textarea = TextArea::new(query.lines().map(|s| s.to_string()).collect());
            self.nav_state.active_pane = ActivePane::Results;
            self.nav_state.set_mode(NavMode::Insert);
            self.status_message = Some("Loaded historical pattern into editor (Insert Mode)".to_string());
        }
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
                            entry.set_results(self.results.clone());
                        }
                    }
                    Message::Error(err) => {
                        self.is_searching = false;
                        self.status_message = Some(format!("✗ {}", err));
                    }
                }
            }
        }

        let events: Vec<AiEvent> = if let Some(rx) = &self.ai_receiver {
            let mut evs = Vec::new();
            while let Ok(event) = rx.try_recv() {
                evs.push(event);
            }
            evs
        } else {
            Vec::new()
        };

        for event in events {
            self.is_ai_processing = false;
            match event {
                AiEvent::QuerySuccess { pattern, explanation } => {
                    let ta = TextArea::new(vec![pattern.clone()]);
                    self.textarea = ta;
                    self.status_message = Some(format!("✓ AI: {} -> {}", explanation, pattern));
                    self.start_search(Some(pattern));
                }
                AiEvent::ReportSuccess { report_path } => {
                    self.status_message = Some(format!("✓ AI Audit Report saved to {}", report_path));
                }
                AiEvent::Error(err) => {
                    self.status_message = Some(format!("✗ AI: {}", err));
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

        if self.nav_state.mode == NavMode::Insert {
            match key.code {
                KeyCode::Esc => { self.nav_state.set_mode(NavMode::Normal); }
                KeyCode::Enter if !self.is_searching => self.start_search(None),
                _ => { self.textarea.input(key); }
            }
            return false;
        }

        // Normal mode
        match key.code {
            KeyCode::Esc => {
                if self.nav_state.active_pane == ActivePane::History {
                    self.nav_state.active_pane = ActivePane::Results;
                    self.status_message = Some("Focused Results Grid".to_string());
                } else {
                    return true;
                }
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.nav_state.toggle_pane();
                if self.nav_state.active_pane == ActivePane::History {
                    self.preview_selected_history();
                    self.status_message = Some("Focused 📜 History (j/k: navigate, Enter/r: re-run, e: edit, Tab: back)".to_string());
                } else {
                    self.status_message = Some("Focused Results Grid (j/k: select, h/l: pages)".to_string());
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.nav_state.active_pane == ActivePane::History {
                    self.nav_state.active_pane = ActivePane::Results;
                    self.status_message = Some("Focused Results Grid".to_string());
                } else {
                    self.nav_state.next_page();
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.nav_state.active_pane == ActivePane::History {
                    // already on history
                } else if self.nav_state.page == 0 && self.nav_state.sidebar_visible {
                    self.nav_state.active_pane = ActivePane::History;
                    self.preview_selected_history();
                    self.status_message = Some("Focused 📜 History (j/k: navigate, Enter/r: re-run)".to_string());
                } else {
                    self.nav_state.prev_page();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.nav_state.active_pane == ActivePane::History {
                    let total = self.chat_history.len();
                    self.nav_state.history_next(total);
                    self.preview_selected_history();
                } else {
                    self.nav_state.select_next();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.nav_state.active_pane == ActivePane::History {
                    self.nav_state.history_prev();
                    self.preview_selected_history();
                } else {
                    self.nav_state.select_prev();
                }
            }
            KeyCode::Enter => {
                if self.nav_state.active_pane == ActivePane::History {
                    self.replay_selected_history();
                } else if !self.is_searching {
                    self.start_search(None);
                }
            }
            KeyCode::Char('r') if self.nav_state.active_pane == ActivePane::History => {
                self.replay_selected_history();
            }
            KeyCode::Char('e') if self.nav_state.active_pane == ActivePane::History => {
                self.edit_selected_history();
            }
            KeyCode::Char('/') | KeyCode::Char(':') => { self.nav_state.enter_command_mode(); }
            KeyCode::Char('v') => { self.nav_state.set_mode(NavMode::Visual); }
            KeyCode::Char('i') | KeyCode::Char('a') => {
                if self.nav_state.active_pane == ActivePane::History {
                    self.edit_selected_history();
                } else {
                    self.nav_state.set_mode(NavMode::Insert);
                }
            }
            KeyCode::Char('?') => { self.nav_state.set_mode(NavMode::Help); }
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => { self.nav_state.toggle_sidebar(); }
            _ => {}
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
            CommandResult::FocusHistory => {
                self.nav_state.set_active_pane(ActivePane::History);
                self.preview_selected_history();
                self.status_message = Some("Focused 📜 History (j/k: navigate, Enter/r: re-run, Tab: back)".to_string());
            }
            CommandResult::ReplayHistory(idx) => {
                let len = self.chat_history.len();
                if len == 0 {
                    self.status_message = Some("History is empty.".to_string());
                } else if idx < len {
                    self.nav_state.history_selected_idx = idx;
                    self.replay_selected_history();
                } else {
                    self.status_message = Some(format!("✗ History index {} out of range (total: {})", idx, len));
                }
            }
            CommandResult::AiQuery(prompt) => {
                self.is_ai_processing = true;
                self.status_message = Some(format!("⟳ Consulting Ollama: \"{}\"...", prompt));
                let (tx, rx) = mpsc::channel();
                self.ai_receiver = Some(rx);
                std::thread::spawn(move || {
                    let client = OllamaClient::new();
                    match client.query_to_pattern(&prompt) {
                        Ok(resp) => {
                            let _ = tx.send(AiEvent::QuerySuccess {
                                pattern: resp.pattern,
                                explanation: resp.explanation,
                            });
                        }
                        Err(e) => {
                            let _ = tx.send(AiEvent::Error(e));
                        }
                    }
                });
            }
            CommandResult::AiReport(topic) => {
                if self.results.is_empty() {
                    self.status_message = Some("✗ No search results to generate report. Run a search first.".to_string());
                } else {
                    self.is_ai_processing = true;
                    self.status_message = Some("⟳ Ollama generating audit report from results...".to_string());
                    let (tx, rx) = mpsc::channel();
                    self.ai_receiver = Some(rx);
                    let results_clone = self.results.clone();
                    let default_topic = self.chat_history.last().map(|c| c.query.clone()).unwrap_or_else(|| "AST Search".to_string());
                    let topic_str = topic.unwrap_or(default_topic);
                    std::thread::spawn(move || {
                        let client = OllamaClient::new();
                        match client.generate_report(&topic_str, &results_clone) {
                            Ok(markdown) => {
                                let path = "audit_report.md";
                                if let Err(e) = std::fs::write(path, markdown) {
                                    let _ = tx.send(AiEvent::Error(format!("Failed to write report: {}", e)));
                                } else {
                                    let _ = tx.send(AiEvent::ReportSuccess { report_path: path.to_string() });
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(AiEvent::Error(e));
                            }
                        }
                    });
                }
            }
        }
    }
}
