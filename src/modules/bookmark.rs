use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub result_id: String,
    pub label: String,
    pub note: Option<String>,
    pub created_at: String,
}

impl Bookmark {
    pub fn new(result_id: String, label: String) -> Self {
        Self {
            id: Self::generate_id(),
            result_id,
            label,
            note: None,
            created_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        }
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("bm_{:x}", now)
    }

    pub fn display(&self) -> String {
        let note_str = self.note.as_deref().unwrap_or("");
        if note_str.is_empty() {
            format!("[{}] {} ({})", self.id, self.label, self.created_at)
        } else {
            format!("[{}] {} — {} ({})", self.id, self.label, note_str, self.created_at)
        }
    }
}

pub struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
}

impl BookmarkManager {
    pub fn new() -> Self {
        Self { bookmarks: Vec::new() }
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.id != id);
        self.bookmarks.len() < initial_len
    }

    pub fn len(&self) -> usize {
        self.bookmarks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    pub fn list(&self) -> &[Bookmark] {
        &self.bookmarks
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new()
    }
}
