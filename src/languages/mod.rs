pub mod detector;
pub mod patterns;
pub mod registry;

pub use detector::LanguageDetector;
pub use patterns::get_patterns_for_lang;
pub use registry::{BuiltinPattern, Language, LanguageRegistry, Severity};

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "rust", "javascript", "typescript", "go", "python", "java", "c", "cpp",
    "ruby", "php", "swift", "kotlin",
];

pub fn supported_languages_display() -> String {
    SUPPORTED_LANGUAGES
        .iter()
        .map(|l| {
            let mut c = l.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().to_string() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
