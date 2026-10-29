use crate::commands::parser::{Command, ParsedCommand};
use crate::commands::registry::CommandRegistry;
use crate::languages::{get_patterns_for_lang, Language, LanguageRegistry};
use crate::languages::patterns::all_builtin_patterns;
use crate::modules::{AppConfig, Bookmark, BookmarkManager, ChatEntry, SearchResult, SessionManager};
use std::path::Path;

use super::commands::{export_csv, export_json, export_sarif, execute_search_command};

pub struct CommandExecutor {
    repo_path: String,
    config: AppConfig,
    session_manager: SessionManager,
    bookmark_manager: BookmarkManager,
    registry: CommandRegistry,
    lang_registry: LanguageRegistry,
}

#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(Option<String>),
    ShowOverlay { title: String, content: String },
    Error(String),
    Search { pattern: String, filters: Vec<crate::modules::Filter> },
    ApplyFilters(Vec<crate::modules::Filter>),
    Quit,
    Redraw,
    Clear,
    ToggleSidebar,
    NextPage,
    PrevPage,
    FirstResult,
    LastResult,
    GotoPage(usize),
    GotoTarget { commit: Option<String>, file: Option<String>, line: Option<usize> },
    SessionLoaded { name: String, entries: Vec<ChatEntry> },
    ExportDone(String),
    AiQuery(String),
    AiReport(Option<String>),
    FocusHistory,
    ReplayHistory(usize),
}

impl CommandExecutor {
    pub fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            config: AppConfig::load(),
            session_manager: SessionManager::new(),
            bookmark_manager: BookmarkManager::new(),
            registry: CommandRegistry::new(),
            lang_registry: LanguageRegistry::new(),
        }
    }

    pub fn set_repo_path(&mut self, repo_path: String) {
        self.repo_path = repo_path;
    }

    pub fn record_entry(&mut self, entry: ChatEntry) {
        if let Some(session) = self.session_manager.get_mut_current() {
            session.add_entry(entry);
        }
        self.session_manager.save_current();
    }

    pub fn execute(&mut self, parsed: ParsedCommand, results: &[SearchResult], selected_idx: usize) -> CommandResult {
        // raw_input is stored for audit/logging purposes
        let raw = parsed.raw_input;
        match parsed.command {
            Command::Search { pattern, filters } => {
                if pattern.is_empty() {
                    return CommandResult::Error("Pattern cannot be empty".to_string());
                }
                let _ = execute_search_command(&pattern, filters.clone());
                let _ = raw;
                self.config.save();
                CommandResult::Search { pattern, filters }
            }
            Command::Filter { filters } => {
                if filters.is_empty() {
                    CommandResult::Error("Usage: /filter lang:rust author:name after:2025-01-01".to_string())
                } else {
                    CommandResult::ApplyFilters(filters)
                }
            }
            Command::Export { format, path } => {
                if results.is_empty() {
                    return CommandResult::Error("No results to export".to_string());
                }
                let path_str = path.unwrap_or_else(|| format!("results.{}", format));
                let export_path = Path::new(&path_str);
                match format.as_str() {
                    "json" => {
                        let content = serde_json::to_string_pretty(results).unwrap_or_else(|_| "[]".to_string());
                        match export_json(export_path, &content) {
                            Ok(msg) => CommandResult::ExportDone(msg),
                            Err(e) => CommandResult::Error(e),
                        }
                    }
                    "csv" => {
                        let headers = &["commit", "author", "file", "line", "lang", "content"];
                        let rows: Vec<Vec<String>> = results.iter().map(|r| {
                            vec![
                                r.commit_short.clone(),
                                r.author.clone().unwrap_or_default(),
                                r.file_path.clone(),
                                r.line_number.to_string(),
                                r.lang.clone(),
                                r.matched_text.clone(),
                            ]
                        }).collect();
                        match export_csv(export_path, headers, &rows) {
                            Ok(msg) => CommandResult::ExportDone(msg),
                            Err(e) => CommandResult::Error(e),
                        }
                    }
                    "sarif" => {
                        match export_sarif(export_path, "ast-query", results) {
                            Ok(msg) => CommandResult::ExportDone(msg),
                            Err(e) => CommandResult::Error(e),
                        }
                    }
                    _ => CommandResult::Error(format!("Unknown format: {}. Use json, csv, or sarif.", format)),
                }
            }
            Command::Goto { commit, file, line } => {
                CommandResult::GotoTarget { commit, file, line }
            }
            Command::Bookmark { label, note } => {
                if results.is_empty() {
                    return CommandResult::Error("No results to bookmark".to_string());
                }
                let result = &results[selected_idx.min(results.len() - 1)];
                let label = label.unwrap_or_else(|| result.summary());
                let mut bookmark = Bookmark::new(result.id.clone(), label.clone());
                if let Some(n) = note {
                    bookmark = bookmark.with_note(n);
                }
                self.bookmark_manager.add(bookmark);
                CommandResult::Success(Some(format!("★ Bookmarked: {} ({} total)", label, self.bookmark_manager.len())))
            }
            Command::RemoveBookmark { id } => {
                if self.bookmark_manager.remove(&id) {
                    CommandResult::Success(Some(format!("Removed bookmark: {}", id)))
                } else {
                    CommandResult::Error(format!("Bookmark not found: {}", id))
                }
            }
            Command::Bookmarks => {
                if self.bookmark_manager.is_empty() {
                    CommandResult::Success(Some("No bookmarks saved.".to_string()))
                } else {
                    let list: Vec<String> = self.bookmark_manager.list().iter().map(|b| format!("  {}", b.display())).collect();
                    CommandResult::ShowOverlay {
                        title: format!(" ★ Bookmarks ({}) ", self.bookmark_manager.len()),
                        content: list.join("\n"),
                    }
                }
            }
            Command::Save { name } => {
                let name = name.unwrap_or_else(|| format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M")));
                let current_entries = self
                    .session_manager
                    .get_mut_current()
                    .map(|s| s.entries.clone())
                    .unwrap_or_default();
                let _ = self.session_manager.create_session(name.clone());
                if let Some(new_sess) = self.session_manager.get_mut_current() {
                    new_sess.entries = current_entries;
                }
                self.session_manager.save_current();
                self.config.save();
                CommandResult::Success(Some(format!("💾 Session saved: {}", name)))
            }
            Command::Load { name } => {
                if let Some(name) = name {
                    if let Some(session) = self.session_manager.load_session_by_name(&name) {
                        CommandResult::SessionLoaded {
                            name: session.name.clone(),
                            entries: session.entries.clone(),
                        }
                    } else {
                        CommandResult::Error(format!("Session not found: '{}'. Use /sessions to list.", name))
                    }
                } else {
                    CommandResult::Error("Usage: /load <session_name>".to_string())
                }
            }
            Command::Sessions => {
                let sessions = self.session_manager.list_sessions();
                if sessions.is_empty() {
                    CommandResult::Success(Some("No saved sessions.".to_string()))
                } else {
                    let list = sessions.iter().map(|s| format!("  • {} (id: {}) — {} queries ({})", s.name, s.id, s.entries.len(), s.updated_at.format("%Y-%m-%d %H:%M"))).collect::<Vec<_>>().join("\n");
                    CommandResult::ShowOverlay {
                        title: " 📋 Saved Sessions ".to_string(),
                        content: list,
                    }
                }
            }
            Command::Patterns { lang } => {
                let mut output = String::from("🔍 Built-in AST Patterns:\n\n");
                for (name, pats) in all_builtin_patterns() {
                    if let Some(ref lf) = lang {
                        if !name.to_lowercase().contains(&lf.to_lowercase()) { continue; }
                    }
                    output.push_str(&format!("  {}:\n", name));
                    for p in pats.iter() {
                        output.push_str(&format!("    • [{}] {} — {}\n", p.name, p.pattern, p.description));
                    }
                    output.push('\n');
                }
                let langs_to_show: Vec<Language> = if let Some(ref lf) = lang {
                    self.lang_registry.enabled_languages().iter()
                        .filter(|l| l.matches_filter(lf))
                        .copied().collect()
                } else {
                    self.lang_registry.enabled_languages().to_vec()
                };
                if !langs_to_show.is_empty() {
                    output.push_str("  Detailed Language Rules:\n");
                    for l in &langs_to_show {
                        if self.lang_registry.is_enabled(*l) {
                            let pats = get_patterns_for_lang(*l);
                            if !pats.is_empty() {
                                output.push_str(&format!("  {} {}:\n", l.icon(), l.name()));
                                for p in &pats {
                                    output.push_str(&format!("    {}\n", p.display()));
                                }
                            }
                        }
                    }
                }
                CommandResult::ShowOverlay {
                    title: " 🔍 Built-in AST Pattern Catalog ".to_string(),
                    content: output,
                }
            }
            Command::Help { topic } => {
                if let Some(ref t) = topic {
                    if let Some(cmd) = self.registry.get(t) {
                        let aliases = if cmd.aliases.is_empty() { String::new() } else { format!(" (aliases: {})", cmd.aliases.join(", ")) };
                        CommandResult::ShowOverlay {
                            title: format!(" 📖 Help: /{} ", cmd.name),
                            content: format!("/{}{}\n\nDescription:\n  {}", cmd.name, aliases, cmd.description),
                        }
                    } else {
                        CommandResult::Error(format!("Unknown command: {}. Try /help", t))
                    }
                } else {
                    CommandResult::ShowOverlay {
                        title: " 📖 Commands & Shortcuts ".to_string(),
                        content: format!("📖 Commands:\n{}\n\n⌨ Keys: j/k (select), h/l (pages), gg/G (first/last), d/Space (blob preview), o (open editor), Tab (History), Esc (cancel search), / (cmd), ? (help)", self.registry.help_text()),
                    }
                }
            }
            Command::Quit => CommandResult::Quit,
            Command::Clear => CommandResult::Clear,
            Command::Refresh => CommandResult::Redraw,
            Command::Next => CommandResult::NextPage,
            Command::Prev => CommandResult::PrevPage,
            Command::First => CommandResult::FirstResult,
            Command::Last => CommandResult::LastResult,
            Command::Page { page } => CommandResult::GotoPage(page),
            Command::ToggleSidebar => CommandResult::ToggleSidebar,
            Command::AiQuery { prompt } => CommandResult::AiQuery(prompt),
            Command::AiReport { topic } => CommandResult::AiReport(topic),
            Command::Refactor { pattern, rewrite, apply } => {
                let dry_run = !apply;
                match crate::engine::RefactorEngine::execute(&self.repo_path, &pattern, &rewrite, dry_run, None) {
                    Ok(res) => {
                        let mode_str = if dry_run { "[DRY RUN PREVIEW - No files modified]" } else { "[APPLIED TO DISK]" };
                        let mut msg = format!("🛠 {}\nPattern: {}\nRewrite: {}\nMatches: {} across {} files\n\n", mode_str, res.pattern, res.rewrite, res.total_matches, res.files_affected);
                        for (idx, d) in res.diffs.iter().enumerate() {
                            msg.push_str(&format!("{}. {}:{}\n  - {}\n  + {}\n\n", idx + 1, d.file_path, d.line_number, d.original, d.replacement));
                        }
                        CommandResult::ShowOverlay {
                            title: format!(" 🛠 AST Refactor ({} matches) ", res.total_matches),
                            content: msg,
                        }
                    }
                    Err(e) => CommandResult::Error(format!("Refactor failed: {}", e)),
                }
            }
            Command::Evolution { pattern, options } => {
                match crate::engine::EvolutionEngine::trace_with_options(&self.repo_path, &pattern, options) {
                    Ok(timeline) => {
                        let mut msg = format!("🏺 Semantic AST Blame for '{}' ({} events across {} commits):\n\n", pattern, timeline.total_occurrences, timeline.total_commits_analyzed);
                        for (idx, h) in timeline.history.iter().enumerate() {
                            msg.push_str(&format!("{}. [{}] {} by {} on {}\n   File: {}:{} — {}\n", idx + 1, h.action, h.commit_short, h.author, h.date, h.file_path, h.line_number, h.message));
                            if let Some(ref prev) = h.previous_snippet {
                                for line in prev.lines() {
                                    msg.push_str(&format!("   - {}\n", line));
                                }
                                for line in h.snippet.lines() {
                                    msg.push_str(&format!("   + {}\n", line));
                                }
                                msg.push('\n');
                            } else {
                                msg.push_str(&format!("   Code:\n     {}\n\n", h.snippet.replace('\n', "\n     ")));
                            }
                        }
                        CommandResult::ShowOverlay {
                            title: format!(" 🏺 Evolution Timeline ({} events) ", timeline.total_occurrences),
                            content: msg,
                        }
                    }
                    Err(e) => CommandResult::Error(format!("Evolution trace failed: {}", e)),
                }
            }
            Command::History => CommandResult::FocusHistory,
            Command::Replay { index } => CommandResult::ReplayHistory(index.unwrap_or(0)),
            Command::Nop => CommandResult::Success(None),
        }
    }

    pub fn get_config(&self) -> &AppConfig { &self.config }
    pub fn get_session_manager(&self) -> &SessionManager { &self.session_manager }
    pub fn get_bookmark_manager(&self) -> &BookmarkManager { &self.bookmark_manager }
}

impl Default for CommandExecutor {
    fn default() -> Self { Self::new(".".to_string()) }
}
