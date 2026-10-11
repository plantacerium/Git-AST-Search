use serde::{Deserialize, Serialize};
use crate::modules::SearchResult;
use super::prompts::{ast_query_system_prompt, report_system_prompt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstQueryResponse {
    pub pattern: String,
    pub lang: Option<String>,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub host: String,
    pub model: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        let host = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "qwen2.5-coder:latest".to_string());
        Self { host, model }
    }

    pub fn with_config(host: Option<String>, model: Option<String>) -> Self {
        let mut client = Self::new();
        if let Some(h) = host {
            client.host = h;
        }
        if let Some(m) = model {
            client.model = m;
        }
        client
    }

    /// Check if Ollama service is reachable
    pub fn is_available(&self) -> bool {
        let endpoint = format!("{}/api/tags", self.host.trim_end_matches('/'));
        ureq::get(&endpoint)
            .timeout(std::time::Duration::from_millis(1500))
            .call()
            .is_ok()
    }

    /// Translate a natural language prompt to an ast-grep pattern
    pub fn query_to_pattern(&self, user_prompt: &str) -> Result<AstQueryResponse, String> {
        let endpoint = format!("{}/api/chat", self.host.trim_end_matches('/'));
        
        let request_payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": ast_query_system_prompt()
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "stream": false,
            "options": {
                "temperature": 0.1
            }
        });

        let response = match ureq::post(&endpoint)
            .timeout(std::time::Duration::from_secs(45))
            .send_json(request_payload)
        {
            Ok(res) => res,
            Err(e) => {
                return Err(format!(
                    "Ollama request failed at {}: {}. Ensure Ollama is running ('ollama serve').",
                    self.host, e
                ));
            }
        };

        let response_body: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("Failed to parse Ollama JSON response: {}", e))?;

        let raw_content = response_body["message"]["content"]
            .as_str()
            .ok_or_else(|| "Ollama returned an empty response message".to_string())?
            .trim();

        // Extract JSON if wrapped in markdown codeblocks
        let clean_json = if let Some(start) = raw_content.find('{') {
            if let Some(end) = raw_content.rfind('}') {
                &raw_content[start..=end]
            } else {
                raw_content
            }
        } else {
            raw_content
        };

        match serde_json::from_str::<AstQueryResponse>(clean_json) {
            Ok(parsed) => Ok(parsed),
            Err(_) => {
                // Fallback attempt: if model just returned a raw pattern string
                let pattern = raw_content.lines().next().unwrap_or(raw_content).trim().to_string();
                Ok(AstQueryResponse {
                    pattern,
                    lang: None,
                    explanation: "Generated pattern from query".to_string(),
                })
            }
        }
    }

    /// Generate an audit report from search results
    pub fn generate_report(&self, topic: &str, results: &[SearchResult]) -> Result<String, String> {
        let endpoint = format!("{}/api/chat", self.host.trim_end_matches('/'));

        let mut sample_items = Vec::new();
        for (idx, r) in results.iter().take(25).enumerate() {
            sample_items.push(format!(
                "{}. File: {} (Line: {}, Commit: {})\n```{}\n{}\n```",
                idx + 1,
                r.file_path,
                r.line_number,
                r.commit_short,
                r.lang,
                r.matched_text.lines().take(5).collect::<Vec<_>>().join("\n")
            ));
        }

        let user_content = format!(
            "Query / Topic: {}\nTotal Results Found in Git History: {}\n\nTop Results Sample:\n{}",
            topic,
            results.len(),
            sample_items.join("\n\n")
        );

        let request_payload = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": report_system_prompt()
                },
                {
                    "role": "user",
                    "content": user_content
                }
            ],
            "stream": false,
            "options": {
                "temperature": 0.3
            }
        });

        let response = match ureq::post(&endpoint)
            .timeout(std::time::Duration::from_secs(60))
            .send_json(request_payload)
        {
            Ok(res) => res,
            Err(e) => {
                return Err(format!(
                    "Ollama request failed at {}: {}. Ensure Ollama is running ('ollama serve').",
                    self.host, e
                ));
            }
        };

        let response_body: serde_json::Value = response
            .into_json()
            .map_err(|e| format!("Failed to parse Ollama JSON response: {}", e))?;

        let report = response_body["message"]["content"]
            .as_str()
            .ok_or_else(|| "Ollama returned an empty report message".to_string())?
            .to_string();

        Ok(report)
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}
