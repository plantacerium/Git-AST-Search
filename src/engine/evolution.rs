use ast_grep_core::{AstGrep, Pattern};
use ast_grep_language::SupportLang;
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
    pub action: String, // "INTRODUCED", "MUTATED", "REMOVED"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_snippet: Option<String>,
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

        // Cache AST matches per blob Oid so unchanged files across commits are parsed only once
        let mut blob_cache: HashMap<Oid, Vec<(usize, String)>> = HashMap::new();
        let mut pattern_cache: HashMap<SupportLang, Pattern> = HashMap::new();
        let literal_prefilter = GitEngine::extract_literal_prefilter(pattern);
        let mut previous_state: HashMap<String, Vec<(usize, String)>> = HashMap::new();
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

            let mut current_commit_matches: HashMap<String, Vec<(usize, String)>> = HashMap::new();

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
                        if !lang.matches_filter(target_l) {
                            return git2::TreeWalkResult::Ok;
                        }
                    }

                    let support_lang = match GitEngine::to_support_lang(lang) {
                        Some(sl) => sl,
                        None => return git2::TreeWalkResult::Ok,
                    };

                    let blob_id = entry.id();
                    let matches_for_blob = if let Some(cached) = blob_cache.get(&blob_id) {
                        cached.clone()
                    } else if let Ok(blob) = entry
                        .to_object(&repo)
                        .and_then(|obj| obj.into_blob().map_err(|_| git2::Error::from_str("Not a blob")))
                    {
                        if blob.is_binary() || blob.size() > 2 * 1024 * 1024 {
                            blob_cache.insert(blob_id, Vec::new());
                            Vec::new()
                        } else {
                            let content = String::from_utf8_lossy(blob.content());
                            if let Some(ref lit) = literal_prefilter {
                                if !content.contains(lit.as_str()) {
                                    blob_cache.insert(blob_id, Vec::new());
                                    return git2::TreeWalkResult::Ok;
                                }
                            }
                            let compiled_pat = pattern_cache
                                .entry(support_lang)
                                .or_insert_with(|| Pattern::new(pattern, support_lang))
                                .clone();
                            let grep = AstGrep::new(content.as_ref(), support_lang);
                            let mut found = Vec::new();
                            for matched in grep.root().find_all(&compiled_pat) {
                                let line = matched.start_pos().line() + 1;
                                let text = matched.text().to_string();
                                found.push((line, text));
                            }
                            blob_cache.insert(blob_id, found.clone());
                            found
                        }
                    } else {
                        Vec::new()
                    };

                    if !matches_for_blob.is_empty() {
                        let file_path = format!("{}{}", root, name);
                        current_commit_matches.insert(file_path, matches_for_blob);
                    }
                }
                git2::TreeWalkResult::Ok
            });

            // Detect INTRODUCED and MUTATED across files present in current commit
            for (file, matches) in &current_commit_matches {
                let current_snippets: Vec<&str> = matches.iter().map(|(_, s)| s.as_str()).collect();
                let (action, prev_snip_opt) = match previous_state.get(file) {
                    None => ("INTRODUCED", None),
                    Some(prev_matches) => {
                        let prev_snippets: Vec<&str> = prev_matches.iter().map(|(_, s)| s.as_str()).collect();
                        if prev_snippets != current_snippets {
                            let prev_combined = if prev_matches.len() == 1 {
                                prev_matches[0].1.clone()
                            } else {
                                prev_matches
                                    .iter()
                                    .map(|(l, s)| format!("// [Line {}]\n{}", l, s))
                                    .collect::<Vec<_>>()
                                    .join("\n---\n")
                            };
                            ("MUTATED", Some(prev_combined))
                        } else {
                            ("PERSISTED", None)
                        }
                    }
                };

                if action != "PERSISTED" {
                    let (first_line, _) = &matches[0];
                    let combined_snippet = if matches.len() == 1 {
                        matches[0].1.clone()
                    } else {
                        matches
                            .iter()
                            .map(|(l, s)| format!("// [Line {}]\n{}", l, s))
                            .collect::<Vec<_>>()
                            .join("\n---\n")
                    };
                    history.push(EvolutionEntry {
                        commit_id: oid.to_string(),
                        commit_short: commit_short.clone(),
                        author: author.clone(),
                        date: date.clone(),
                        message: message.clone(),
                        file_path: file.clone(),
                        line_number: *first_line,
                        action: action.to_string(),
                        previous_snippet: prev_snip_opt,
                        snippet: combined_snippet,
                    });
                }
            }

            // Detect REMOVED for files that previously had the pattern but no longer match in this commit
            for (prev_file, prev_matches) in &previous_state {
                if !current_commit_matches.contains_key(prev_file) {
                    let (last_line, last_snip) = &prev_matches[0];
                    history.push(EvolutionEntry {
                        commit_id: oid.to_string(),
                        commit_short: commit_short.clone(),
                        author: author.clone(),
                        date: date.clone(),
                        message: message.clone(),
                        file_path: prev_file.clone(),
                        line_number: *last_line,
                        action: "REMOVED".to_string(),
                        previous_snippet: Some(last_snip.clone()),
                        snippet: last_snip.clone(),
                    });
                }
            }

            previous_state = current_commit_matches;
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
