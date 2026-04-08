use crate::languages::LanguageDetector;
use crate::modules::SearchResult;
use ast_grep_core::AstGrep;
use ast_grep_language::SupportLang;
use crate::languages::Language;
use dashmap::DashSet;
use git2::{Oid, Repository};
use rayon::prelude::*;
use std::sync::{mpsc::Sender, Arc};

pub struct GitEngine {
    repo_path: String,
}

pub enum Message {
    ResultFound(SearchResult),
    SearchFinished,
    Error(String),
}

impl GitEngine {
    pub fn new(repo_path: String) -> Self {
        Self { repo_path }
    }

    pub fn get_repo_path(&self) -> &str {
        &self.repo_path
    }

    fn to_support_lang(lang: Language) -> Option<SupportLang> {
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

    pub fn run_search(repo_path: &str, pattern: String, tx: Sender<Message>) {
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
        if let Err(e) = revwalk.push_head() {
            let _ = tx.send(Message::Error(format!("No HEAD: {}", e)));
            return;
        }

        let oids: Vec<Oid> = revwalk.filter_map(|id| id.ok()).collect();
        let visited_blobs = Arc::new(DashSet::new());
        let repo_path_owned = repo_path.to_string();

        oids.par_chunks(100).for_each(|chunk| {
            if let Ok(thread_repo) = Repository::open(&repo_path_owned) {
                for oid in chunk {
                    if let Ok(commit) = thread_repo.find_commit(*oid) {
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
                                    let support_lang = match Self::to_support_lang(lang) {
                                        Some(sl) => sl,
                                        None => return git2::TreeWalkResult::Ok,
                                    };

                                    let blob_id = entry.id();
                                    if !visited_blobs.insert(blob_id) {
                                        return git2::TreeWalkResult::Ok;
                                    }

                                    if let Ok(blob) = entry.to_object(&thread_repo).and_then(|obj| {
                                        obj.into_blob().map_err(|_| git2::Error::from_str("Not a blob"))
                                    }) {
                                        let content = String::from_utf8_lossy(blob.content());
                                        let grep = AstGrep::new(content.as_ref(), support_lang);

                                        for matched in grep.root().find_all(pattern.as_str()) {
                                            let range = matched.range();
                                            let line_number = content[..range.start].lines().count();
                                            let result = SearchResult::new(
                                                oid.to_string(),
                                                format!("{}{}", root, name),
                                                matched.text().to_string(),
                                                matched.text().to_string(),
                                                line_number,
                                                lang.name().to_string(),
                                            );
                                            let _ = tx.send(Message::ResultFound(result));
                                        }
                                    }
                                }
                                git2::TreeWalkResult::Ok
                            });
                        }
                    }
                }
            }
        });

        let _ = tx.send(Message::SearchFinished);
    }
}
