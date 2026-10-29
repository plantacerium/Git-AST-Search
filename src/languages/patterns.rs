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
        Language::Kotlin => vec![
            Pattern { name: "force_unwrap".into(), pattern: "$X!!".into(), description: "Unsafe non-null assertion".into(), severity: Severity::Warning },
            Pattern { name: "global_scope".into(), pattern: "GlobalScope.launch { $$$ }".into(), description: "Unscoped coroutine leak".into(), severity: Severity::Warning },
            Pattern { name: "android_log".into(), pattern: "Log.$M($TAG, $MSG)".into(), description: "Android debug logging".into(), severity: Severity::Info },
        ],
        Language::Dart => vec![
            Pattern { name: "set_state".into(), pattern: "setState(() { $$$ })".into(), description: "Flutter state update".into(), severity: Severity::Info },
            Pattern { name: "debug_print".into(), pattern: "debugPrint($$$)".into(), description: "Flutter debug output".into(), severity: Severity::Info },
        ],
        Language::Swift => vec![
            Pattern { name: "force_try".into(), pattern: "try! $F($$$)".into(), description: "Unsafe force try".into(), severity: Severity::Warning },
            Pattern { name: "force_unwrap".into(), pattern: "$X!".into(), description: "Force unwrap optional".into(), severity: Severity::Warning },
        ],
        Language::CSharp => vec![
            Pattern { name: "unsafe_block".into(), pattern: "unsafe { $$$ }".into(), description: "Unsafe C# block".into(), severity: Severity::Warning },
            Pattern { name: "thread_sleep".into(), pattern: "Thread.Sleep($$$)".into(), description: "Thread blocking call".into(), severity: Severity::Warning },
        ],
        Language::Solidity => vec![
            Pattern { name: "tx_origin".into(), pattern: "tx.origin".into(), description: "Phishing vulnerability risk".into(), severity: Severity::Error },
            Pattern { name: "selfdestruct".into(), pattern: "selfdestruct($$$)".into(), description: "Contract destruction".into(), severity: Severity::Error },
        ],
        Language::Bash => vec![
            Pattern { name: "eval".into(), pattern: "eval $$$".into(), description: "Unsafe eval execution".into(), severity: Severity::Error },
        ],
        Language::Ruby => vec![
            Pattern { name: "eval".into(), pattern: "eval($$$)".into(), description: "Arbitrary code execution".into(), severity: Severity::Error },
        ],
        Language::Php => vec![
            Pattern { name: "eval".into(), pattern: "eval($$$)".into(), description: "Arbitrary code execution".into(), severity: Severity::Error },
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

pub static KOTLIN_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "force_unwrap", pattern: "$X!!", description: "Unsafe non-null assertion" },
    BuiltinPattern { name: "global_scope", pattern: "GlobalScope.launch { $$$ }", description: "Unscoped coroutine leak" },
    BuiltinPattern { name: "android_log", pattern: "Log.$M($TAG, $MSG)", description: "Android debug logging" },
];

pub static DART_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "set_state", pattern: "setState(() { $$$ })", description: "Flutter state update" },
    BuiltinPattern { name: "debug_print", pattern: "debugPrint($$$)", description: "Flutter debug output" },
];

pub static SOLIDITY_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "tx_origin", pattern: "tx.origin", description: "Phishing vulnerability risk" },
    BuiltinPattern { name: "selfdestruct", pattern: "selfdestruct($$$)", description: "Contract destruction" },
];

pub static CSHARP_PATTERNS: &[BuiltinPattern] = &[
    BuiltinPattern { name: "unsafe_block", pattern: "unsafe { $$$ }", description: "Unsafe C# block" },
    BuiltinPattern { name: "thread_sleep", pattern: "Thread.Sleep($$$)", description: "Thread blocking call" },
];

pub fn all_builtin_patterns() -> Vec<(&'static str, &'static [BuiltinPattern])> {
    vec![
        ("Rust", RUST_PATTERNS),
        ("JavaScript/TypeScript", JS_PATTERNS),
        ("Python", PYTHON_PATTERNS),
        ("Go", GO_PATTERNS),
        ("Kotlin (Android)", KOTLIN_PATTERNS),
        ("Dart (Flutter)", DART_PATTERNS),
        ("C# (.NET)", CSHARP_PATTERNS),
        ("Solidity (Web3)", SOLIDITY_PATTERNS),
    ]
}
