use super::Language;

pub struct LanguageDetector;

impl LanguageDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_from_extension(&self, ext: &str) -> Option<Language> {
        Language::from_extension(ext)
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}
