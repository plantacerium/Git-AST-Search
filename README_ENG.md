# 🦀 Git AST Search TUI v0.3.0

**Git AST Search** is a high-performance terminal user interface (TUI) tool designed for historical code mining. Unlike traditional search tools based on plain text or Regular Expressions (Regex), this tool leverages **Abstract Syntax Tree (AST) Analysis** to find exact code structures—ignoring comments, whitespace, or line breaks—across the *entire* history of a Git repository.

### ✨ What's New in v0.3.0
* **High-Level Modular UI**: Full interface refactor into independent components (`sidebar`, `results_grid`, `search_bar`, `status_bar`).
* **Atomic Layout Logic**: Dimension calculations decoupled from rendering logic for improved stability and smoothness.
* **Command System (Slash Commands)**: Full support for `/search`, `/export`, `/bookmark`, `/sessions`, `/patterns` commands and their aliases.
* **Advanced Vim-style Navigation**: Autocomplete history with `Tab`, `j/k/h/l` keys for results/pages, and Visual Mode `v`.

---

![GIT AST SEARCH](./assets/GIT_AST_SEARCH_V2.JPG)

---

## 🌟 Why Git AST Search? (Unique Value Proposition)

### 🗺️ Capabilities & Features Matrix at a Glance

| Pillar / Capability | What it Does | Interface / Command | Key Value for Developers & AI Agents |
|---|---|---|---|
| **⏳ Historical Mining** | Scans thousands of Git commits without checkouts | TUI, `query`, MCP | Deep historical discovery without disk I/O or altering working tree |
| **⚡ O(1) Deduplication** | Reads Git blobs directly in RAM (`libgit2` + `DashSet`) | Core Engine (Auto) | Reduces search times from minutes to milliseconds |
| **🌳 AST Precision** | Exact syntax pattern matching (`$$$`, `$VAR`) via Tree-Sitter | `/search`, `query` | Eliminates false positives from whitespace and formatting noise |
| **🌐 Polyglot Detection** | Real-time parser assignment per commit based on extension | Auto (`.rs`, `.js`, `.py`, `.go`...) | Unified auditing across mixed-language codebases & monorepos |
| **🏺 Semantic AST Blame** | Bug archeology: detects when a block mutated or was introduced | `/evolution`, `evolution` | Uncovers root cause of logical mutations, immune to `fmt` diffs |
| **🛠️ Safe Auto-Refactor** | Structural code rewriting with default dry-run guardrails | `/refactor`, `refactor --apply` | Safe batch refactoring with unified diff preview before writing |
| **🦙 Local AI (Ollama)** | Natural language to AST translation and audit reports | `/ai`, `/report`, `ai` | No need to memorize AST syntax; auto-generates Markdown audits |
| **🔌 Native MCP Server** | JSON-RPC stdio protocol for AI agents (Cursor, Claude, IDE) | `mcp [REPO]` | Empowers autonomous agents to run AST mining tools natively |
| **🤝 Symbiotic Workflow** | Interactive TUI running in tandem with AI agent operations | TUI Vim (`j/k/h/l`) + MCP | Shared mental model: human and AI inspect the exact same results |

---

### 1. Symbiotic Human ↔ AI Collaboration (Shared Mental Model)
* **The issue with traditional AI coding tools:** When an autonomous AI agent searches codebases in the background using invisible tools, developers lose context, trust, and real-time oversight.
* **The Git-AST-Search solution:** Developers keep the **interactive TUI open in their terminal** navigating results effortlessly with Vim keybindings (`j/k/h/l`), while AI agents in the IDE query patterns, audit vulnerabilities, or suggest refactors through the **MCP Server**. Both human and AI share the exact same mental model of the codebase.

### 2. Zero-Checkout O(1) History Mining
The engine never performs Git checkouts or touches the hard drive during scans. It reads directly from the Git object database (Blobs) via `libgit2`. Furthermore, it implements **aggressive concurrent deduplication**: if a file has not changed across 1,000 commits, the engine scans it *only once*. This reduces search times from minutes to milliseconds even in massive repositories.

### 3. Semantic Code Archaeology (Git AST Blame)
Unlike standard `git blame` (which attributes changes to whitespace or `rustfmt`/`prettier` modifications), **Semantic AST Blame** tracks genuine structural code evolution:
* Pinpoints the exact commit where a function changed from synchronous to `async`.
* Identifies which commit and author introduced an `unsafe` block or risky `unwrap()` call.

### 4. First-Class Multi-Polyglot Support
Real-time Tree-Sitter language parser auto-detection per commit:
* 🦀 Rust (`.rs`)
* 🌐 JavaScript / TypeScript (`.js`, `.jsx`, `.ts`, `.tsx`)
* 🐹 Go (`.go`)
* 🐍 Python (`.py`)
* ☕ Java (`.java`)
* ⚙️ C / C++ (`.c`, `.cpp`, `.cc`, `.cxx`)

### 5. Safe Refactoring with Dry-Run Guardrails
* **Strict Separation of Read vs Rewrite Modes:** Historical mining is strictly read-only. The structural refactoring engine (`/refactor` or `git-ast-search refactor`) operates under a **`dry-run` by default philosophy**, presenting unified diff previews before allowing any changes to be written to disk (`--apply`).

---

## 📁 Project Structure v0.3.0

```
Git-AST-Search/
├── src/
│   ├── main.rs                   # Main loop and orchestration
│   ├── ui/                        # Modular User Interface
│   │   ├── mod.rs                # UI Entry point
│   │   ├── app.rs                # Global state and App handlers
│   │   ├── render.rs             # Drawing functions (Frame orchestration)
│   │   ├── layout.rs             # Geometric calculations and constraints
│   │   ├── events.rs             # Crossterm event bridge (WIP)
│   │   └── components/           # Atomic widgets
│   │       ├── mod.rs            # Component exports
│   │       ├── sidebar.rs        # Historical sidebar panel
│   │       ├── results_grid.rs   # Dynamic results grid
│   │       ├── search_bar.rs     # Command/Pattern input
│   │       ├── status_bar.rs     # Mode and status information
│   │       └── help_overlay.rs   # Help and Welcome screens
│   ├── modules/                   # Data models
│   │   ├── mod.rs
│   │   ├── search_result.rs       # SearchResult entity
│   │   ├── chat_entry.rs          # Query history model
│   │   ├── session.rs             # Session persistence and management
│   │   ├── bookmark.rs            # Result bookmarks
│   │   ├── filter.rs              # Filtering logic
│   │   └── config.rs              # Configuration and Themes
│   ├── commands/                  # Slash Command Engine
│   │   ├── mod.rs
│   │   ├── parser.rs              # / command parsing
│   │   ├── executor.rs            # Command logic executor
│   │   ├── registry.rs            # Central command registry
│   │   ├── autocomplete.rs         # Suggestions and autocompletion
│   │   └── commands/              # Individual command implementations
│   │       ├── mod.rs
│   │       ├── search.rs          # /search command
│   │       └── export.rs          # /export command
│   ├── navigation/               # Navigation and Modes (Vim-style)
│   │   ├── mod.rs                 # NavigationState
│   │   └── modes.rs               # NavMode enum
│   ├── languages/                 # Parsers and Multi-polyglot Detection
│   │   ├── mod.rs
│   │   ├── registry.rs            # Language and pattern registry
│   │   ├── detector.rs            # Auto-detection by extension
│   │   └── patterns.rs            # Base AST pattern definitions
│   └── engine/                    # Core: Git2 + AST-Grep + Rayon
│       └── mod.rs                 # Search engine and Git integration
├── assets/                        # Visual assets and screenshots
├── docs/                          # Detailed documentation
├── Cargo.toml                     # Dependencies and project metadata
├── README.md                      # Spanish Documentation
└── README_ENG.md                  # English Documentation
```

---

## 🚀 Tech Stack

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Interface (TUI)** | `ratatui` | Immediate-mode rendering framework for fluid terminal interfaces. |
| **AST Engine** | `ast-grep` | Super-fast structural search framework based on `tree-sitter`. |
| **Git Engine** | `git2` | Low-level C-based interaction with the Git object database. |
| **Deduplication** | `dashmap` | Lock-free concurrent data structure for global Blob registration. |
| **Parallelism** | `rayon` | Dynamic distribution of commit chunks across all CPU cores. |
| **Async & Events** | `tokio` / `mpsc` | Channels to stream results from the engine to the UI without blocking. |

---

## 📦 Installation

To compile this project, you need the system development libraries.

**On Fedora / RHEL:**
```bash
sudo dnf install cmake openssl-devel libgit2-devel zlib-devel
```

**On Ubuntu / Debian:**
```bash
sudo apt install cmake libssl-dev libgit2-dev zlib1g-dev
```

**Compiling the project:**
```bash
git clone https://github.com/plantacerium/Git-AST-Search
cd Git-AST-Search
cargo build --release
```

---

## 🛠️ Usage Guide: Targeting Any Repository

`Git-AST-Search` is fully portable and completely independent of the folder where it is compiled. It can audit **any Git repository on your machine** (local or on external drives).

### 1. In Interactive TUI Mode
Pass the repository path as the first positional argument (defaults to `.`, the directory where you open the terminal):

```bash
# Audit the current repository in terminal
git-ast-search .

# Audit any other project via relative or absolute path
git-ast-search /home/user/dev/linux
git-ast-search "D:\projects\backend-api"
```

### 2. In Headless CLI Mode
Use the `--repo <PATH>` flag to target queries at any repository:

```bash
# Direct structural search in another repository
git-ast-search query "$A.unwrap()" --repo "D:\projects\my-app" --lang rust --format json

# Bug archeology / Semantic AST blame in another repository
git-ast-search evolution "std::thread::spawn($$$)" --repo "D:\projects\my-app" --format text

# Safe structural refactor preview in another repository
git-ast-search refactor "$A.unwrap()" "$A.unwrap_or_default()" --repo "D:\projects\my-app"
```

### 3. In MCP Server Mode (For AI Agents in the IDE)
Launch the MCP server targeting a default repository:
```bash
git-ast-search mcp "D:\projects\my-app"
```
Furthermore, on every individual tool invocation (`git_ast_search`, `git_ast_refactor`, `git_ast_evolution`), the AI agent can pass `"repo_path": "/another/path"` to inspect repositories dynamically without restarting the server.

---

## 🎮 TUI Controls

### Insert Mode (Default)

| Shortcut | Action |
|----------|--------|
| `Any` | Type AST Pattern |
| `Enter` | Start search |
| `Esc` | Exit to Normal Mode |

### Normal Mode

| Shortcut | Action |
|----------|--------|
| `Tab` | Toggle active pane between Results and History |
| `i` / `a` | Enter Insert Mode |
| `j` / `↓` | Next result (or next query in History) |
| `k` / `↑` | Previous result (or previous query in History) |
| `Enter` / `r` | Re-run historical query (in History) or Start search |
| `e` / `i` | Load historical query into text editor (in History) |
| `h` / `←` | Previous page / Jump to History |
| `l` / `→` | Next page / Jump to Results |
| `Esc` | Clear selection / Focus Results / Exit |
| `Ctrl+B` | Toggle sidebar visibility |
| `gg` | Go to first result |
| `G` | Go to last result |

### Command Mode

| Shortcut | Action |
|----------|--------|
| `/` | Enter command mode |
| `:` | Vim command mode |
| `Tab` | Autocomplete |
| `↑` / `↓` | Command history |

---

## 💻 Available Commands

### Search Commands

| Command | Description |
|---------|-------------|
| `/search <pattern>` | Search AST pattern in history |
| `/search <pattern> --lang <language>` | Filter by language |
| `/search <pattern> --author <name>` | Filter by author |
| `/search <pattern> --after <date>` | Filter by date |

### Navigation Commands

| Command | Description |
|---------|-------------|
| `/history` or `/hist` | Focus and activate interactive History sidebar |
| `/replay [N]` or `/rerun` | Re-run historical query by index (default: latest) |
| `/goto <target>` | Go to commit/file/line |
| `/next` | Next result |
| `/prev` | Previous result |
| `/first` | First result |
| `/last` | Last result |
| `/page <n>` | Go to page n |

### Export Commands

| Command | Description |
|---------|-------------|
| `/export json <path>` | Export to JSON |
| `/export csv <path>` | Export to CSV |
| `/export csv --all` | Include all results |

### 🤖 AI Commands (Ollama)

| Command | Description |
|---------|-------------|
| `/ai <prompt>` | Translate natural language query to an AST pattern via Ollama & auto-search |
| `/report [topic]` | Generate an executive AI code health & audit report saved to `audit_report.md` |
| `/refactor <P> -> <R>` | Safe AST structural rewrite preview (dry-run by default; `--apply` to write) |
| `/evolution <pattern>` | Bug archeology / Semantic Blame: Chronological history & mutation timeline |

---

## 🤖 AI & Agent Integration (CLI & MCP Server)

`Git-AST-Search` is built for direct interoperability with AI agents and coding assistants (**Antigravity, Cursor, Claude Code, Windsurf**):

### 1. Model Context Protocol (MCP) Server
Launch the standard JSON-RPC 2.0 server over `stdio` for agent tool-use:
```bash
git-ast-search mcp .
```
**Exposed Tools for Agents:**
* `git_ast_search`: Fast historical AST pattern search across all Git commits.
* `ai_query_ast`: Natural language to AST pattern translation via Ollama.
* `generate_git_ast_report`: Automated security audit and vulnerability reports.
* `list_ast_patterns`: Access to the categorized pattern catalog.
* `git_ast_refactor`: Safe structural code rewrite with dry-run diff preview.
* `git_ast_evolution`: Semantic AST Blame tracking structural mutations across Git history.

### 2. Headless CLI Mode (Scripts & CI/CD)
```bash
# Direct structural search returning JSON
git-ast-search query '$A.unwrap()' --lang rust --format json

# Natural language query translated by Ollama in terminal
git-ast-search ai "find empty catch blocks in typescript"

# Safe dry-run preview of structural refactor
git-ast-search refactor '$A.unwrap()' '$A.unwrap_or_default()' --lang rust

# Apply structural refactor directly to disk
git-ast-search refactor '$A.unwrap()' '$A.unwrap_or_default()' --apply

# Bug archeology: track how code structure mutated across history
git-ast-search evolution 'std::thread::spawn($$$)' --format text
```

---

## 🔍 Examples of Multi-Polyglot Semantic Search

> **📖 Pro Tip:** Check out our newly expanded [Code Exploration Catalog](./code_exploration.md) which contains **260 ready-to-use AST patterns** (including 88 Fundamental and 88 Professional Rust patterns) to supercharge your codebase audits!

Harness the power of `ast-grep` using the `$$$` wildcard (zero or more nodes) and variables like `$A`, `$B` (specific nodes). Here are real-world use cases for code auditing in history:

### 🦀 Rust (`.rs`)
* **Search for "unsafe" code blocks:**
    `unsafe { $$$ }`
* **Find old memory leaks or risky unwrap calls:**
    `$OBJ.unwrap()` or `$OBJ.expect($ANY)`
* **Locate explicitly silenced errors:**
    `let _ = $FUNC($$$);`

### 🌐 JavaScript / TypeScript (`.js`, `.ts`, `.jsx`, `.tsx`)
* **Search for `console.log` statements that leaked to production:**
    `console.log($$$)`
* **Find "Callback Hell" or nested promises:**
    `$A.then(($B) => { $$$}).then(($C) => {$$$ })`
* **Empty `catch` blocks (Silent failures):**
    ```javascript
    catch ($E) { }
    ```

### 🐍 Python (`.py`)
* **Mutable default argument traps:**
    `def $FUNC($ARG = []): $$$` or `def $FUNC($ARG = {}): $$$`
* **Silenced error capture blocks:**
    ```python
    try:
        $$$
    except $ERR:
        pass
    ```

### 🐹 Go (`.go`)
* **Identify where errors were deliberately ignored:**
    `$VAL, _ := $FUNC($$$)`
* **Search for anonymous goroutines that might be leaking:**
    ```go
    go func() {
        $$$
    }()
    ```

### ☕ Java (`.java`)
* **Leftover "Print Debugging" traces:**
    `System.out.println($$$);`
* **Excessively generic exception catching:**
    ```java
    catch (Exception $E) {
        $$$
    }
    ```

### ⚙️ C / C++ (`.c`, `.cpp`)
* **Insecure string manipulation functions:**
    `strcpy($DEST, $SRC)` or `sprintf($$$)`
* **Manual memory management:**
    `delete $PTR;` or `free($PTR);`

---

## ⚙️ Systems Architecture (v0.3.0)

```
┌─────────────────────────────────────────────────────────────┐
│                          main.rs                            │
│           (Entry point ~50 lines, Event Loop)               │
└─────────────────────────────┬───────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐      ┌───────────────┐     ┌───────────────┐
│      ui/      │      │   commands/   │     │  navigation/  │
│ (Components)  │      │   (Ingest)    │     │   (FSM)       │
├───────────────┤      ├───────────────┤     ├───────────────┤
│ app.rs        │◄────►│ CommandParser │◄───►│ NavMode       │
│ layout.rs     │      │ Executor      │     │ NavState      │
│ render.rs     │      │ Autocomplete  │     │               │
│ components/   │      └──────┬────────┘     └───────────────┘
└───────────────┘             │
        │                     │
        ▼                     ▼
┌───────────────┐      ┌───────────────┐
│     modules/  │      │   languages/  │
│    (Models)   │      │  (TreeSitter) │
├───────────────┤      ├───────────────┤
│ SearchResult  │      │ LanguageReg   │
│ ChatEntry     │      │ Detector      │
│ Session       │      │ Patterns      │
└───────────────┘      └──────┬────────┘
                              │
                              ▼
                     ┌───────────────┐
                     │    engine/    │
                     │ (Git + AST)   │
                     ├───────────────┤
                     │ RevWalk       │
                     │ Blobs Cache   │
                     └───────────────┘
```

### Data Flow

```
UI Input (TextArea) → CommandParser → CommandExecutor
                                              │
                                              ▼
                                     start_search()
                                              │
                                              ▼
                                    mpsc::channel()
                                              │
                    ┌─────────────────────────┤
                    ▼                         ▼
             RevWalk (commits)        DashSet (blob cache)
                    │                         │
                    └─────────┬───────────────┘
                              ▼
                       par_chunks(100)
                              │
                              ▼
                       AstGrep (pattern)
                              │
                              ▼
                     Message::ResultFound
```

---

## 📄 License

This project is distributed under the MIT License. Feel free to use, modify, and distribute it to push the limits of static code mining.
