use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatEntry {
    pub id: String,
    pub query: String,
    pub timestamp: DateTime<Local>,
    pub result_count: usize,
    pub filters_applied: Vec<String>,
}

impl ChatEntry {
    pub fn new(query: String) -> Self {
        Self {
            id: Self::generate_id(),
            query,
            timestamp: Local::now(),
            result_count: 0,
            filters_applied: Vec::new(),
        }
    }

    pub fn with_results(mut self, count: usize) -> Self {
        self.result_count = count;
        self
    }

    pub fn set_result_count(&mut self, count: usize) {
        self.result_count = count;
    }

    pub fn with_filters(mut self, filters: Vec<String>) -> Self {
        self.filters_applied = filters;
        self
    }

    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("entry_{:x}", now)
    }
}
