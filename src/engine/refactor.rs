use ast_grep_core::AstGrep;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::languages::LanguageDetector;
use super::GitEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorDiff {
    pub file_path: String,
    pub line_number: usize,
    pub original: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorResult {
    pub pattern: String,
    pub rewrite: String,
    pub dry_run: bool,
    pub total_matches: usize,
    pub files_affected: usize,
    pub diffs: Vec<RefactorDiff>,
}

pub struct RefactorEngine;

impl RefactorEngine {
    /// Substitute meta-variables ($VAR) in rewrite template with matched node text
    fn interpolate_rewrite<D: ast_grep_core::Doc>(
        rewrite_template: &str,
        matched: &ast_grep_core::NodeMatch<D>,
    ) -> String {
        let env = matched.get_env();
        let mut result = rewrite_template.to_string();

        // Extract potential $VAR or $A tokens
        // Check single uppercase letters ($A..$Z) and common variable names
        for c in 'A'..='Z' {
            let var_name = c.to_string();
            let var_token = format!("${}", var_name);
            if result.contains(&var_token) {
                if let Some(node) = env.get_match(&var_name) {
                    result = result.replace(&var_token, &node.text());
                }
            }
        }

        // Also check if any $$$ wildcard exists
        if result.contains("$$$") {
            let nodes = env.get_multiple_matches("$$$");
            if !nodes.is_empty() {
                let joined = nodes.iter().map(|n| n.text().to_string()).collect::<Vec<_>>().join(", ");
                result = result.replace("$$$", &joined);
            }
        }

        result
    }

    /// Perform AST-based refactor across working directory files
    pub fn execute(
        repo_path: &str,
        pattern: &str,
        rewrite: &str,
        dry_run: bool,
        lang_filter: Option<&str>,
    ) -> Result<RefactorResult, String> {
        let detector = LanguageDetector::new();
        let mut all_diffs = Vec::new();
        let mut files_changed_count = 0;

        let walker = ignore::WalkBuilder::new(repo_path)
            .hidden(true)
            .git_ignore(true)
            .build();

        for entry_res in walker {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let lang = match detector.detect_from_extension(ext) {
                Some(l) => l,
                None => continue,
            };

            if let Some(target_lang) = lang_filter {
                if !lang.name().eq_ignore_ascii_case(target_lang) {
                    continue;
                }
            }

            let support_lang = match GitEngine::to_support_lang(lang) {
                Some(sl) => sl,
                None => continue,
            };

            let file_content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let grep = AstGrep::new(&file_content, support_lang);
            let matches: Vec<_> = grep.root().find_all(pattern).collect();
            if matches.is_empty() {
                continue;
            }

            files_changed_count += 1;
            let display_path = path.strip_prefix(repo_path).unwrap_or(path).to_string_lossy().to_string();

            // Store replacement ranges (start, end, replacement_text, line, original_text)
            let mut replacements = Vec::new();
            for m in matches {
                let range = m.range();
                let original_text = m.text().to_string();
                let replacement_text = Self::interpolate_rewrite(rewrite, &m);
                let line_number = file_content[..range.start].lines().count();

                all_diffs.push(RefactorDiff {
                    file_path: display_path.clone(),
                    line_number,
                    original: original_text.clone(),
                    replacement: replacement_text.clone(),
                });

                replacements.push((range.start, range.end, replacement_text));
            }

            if !dry_run {
                // Apply replacements from back to front to avoid invalidating offsets
                replacements.sort_by(|a, b| b.0.cmp(&a.0));
                let mut new_content = file_content.clone();
                for (start, end, rep) in replacements {
                    if start <= end && end <= new_content.len() {
                        new_content.replace_range(start..end, &rep);
                    }
                }
                let _ = fs::write(path, new_content);
            }
        }

        let total_matches = all_diffs.len();
        Ok(RefactorResult {
            pattern: pattern.to_string(),
            rewrite: rewrite.to_string(),
            dry_run,
            total_matches,
            files_affected: files_changed_count,
            diffs: all_diffs,
        })
    }
}
