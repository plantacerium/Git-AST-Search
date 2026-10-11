use ast_grep_core::AstGrep;
use git2::{Oid, Repository, Sort};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::languages::LanguageDetector;
use super::GitEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    pub commit_id: String,
    pub commit_short: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub file_path: String,
    pub line_number: usize,
    pub action: String, // "INTRODUCED", "MUTATED", "PRESENT"
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionTimeline {
    pub pattern: String,
    pub total_commits_analyzed: usize,
    pub total_occurrences: usize,
    pub history: Vec<EvolutionEntry>,
}

pub struct EvolutionEngine;

impl EvolutionEngine {
    pub fn trace(repo_path: &str, pattern: &str, lang_filter: Option<&str>) -> Result<EvolutionTimeline, String> {
        let repo = Repository::open(repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;
        let detector = LanguageDetector::new();

        let mut revwalk = repo.revwalk()
            .map_err(|e| format!("Revwalk error: {}", e))?;
        // Chronological order: oldest commits first
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
            .map_err(|e| format!("Sort error: {}", e))?;
        revwalk.push_head()
            .map_err(|e| format!("Push HEAD error: {}", e))?;

        let oids: Vec<Oid> = revwalk.filter_map(|id| id.ok()).collect();
        let total_commits = oids.len();

        let mut previous_state: HashMap<String, String> = HashMap::new();
        let mut history = Vec::new();

        for oid in oids {
            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let author_sig = commit.author();
            let author = author_sig.name().unwrap_or("Unknown").to_string();
            let time = commit.time();
            let date = chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown date".to_string());
            let message = commit.summary().unwrap_or("").trim().to_string();
            let commit_short = oid.to_string().chars().take(7).collect::<String>();

            let tree = match commit.tree() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let mut current_commit_matches: HashMap<String, (usize, String)> = HashMap::new();

            let _ = tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                if let Some(name) = entry.name() {
                    let ext = std::path::Path::new(name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    let lang = match detector.detect_from_extension(ext) {
                        Some(l) => l,
                        None => return git2::TreeWalkResult::Ok,
                    };

                    if let Some(target_l) = lang_filter {
                        if !lang.name().eq_ignore_ascii_case(target_l) {
                            return git2::TreeWalkResult::Ok;
                        }
                    }

                    let support_lang = match GitEngine::to_support_lang(lang) {
                        Some(sl) => sl,
                        None => return git2::TreeWalkResult::Ok,
                    };

                    if let Ok(blob) = entry.to_object(&repo).and_then(|obj| obj.into_blob().map_err(|_| git2::Error::from_str("Not a blob"))) {
                        let content = String::from_utf8_lossy(blob.content());
                        let grep = AstGrep::new(content.as_ref(), support_lang);

                        let file_path = format!("{}{}", root, name);
                        for matched in grep.root().find_all(pattern) {
                            let range = matched.range();
                            let line = content[..range.start].lines().count();
                            let text = matched.text().to_string();
                            current_commit_matches.insert(file_path.clone(), (line, text));
                        }
                    }
                }
                git2::TreeWalkResult::Ok
            });

            // Compare against previous state to determine action
            for (file, (line, snippet)) in &current_commit_matches {
                let action = match previous_state.get(file) {
                    None => "INTRODUCED",
                    Some(prev_snip) if prev_snip != snippet => "MUTATED",
                    Some(_) => "PERSISTED",
                };

                // Only record meaningful state changes or key milestone events
                if action != "PERSISTED" || history.is_empty() {
                    history.push(EvolutionEntry {
                        commit_id: oid.to_string(),
                        commit_short: commit_short.clone(),
                        author: author.clone(),
                        date: date.clone(),
                        message: message.clone(),
                        file_path: file.clone(),
                        line_number: *line,
                        action: action.to_string(),
                        snippet: snippet.clone(),
                    });
                }

                previous_state.insert(file.clone(), snippet.clone());
            }
        }

        let total_occurrences = history.len();
        Ok(EvolutionTimeline {
            pattern: pattern.to_string(),
            total_commits_analyzed: total_commits,
            total_occurrences,
            history,
        })
    }
}
