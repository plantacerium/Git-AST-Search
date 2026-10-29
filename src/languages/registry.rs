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
    CSharp,
    Dart,
    Bash,
    Html,
    Css,
    Json,
    Yaml,
    Elixir,
    Lua,
    Scala,
    Solidity,
    Hcl,
    Nix,
    Haskell,
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
            Self::CSharp => "C#",
            Self::Dart => "Dart",
            Self::Bash => "Bash",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Elixir => "Elixir",
            Self::Lua => "Lua",
            Self::Scala => "Scala",
            Self::Solidity => "Solidity",
            Self::Hcl => "HCL",
            Self::Nix => "Nix",
            Self::Haskell => "Haskell",
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
            Self::CSharp => "🔷",
            Self::Dart => "🎯",
            Self::Bash => "🐚",
            Self::Html => "🌐",
            Self::Css => "🎨",
            Self::Json => "📋",
            Self::Yaml => "📄",
            Self::Elixir => "💧",
            Self::Lua => "🌙",
            Self::Scala => "🔴",
            Self::Solidity => "⛓️",
            Self::Hcl => "🏗️",
            Self::Nix => "❄️",
            Self::Haskell => "λ",
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
            "kt" | "kts" => Some(Self::Kotlin),
            "cs" => Some(Self::CSharp),
            "dart" => Some(Self::Dart),
            "sh" | "bash" | "zsh" => Some(Self::Bash),
            "html" | "htm" => Some(Self::Html),
            "css" | "scss" => Some(Self::Css),
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "ex" | "exs" => Some(Self::Elixir),
            "lua" => Some(Self::Lua),
            "scala" | "sc" => Some(Self::Scala),
            "sol" => Some(Self::Solidity),
            "tf" | "hcl" => Some(Self::Hcl),
            "nix" => Some(Self::Nix),
            "hs" => Some(Self::Haskell),
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
            Self::Swift => matches!(f.as_str(), "swift" | "ios"),
            Self::Kotlin => matches!(f.as_str(), "kt" | "kotlin" | "kts" | "android"),
            Self::CSharp => matches!(f.as_str(), "cs" | "csharp" | "c#" | "dotnet"),
            Self::Dart => matches!(f.as_str(), "dart" | "flutter"),
            Self::Bash => matches!(f.as_str(), "sh" | "bash" | "shell" | "zsh"),
            Self::Html => matches!(f.as_str(), "html" | "htm"),
            Self::Css => matches!(f.as_str(), "css" | "scss"),
            Self::Json => matches!(f.as_str(), "json"),
            Self::Yaml => matches!(f.as_str(), "yaml" | "yml"),
            Self::Elixir => matches!(f.as_str(), "ex" | "elixir" | "exs"),
            Self::Lua => matches!(f.as_str(), "lua"),
            Self::Scala => matches!(f.as_str(), "scala" | "sc"),
            Self::Solidity => matches!(f.as_str(), "sol" | "solidity" | "web3"),
            Self::Hcl => matches!(f.as_str(), "tf" | "hcl" | "terraform"),
            Self::Nix => matches!(f.as_str(), "nix"),
            Self::Haskell => matches!(f.as_str(), "hs" | "haskell"),
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
                Language::Ruby,
                Language::Php,
                Language::Swift,
                Language::Kotlin,
                Language::CSharp,
                Language::Dart,
                Language::Bash,
                Language::Html,
                Language::Css,
                Language::Json,
                Language::Yaml,
                Language::Elixir,
                Language::Lua,
                Language::Scala,
                Language::Solidity,
                Language::Hcl,
                Language::Nix,
                Language::Haskell,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_count_and_enabled() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.enabled_count(), 26);
        assert!(registry.is_enabled(Language::Kotlin));
        assert!(registry.is_enabled(Language::Dart));
        assert!(registry.is_enabled(Language::Solidity));
        assert!(registry.is_enabled(Language::CSharp));
    }

    #[test]
    fn test_mobile_and_android_detection() {
        assert_eq!(Language::from_extension("kt"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("kts"), Some(Language::Kotlin));
        assert_eq!(Language::from_extension("dart"), Some(Language::Dart));
        assert_eq!(Language::from_extension("swift"), Some(Language::Swift));

        // Filter aliases
        assert!(Language::Kotlin.matches_filter("android"));
        assert!(Language::Kotlin.matches_filter("kt"));
        assert!(Language::Kotlin.matches_filter("kotlin"));
        assert!(Language::Dart.matches_filter("flutter"));
        assert!(Language::Dart.matches_filter("dart"));
        assert!(Language::Swift.matches_filter("ios"));
    }

    #[test]
    fn test_new_generation_and_consolidated_languages() {
        assert_eq!(Language::from_extension("sol"), Some(Language::Solidity));
        assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));
        assert_eq!(Language::from_extension("sh"), Some(Language::Bash));
        assert_eq!(Language::from_extension("ex"), Some(Language::Elixir));
        assert_eq!(Language::from_extension("lua"), Some(Language::Lua));
        assert_eq!(Language::from_extension("scala"), Some(Language::Scala));
        assert_eq!(Language::from_extension("tf"), Some(Language::Hcl));
        assert_eq!(Language::from_extension("nix"), Some(Language::Nix));
        assert_eq!(Language::from_extension("hs"), Some(Language::Haskell));

        // Aliases
        assert!(Language::Solidity.matches_filter("web3"));
        assert!(Language::Solidity.matches_filter("sol"));
        assert!(Language::CSharp.matches_filter("c#"));
        assert!(Language::CSharp.matches_filter("dotnet"));
        assert!(Language::Hcl.matches_filter("terraform"));
        assert!(Language::Bash.matches_filter("shell"));
    }
}
