use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use crate::ai::OllamaClient;
use crate::engine::{EvolutionEngine, EvolutionOptions, GitEngine, RefactorEngine};
use crate::languages::patterns::all_builtin_patterns;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    #[serde(default)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub struct McpServer {
    repo_path: String,
    ollama: OllamaClient,
}

impl McpServer {
    pub fn new(repo_path: String) -> Self {
        Self {
            repo_path,
            ollama: OllamaClient::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(req) => req,
                Err(e) => {
                    let err_resp = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": format!("Parse error: {}", e)
                        }
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&err_resp)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request);
            if let Some(resp) = response {
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    fn handle_request(&mut self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id;

        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "git-ast-search",
                        "version": "0.2.1"
                    }
                });
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                })
            }
            "notifications/initialized" => {
                // Initialized notification, no response needed
                None
            }
            "ping" => {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({})),
                    error: None,
                })
            }
            "tools/list" => {
                let tools = json!([
                    {
                        "name": "git_ast_search",
                        "description": "Perform structural AST pattern search across all Git commits in the repository history without checkouts.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "pattern": {
                                    "type": "string",
                                    "description": "The ast-grep pattern to find (e.g. 'unsafe { $$$ }', '$A.unwrap()', 'console.log($$$)')"
                                },
                                "repo_path": {
                                    "type": "string",
                                    "description": "Optional repository path (defaults to current repository)"
                                },
                                "lang": {
                                    "type": "string",
                                    "description": "Optional language filter (e.g. 'rust', 'javascript', 'python', 'go')"
                                },
                                "branch": {
                                    "type": "string",
                                    "description": "Optional git branch name to scan instead of HEAD"
                                },
                                "range": {
                                    "type": "string",
                                    "description": "Optional commit range A..B (e.g. 'HEAD~20..HEAD')"
                                },
                                "all": {
                                    "type": "boolean",
                                    "description": "If true, scan across all local and remote branches"
                                },
                                "max_results": {
                                    "type": "integer",
                                    "description": "Maximum number of matching results to return (limits response size)"
                                },
                                "max_commits": {
                                    "type": "integer",
                                    "description": "Maximum number of commits to traverse"
                                },
                                "author": {
                                    "type": "string",
                                    "description": "Optional commit author filter (substring match)"
                                },
                                "path": {
                                    "type": "string",
                                    "description": "Optional file path filter (substring match)"
                                },
                                "after": {
                                    "type": "string",
                                    "description": "Optional date filter: commits on or after YYYY-MM-DD"
                                },
                                "before": {
                                    "type": "string",
                                    "description": "Optional date filter: commits on or before YYYY-MM-DD"
                                }
                            },
                            "required": ["pattern"]
                        }
                    },
                    {
                        "name": "ai_query_ast",
                        "description": "Translate a natural language question (e.g. 'find where memory is leaked' or 'find empty catch blocks') into an ast-grep pattern using Ollama.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "prompt": {
                                    "type": "string",
                                    "description": "Natural language query"
                                }
                            },
                            "required": ["prompt"]
                        }
                    },
                    {
                        "name": "generate_git_ast_report",
                        "description": "Execute an AST search across Git history and synthesize findings into an AI Audit Report with hotspots and recommendations.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "pattern": {
                                    "type": "string",
                                    "description": "The ast-grep pattern or topic to search"
                                },
                                "repo_path": {
                                    "type": "string",
                                    "description": "Optional repository path"
                                }
                            },
                            "required": ["pattern"]
                        }
                    },
                    {
                        "name": "list_ast_patterns",
                        "description": "List built-in ast-grep patterns categorized by language from the catalog.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "lang": {
                                    "type": "string",
                                    "description": "Optional language filter (e.g. 'rust', 'javascript')"
                                }
                            }
                        }
                    },
                    {
                        "name": "git_ast_refactor",
                        "description": "Perform structural code rewriting (ast-grep rewrite) across working tree files with safe dry-run preview.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "pattern": {
                                    "type": "string",
                                    "description": "The ast-grep pattern to match (e.g. '$A.unwrap()')"
                                },
                                "rewrite": {
                                    "type": "string",
                                    "description": "The replacement template (e.g. '$A.unwrap_or_default()')"
                                },
                                "dry_run": {
                                    "type": "boolean",
                                    "description": "If true (default), returns the diff preview without modifying files on disk."
                                },
                                "repo_path": {
                                    "type": "string",
                                    "description": "Optional repository path (defaults to current repository)"
                                },
                                "lang": {
                                    "type": "string",
                                    "description": "Optional language filter (e.g. 'rust', 'javascript')"
                                }
                            },
                            "required": ["pattern", "rewrite"]
                        }
                    },
                    {
                        "name": "git_ast_evolution",
                        "description": "Semantic AST Blame / Archeology: Trace the chronological evolution and mutations of an AST structure across Git commits.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "pattern": {
                                    "type": "string",
                                    "description": "The AST pattern to track historically (e.g. 'fn $F($$$)' or 'std::thread::spawn($$$)')"
                                },
                                "repo_path": {
                                    "type": "string",
                                    "description": "Optional repository path"
                                },
                                "lang": {
                                    "type": "string",
                                    "description": "Optional language filter"
                                },
                                "branch": {
                                    "type": "string",
                                    "description": "Optional branch name to trace instead of HEAD"
                                },
                                "range": {
                                    "type": "string",
                                    "description": "Optional commit range A..B to trace"
                                },
                                "max_commits": {
                                    "type": "integer",
                                    "description": "Maximum number of commits to trace"
                                }
                            },
                            "required": ["pattern"]
                        }
                    }
                ]);

                Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({ "tools": tools })),
                    error: None,
                })
            }
            "tools/call" => {
                let params = req.params.unwrap_or(json!({}));
                let tool_name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];

                let call_result = match tool_name {
                    "git_ast_search" => self.tool_git_ast_search(args),
                    "ai_query_ast" => self.tool_ai_query_ast(args),
                    "generate_git_ast_report" => self.tool_generate_report(args),
                    "list_ast_patterns" => self.tool_list_patterns(args),
                    "git_ast_refactor" => self.tool_git_ast_refactor(args),
                    "git_ast_evolution" => self.tool_git_ast_evolution(args),
                    _ => Err(format!("Unknown tool: {}", tool_name)),
                };

                match call_result {
                    Ok(content) => Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": content
                                }
                            ]
                        })),
                        error: None,
                    }),
                    Err(e) => Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32603,
                            "message": e
                        })),
                    }),
                }
            }
            _ => Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": format!("Method not found: {}", req.method)
                })),
            }),
        }
    }

    fn tool_git_ast_search(&self, args: &Value) -> Result<String, String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| "Missing 'pattern' parameter".to_string())?;
        let repo_path = args["repo_path"].as_str().unwrap_or(&self.repo_path);

        let mut filters = Vec::new();
        if let Some(l) = args["lang"].as_str() {
            filters.push(crate::modules::Filter::new("lang", l));
        }
        if let Some(a) = args["author"].as_str() {
            filters.push(crate::modules::Filter::new("author", a));
        }
        if let Some(p) = args["path"].as_str() {
            filters.push(crate::modules::Filter::new("path", p));
        }
        if let Some(after) = args["after"].as_str() {
            filters.push(crate::modules::Filter::new("after", after));
        }
        if let Some(before) = args["before"].as_str() {
            filters.push(crate::modules::Filter::new("before", before));
        }
        if let Some(br) = args["branch"].as_str() {
            filters.push(crate::modules::Filter::new("branch", br));
        }
        if let Some(rng) = args["range"].as_str() {
            filters.push(crate::modules::Filter::new("range", rng));
        }
        if let Some(all) = args["all"].as_bool() {
            if all {
                filters.push(crate::modules::Filter::new("all", "true"));
            }
        }
        if let Some(mr) = args["max_results"].as_u64() {
            filters.push(crate::modules::Filter::new("max_results", &mr.to_string()));
        }
        if let Some(mc) = args["max_commits"].as_u64() {
            filters.push(crate::modules::Filter::new("max_commits", &mc.to_string()));
        }

        let results = GitEngine::search_sync_with_filters(repo_path, pattern, filters)?;

        serde_json::to_string_pretty(&results)
            .map_err(|e| format!("Failed to serialize results: {}", e))
    }

    fn tool_ai_query_ast(&self, args: &Value) -> Result<String, String> {
        let prompt = args["prompt"]
            .as_str()
            .ok_or_else(|| "Missing 'prompt' parameter".to_string())?;

        let response = self.ollama.query_to_pattern(prompt)?;
        serde_json::to_string_pretty(&response)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    fn tool_generate_report(&self, args: &Value) -> Result<String, String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| "Missing 'pattern' parameter".to_string())?;
        let repo_path = args["repo_path"].as_str().unwrap_or(&self.repo_path);

        let results = GitEngine::search_sync(repo_path, pattern)?;
        self.ollama.generate_report(pattern, &results)
    }

    fn tool_list_patterns(&self, args: &Value) -> Result<String, String> {
        let filter = args["lang"].as_str();
        let all = all_builtin_patterns();
        let mut filtered = Vec::new();

        for (lang, patterns) in all {
            if let Some(f) = filter {
                if !lang.to_lowercase().contains(&f.to_lowercase()) {
                    continue;
                }
            }
            let pats_json: Vec<Value> = patterns.iter().map(|p| {
                json!({
                    "name": p.name,
                    "pattern": p.pattern,
                    "description": p.description
                })
            }).collect();

            filtered.push(json!({
                "language": lang,
                "patterns": pats_json
            }));
        }

        serde_json::to_string_pretty(&filtered)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    fn tool_git_ast_refactor(&self, args: &Value) -> Result<String, String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| "Missing 'pattern' parameter".to_string())?;
        let rewrite = args["rewrite"]
            .as_str()
            .ok_or_else(|| "Missing 'rewrite' parameter".to_string())?;
        let dry_run = args["dry_run"].as_bool().unwrap_or(true);
        let repo_path = args["repo_path"].as_str().unwrap_or(&self.repo_path);
        let lang = args["lang"].as_str();

        let result = RefactorEngine::execute(repo_path, pattern, rewrite, dry_run, lang)?;
        serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    fn tool_git_ast_evolution(&self, args: &Value) -> Result<String, String> {
        let pattern = args["pattern"]
            .as_str()
            .ok_or_else(|| "Missing 'pattern' parameter".to_string())?;
        let repo_path = args["repo_path"].as_str().unwrap_or(&self.repo_path);
        let lang = args["lang"].as_str().map(|s| s.to_string());
        let branch = args["branch"].as_str().map(|s| s.to_string());
        let range = args["range"].as_str().map(|s| s.to_string());
        let max_commits = args["max_commits"].as_u64().map(|n| n as usize);

        let options = EvolutionOptions {
            lang,
            branch,
            range,
            max_commits,
        };

        let result = EvolutionEngine::trace_with_options(repo_path, pattern, options)?;
        serde_json::to_string_pretty(&result)
            .map_err(|e| format!("Serialization error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tools_list_schema_parity() {
        let mut server = McpServer::new(".".to_string());
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };
        let resp = server.handle_request(req).expect("Response expected");
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        let search_tool = tools.iter().find(|t| t["name"] == "git_ast_search").unwrap();
        let search_props = &search_tool["inputSchema"]["properties"];
        assert!(search_props["branch"].is_object());
        assert!(search_props["range"].is_object());
        assert!(search_props["all"].is_object());
        assert!(search_props["max_results"].is_object());
        assert!(search_props["max_commits"].is_object());
        assert!(search_props["author"].is_object());
        assert!(search_props["path"].is_object());
        assert!(search_props["after"].is_object());
        assert!(search_props["before"].is_object());

        let evo_tool = tools.iter().find(|t| t["name"] == "git_ast_evolution").unwrap();
        let evo_props = &evo_tool["inputSchema"]["properties"];
        assert!(evo_props["branch"].is_object());
        assert!(evo_props["range"].is_object());
        assert!(evo_props["max_commits"].is_object());
    }

    #[test]
    fn test_mcp_tool_call_git_ast_search_max_results() {
        let mut server = McpServer::new(".".to_string());
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "git_ast_search",
                "arguments": {
                    "pattern": "fn $F($$$)",
                    "lang": "rust",
                    "max_results": 2
                }
            })),
        };
        let resp = server.handle_request(req).expect("Response expected");
        assert!(resp.error.is_none());
        let content_arr = resp.result.unwrap()["content"].as_array().unwrap().clone();
        let text = content_arr[0]["text"].as_str().unwrap();
        let parsed_results: Vec<Value> = serde_json::from_str(text).unwrap();
        assert!(parsed_results.len() <= 2);
    }
}
