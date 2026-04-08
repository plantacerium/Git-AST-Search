# 🦀 Git AST Search TUI

**Git AST Search** es una herramienta de terminal (TUI) de alto rendimiento diseñada para la minería de código histórica. A diferencia de las herramientas de búsqueda tradicionales basadas en texto plano o expresiones regulares (Regex), esta herramienta utiliza **Análisis de Árboles de Sintaxis Abstracta (AST)** para encontrar estructuras de código exactas, ignorando comentarios, espacios en blanco o saltos de línea, a través de *toda* la historia de un repositorio Git.

### ✨ Novedades en v0.2.0
* **Arquitectura 100% Modular**: Lógica principal desacoplada en `engine`, `commands` y `modules`. El `main.rs` ahora está dedicado exclusivamente a la UI interactiva.
* **Sistema de Comandos (Slash Commands)**: Soporte completo para comandos `/search`, `/export`, `/bookmark`, `/sessions`, `/patterns` y sus alias.
* **Navegación tipo Vim Avanzada**: Historial de autocompletado con `Tab`, teclas `j/k/h/l` para resultados/páginas, y Modo Visual `v`.
* **Exportación Rápida**: Puedes arrojar resultados de búsquedas en AST directamente a `json` o `csv` con `/export`.

---

![GIT AST SEARCH](./assets/GIT_AST_SEARCH_V2.JPG)

---

## 🌟 ¿Por qué Git AST Search? (Perspectivas del Proyecto)

### 1. Perspectiva de Rendimiento: Escaneo Cero-Redundante en O(1)
El motor no realiza *checkouts* de Git ni toca el disco duro. Lee directamente de la base de datos de objetos (Blobs). Además, implementa una **deduplicación concurrente agresiva**: si un archivo no ha cambiado entre 1,000 commits, el motor solo lo escanea *una vez*. Esto reduce tiempos de búsqueda de minutos a milisegundos en repositorios masivos.

### 2. Perspectiva Multipolíglota
Ya no está limitado a Rust. El motor detecta automáticamente la extensión del archivo en cada commit y asigna el parser de lenguaje correcto en tiempo real. Soporta:
* 🦀 Rust (`.rs`)
* 🌐 JavaScript / TypeScript (`.js`, `.jsx`, `.ts`, `.tsx`)
* 🐹 Go (`.go`)
* 🐍 Python (`.py`)
* ☕ Java (`.java`)
* ⚙️ C / C++ (`.c`, `.cpp`, `.cc`, `.cxx`)

### 3. Perspectiva de Experiencia de Usuario (UI/UX)
Diseñado para el "Flujo de trabajo Terminal-First". Incluye un historial de búsquedas lateral para mantener el contexto de tus investigaciones y una vista de resultados en formato de tarjetas con paginación fluida, evitando saturar la pantalla.

### 4. Perspectiva de Navegación (Vim-style)
Controles familiares para usuarios de Vim con modo Normal, Comando, Visual y Goto. Incluye autocompletado de comandos y shortcuts configurables.

---

## 📁 Estructura del Proyecto v2

```
Git-AST-Search/
├── src/
│   ├── main.rs                   # Entry point
│   ├── modules/                   # Modelos de datos
│   │   ├── mod.rs
│   │   ├── search_result.rs       # SearchResult
│   │   ├── chat_entry.rs          # ChatEntry
│   │   ├── session.rs             # Session, SessionManager
│   │   ├── bookmark.rs            # Bookmark, BookmarkManager
│   │   ├── filter.rs              # Filter
│   │   └── config.rs              # AppConfig
│   ├── commands/                  # Sistema de comandos slash
│   │   ├── mod.rs
│   │   ├── parser.rs              # CommandParser, ParsedCommand
│   │   ├── executor.rs            # CommandExecutor
│   │   ├── registry.rs            # CommandRegistry
│   │   ├── autocomplete.rs        # Autocomplete
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── search.rs          # /search
│   │       └── export.rs          # /export
│   ├── navigation/               # Navegación y modos
│   │   ├── mod.rs                 # NavigationState
│   │   └── modes.rs               # NavMode enum
│   ├── languages/                 # Detección de lenguajes
│   │   ├── mod.rs
│   │   ├── registry.rs            # Language, LanguageRegistry, BuiltinPattern
│   │   ├── detector.rs            # LanguageDetector
│   │   └── patterns.rs            # Patrones por lenguaje
│   └── engine/
│       └── mod.rs                 # GitEngine
├── docs/                          # Documentación
├── assets/
├── Cargo.toml
└── README.md
```

---

## 🚀 Stack Tecnológico

| Componente | Tecnología | Descripción |
| :--- | :--- | :--- |
| **Interfaz (TUI)** | `ratatui` | Framework de renderizado inmediato para interfaces de terminal fluidas. |
| **Motor AST** | `ast-grep` | Framework de búsqueda estructural super-rápido basado en `tree-sitter`. |
| **Motor Git** | `git2` | Interacción en C puro y bajo nivel con la base de datos de objetos de Git. |
| **Deduplicación** | `dashmap` | Estructura de datos concurrente `Lock-Free` para el registro global de Blobs. |
| **Paralelismo** | `rayon` | Distribución dinámica de *chunks* de commits a través de todos los núcleos de CPU. |
| **Async & Eventos** | `tokio` / `mpsc` | Canales para enviar resultados desde el motor hacia la UI sin bloquear el renderizado. |

---

## 📦 Instalación

Para compilar este proyecto, necesitas las librerías de desarrollo del sistema operativo.

**En Fedora / RHEL:**
```bash
sudo dnf install cmake openssl-devel libgit2-devel zlib-devel
```

**En Ubuntu / Debian:**
```bash
sudo apt install cmake libssl-dev libgit2-dev zlib1g-dev
```

**Compilación del proyecto:**
```bash
git clone https://github.com/plantacerium/Git-AST-Search
cd Git-AST-Search
cargo build --release
```

---

## 🛠️ Guía de Uso

Inicia la herramienta pasando la ruta del repositorio Git como argumento (por defecto es el directorio actual):

```bash
./target/release/git-ast-search .
./target/release/git-ast-search /home/user/dev/linux
```

---

## 🎮 Controles de la TUI

### Modo Normal (Default)

| Atajo | Acción |
|-------|--------|
| `j` / `↓` | Siguiente resultado |
| `k` / `↑` | Resultado anterior |
| `h` / `←` | Página anterior |
| `l` / `→` | Página siguiente |
| `Enter` | Iniciar búsqueda |
| `Esc` | Abortar búsqueda / Cerrar |
| `Ctrl+H` | Panel de historial |
| `Ctrl+B` | Toggle sidebar |
| `gg` | Ir al primer resultado |
| `G` | Ir al último resultado |

### Modo Comando

| Atajo | Acción |
|-------|--------|
| `/` | Entrar en modo comando |
| `:` | Modo comando vim |
| `Tab` | Autocompletar |
| `↑` / `↓` | Historial de comandos |

---

## 💻 Comandos Disponibles

### Comandos de Búsqueda

| Comando | Descripción |
|---------|-------------|
| `/search <patrón>` | Buscar patrón AST en el historial |
| `/search <patrón> --lang <lenguaje>` | Filtrar por lenguaje |
| `/search <patrón> --author <nombre>` | Filtrar por autor |
| `/search <patrón> --after <fecha>` | Filtrar por fecha |

### Comandos de Navegación

| Comando | Descripción |
|---------|-------------|
| `/goto <destino>` | Ir a commit/archivo/línea |
| `/next` | Siguiente resultado |
| `/prev` | Resultado anterior |
| `/first` | Primer resultado |
| `/last` | Último resultado |
| `/page <n>` | Ir a página n |

### Comandos de Exportación

| Comando | Descripción |
|---------|-------------|
| `/export json <path>` | Exportar a JSON |
| `/export csv <path>` | Exportar a CSV |
| `/export csv --all` | Incluir todos los resultados |

### Comandos de Sesión

| Comando | Descripción |
|---------|-------------|
| `/save [nombre]` | Guardar sesión actual |
| `/load [nombre]` | Cargar sesión |
| `/sessions` | Listar sesiones guardadas |

### Comandos de Bookmark

| Comando | Descripción |
|---------|-------------|
| `/bookmark <label>` | Guardar bookmark |
| `/bookmarks` | Listar bookmarks |

### Comandos de Configuración y Ayuda

| Comando | Descripción |
|---------|-------------|
| `/patterns [lang]` | Ver patrones AST incorporados |
| `/help [topic]` | Ayuda sobre atajos y comandos |
| `/clear` | Limpiar resultados |
| `/toggle` | Mostrar/Ocultar barra lateral |

---

## 🔍 Ejemplos de Búsqueda Semántica Multipolíglota

Aprovecha el poder de `ast-grep` usando el comodín `$$$` (cero o múltiples nodos) y variables como `$A`, `$B` (nodos específicos). Aquí tienes casos de uso reales para auditoría de código en el historial:

### 🦀 Rust (`.rs`)
* **Buscar código "inseguro" (Unsafe blocks):**
    `unsafe { $$$ }`
* **Encontrar antiguas fugas de memoria o desempaquetados de riesgo:**
    `$OBJ.unwrap()` o `$OBJ.expect($ANY)`
* **Localizar errores silenciados explícitamente:**
    `let _ = $FUNC($$$);`

### 🌐 JavaScript / TypeScript (`.js`, `.ts`, `.jsx`, `.tsx`)
* **Buscar `console.log` que se escaparon a producción:**
    `console.log($$$)`
* **Encontrar "Callback Hell" o promesas anidadas:**
    `$A.then(($B) => { $$$}).then(($C) => {$$$ })`
* **Bloques `catch` vacíos (Fallos silenciosos):**
    ```javascript
    catch ($E) { }
    ```

### 🐍 Python (`.py`)
* **Trampas de argumentos por defecto mutables:**
    `def $FUNC($ARG = []): $$$` o `def $FUNC($ARG = {}): $$$`
* **Bloques de captura de errores silenciados:**
    ```python
    try:
        $$$
    except $ERR:
        pass
    ```

### 🐹 Go (`.go`)
* **Identificar dónde se ignoraron errores deliberadamente:**
    `$VAL, _ := $FUNC($$$)`
* **Buscar goroutines anónimas que podrían estar causando fugas:**
    ```go
    go func() {
        $$$
    }()
    ```

### ☕ Java (`.java`)
* **Rastros de "Print Debugging" dejados:**
    `System.out.println($$$);`
* **Captura excesivamente genérica de excepciones:**
    ```java
    catch (Exception $E) {
        $$$
    }
    ```

### ⚙️ C / C++ (`.c`, `.cpp`)
* **Funciones inseguras de manipulación de cadenas:**
    `strcpy($DEST, $SRC)` o `sprintf($$$)`
* **Gestión de memoria manual:**
    `delete $PTR;` o `free($PTR);`

---

## ⚙️ Arquitectura Modular

```
┌─────────────────────────────────────────────────────────────┐
│                          main.rs                            │
│           (Entry point ~400 líneas, delegación a UI)        │
└─────────────────────────────┬───────────────────────────────┘
                              │
        ┌────────────────────┼────────────────────┐
        ▼                    ▼                    ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│    modules/   │     │   commands/   │     │  navigation/  │
│   (Modelos)   │     │   (Parser)    │     │   (Estados)   │
├───────────────┤     ├───────────────┤     ├───────────────┤
│ SearchResult  │     │ CommandParser │     │ NavigationState│
│ ChatEntry     │     │ Executor     │     │ NavMode       │
│ Session       │     │ Registry     │     │               │
│ Filter        │     │ Autocomplete │     │               │
│ Config        │     │             │     │               │
└───────────────┘     └───────────────┘     └───────────────┘
        │                    │                    │
        └────────────────────┼────────────────────┘
                             ▼
                    ┌───────────────┐
                    │   languages/   │
                    │ (Detección)   │
                    ├───────────────┤
                    │ Language      │
                    │ Detector      │
                    │ Registry      │
                    │ Patterns      │
                    └───────────────┘
                             │
                             ▼
                    ┌───────────────┐
                    │    engine/    │
                    │ (Git + AST)   │
                    ├───────────────┤
                    │ GitEngine     │
                    │ AST Search    │
                    └───────────────┘
```

### Flujo de Datos

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
                              │
                              ▼
                    App.results (UI update)
```

---

## 📄 Licencia

Este proyecto está distribuido bajo la Licencia MIT. Siéntete libre de usarlo, modificarlo y distribuirlo para empujar los límites de la minería de código estática.
