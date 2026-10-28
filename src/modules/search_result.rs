use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub commit_id: String,
    pub commit_short: String,
    pub file_path: String,
    pub content: String,
    pub matched_text: String,
    pub line_number: usize,
    pub lang: String,
    pub author: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl SearchResult {
    pub fn new(
        commit_id: String,
        file_path: String,
        content: String,
        matched_text: String,
        line_number: usize,
        lang: String,
    ) -> Self {
        let commit_short = commit_id.chars().take(7).collect();
        Self {
            id: uuid_v4(),
            commit_id,
            commit_short,
            file_path,
            content,
            matched_text,
            line_number,
            lang,
            author: None,
            timestamp: None,
        }
    }

    pub fn with_metadata(
        mut self,
        author: Option<String>,
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        self.author = author;
        self.timestamp = timestamp;
        self
    }

    /// Check if this result satisfies all given filters (lang, author, file/path, commit, after/since, before/until)
    pub fn matches_filters(&self, filters: &[super::Filter]) -> bool {
        for f in filters {
            let key = f.field.to_lowercase();
            let val = f.value.to_lowercase();
            match key.as_str() {
                "lang" | "language" | "l" => {
                    let lang_ok = crate::languages::Language::from_extension(&val)
                        .map(|l| l.name().eq_ignore_ascii_case(&self.lang))
                        .unwrap_or(false)
                        || self.lang.to_lowercase() == val
                        || (val == "cpp" && self.lang == "C++");
                    if !lang_ok {
                        return false;
                    }
                }
                "author" | "a" | "user" => {
                    let author_ok = self
                        .author
                        .as_ref()
                        .map(|a| a.to_lowercase().contains(&val))
                        .unwrap_or(false);
                    if !author_ok {
                        return false;
                    }
                }
                "file" | "path" | "f" => {
                    if !self.file_path.to_lowercase().contains(&val) {
                        return false;
                    }
                }
                "commit" | "c" => {
                    if !self.commit_id.to_lowercase().starts_with(&val) {
                        return false;
                    }
                }
                "after" | "since" => {
                    if let Some(ts) = &self.timestamp {
                        let date_str = ts.format("%Y-%m-%d").to_string();
                        if date_str < val {
                            return false;
                        }
                    }
                }
                "before" | "until" => {
                    if let Some(ts) = &self.timestamp {
                        let date_str = ts.format("%Y-%m-%d").to_string();
                        if date_str > val {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// One-line summary for display in lists
    pub fn summary(&self) -> String {
        format!(
            "[{}] {}:{} ({})",
            self.commit_short, self.file_path, self.line_number, self.lang
        )
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}
