use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct CodeHighlighter {
    ps: SyntaxSet,
    ts: ThemeSet,
}

impl CodeHighlighter {
    pub fn new() -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_blob(&self, content: &str, file_path: &str, target_line: usize) -> Vec<Line<'static>> {
        let syntax = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.ps.find_syntax_by_extension(ext))
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let theme = self
            .ts
            .themes
            .get("base16-ocean.dark")
            .or_else(|| self.ts.themes.values().next())
            .expect("Theme required");

        let mut h = HighlightLines::new(syntax, theme);
        let mut lines = Vec::new();

        for (idx, line_str) in content.lines().enumerate() {
            let line_no = idx + 1;
            let is_target = line_no == target_line;

            let prefix = if is_target {
                Span::styled(
                    format!(">> {:4} | ", line_no),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!("   {:4} | ", line_no),
                    Style::default().fg(Color::DarkGray),
                )
            };

            let mut spans = vec![prefix];

            let line_with_nl = format!("{}\n", line_str);
            if let Ok(ranges) = h.highlight_line(&line_with_nl, &self.ps) {
                for (style, text) in ranges {
                    let clean_text = text.trim_end_matches('\n').to_string();
                    if clean_text.is_empty() {
                        continue;
                    }
                    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                    let mut s = Style::default().fg(fg);
                    if is_target {
                        s = s.add_modifier(Modifier::BOLD);
                    }
                    spans.push(Span::styled(clean_text, s));
                }
            } else {
                let mut s = Style::default();
                if is_target {
                    s = s.fg(Color::Yellow).add_modifier(Modifier::BOLD);
                }
                spans.push(Span::styled(line_str.to_string(), s));
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}

impl Default for CodeHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_blob_rust() {
        let hl = CodeHighlighter::new();
        let code = "fn main() {\n    println!(\"hello\");\n}\n";
        let lines = hl.highlight_blob(code, "main.rs", 2);
        assert_eq!(lines.len(), 3);
        // Target line should have prefix >>
        let line2_str = lines[1].spans.iter().map(|s| s.content.as_ref()).collect::<String>();
        assert!(line2_str.starts_with(">>    2 |"));
        // First line should have prefix
        let line1_str = lines[0].spans.iter().map(|s| s.content.as_ref()).collect::<String>();
        assert!(line1_str.starts_with("      1 |"));
    }
}
