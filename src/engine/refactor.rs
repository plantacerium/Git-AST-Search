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
    /// Substitute meta-variables ($VAR, $FUNC, $A, $$$) in rewrite template in a single safe pass
    pub fn interpolate_rewrite<D: ast_grep_core::Doc>(
        rewrite_template: &str,
        matched: &ast_grep_core::NodeMatch<D>,
    ) -> String {
        let env = matched.get_env();
        let chars: Vec<char> = rewrite_template.chars().collect();
        let mut out = String::with_capacity(rewrite_template.len());
        let mut i = 0;

        while i < chars.len() {
            // Check for $$$ (variadic wildcard, optionally named $$$ARGS)
            if i + 2 < chars.len() && chars[i] == '$' && chars[i + 1] == '$' && chars[i + 2] == '$' {
                i += 3;
                let start_name = i;
                while i < chars.len() && (chars[i].is_ascii_uppercase() || chars[i].is_ascii_digit() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = if i > start_name {
                    chars[start_name..i].iter().collect()
                } else {
                    "$$$".to_string()
                };
                let mut nodes = env.get_multiple_matches(&var_name);
                if nodes.is_empty() && var_name != "$$$" {
                    nodes = env.get_multiple_matches("$$$");
                }
                if !nodes.is_empty() {
                    let joined = nodes
                        .iter()
                        .map(|n| n.text().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&joined);
                }
                continue;
            }

            // Check for $VAR (single or multi-letter uppercase identifier)
            if chars[i] == '$' && i + 1 < chars.len() && (chars[i + 1].is_ascii_uppercase() || chars[i + 1] == '_') {
                let start_name = i + 1;
                let mut j = start_name;
                while j < chars.len() && (chars[j].is_ascii_uppercase() || chars[j].is_ascii_digit() || chars[j] == '_') {
                    j += 1;
                }
                let var_name: String = chars[start_name..j].iter().collect();
                if let Some(node) = env.get_match(&var_name) {
                    out.push_str(&node.text());
                    i = j;
                    continue;
                }
            }

            out.push(chars[i]);
            i += 1;
        }

        out
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
                if !lang.matches_filter(target_lang) {
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
                let line_number = m.start_pos().line() + 1;

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
                fs::write(path, new_content)
                    .map_err(|e| format!("Failed to write refactored file '{}': {}", display_path, e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ast_grep_language::SupportLang;

    #[test]
    fn test_interpolate_multi_letter_and_no_prefix_collision() {
        let code = "fn main() { my_func(alpha, beta); }";
        let grep = AstGrep::new(code, SupportLang::Rust);
        let matched = grep.root().find("$FUNC($A, $AB)").expect("should match");
        let rewritten = RefactorEngine::interpolate_rewrite("$FUNC($AB, $A)", &matched);
        assert_eq!(rewritten, "my_func(beta, alpha)");
    }

    #[test]
    fn test_interpolate_variadic_wildcard() {
        let code = "fn main() { println!(a, b, c); }";
        let grep = AstGrep::new(code, SupportLang::Rust);
        let matched = grep.root().find("println!($$$)").expect("should match");
        let rewritten = RefactorEngine::interpolate_rewrite("log::info!($$$)", &matched);
        assert!(rewritten.starts_with("log::info!("));
    }
}
