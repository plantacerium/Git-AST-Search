#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub description: &'static str,
}

pub struct CommandRegistry {
    commands: Vec<CommandInfo>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: vec![
                CommandInfo { name: "search", aliases: vec!["s"], description: "Search AST pattern in git history" },
                CommandInfo { name: "filter", aliases: vec!["f"], description: "Apply filters (lang:rust, author:name)" },
                CommandInfo { name: "export", aliases: vec!["e"], description: "Export results (json/csv)" },
                CommandInfo { name: "goto", aliases: vec!["g"], description: "Navigate to commit/file/line" },
                CommandInfo { name: "bookmark", aliases: vec!["b"], description: "Bookmark selected result" },
                CommandInfo { name: "unbookmark", aliases: vec!["rmbookmark"], description: "Remove bookmark by ID" },
                CommandInfo { name: "bookmarks", aliases: vec![], description: "List all bookmarks" },
                CommandInfo { name: "save", aliases: vec![], description: "Save current session" },
                CommandInfo { name: "load", aliases: vec!["l"], description: "Load a saved session" },
                CommandInfo { name: "sessions", aliases: vec![], description: "List all sessions" },
                CommandInfo { name: "patterns", aliases: vec![], description: "Show builtin AST patterns" },
                CommandInfo { name: "help", aliases: vec!["h", "?"], description: "Show help [topic]" },
                CommandInfo { name: "clear", aliases: vec!["c"], description: "Clear results" },
                CommandInfo { name: "toggle", aliases: vec!["sidebar"], description: "Toggle sidebar" },
                CommandInfo { name: "quit", aliases: vec!["q", "exit"], description: "Exit" },
            ],
        }
    }

    pub fn get(&self, name: &str) -> Option<&CommandInfo> {
        self.commands.iter().find(|c| c.name == name || c.aliases.contains(&name))
    }

    pub fn list_all(&self) -> &[CommandInfo] {
        &self.commands
    }

    pub fn help_text(&self) -> String {
        self.commands.iter().map(|c| {
            let aliases = if c.aliases.is_empty() { String::new() } else { format!(" ({})", c.aliases.join(", ")) };
            format!("  /{}{} — {}", c.name, aliases, c.description)
        }).collect::<Vec<_>>().join("\n")
    }
}

impl Default for CommandRegistry {
    fn default() -> Self { Self::new() }
}
