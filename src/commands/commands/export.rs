use crate::modules::SearchResult;
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn export_json(path: &Path, content: &str) -> Result<String, String> {
    if path == Path::new("-") {
        Ok(content.to_string())
    } else {
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(format!("Exported JSON to {}", path.display()))
    }
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

pub fn export_csv(path: &Path, headers: &[&str], rows: &[Vec<String>]) -> Result<String, String> {
    let mut csv = headers.join(",") + "\n";
    for row in rows {
        let escaped: Vec<String> = row.iter().map(|s| escape_csv_field(s)).collect();
        csv += &escaped.join(",");
        csv += "\n";
    }
    if path == Path::new("-") {
        Ok(csv)
    } else {
        fs::write(path, csv).map_err(|e| e.to_string())?;
        Ok(format!(
            "Exported {} rows to {}",
            rows.len(),
            path.display()
        ))
    }
}

pub fn to_sarif_json(pattern: &str, results: &[SearchResult]) -> String {
    let sarif_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            json!({
                "ruleId": "git-ast-match",
                "level": "warning",
                "message": {
                    "text": format!("AST match in commit {} (by {}): {}", r.commit_short, r.author.as_deref().unwrap_or("unknown"), r.matched_text.lines().next().unwrap_or(""))
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": r.file_path
                            },
                            "region": {
                                "startLine": r.line_number.max(1),
                                "snippet": {
                                    "text": r.matched_text
                                }
                            }
                        }
                    }
                ],
                "properties": {
                    "commitId": r.commit_id,
                    "commitShort": r.commit_short,
                    "author": r.author,
                    "timestamp": r.timestamp,
                    "language": r.lang
                }
            })
        })
        .collect();

    let sarif_doc = json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/Sarif-2.1.0/rt/schemas/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "Git-AST-Search",
                        "informationUri": "https://github.com/plantacerium/Git-AST-Search",
                        "version": "0.2.1",
                        "rules": [
                            {
                                "id": "git-ast-match",
                                "shortDescription": {
                                    "text": format!("Structural AST pattern match: {}", pattern)
                                }
                            }
                        ]
                    }
                },
                "results": sarif_results
            }
        ]
    });

    serde_json::to_string_pretty(&sarif_doc).unwrap_or_else(|_| "{}".to_string())
}

pub fn export_sarif(path: &Path, pattern: &str, results: &[SearchResult]) -> Result<String, String> {
    let content = to_sarif_json(pattern, results);
    if path == Path::new("-") {
        Ok(content)
    } else {
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(format!(
            "Exported {} results in SARIF v2.1.0 format to {}",
            results.len(),
            path.display()
        ))
    }
}
