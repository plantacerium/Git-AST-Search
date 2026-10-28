#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Go,
    Python,
    Java,
    C,
    Cpp,
    Ruby,
    Php,
    Swift,
    Kotlin,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Python => "Python",
            Self::Java => "Java",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Swift => "Swift",
            Self::Kotlin => "Kotlin",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Rust => "🦀",
            Self::JavaScript => "🌐",
            Self::TypeScript => "🔷",
            Self::Go => "🐹",
            Self::Python => "🐍",
            Self::Java => "☕",
            Self::C | Self::Cpp => "⚙️",
            Self::Ruby => "💎",
            Self::Php => "🐘",
            Self::Swift => "🐦",
            Self::Kotlin => "🟣",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "js" | "jsx" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "py" => Some(Self::Python),
            "java" => Some(Self::Java),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" => Some(Self::Cpp),
            "rb" => Some(Self::Ruby),
            "php" => Some(Self::Php),
            "swift" => Some(Self::Swift),
            "kt" => Some(Self::Kotlin),
            _ => None,
        }
    }

    /// Match a user-provided language filter (full name or short alias like js, py, ts, cpp, rs)
    pub fn matches_filter(&self, filter: &str) -> bool {
        let f = filter.trim().to_lowercase();
        if self.name().to_lowercase() == f {
            return true;
        }
        match self {
            Self::Rust => matches!(f.as_str(), "rs" | "rust"),
            Self::JavaScript => matches!(f.as_str(), "js" | "jsx" | "javascript"),
            Self::TypeScript => matches!(f.as_str(), "ts" | "tsx" | "typescript"),
            Self::Go => matches!(f.as_str(), "go" | "golang"),
            Self::Python => matches!(f.as_str(), "py" | "python" | "python3"),
            Self::Java => matches!(f.as_str(), "java"),
            Self::C => matches!(f.as_str(), "c" | "h"),
            Self::Cpp => matches!(f.as_str(), "cpp" | "c++" | "cc" | "cxx" | "hpp"),
            Self::Ruby => matches!(f.as_str(), "rb" | "ruby"),
            Self::Php => matches!(f.as_str(), "php"),
            Self::Swift => matches!(f.as_str(), "swift"),
            Self::Kotlin => matches!(f.as_str(), "kt" | "kotlin"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltinPattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

pub struct LanguageRegistry {
    enabled: Vec<Language>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self {
            enabled: vec![
                Language::Rust,
                Language::JavaScript,
                Language::TypeScript,
                Language::Go,
                Language::Python,
                Language::Java,
                Language::C,
                Language::Cpp,
            ],
        }
    }

    pub fn is_enabled(&self, lang: Language) -> bool {
        self.enabled.contains(&lang)
    }

    pub fn enabled_count(&self) -> usize {
        self.enabled.len()
    }

    pub fn enabled_languages(&self) -> &[Language] {
        &self.enabled
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
