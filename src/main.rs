use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, time::Duration};

pub mod ai;
mod commands;
mod engine;
mod languages;
pub mod mcp;
mod modules;
mod navigation;
pub mod ui;

use ai::OllamaClient;
use commands::commands::to_sarif_json;
use engine::{EvolutionEngine, EvolutionOptions, GitEngine, RefactorEngine};
use mcp::McpServer;
use ui::app::App;
use ui::render::draw_ui;

fn print_cli_help() {
    println!(
        r#"🦀 Git AST Search v0.2.1
High-performance structural code mining across full Git history.

USAGE:
    git-ast-search [OPTIONS] [REPO_PATH]
    git-ast-search query <PATTERN> [OPTIONS]
    git-ast-search ai <PROMPT> [OPTIONS]
    git-ast-search refactor <PATTERN> <REWRITE> [OPTIONS]
    git-ast-search evolution <PATTERN> [OPTIONS]
    git-ast-search mcp [REPO_PATH]

COMMANDS:
    query <PATTERN>             Run headless AST search and output results directly
    ai <PROMPT>                 Translate natural language into an AST pattern via Ollama & search
    refactor <P> <R>            Safe AST-based code rewrite (dry-run by default; use --apply to write)
    evolution <PATTERN>         Semantic AST Blame: Chronological history of a code structure
    mcp [REPO_PATH]             Run as a Model Context Protocol (MCP) server over stdio for IDEs & AI

OPTIONS:
    --repo <PATH>        Target git repository path (default: ".")
    --lang <LANG>        Filter results by language (rust, js, ts, py, go, java, c, cpp)
    --branch <NAME>      Scan a specific branch instead of HEAD
    --range <A..B>       Scan commit range A..B
    --all                Scan across all branches/references
    --max-commits <N>    Limit number of commits traversed
    --max-results <N>    Stop after collecting N matches
    --format <FORMAT>    Output format for headless commands: json, text, sarif (default: json)
    --apply              Actually apply changes to disk for 'refactor' command (disables dry-run)
    --mcp                Start in MCP server mode
    -h, --help           Print help information
    -v, --version        Print version information

EXAMPLES:
    # Open interactive TUI
    git-ast-search .

    # Headless search for unwrap calls in JSON
    git-ast-search query "$A.unwrap()" --lang rust --format json

    # Dry-run AST refactoring preview
    git-ast-search refactor "$A.unwrap()" "$A.unwrap_or_default()" --lang rust

    # Apply structural refactor to codebase
    git-ast-search refactor "$A.unwrap()" "$A.unwrap_or_default()" --apply

    # Track structural evolution across git history
    git-ast-search evolution "std::thread::spawn($$$)" --format text

    # Connect to AI coding agents (Claude, Cursor, Antigravity)
    git-ast-search mcp .
"#
    );
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_cli_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("git-ast-search v0.2.1");
        return Ok(());
    }

    // Check for MCP server mode
    if args.get(1).map(|s| s.as_str()) == Some("mcp") || args.iter().any(|a| a == "--mcp") {
        let repo_path = args.get(2).cloned().unwrap_or_else(|| ".".to_string());
        let mut server = McpServer::new(repo_path);
        server.run()?;
        return Ok(());
    }

    // Check for Headless Query mode
    if args.get(1).map(|s| s.as_str()) == Some("query") || args.get(1).map(|s| s.as_str()) == Some("-q") {
        let pattern = args.get(2).cloned().unwrap_or_default();
        if pattern.is_empty() {
            eprintln!("Error: Missing pattern. Usage: git-ast-search query <PATTERN> [--repo <path>] [--format json|text]");
            std::process::exit(1);
        }

        let mut repo_path = ".".to_string();
        let mut lang_filter = None;
        let mut format = "json".to_string();
        let mut branch = None;
        let mut range = None;
        let mut all_branches = false;
        let mut max_commits: Option<usize> = None;
        let mut max_results: Option<usize> = None;

        let mut i = 3;
        while i < args.len() {
            match args[i].as_str() {
                "--repo" => {
                    if i + 1 < args.len() { repo_path = args[i + 1].clone(); i += 1; }
                }
                "--lang" => {
                    if i + 1 < args.len() { lang_filter = Some(args[i + 1].clone()); i += 1; }
                }
                "--format" => {
                    if i + 1 < args.len() { format = args[i + 1].clone(); i += 1; }
                }
                "--branch" => {
                    if i + 1 < args.len() { branch = Some(args[i + 1].clone()); i += 1; }
                }
                "--range" => {
                    if i + 1 < args.len() { range = Some(args[i + 1].clone()); i += 1; }
                }
                "--all" => {
                    all_branches = true;
                }
                "--max-commits" => {
                    if i + 1 < args.len() { max_commits = args[i + 1].parse().ok(); i += 1; }
                }
                "--max-results" => {
                    if i + 1 < args.len() { max_results = args[i + 1].parse().ok(); i += 1; }
                }
                _ => {}
            }
            i += 1;
        }

        let mut filters = Vec::new();
        if let Some(ref l) = lang_filter {
            filters.push(crate::modules::Filter::new("lang", l));
        }
        if let Some(ref b) = branch {
            filters.push(crate::modules::Filter::new("branch", b));
        }
        if let Some(ref r) = range {
            filters.push(crate::modules::Filter::new("range", r));
        }
        if all_branches {
            filters.push(crate::modules::Filter::new("all", "true"));
        }
        if let Some(mc) = max_commits {
            filters.push(crate::modules::Filter::new("max_commits", &mc.to_string()));
        }
        if let Some(mr) = max_results {
            filters.push(crate::modules::Filter::new("max_results", &mr.to_string()));
        }

        let results = match GitEngine::search_sync_with_filters(&repo_path, &pattern, filters) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error executing search: {}", e);
                std::process::exit(1);
            }
        };

        if format == "text" {
            println!("Found {} results for pattern '{}':", results.len(), pattern);
            for r in results {
                println!("[{}] {}:{} ({})\n{}", r.commit_short, r.file_path, r.line_number, r.lang, r.matched_text);
                println!("---");
            }
        } else if format == "sarif" {
            println!("{}", to_sarif_json(&pattern, &results));
        } else {
            let json_out = serde_json::to_string_pretty(&results)?;
            println!("{}", json_out);
        }
        return Ok(());
    }

    // Check for Headless AI mode
    if args.get(1).map(|s| s.as_str()) == Some("ai") {
        let prompt = args.get(2).cloned().unwrap_or_default();
        if prompt.is_empty() {
            eprintln!("Error: Missing prompt. Usage: git-ast-search ai \"<PROMPT>\" [--repo <path>]");
            std::process::exit(1);
        }

        let mut repo_path = ".".to_string();
        let mut i = 3;
        while i < args.len() {
            if args[i] == "--repo" && i + 1 < args.len() {
                repo_path = args[i + 1].clone();
                i += 1;
            }
            i += 1;
        }

        eprintln!("⟳ Consulting Ollama: \"{}\"...", prompt);
        let client = OllamaClient::new();
        let query_resp = match client.query_to_pattern(&prompt) {
            Ok(q) => q,
            Err(e) => {
                eprintln!("Error contacting Ollama: {}", e);
                std::process::exit(1);
            }
        };

        eprintln!("✓ Generated AST Pattern: {} ({})", query_resp.pattern, query_resp.explanation);
        eprintln!("⟳ Scanning Git repository '{}'...", repo_path);

        let results = match GitEngine::search_sync(&repo_path, &query_resp.pattern) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error executing search: {}", e);
                std::process::exit(1);
            }
        };

        eprintln!("✓ Found {} occurrences.", results.len());
        let json_out = serde_json::json!({
            "prompt": prompt,
            "pattern": query_resp.pattern,
            "lang": query_resp.lang,
            "explanation": query_resp.explanation,
            "results_count": results.len(),
            "results": results
        });
        println!("{}", serde_json::to_string_pretty(&json_out)?);
        return Ok(());
    }

    // Check for Refactor mode
    if args.get(1).map(|s| s.as_str()) == Some("refactor") || args.get(1).map(|s| s.as_str()) == Some("rewrite") {
        let pattern = args.get(2).cloned().unwrap_or_default();
        let rewrite = args.get(3).cloned().unwrap_or_default();
        if pattern.is_empty() || rewrite.is_empty() {
            eprintln!("Error: Usage: git-ast-search refactor <PATTERN> <REWRITE> [--apply] [--repo <path>] [--lang <lang>]");
            std::process::exit(1);
        }

        let mut repo_path = ".".to_string();
        let mut dry_run = true;
        let mut lang_filter = None;

        let mut i = 4;
        while i < args.len() {
            match args[i].as_str() {
                "--apply" => dry_run = false,
                "--repo" => {
                    if i + 1 < args.len() { repo_path = args[i + 1].clone(); i += 1; }
                }
                "--lang" => {
                    if i + 1 < args.len() { lang_filter = Some(args[i + 1].clone()); i += 1; }
                }
                _ => {}
            }
            i += 1;
        }

        let result = match RefactorEngine::execute(&repo_path, &pattern, &rewrite, dry_run, lang_filter.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Refactor error: {}", e);
                std::process::exit(1);
            }
        };

        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    // Check for Evolution / Semantic Blame mode
    if args.get(1).map(|s| s.as_str()) == Some("evolution") || args.get(1).map(|s| s.as_str()) == Some("blame") {
        let pattern = args.get(2).cloned().unwrap_or_default();
        if pattern.is_empty() {
            eprintln!("Error: Usage: git-ast-search evolution <PATTERN> [--repo <path>] [--lang <lang>] [--branch <branch>] [--range <A..B>] [--max-commits <N>] [--format json|text]");
            std::process::exit(1);
        }

        let mut repo_path = ".".to_string();
        let mut lang_filter = None;
        let mut format = "json".to_string();
        let mut branch = None;
        let mut range = None;
        let mut max_commits = None;

        let mut i = 3;
        while i < args.len() {
            match args[i].as_str() {
                "--repo" => {
                    if i + 1 < args.len() { repo_path = args[i + 1].clone(); i += 1; }
                }
                "--lang" => {
                    if i + 1 < args.len() { lang_filter = Some(args[i + 1].clone()); i += 1; }
                }
                "--format" => {
                    if i + 1 < args.len() { format = args[i + 1].clone(); i += 1; }
                }
                "--branch" => {
                    if i + 1 < args.len() { branch = Some(args[i + 1].clone()); i += 1; }
                }
                "--range" => {
                    if i + 1 < args.len() { range = Some(args[i + 1].clone()); i += 1; }
                }
                "--max-commits" => {
                    if i + 1 < args.len() { max_commits = args[i + 1].parse().ok(); i += 1; }
                }
                _ => {}
            }
            i += 1;
        }

        let options = EvolutionOptions {
            lang: lang_filter,
            branch,
            range,
            max_commits,
        };

        let result = match EvolutionEngine::trace_with_options(&repo_path, &pattern, options) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Evolution error: {}", e);
                std::process::exit(1);
            }
        };

        if format == "text" {
            println!("🏺 Semantic AST Blame for '{}' (analyzed {} commits):", result.pattern, result.total_commits_analyzed);
            for h in result.history {
                println!("[{}] {} by {} on {}", h.action, h.commit_short, h.author, h.date);
                println!("  File: {}:{}", h.file_path, h.line_number);
                println!("  Message: {}", h.message);
                if let Some(ref prev) = h.previous_snippet {
                    println!("  Diff:");
                    for l in prev.lines() {
                        println!("    - {}", l);
                    }
                    for l in h.snippet.lines() {
                        println!("    + {}", l);
                    }
                } else {
                    println!("  Code:\n{}", h.snippet);
                }
                println!("---");
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        return Ok(());
    }

    // Interactive TUI Mode
    let repo_path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, event::EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repo_path);

    loop {
        app.update();
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.handle_key(key) {
                        break;
                    }
                }
                Event::Paste(text) => {
                    app.handle_paste(text);
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture, event::DisableBracketedPaste)?;
    terminal.show_cursor()?;
    Ok(())
}
