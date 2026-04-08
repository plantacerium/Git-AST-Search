use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub entries: Vec<super::ChatEntry>,
    pub tags: Vec<String>,
}

impl Session {
    pub fn new(name: String) -> Self {
        let now = Local::now();
        Self {
            id: Self::generate_id(),
            name,
            created_at: now,
            updated_at: now,
            entries: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: super::ChatEntry) {
        self.entries.insert(0, entry);
        self.updated_at = Local::now();
    }

    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("session_{:x}", now)
    }
}

pub struct SessionManager {
    sessions_dir: PathBuf,
    sessions: HashMap<String, Session>,
    current_session: Option<String>,
}

impl SessionManager {
    pub fn new() -> Self {
        let sessions_dir = Self::get_sessions_dir();
        let mut manager = Self {
            sessions_dir,
            sessions: HashMap::new(),
            current_session: None,
        };
        manager.load_all();
        if manager.sessions.is_empty() {
            let _ = manager.create_session("default".to_string());
        }
        manager
    }

    fn get_sessions_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "git-ast-search", "GitASTSearch")
            .map(|dirs| dirs.data_dir().join("sessions"))
            .unwrap_or_else(|| PathBuf::from("./sessions"))
    }

    fn load_all(&mut self) {
        if self.sessions_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.sessions_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(session) = serde_json::from_str::<Session>(&content) {
                                self.sessions.insert(session.id.clone(), session);
                            }
                        }
                    }
                }
            }
        } else {
            let _ = fs::create_dir_all(&self.sessions_dir);
        }
    }

    pub fn create_session(&mut self, name: String) -> &Session {
        let session = Session::new(name);
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.current_session = Some(id.clone());
        self.save(&id);
        self.sessions.get(&id).unwrap()
    }

    pub fn get_mut_current(&mut self) -> Option<&mut Session> {
        self.current_session
            .as_ref()
            .and_then(|id| self.sessions.get_mut(id))
    }

    pub fn save_current(&self) {
        if let Some(id) = &self.current_session {
            self.save(id);
        }
    }

    pub fn save(&self, id: &str) {
        if let Some(session) = self.sessions.get(id) {
            let path = self.sessions_dir.join(format!("{}.json", id));
            if let Ok(json) = serde_json::to_string_pretty(session) {
                let _ = fs::write(path, json);
            }
        }
    }

    pub fn list_sessions(&self) -> Vec<&Session> {
        let mut sessions: Vec<_> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        sessions
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
