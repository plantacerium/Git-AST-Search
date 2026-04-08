use crate::modules::Filter;

#[derive(Debug, Clone)]
pub enum Command {
    Search { pattern: String, filters: Vec<Filter> },
    Filter { filters: Vec<Filter> },
    Export { format: String, path: Option<String> },
    Goto { commit: Option<String>, file: Option<String>, line: Option<usize> },
    Bookmark { label: Option<String>, note: Option<String> },
    RemoveBookmark { id: String },
    Bookmarks,
    Save { name: Option<String> },
    Load { name: Option<String> },
    Sessions,
    Help { topic: Option<String> },
    Quit,
    Clear,
    Refresh,
    Next,
    Prev,
    ToggleSidebar,
    Patterns { lang: Option<String> },
    Nop,
}

#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub command: Command,
    pub raw_input: String,
}

pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str) -> Result<ParsedCommand, String> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(ParsedCommand { command: Command::Nop, raw_input: input.to_string() });
        }

        if !input.starts_with('/') {
            return Ok(ParsedCommand {
                command: Command::Search { pattern: input.to_string(), filters: Vec::new() },
                raw_input: input.to_string(),
            });
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().map(|s| s.trim_start_matches('/')).unwrap_or("");

        let command = match cmd {
            "s" | "search" => {
                let mut pattern_parts = Vec::new();
                let mut filters = Vec::new();
                let mut i = 1;
                while i < parts.len() {
                    if parts[i].starts_with("--") && i + 1 < parts.len() {
                        let key = parts[i].trim_start_matches("--");
                        let val = parts[i + 1];
                        filters.push(Filter::new(key, val));
                        i += 2;
                    } else if let Some(f) = Filter::parse(parts[i]) {
                        filters.push(f);
                        i += 1;
                    } else {
                        pattern_parts.push(parts[i]);
                        i += 1;
                    }
                }
                Command::Search { pattern: pattern_parts.join(" "), filters }
            }
            "f" | "filter" => {
                let filters: Vec<Filter> = parts.iter().skip(1).filter_map(|s| Filter::parse(s)).collect();
                Command::Filter { filters }
            }
            "e" | "export" => {
                let format = parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "json".to_string());
                let path = parts.get(2).map(|s| s.to_string());
                Command::Export { format, path }
            }
            "g" | "goto" => Command::Goto {
                commit: parts.get(1).map(|s| s.to_string()),
                file: parts.get(2).map(|s| s.to_string()),
                line: parts.get(3).and_then(|s| s.parse().ok()),
            },
            "b" | "bookmark" => {
                let label = parts.get(1).map(|s| s.to_string());
                let note_idx = parts.iter().position(|s| *s == "--note");
                let note = note_idx.map(|i| parts[i + 1..].join(" "));
                Command::Bookmark { label, note }
            }
            "unbookmark" | "rmbookmark" => {
                let id = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
                Command::RemoveBookmark { id }
            }
            "bookmarks" => Command::Bookmarks,
            "save" => Command::Save { name: parts.get(1).map(|s| s.to_string()) },
            "l" | "load" => Command::Load { name: parts.get(1).map(|s| s.to_string()) },
            "sessions" => Command::Sessions,
            "h" | "help" | "?" => Command::Help { topic: parts.get(1).map(|s| s.to_string()) },
            "q" | "quit" | "exit" => Command::Quit,
            "c" | "clear" => Command::Clear,
            "r" | "refresh" => Command::Refresh,
            "next" | "n" => Command::Next,
            "prev" | "p" => Command::Prev,
            "toggle" | "sidebar" => Command::ToggleSidebar,
            "patterns" => Command::Patterns { lang: parts.get(1).map(|s| s.to_string()) },
            _ => return Err(format!("Unknown command: /{}", cmd)),
        };

        Ok(ParsedCommand { command, raw_input: input.to_string() })
    }
}
