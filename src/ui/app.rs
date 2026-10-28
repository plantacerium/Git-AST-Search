use ratatui::widgets::{Block, Borders, BorderType};
use ratatui::style::Style;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use tui_textarea::TextArea;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::ai::OllamaClient;
use crate::commands::{Autocomplete, CommandExecutor, CommandParser, CommandResult};
use crate::engine::{GitEngine, Message};
use crate::modules::{ChatEntry, Filter, SearchResult};
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
    pub active_filters: Vec<Filter>,
    pub is_searching: bool,
    pub should_quit: bool,
    pub receiver: Option<Receiver<Message>>,
    pub nav_state: NavigationState,
    pub command_executor: CommandExecutor,
    pub autocomplete: Autocomplete,
    pub status_message: Option<String>,
    pub engine: GitEngine,
    pub ai_receiver: Option<Receiver<AiEvent>>,
    pub is_ai_processing: bool,
    pub cancel_flag: Arc<AtomicBool>,
    pub scan_progress: Option<(usize, usize)>,
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

        let command_executor = CommandExecutor::new(repo_path.clone());
        let mut initial_history = command_executor
            .get_session_manager()
            .get_current()
            .map(|s| s.entries.clone())
            .unwrap_or_default();
        initial_history.reverse();

        Self {
            textarea,
            chat_history: initial_history,
            results: Vec::new(),
            active_filters: Vec::new(),
            is_searching: false,
            should_quit: false,
            receiver: None,
            nav_state: NavigationState::new(),
            command_executor,
            autocomplete: Autocomplete::new(),
            status_message: None,
            engine: GitEngine::new(repo_path),
            ai_receiver: None,
            is_ai_processing: false,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            scan_progress: None,
        }
    }

    pub fn start_search(&mut self, pattern_override: Option<String>) {
        let filters = self.active_filters.clone();
        self.start_search_with_filters(pattern_override, filters);
    }

    pub fn start_search_with_filters(&mut self, pattern_override: Option<String>, filters: Vec<Filter>) {
        let pattern = pattern_override.unwrap_or_else(|| self.textarea.lines().join("\n"));
        if pattern.trim().is_empty() { return; }

        // Cancel any running search before starting a new one
        self.cancel_flag.store(true, Ordering::Relaxed);
        let new_cancel = Arc::new(AtomicBool::new(false));
        self.cancel_flag = new_cancel.clone();
        self.scan_progress = None;

        self.is_searching = true;
        self.results.clear();
        self.nav_state.update_results(0);
        self.nav_state.close_overlay();
        self.nav_state.active_pane = ActivePane::Results;
        self.nav_state.history_selected_idx = 0;

        let filter_strs: Vec<String> = filters.iter().map(|f| f.to_string()).collect();
        self.chat_history.push(ChatEntry::new(pattern.clone()).with_filters(filter_strs));

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let path = self.engine.get_repo_path().to_string();
        let clean_pattern = pattern.trim().to_string();

        std::thread::spawn(move || {
            GitEngine::run_search_with_options(&path, clean_pattern, filters, new_cancel, tx);
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
                    Message::Progress { processed, total } => {
                        self.scan_progress = Some((processed, total));
                        let pct = (processed * 100) / total.max(1);
                        self.status_message = Some(format!(
                            "⟳ Scanning blobs: {}/{} ({}%) — {} matches [Esc to cancel]",
                            processed,
                            total,
                            pct,
                            self.results.len()
                        ));
                    }
                    Message::SearchFinished => {
                        self.is_searching = false;
                        self.scan_progress = None;
                        let count = self.results.len();
                        let filter_note = if self.active_filters.is_empty() {
                            String::new()
                        } else {
                            format!(
                                " [filters: {}]",
                                self.active_filters
                                    .iter()
                                    .map(|f| f.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        };
                        let cancelled = self.cancel_flag.load(Ordering::Relaxed);
                        if cancelled {
                            self.status_message = Some(format!("⏹ Search cancelled ({} partial results{})", count, filter_note));
                        } else {
                            self.status_message = Some(format!("✓ Found {} results{}", count, filter_note));
                        }
                        if let Some(entry) = self.chat_history.last_mut() {
                            // Cap cached results per history entry to 500 to keep memory bounded
                            let capped: Vec<SearchResult> = self.results.iter().take(500).cloned().collect();
                            entry.set_results(capped);
                            let saved_entry = entry.clone();
                            self.command_executor.record_entry(saved_entry);
                        }
                    }
                    Message::Error(err) => {
                        self.is_searching = false;
                        self.scan_progress = None;
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

    pub fn open_selected_blob_preview(&mut self) {
        if self.results.is_empty() {
            self.status_message = Some("✗ No result selected to preview.".to_string());
            return;
        }
        let idx = self.nav_state.global_selected().min(self.results.len() - 1);
        let res = &self.results[idx];
        match GitEngine::read_blob_at_commit(self.engine.get_repo_path(), &res.commit_id, &res.file_path) {
            Ok(content) => {
                let mut formatted = String::new();
                formatted.push_str(&format!(
                    "Commit: {} ({}) | File: {}:{}\n\n",
                    res.commit_short,
                    res.author.as_deref().unwrap_or("unknown"),
                    res.file_path,
                    res.line_number
                ));
                for (i, line) in content.lines().enumerate() {
                    let line_no = i + 1;
                    if line_no == res.line_number {
                        formatted.push_str(&format!(">> {:4} | {}\n", line_no, line));
                    } else {
                        formatted.push_str(&format!("   {:4} | {}\n", line_no, line));
                    }
                }
                let title = format!(" 📄 Historical Blob [{}] {}:{} ", res.commit_short, res.file_path, res.line_number);
                self.nav_state.show_overlay(title, formatted);
                self.nav_state.overlay_scroll = res.line_number.saturating_sub(6) as u16;
            }
            Err(e) => {
                self.status_message = Some(format!("✗ Could not read historical blob: {}", e));
            }
        }
    }

    pub fn open_selected_in_editor(&mut self) {
        if self.results.is_empty() {
            self.status_message = Some("✗ No result selected to open.".to_string());
            return;
        }
        let idx = self.nav_state.global_selected().min(self.results.len() - 1);
        let res = &self.results[idx];
        let full_path = std::path::Path::new(self.engine.get_repo_path()).join(&res.file_path);
        if !full_path.exists() {
            self.status_message = Some(format!(
                "ℹ '{}' does not exist in working tree; opening historical blob preview instead.",
                res.file_path
            ));
            self.open_selected_blob_preview();
            return;
        }
        let path_str = full_path.to_string_lossy().to_string();
        let line = res.line_number;
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "code".to_string());

        let spawned = if editor.contains("code") || editor.contains("cursor") {
            std::process::Command::new(&editor)
                .arg("-g")
                .arg(format!("{}:{}", path_str, line))
                .spawn()
        } else {
            std::process::Command::new(&editor)
                .arg(format!("+{}", line))
                .arg(&path_str)
                .spawn()
        };

        match spawned {
            Ok(_) => {
                self.status_message = Some(format!("↗ Opened {}:{} in '{}'", res.file_path, line, editor));
            }
            Err(e) => {
                self.status_message = Some(format!("✗ Failed to launch editor '{}': {}", editor, e));
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
            return self.should_quit;
        }

        if self.nav_state.mode == NavMode::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
                self.nav_state.exit_mode();
            }
            return false;
        }

        // Handle scrollable command output overlay modal if active
        if self.nav_state.overlay_content.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.nav_state.close_overlay();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.nav_state.overlay_scroll = self.nav_state.overlay_scroll.saturating_add(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.nav_state.overlay_scroll = self.nav_state.overlay_scroll.saturating_sub(1);
                }
                KeyCode::Char('/') | KeyCode::Char(':') => {
                    self.nav_state.enter_command_mode();
                }
                _ => {}
            }
            return false;
        }

        if self.nav_state.mode == NavMode::Visual {
            match key.code {
                KeyCode::Esc => {
                    self.nav_state.pending_g = false;
                    self.nav_state.exit_mode();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.nav_state.pending_g = false;
                    self.nav_state.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.nav_state.pending_g = false;
                    self.nav_state.select_prev();
                }
                KeyCode::Char('g') => {
                    if self.nav_state.pending_g {
                        self.nav_state.select_first();
                        self.nav_state.pending_g = false;
                    } else {
                        self.nav_state.pending_g = true;
                    }
                }
                KeyCode::Char('G') => {
                    self.nav_state.pending_g = false;
                    self.nav_state.select_last();
                }
                KeyCode::Char('b') => {
                    self.nav_state.pending_g = false;
                    let parsed = CommandParser::parse("/bookmark").unwrap();
                    let res = self.command_executor.execute(parsed, &self.results, self.nav_state.global_selected());
                    self.apply_command_result(res);
                }
                _ => {
                    self.nav_state.pending_g = false;
                }
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
        if key.code != KeyCode::Char('g') {
            self.nav_state.pending_g = false;
        }

        match key.code {
            KeyCode::Esc => {
                if self.is_searching {
                    self.cancel_flag.store(true, Ordering::Relaxed);
                    self.is_searching = false;
                    self.scan_progress = None;
                    self.status_message = Some("⏹ Search cancelled by user (Esc)".to_string());
                } else if self.nav_state.active_pane == ActivePane::History {
                    self.nav_state.active_pane = ActivePane::Results;
                    self.status_message = Some("Focused Results Grid".to_string());
                } else {
                    return true;
                }
            }
            KeyCode::Char('d') | KeyCode::Char(' ') if self.nav_state.active_pane == ActivePane::Results => {
                self.open_selected_blob_preview();
            }
            KeyCode::Char('o') if self.nav_state.active_pane == ActivePane::Results => {
                self.open_selected_in_editor();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                self.nav_state.toggle_pane();
                if self.nav_state.active_pane == ActivePane::History {
                    self.preview_selected_history();
                    self.status_message = Some("Focused 📜 History (j/k: navigate, Enter/r: re-run, e: edit, Tab: back)".to_string());
                } else {
                    self.status_message = Some("Focused Results Grid (j/k: select, h/l: pages, gg/G: first/last)".to_string());
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
            KeyCode::Char('g') => {
                if self.nav_state.pending_g {
                    self.nav_state.select_first();
                    self.nav_state.pending_g = false;
                    self.status_message = Some("Jumped to first result (gg)".to_string());
                } else {
                    self.nav_state.pending_g = true;
                }
            }
            KeyCode::Char('G') => {
                self.nav_state.select_last();
                self.status_message = Some("Jumped to last result (G)".to_string());
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
        self.should_quit
    }

    pub fn apply_command_result(&mut self, result: CommandResult) {
        match result {
            CommandResult::Search { pattern, filters } => {
                self.active_filters = filters.clone();
                self.start_search_with_filters(Some(pattern), filters);
            }
            CommandResult::ApplyFilters(filters) => {
                let display = filters.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
                self.active_filters = filters.clone();
                if !self.results.is_empty() {
                    self.results.retain(|r| r.matches_filters(&filters));
                    self.nav_state.update_results(self.results.len());
                }
                self.status_message = Some(format!("✓ Filters active ({}): {} results shown", display, self.results.len()));
            }
            CommandResult::ShowOverlay { title, content } => {
                self.nav_state.show_overlay(title, content);
            }
            CommandResult::Quit => {
                self.should_quit = true;
            }
            CommandResult::Redraw => {}
            CommandResult::Clear => {
                self.results.clear();
                self.chat_history.clear();
                self.active_filters.clear();
                self.nav_state.close_overlay();
                self.nav_state.update_results(0);
                self.status_message = Some("Cleared results, history, and filters.".to_string());
            }
            CommandResult::ToggleSidebar => { self.nav_state.toggle_sidebar(); }
            CommandResult::NextPage => { self.nav_state.next_page(); }
            CommandResult::PrevPage => { self.nav_state.prev_page(); }
            CommandResult::FirstResult => {
                self.nav_state.select_first();
                self.status_message = Some("→ First result".to_string());
            }
            CommandResult::LastResult => {
                self.nav_state.select_last();
                self.status_message = Some("→ Last result".to_string());
            }
            CommandResult::GotoPage(page) => {
                if self.nav_state.goto_page(page) {
                    self.status_message = Some(format!("→ Page {}/{}", page, self.nav_state.total_pages()));
                } else {
                    self.status_message = Some(format!("✗ Page {} out of range (1..={})", page, self.nav_state.total_pages()));
                }
            }
            CommandResult::GotoTarget { commit, file, line } => {
                let found_idx = self.results.iter().position(|r| {
                    let c_ok = commit.as_ref().map(|c| r.commit_id.starts_with(c)).unwrap_or(true);
                    let f_ok = file.as_ref().map(|f| r.file_path.contains(f)).unwrap_or(true);
                    let l_ok = line.map(|l| r.line_number == l).unwrap_or(true);
                    c_ok && f_ok && l_ok
                });
                if let Some(idx) = found_idx {
                    let per_page = self.nav_state.results_per_page;
                    self.nav_state.page = idx / per_page;
                    self.nav_state.selected_idx = idx % per_page;
                    let r = &self.results[idx];
                    self.status_message = Some(format!("→ Jumped to [{}] {}:{}", r.commit_short, r.file_path, r.line_number));
                } else {
                    self.status_message = Some("✗ No matching result found for /goto target".to_string());
                }
            }
            CommandResult::SessionLoaded { name, entries } => {
                let mut chronological = entries;
                chronological.reverse();
                self.chat_history = chronological;
                self.nav_state.history_selected_idx = 0;
                self.preview_selected_history();
                self.status_message = Some(format!("📂 Loaded session '{}' ({} queries)", name, self.chat_history.len()));
            }
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
