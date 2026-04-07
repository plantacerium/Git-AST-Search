use anyhow::Result;
use ast_grep_core::AstGrep;
use ast_grep_language::SupportLang;
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dashmap::DashSet;
use git2::{Oid, Repository};
use ratatui::{prelude::*, widgets::*};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    io,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};
use tui_textarea::TextArea;

// --- MODELOS DE DATOS ---

#[derive(Clone, Serialize, Deserialize)]
struct ChatEntry {
    query: String,
    timestamp: DateTime<Local>,
}

#[derive(Clone)]
struct SearchResult {
    commit_id: String,
    file_path: String,
    content: String,
    line_number: usize,
    lang: String,
}

enum Message {
    ResultFound(SearchResult),
    SearchFinished,
    Error(String),
}

struct App<'a> {
    textarea: TextArea<'a>,
    chat_history: Vec<ChatEntry>,
    results: Vec<SearchResult>,
    is_searching: bool,
    repo_path: String,
    receiver: Option<Receiver<Message>>,
    page: usize,
}

impl<'a> App<'a> {
    fn new(repo_path: String) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title(" 󰭻  Patrón AST (Ej: fn $A($$$) {$$$ }) ")
            .border_type(BorderType::Rounded),
        );
        textarea.set_cursor_line_style(Style::default());

        Self {
            textarea,
            chat_history: Vec::new(),
            results: Vec::new(),
            is_searching: false,
            repo_path,
            receiver: None,
            page: 0,
        }
    }

    fn start_search(&mut self) {
        let pattern = self.textarea.lines().join("\n");
        if pattern.trim().is_empty() {
            return;
        }

        self.is_searching = true;
        self.results.clear();
        self.page = 0;
        self.chat_history.push(ChatEntry {
            query: pattern.clone(),
                               timestamp: Local::now(),
        });

        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let path = self.repo_path.clone();

        std::thread::spawn(move || {
            if let Err(e) = run_git_ast_engine(path, pattern, tx.clone()) {
                let _ = tx.send(Message::Error(e.to_string()));
            }
            let _ = tx.send(Message::SearchFinished);
        });
    }

    fn update(&mut self) {
        if let Some(rx) = &self.receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Message::ResultFound(res) => self.results.push(res),
                    Message::SearchFinished => self.is_searching = false,
                    // Bind the string to _err to satisfy the compiler
                    Message::Error(_err) => self.is_searching = false,
                }
            }
        }
    }

    fn next_page(&mut self) {
        if (self.page + 1) * 6 < self.results.len() {
            self.page += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }
}

// --- MOTOR DE BÚSQUEDA OPTIMIZADO ---

fn run_git_ast_engine(repo_path: String, pattern: String, tx: Sender<Message>) -> Result<()> {
    let repo = Repository::open(&repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let oids: Vec<Oid> = revwalk.filter_map(|id| id.ok()).collect();

    // OPTIMIZACIÓN 1: Caché global de Blobs procesados.
    // Un archivo (blob) exacto nunca se escanea dos veces, independientemente de en cuántos commits aparezca.
    let visited_blobs = Arc::new(DashSet::new());

    // OPTIMIZACIÓN 2: Chunking para no instanciar `Repository` por cada commit individual.
    oids.par_chunks(100).for_each(|chunk| {
        // Instanciar un repo local al hilo para el chunk actual
        if let Ok(thread_repo) = Repository::open(&repo_path) {
            for oid in chunk {
                if let Ok(commit) = thread_repo.find_commit(*oid) {
                    if let Ok(tree) = commit.tree() {
                        let _ = tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
                            if let Some(name) = entry.name() {
                                // Soporte multipoliglota rápido por extensión
                                let ext = std::path::Path::new(name)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("");

                                let lang = match ext {
                                    "rs" => SupportLang::Rust,
                                    "ts" | "tsx" => SupportLang::TypeScript,
                                    "js" | "jsx" => SupportLang::JavaScript,
                                    "go" => SupportLang::Go,
                                    "c" => SupportLang::C,
                                    "cpp" | "cc" | "cxx" => SupportLang::Cpp,
                                    "py" => SupportLang::Python,
                                    "java" => SupportLang::Java,
                                    _ => return git2::TreeWalkResult::Ok, // Ignorar lenguajes no soportados
                                };

                                let blob_id = entry.id();

                                // Si este blob ya se ha escaneado en otro commit, saltarlo (O(1))
                                if !visited_blobs.insert(blob_id) {
                                    return git2::TreeWalkResult::Ok;
                                }

                                if let Ok(blob) = entry.to_object(&thread_repo).and_then(|obj| {
                                    obj.into_blob().map_err(|_| git2::Error::from_str("Not a blob"))
                                }) {
                                    let content = String::from_utf8_lossy(blob.content());
                                    let grep = AstGrep::new(content.as_ref(), lang);

                                for matched in grep.root().find_all(pattern.as_str())       {
                                        let range = matched.range();
                                        let line_number = content[..range.start].lines().count();

                                        let _ = tx.send(Message::ResultFound(SearchResult {
                                            commit_id: oid.to_string()[..7].to_string(),
                                                                             file_path: format!("{}{}", root, name),
                                                                             content: matched.text().to_string(),
                                                                             line_number,
                                                                             lang: ext.to_string(),
                                        }));
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

    Ok(())
}

// --- INTERFAZ DE USUARIO (RATATUI) ---

fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(7)])
    .split(f.area());

    let main_area = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
    .split(chunks[0]);

    // 1. Sidebar (Historial)
    let history_items: Vec<ListItem> = app
    .chat_history
    .iter()
    .rev()
    .map(|entry| {
        let time = entry.timestamp.format("%H:%M").to_string();
        ListItem::new(vec![
            Line::from(vec![Span::styled(
                format!(" 󰍩  {}", time),
                    Style::default().fg(Color::DarkGray),
            )]),
            Line::from(format!("   {}", entry.query.lines().next().unwrap_or(""))),
        ])
    })
    .collect();

    let history_list = List::new(history_items)
    .block(Block::default().borders(Borders::ALL).title(" Historial "));
    f.render_widget(history_list, main_area[0]);

    // 2. Resultados (Cuadrícula con Paginación)
    if app.results.is_empty() && !app.is_searching {
        let welcome = Paragraph::new("Escribe un patrón AST abajo y presiona Enter para buscar en la historia.\n\nSoporta: Rust, JS/TS, Go, C/C++, Python, Java.\nUsa <- y -> para paginar.")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(welcome, main_area[1]);
    } else {
        render_results_grid(f, main_area[1], app);
    }

    // 3. Barra de Búsqueda
    let mut input_style = Style::default();
    if app.is_searching {
        input_style = input_style.fg(Color::Yellow);
        app.textarea.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " 🔍 Buscando en Git History... ({} resultados) ",
                           app.results.len()
            ))
            .border_style(input_style),
        );
    } else {
        app.textarea.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title(" 󰭻  AST Pattern (Enter para buscar) ")
            .border_type(BorderType::Rounded),
        );
    }
    f.render_widget(&app.textarea, chunks[1]);
}

fn render_results_grid(f: &mut Frame, area: Rect, app: &App) {
    let total_results = app.results.len();
    let start_idx = app.page * 6;
    let items_to_show = app.results.iter().skip(start_idx).take(6).collect::<Vec<_>>();
    let count = items_to_show.len();

    let title = format!(
        " Resultados (Pag {}/{} - Total: {}) ",
                        app.page + 1,
                        (total_results.saturating_sub(1) / 6) + 1,
                        total_results
    );

    let main_block = Block::default()
    .borders(Borders::ALL)
    .title(title)
    .border_type(BorderType::Plain);

    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);

    if count == 0 {
        return;
    }

    let rows_count = if count > 3 { 2 } else { 1 };
    let vertical_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(vec![Constraint::Percentage(100 / rows_count); rows_count as usize])
    .split(inner_area);

    for (i, res) in items_to_show.into_iter().enumerate() {
        let row_idx = i / 3;
        let col_idx = i % 3;
        let cols_in_row = if row_idx == 0 && count < 3 { count } else if row_idx == 1 { count - 3 } else { 3 };

        let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(100 / cols_in_row as u16); cols_in_row])
        .split(vertical_chunks[row_idx]);

        let text = vec![
            Line::from(vec![
                Span::styled(" Commit: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                       Span::raw(&res.commit_id),
                       Span::styled(format!(" [{}]", res.lang), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(" File:   ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                       Span::raw(&res.file_path),
                       Span::raw(format!(":{}", res.line_number)),
            ]),
            Line::from("─".repeat(inner_area.width as usize)),
            Line::from(res.content.clone()),
        ];

        let card = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
        .wrap(Wrap { trim: false });

        f.render_widget(card, horizontal_chunks[col_idx]);
    }
}

// --- MAIN LOOP ---

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let repo_path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repo_path);

    loop {
        app.update();
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Right => app.next_page(),
                    KeyCode::Left => app.prev_page(),
                    KeyCode::Enter if !app.is_searching => {
                        app.start_search();
                    }
                    _ => {
                        app.textarea.input(key);
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
             LeaveAlternateScreen,
             DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
