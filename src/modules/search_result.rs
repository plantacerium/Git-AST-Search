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
