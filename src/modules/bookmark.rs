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
    storage_path: std::path::PathBuf,
}

impl BookmarkManager {
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        let mut manager = Self {
            bookmarks: Vec::new(),
            storage_path,
        };
        manager.load();
        manager
    }

    fn get_storage_path() -> std::path::PathBuf {
        directories::ProjectDirs::from("com", "git-ast-search", "GitASTSearch")
            .map(|dirs| dirs.data_dir().join("bookmarks.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("./bookmarks.json"))
    }

    pub fn load(&mut self) {
        if self.storage_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.storage_path) {
                if let Ok(bms) = serde_json::from_str::<Vec<Bookmark>>(&content) {
                    self.bookmarks = bms;
                }
            }
        }
    }

    pub fn save(&self) {
        if let Some(parent) = self.storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.bookmarks) {
            let _ = std::fs::write(&self.storage_path, json);
        }
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
        self.save();
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.id != id);
        let removed = self.bookmarks.len() < initial_len;
        if removed {
            self.save();
        }
        removed
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
