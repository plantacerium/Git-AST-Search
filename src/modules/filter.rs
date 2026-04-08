use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub field: String,
    pub value: String,
}

impl Filter {
    pub fn new(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            value: value.to_string(),
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        if let Some(pos) = s.find(':') {
            let field = s[..pos].trim().to_string();
            let value = s[pos + 1..].trim().to_string();
            Some(Self { field, value })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.field, self.value)
    }
}
