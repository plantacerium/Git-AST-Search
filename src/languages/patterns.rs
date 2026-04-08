use super::{BuiltinPattern, Language, Severity};

#[derive(Debug, Clone)]
pub struct Pattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub severity: Severity,
}

impl Pattern {
    pub fn display(&self) -> String {
        let sev = match self.severity {
            Severity::Info => "ℹ",
            Severity::Warning => "⚠",
            Severity::Error => "✗",
        };
        format!("{} [{}] {} — {}", sev, self.name, self.pattern, self.description)
    }
}

pub fn get_patterns_for_lang(lang: Language) -> Vec<Pattern> {
    match lang {
        Language::Rust => vec![
            Pattern { name: "unsafe_block".into(), pattern: "unsafe { $$$ }".into(), description: "Unsafe Rust block".into(), severity: Severity::Warning },
            Pattern { name: "unwrap".into(), pattern: "$A.unwrap()".into(), description: "unwrap() call".into(), severity: Severity::Warning },
            Pattern { name: "panic".into(), pattern: "panic!($$$)".into(), description: "panic! macro".into(), severity: Severity::Error },
        ],
        Language::JavaScript | Language::TypeScript => vec![
            Pattern { name: "console_log".into(), pattern: "console.log($$$)".into(), description: "console.log statement".into(), severity: Severity::Info },
            Pattern { name: "eval".into(), pattern: "eval($$$)".into(), description: "eval() usage - security risk".into(), severity: Severity::Error },
        ],
        Language::Python => vec![
            Pattern { name: "mutable_default".into(), pattern: "def $FUNC($ARG = []):".into(), description: "Mutable default argument".into(), severity: Severity::Error },
            Pattern { name: "eval".into(), pattern: "eval($$$)".into(), description: "eval() usage".into(), severity: Severity::Error },
        ],
        Language::Go => vec![
            Pattern { name: "ignored_error".into(), pattern: "$VAL, _ := $FUNC($$$)".into(), description: "Ignored error return".into(), severity: Severity::Warning },
            Pattern { name: "panic".into(), pattern: "panic($$$)".into(), description: "panic() call".into(), severity: Severity::Error },
        ],
        Language::Java => vec![
            Pattern { name: "sysout".into(), pattern: "System.out.println($$$)".into(), description: "Debug print statement".into(), severity: Severity::Info },
            Pattern { name: "catch_all".into(), pattern: "catch (Exception $E) { $$$ }".into(), description: "Generic exception catch".into(), severity: Severity::Warning },
        ],
        _ => Vec::new(),
    }
}

pub static RUST_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "unsafe_block", pattern: "unsafe { $$$ }", description: "Unsafe Rust block" },
    BuiltinPattern { name: "unwrap", pattern: "$A.unwrap()", description: "unwrap() call" },
    BuiltinPattern { name: "panic", pattern: "panic!($$$)", description: "panic! macro" },
];

pub static JS_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "console_log", pattern: "console.log($$$)", description: "console.log statement" },
    BuiltinPattern { name: "eval", pattern: "eval($$$)", description: "eval() usage" },
];

pub static PYTHON_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "mutable_default", pattern: "def $FUNC($ARG = []):", description: "Mutable default" },
    BuiltinPattern { name: "eval", pattern: "eval($$$)", description: "eval() usage" },
];

pub static GO_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "ignored_error", pattern: "$VAL, _ := $FUNC($$$)", description: "Ignored error" },
    BuiltinPattern { name: "panic", pattern: "panic($$$)", description: "panic() call" },
];

pub fn all_builtin_patterns() -> Vec<(&'static str, &'static [BuiltinPattern])> {
    vec![
        ("Rust", RUST_PATTERNS),
        ("JavaScript/TypeScript", JS_PATTERNS),
        ("Python", PYTHON_PATTERNS),
        ("Go", GO_PATTERNS),
    ]
}
