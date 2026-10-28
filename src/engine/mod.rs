use crate::languages::LanguageDetector;
use crate::modules::{Filter, SearchResult};
use ast_grep_core::{matcher::MatcherExt, AstGrep, Pattern};
use ast_grep_language::SupportLang;
use crate::languages::Language;
use git2::Repository;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::Sender,
    Arc,
};

pub mod evolution;
pub mod refactor;

pub use evolution::EvolutionEngine;
pub use refactor::RefactorEngine;

/// Maximum blob size to parse with Tree-Sitter (2 MB)
const MAX_BLOB_BYTES: usize = 2 * 1024 * 1024;

pub struct GitEngine {
    repo_path: String,
}

pub enum Message {
    ResultFound(SearchResult),
    Progress { processed: usize, total: usize },
    SearchFinished,
    Error(String),
}

struct BlobTask {
    blob_id: git2::Oid,
    commit_id: String,
    author: Option<String>,
    timestamp: Option<chrono::DateTime<chrono::Utc>>,
    file_path: String,
    lang: Language,
    support_lang: SupportLang,
}

/// Composite structural query supporting inline/YAML-like relational constraints (inside, has, not)
#[derive(Debug, Clone)]
pub struct CompositeQuery {
    pub primary_pattern: String,
    pub inside_pattern: Option<String>,
    pub has_pattern: Option<String>,
    pub not_pattern: Option<String>,
}

impl CompositeQuery {
    /// Parse either a standard AST pattern or a composite rule (YAML-style or inline `inside:`, `has:`, `not:`)
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        let mut primary = String::new();
        let mut inside = None;
        let mut has = None;
        let mut not = None;

        // Check if YAML-like multi-line rule block (`rule:`, `pattern:`, `inside:`, `has:`, `not:`)
        if trimmed.lines().count() > 1
            && (trimmed.contains("pattern:") || trimmed.contains("rule:"))
        {
            let mut current_section = "root";
            for line in trimmed.lines() {
                let l = line.trim();
                if l.starts_with("inside:") {
                    let rest = l.trim_start_matches("inside:").trim();
                    if !rest.is_empty() && rest != "|" {
                        inside = Some(rest.trim_matches(|c| c == '"' || c == '\'').to_string());
                    } else {
                        current_section = "inside";
                    }
                } else if l.starts_with("has:") {
                    let rest = l.trim_start_matches("has:").trim();
                    if !rest.is_empty() && rest != "|" {
                        has = Some(rest.trim_matches(|c| c == '"' || c == '\'').to_string());
                    } else {
                        current_section = "has";
                    }
                } else if l.starts_with("not:") {
                    let rest = l.trim_start_matches("not:").trim();
                    if !rest.is_empty() && rest != "|" {
                        not = Some(rest.trim_matches(|c| c == '"' || c == '\'').to_string());
                    } else {
                        current_section = "not";
                    }
                } else if l.starts_with("pattern:") {
                    let val = l
                        .trim_start_matches("pattern:")
                        .trim()
                        .trim_matches(|c| c == '"' || c == '\'')
                        .to_string();
                    match current_section {
                        "inside" => {
                            inside = Some(val);
                            current_section = "root";
                        }
                        "has" => {
                            has = Some(val);
                            current_section = "root";
                        }
                        "not" => {
                            not = Some(val);
                            current_section = "root";
                        }
                        _ => primary = val,
                    }
                }
            }
            if !primary.is_empty() {
                return Self {
                    primary_pattern: primary,
                    inside_pattern: inside,
                    has_pattern: has,
                    not_pattern: not,
                };
            }
        }

        Self {
            primary_pattern: trimmed.to_string(),
            inside_pattern: None,
            has_pattern: None,
            not_pattern: None,
        }
    }
}

impl GitEngine {
    pub fn new(repo_path: String) -> Self {
        Self { repo_path }
    }

    pub fn get_repo_path(&self) -> &str {
        &self.repo_path
    }

    pub fn to_support_lang(lang: Language) -> Option<SupportLang> {
        match lang {
            Language::Rust => Some(SupportLang::Rust),
            Language::JavaScript => Some(SupportLang::JavaScript),
            Language::TypeScript => Some(SupportLang::TypeScript),
            Language::Go => Some(SupportLang::Go),
            Language::C => Some(SupportLang::C),
            Language::Cpp => Some(SupportLang::Cpp),
            Language::Python => Some(SupportLang::Python),
            Language::Java => Some(SupportLang::Java),
            _ => None,
        }
    }

    /// Extract the longest fixed literal substring (not part of $VAR or $$$) for fast O(N) pre-filtering
    pub fn extract_literal_prefilter(pattern: &str) -> Option<String> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut literals: Vec<String> = Vec::new();
        let mut current = String::new();
        let mut i = 0;

        while i < chars.len() {
            if i + 2 < chars.len() && chars[i] == '$' && chars[i + 1] == '$' && chars[i + 2] == '$' {
                if !current.trim().is_empty() {
                    literals.push(current.trim().to_string());
                }
                current.clear();
                i += 3;
                while i < chars.len()
                    && (chars[i].is_ascii_uppercase() || chars[i].is_ascii_digit() || chars[i] == '_')
                {
                    i += 1;
                }
                continue;
            }
            if chars[i] == '$'
                && i + 1 < chars.len()
                && (chars[i + 1].is_ascii_uppercase() || chars[i + 1] == '_')
            {
                if !current.trim().is_empty() {
                    literals.push(current.trim().to_string());
                }
                current.clear();
                i += 1;
                while i < chars.len()
                    && (chars[i].is_ascii_uppercase() || chars[i].is_ascii_digit() || chars[i] == '_')
                {
                    i += 1;
                }
                continue;
            }
            if chars[i].is_whitespace() {
                if !current.trim().is_empty() {
                    literals.push(current.trim().to_string());
                }
                current.clear();
                i += 1;
                continue;
            }
            current.push(chars[i]);
            i += 1;
        }
        if !current.trim().is_empty() {
            literals.push(current.trim().to_string());
        }

        literals
            .into_iter()
            .filter(|s| s.len() >= 2)
            .max_by_key(|s| s.len())
    }

    /// Extract a snippet with +-2 lines of surrounding context around 1-based `line_number`
    pub fn extract_context_snippet(content: &str, line_number: usize, matched_text: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return matched_text.to_string();
        }
        let idx = line_number.saturating_sub(1);
        let match_line_count = matched_text.lines().count().max(1);
        let start = idx.saturating_sub(2);
        let end = (idx + match_line_count + 2).min(lines.len());
        if start < end {
            lines[start..end].join("\n")
        } else {
            matched_text.to_string()
        }
    }

    /// Read the full file content from a historical Git commit without performing a checkout
    pub fn read_blob_at_commit(
        repo_path: &str,
        commit_id: &str,
        file_path: &str,
    ) -> Result<String, String> {
        let repo = Repository::open(repo_path)
            .map_err(|e| format!("Failed to open repository: {}", e))?;
        let obj = repo
            .revparse_single(commit_id)
            .map_err(|e| format!("Commit '{}' not found: {}", commit_id, e))?;
        let commit = obj
            .peel_to_commit()
            .map_err(|e| format!("Not a commit '{}': {}", commit_id, e))?;
        let tree = commit
            .tree()
            .map_err(|e| format!("Failed to read commit tree: {}", e))?;
        let entry = tree
            .get_path(std::path::Path::new(file_path))
            .map_err(|e| format!("File '{}' not found in commit {}: {}", file_path, commit_id, e))?;
        let blob = entry
            .to_object(&repo)
            .and_then(|o| o.into_blob().map_err(|_| git2::Error::from_str("Not a blob")))
            .map_err(|e| format!("Failed to read blob: {}", e))?;
        Ok(String::from_utf8_lossy(blob.content()).to_string())
    }

    pub fn run_search(repo_path: &str, pattern: String, tx: Sender<Message>) {
        Self::run_search_with_options(
            repo_path,
            pattern,
            Vec::new(),
            Arc::new(AtomicBool::new(false)),
            tx,
        );
    }

    pub fn run_search_with_filters(
        repo_path: &str,
        pattern: String,
        filters: Vec<Filter>,
        tx: Sender<Message>,
    ) {
        Self::run_search_with_options(
            repo_path,
            pattern,
            filters,
            Arc::new(AtomicBool::new(false)),
            tx,
        );
    }

    pub fn run_search_with_options(
        repo_path: &str,
        pattern: String,
        filters: Vec<Filter>,
        cancel_flag: Arc<AtomicBool>,
        tx: Sender<Message>,
    ) {
        let detector = LanguageDetector::new();
        let repo = match Repository::open(repo_path) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Message::Error(format!("Failed to open repo: {}", e)));
                return;
            }
        };
        let mut revwalk = match repo.revwalk() {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Message::Error(format!("Revwalk error: {}", e)));
                return;
            }
        };
        let _ = revwalk.set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL);

        // Pre-parse filters and scope options (branch, all, range, max_commits, max_results)
        let mut lang_filters = Vec::new();
        let mut author_filters = Vec::new();
        let mut path_filters = Vec::new();
        let mut commit_filters = Vec::new();
        let mut after_filters = Vec::new();
        let mut before_filters = Vec::new();
        let mut branch_opt: Option<String> = None;
        let mut range_opt: Option<String> = None;
        let mut walk_all = false;
        let mut max_commits: Option<usize> = None;
        let mut max_results: usize = 5000;

        for f in &filters {
            let k = f.field.to_lowercase();
            let v = f.value.trim();
            let vl = v.to_lowercase();
            match k.as_str() {
                "lang" | "language" | "l" => lang_filters.push(vl),
                "author" | "a" | "user" => author_filters.push(vl),
                "file" | "path" | "f" => path_filters.push(vl),
                "commit" | "c" => commit_filters.push(vl),
                "after" | "since" => after_filters.push(vl),
                "before" | "until" => before_filters.push(vl),
                "branch" | "b" => branch_opt = Some(v.to_string()),
                "range" => range_opt = Some(v.to_string()),
                "all" => walk_all = vl == "true" || vl == "1" || vl == "yes",
                "max_commits" | "commits" => {
                    if let Ok(n) = v.parse::<usize>() {
                        max_commits = Some(n);
                    }
                }
                "max_results" | "limit" => {
                    if let Ok(n) = v.parse::<usize>() {
                        max_results = n;
                    }
                }
                _ => {
                    // Also check if composite filter like `inside:fn $F()` was passed as filter
                }
            }
        }

        // Configure revwalk scope (--range, --branch, --all, or default HEAD)
        if let Some(ref rng) = range_opt {
            if let Err(e) = revwalk.push_range(rng) {
                let _ = tx.send(Message::Error(format!("Invalid commit range '{}': {}", rng, e)));
                return;
            }
        } else if walk_all {
            let _ = revwalk.push_glob("refs/heads/*");
            let _ = revwalk.push_glob("refs/remotes/*");
            let _ = revwalk.push_head();
        } else if let Some(ref br) = branch_opt {
            let pushed = repo
                .find_branch(br, git2::BranchType::Local)
                .or_else(|_| repo.find_branch(br, git2::BranchType::Remote))
                .and_then(|b| {
                    b.get()
                        .target()
                        .ok_or_else(|| git2::Error::from_str("Branch has no target"))
                })
                .and_then(|oid| revwalk.push(oid));
            if let Err(e) = pushed {
                let _ = tx.send(Message::Error(format!("Branch '{}' not found: {}", br, e)));
                return;
            }
        } else if let Err(e) = revwalk.push_head() {
            let _ = tx.send(Message::Error(format!("No HEAD: {}", e)));
            return;
        }

        let mut composite = CompositeQuery::parse(&pattern);
        for f in &filters {
            match f.field.to_lowercase().as_str() {
                "inside" => composite.inside_pattern = Some(f.value.clone()),
                "has" => composite.has_pattern = Some(f.value.clone()),
                "not" => composite.not_pattern = Some(f.value.clone()),
                _ => {}
            }
        }

        let literal_prefilter = Self::extract_literal_prefilter(&composite.primary_pattern);

        // Phase 1: Deterministic O(1) blob deduplication & metadata collection
        let mut visited_blobs = HashSet::new();
        let mut tasks = Vec::new();
        let mut commits_seen = 0usize;

        for oid_res in revwalk {
            if cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(Message::SearchFinished);
                return;
            }
            if let Some(limit_c) = max_commits {
                if commits_seen >= limit_c {
                    break;
                }
            }

            let oid = match oid_res {
                Ok(id) => id,
                Err(_) => continue,
            };
            commits_seen += 1;

            let oid_str = oid.to_string();
            if !commit_filters.is_empty()
                && !commit_filters.iter().any(|cf| oid_str.to_lowercase().starts_with(cf))
            {
                continue;
            }

            let commit = match repo.find_commit(oid) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let author_name = commit.author().name().map(|s| s.to_string());
            if !author_filters.is_empty() {
                let matches_author = author_name
                    .as_ref()
                    .map(|a| {
                        let al = a.to_lowercase();
                        author_filters.iter().any(|af| al.contains(af))
                    })
                    .unwrap_or(false);
                if !matches_author {
                    continue;
                }
            }

            let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0);
            if !after_filters.is_empty() || !before_filters.is_empty() {
                if let Some(ts) = &timestamp {
                    let date_str = ts.format("%Y-%m-%d").to_string();
                    if after_filters.iter().any(|af| &date_str < af) {
                        continue;
                    }
                    if before_filters.iter().any(|bf| &date_str > bf) {
                        continue;
                    }
                }
            }

            if let Ok(tree) = commit.tree() {
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

                        if !lang_filters.is_empty()
                            && !lang_filters.iter().any(|lf| lang.matches_filter(lf))
                        {
                            return git2::TreeWalkResult::Ok;
                        }

                        let support_lang = match Self::to_support_lang(lang) {
                            Some(sl) => sl,
                            None => return git2::TreeWalkResult::Ok,
                        };

                        let full_path = format!("{}{}", root, name);
                        if !path_filters.is_empty() {
                            let fp_lower = full_path.to_lowercase();
                            if !path_filters.iter().any(|pf| fp_lower.contains(pf)) {
                                return git2::TreeWalkResult::Ok;
                            }
                        }

                        let blob_id = entry.id();
                        if !visited_blobs.insert(blob_id) {
                            return git2::TreeWalkResult::Ok;
                        }

                        tasks.push(BlobTask {
                            blob_id,
                            commit_id: oid_str.clone(),
                            author: author_name.clone(),
                            timestamp,
                            file_path: full_path,
                            lang,
                            support_lang,
                        });
                    }
                    git2::TreeWalkResult::Ok
                });
            }
        }

        let total_blobs = tasks.len();
        let processed_blobs = Arc::new(AtomicUsize::new(0));
        let results_count = Arc::new(AtomicUsize::new(0));

        // Phase 2: Parallel AST parsing across unique blobs
        // Each worker thread opens Repository once and caches compiled Pattern per SupportLang
        let repo_path_owned = repo_path.to_string();
        tasks.par_iter().for_each_init(
            || {
                (
                    Repository::open(&repo_path_owned).ok(),
                    HashMap::<SupportLang, Pattern>::new(),
                )
            },
            |(thread_repo_opt, pattern_cache), task| {
                if cancel_flag.load(Ordering::Relaxed)
                    || results_count.load(Ordering::Relaxed) >= max_results
                {
                    return;
                }

                let done = processed_blobs.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 25 == 0 || done == total_blobs {
                    let _ = tx.send(Message::Progress {
                        processed: done,
                        total: total_blobs,
                    });
                }

                if let Some(thread_repo) = thread_repo_opt {
                    if let Ok(blob) = thread_repo.find_blob(task.blob_id) {
                        // Skip binary or huge blobs (> 2 MB)
                        if blob.is_binary() || blob.size() > MAX_BLOB_BYTES {
                            return;
                        }

                        let content = String::from_utf8_lossy(blob.content());

                        // Fast SIMD substring prefilter before invoking Tree-Sitter
                        if let Some(ref lit) = literal_prefilter {
                            if !content.contains(lit.as_str()) {
                                return;
                            }
                        }

                        let compiled_pat = pattern_cache
                            .entry(task.support_lang)
                            .or_insert_with(|| Pattern::new(&composite.primary_pattern, task.support_lang))
                            .clone();

                        let grep = AstGrep::new(content.as_ref(), task.support_lang);

                        for matched in grep.root().find_all(&compiled_pat) {
                            if results_count.load(Ordering::Relaxed) >= max_results {
                                break;
                            }

                            // Optional relational filters: `not`, `has`, `inside`
                            if let Some(ref not_pat_str) = composite.not_pattern {
                                let not_pat = Pattern::new(not_pat_str, task.support_lang);
                                if not_pat.match_node(matched.get_node().clone()).is_some() {
                                    continue;
                                }
                            }
                            if let Some(ref has_pat_str) = composite.has_pattern {
                                let has_pat = Pattern::new(has_pat_str, task.support_lang);
                                if matched.get_node().find(&has_pat).is_none() {
                                    continue;
                                }
                            }
                            if let Some(ref inside_pat_str) = composite.inside_pattern {
                                let inside_pat = Pattern::new(inside_pat_str, task.support_lang);
                                let mut found_anc = false;
                                for anc in matched.get_node().ancestors() {
                                    if inside_pat.match_node(anc).is_some() {
                                        found_anc = true;
                                        break;
                                    }
                                }
                                if !found_anc {
                                    continue;
                                }
                            }

                            let line_number = matched.start_pos().line() + 1;
                            let matched_str = matched.text().to_string();
                            let context_snippet =
                                Self::extract_context_snippet(&content, line_number, &matched_str);

                            let result = SearchResult::new(
                                task.commit_id.clone(),
                                task.file_path.clone(),
                                context_snippet,
                                matched_str,
                                line_number,
                                task.lang.name().to_string(),
                            )
                            .with_metadata(task.author.clone(), task.timestamp);

                            results_count.fetch_add(1, Ordering::Relaxed);
                            let _ = tx.send(Message::ResultFound(result));
                        }
                    }
                }
            },
        );

        let _ = tx.send(Message::SearchFinished);
    }

    /// Synchronous search returning all collected results (used by CLI headless and MCP)
    pub fn search_sync(repo_path: &str, pattern: &str) -> Result<Vec<SearchResult>, String> {
        Self::search_sync_with_filters(repo_path, pattern, Vec::new())
    }

    pub fn search_sync_with_filters(
        repo_path: &str,
        pattern: &str,
        filters: Vec<Filter>,
    ) -> Result<Vec<SearchResult>, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        Self::run_search_with_filters(repo_path, pattern.to_string(), filters, tx);
        let mut results = Vec::new();
        while let Ok(msg) = rx.recv() {
            match msg {
                Message::ResultFound(r) => results.push(r),
                Message::Progress { .. } => {}
                Message::SearchFinished => break,
                Message::Error(e) => return Err(e),
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_literal_prefilter() {
        assert_eq!(
            GitEngine::extract_literal_prefilter("$A.unwrap()"),
            Some(".unwrap()".to_string())
        );
        assert_eq!(
            GitEngine::extract_literal_prefilter("unsafe { $$$ }"),
            Some("unsafe".to_string())
        );
        assert_eq!(
            GitEngine::extract_literal_prefilter("$A"),
            None
        );
    }

    #[test]
    fn test_composite_yaml_query_parse() {
        let yaml = r#"
rule:
  pattern: $A.unwrap()
  inside: fn $FUNC($$$) { $$$ }
  not: test_unwrap()
"#;
        let q = CompositeQuery::parse(yaml);
        assert_eq!(q.primary_pattern, "$A.unwrap()");
        assert_eq!(q.inside_pattern.as_deref(), Some("fn $FUNC($$$) { $$$ }"));
        assert_eq!(q.not_pattern.as_deref(), Some("test_unwrap()"));
    }
}
