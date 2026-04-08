use std::fs;
use std::path::Path;

pub fn export_json(path: &Path, content: &str) -> Result<String, String> {
    if path == Path::new("-") {
        Ok(content.to_string())
    } else {
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(format!("Exported to {}", path.display()))
    }
}

pub fn export_csv(path: &Path, headers: &[&str], rows: &[Vec<String>]) -> Result<String, String> {
    let mut csv = headers.join(",") + "\n";
    for row in rows {
        csv += &row.join(",");
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
