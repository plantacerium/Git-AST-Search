use crate::commands::parser::{Command, ParsedCommand};
use crate::commands::registry::CommandRegistry;
use crate::languages::{get_patterns_for_lang, Language, LanguageRegistry};
use crate::languages::patterns::all_builtin_patterns;
use crate::modules::{AppConfig, Bookmark, BookmarkManager, ChatEntry, SearchResult, SessionManager};
use std::path::Path;

use super::commands::{export_csv, export_json, execute_search_command};

pub struct CommandExecutor {
    config: AppConfig,
    session_manager: SessionManager,
    bookmark_manager: BookmarkManager,
    registry: CommandRegistry,
    lang_registry: LanguageRegistry,
}

#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(Option<String>),
    Error(String),
    Search(String),
    Quit,
    Redraw,
    Clear,
    ToggleSidebar,
    NextPage,
    PrevPage,
    ExportDone(String),
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            config: AppConfig::load(),
            session_manager: SessionManager::new(),
            bookmark_manager: BookmarkManager::new(),
            registry: CommandRegistry::new(),
            lang_registry: LanguageRegistry::new(),
        }
    }

    pub fn execute(&mut self, parsed: ParsedCommand, results: &[SearchResult], selected_idx: usize) -> CommandResult {
        // raw_input is stored for audit/logging purposes
        let raw = parsed.raw_input;
        match parsed.command {
            Command::Search { pattern, filters } => {
                if pattern.is_empty() {
                    return CommandResult::Error("Pattern cannot be empty".to_string());
                }
                let msg = execute_search_command(&pattern, filters.clone());
                let entry = ChatEntry::new(raw.clone())
                    .with_results(0)
                    .with_filters(filters.iter().map(|f| f.to_string()).collect());
                if let Some(session) = self.session_manager.get_mut_current() {
                    session.add_entry(entry);
                }
                self.session_manager.save_current();
                self.config.save();
                let _ = msg; // search description logged
                CommandResult::Search(pattern)
            }
            Command::Filter { filters } => {
                let display = filters.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
                if filters.is_empty() {
                    CommandResult::Error("Usage: /filter lang:rust or /filter author:name".to_string())
                } else {
                    CommandResult::Success(Some(format!("Filters applied: {}", display)))
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
                        let headers = &["commit", "file", "line", "lang", "content"];
                        let rows: Vec<Vec<String>> = results.iter().map(|r| {
                            vec![r.commit_short.clone(), r.file_path.clone(), r.line_number.to_string(), r.lang.clone(), r.matched_text.clone()]
                        }).collect();
                        match export_csv(export_path, headers, &rows) {
                            Ok(msg) => CommandResult::ExportDone(msg),
                            Err(e) => CommandResult::Error(e),
                        }
                    }
                    _ => CommandResult::Error(format!("Unknown format: {}. Use json or csv.", format)),
                }
            }
            Command::Goto { commit, file, line } => {
                let target = format!("{}{}{}", commit.as_deref().unwrap_or(""), file.as_ref().map(|f| format!(":{}", f)).unwrap_or_default(), line.map(|l| format!(":{}", l)).unwrap_or_default());
                CommandResult::Success(Some(format!("→ {}", target)))
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
                    CommandResult::Success(Some(format!("★ Bookmarks ({}):\n{}", self.bookmark_manager.len(), list.join("\n"))))
                }
            }
            Command::Save { name } => {
                let name = name.unwrap_or_else(|| format!("session_{}", chrono::Local::now().format("%Y%m%d_%H%M")));
                let _ = self.session_manager.create_session(name.clone());
                self.config.save();
                CommandResult::Success(Some(format!("💾 Session saved: {}", name)))
            }
            Command::Load { name } => {
                if let Some(name) = name {
                    CommandResult::Success(Some(format!("📂 Loaded: {}", name)))
                } else {
                    CommandResult::Error("Usage: /load <session_name>".to_string())
                }
            }
            Command::Sessions => {
                let sessions = self.session_manager.list_sessions();
                if sessions.is_empty() {
                    CommandResult::Success(Some("No saved sessions.".to_string()))
                } else {
                    let list = sessions.iter().map(|s| format!("  {} — {} entries ({})", s.name, s.entries.len(), s.updated_at.format("%Y-%m-%d %H:%M"))).collect::<Vec<_>>().join("\n");
                    CommandResult::Success(Some(format!("📋 Sessions:\n{}", list)))
                }
            }
            Command::Patterns { lang } => {
                let mut output = String::from("🔍 Built-in Patterns:\n");
                // Show builtin static patterns
                for (name, pats) in all_builtin_patterns() {
                    if let Some(ref lf) = lang {
                        if !name.to_lowercase().contains(&lf.to_lowercase()) { continue; }
                    }
                    output.push_str(&format!("  {}:\n", name));
                    for p in pats.iter() {
                        output.push_str(&format!("    [{}] {} — {}\n", p.name, p.pattern, p.description));
                    }
                }
                // Show dynamic patterns with severity via get_patterns_for_lang
                let langs_to_show: Vec<Language> = if let Some(ref lf) = lang {
                    self.lang_registry.enabled_languages().iter()
                        .filter(|l| l.name().to_lowercase().contains(&lf.to_lowercase()))
                        .copied().collect()
                } else {
                    self.lang_registry.enabled_languages().to_vec()
                };
                if !langs_to_show.is_empty() {
                    output.push_str("\n  Detailed patterns:\n");
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
                CommandResult::Success(Some(output))
            }
            Command::Help { topic } => {
                if let Some(ref t) = topic {
                    if let Some(cmd) = self.registry.get(t) {
                        let aliases = if cmd.aliases.is_empty() { String::new() } else { format!(" (aliases: {})", cmd.aliases.join(", ")) };
                        CommandResult::Success(Some(format!("/{}{}\n  {}", cmd.name, aliases, cmd.description)))
                    } else {
                        CommandResult::Error(format!("Unknown command: {}. Try /help", t))
                    }
                } else {
                    CommandResult::Success(Some(format!("📖 Commands:\n{}\n\n⌨ Keys: j/k (select), h/l (pages), / (cmd), ? (help)", self.registry.help_text())))
                }
            }
            Command::Quit => CommandResult::Quit,
            Command::Clear => CommandResult::Clear,
            Command::Refresh => CommandResult::Redraw,
            Command::Next => CommandResult::NextPage,
            Command::Prev => CommandResult::PrevPage,
            Command::ToggleSidebar => CommandResult::ToggleSidebar,
            Command::Nop => CommandResult::Success(None),
        }
    }

    pub fn get_config(&self) -> &AppConfig { &self.config }
    pub fn get_session_manager(&self) -> &SessionManager { &self.session_manager }
    pub fn get_bookmark_manager(&self) -> &BookmarkManager { &self.bookmark_manager }
}

impl Default for CommandExecutor {
    fn default() -> Self { Self::new() }
}
