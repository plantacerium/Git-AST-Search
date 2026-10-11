# 🦀 Git AST Search TUI v0.3.0

**Git AST Search** es una herramienta de terminal (TUI) de alto rendimiento diseñada para la minería de código histórica. A diferencia de las herramientas de búsqueda tradicionales basadas en texto plano o expresiones regulares (Regex), esta herramienta utiliza **Análisis de Árboles de Sintaxis Abstracta (AST)** para encontrar estructuras de código exactas, ignorando comentarios, espacios en blanco o saltos de línea, a través de *toda* la historia de un repositorio Git.

### ✨ Novedades en v0.3.0
* **UI Modular de Alto Nivel**: Refactorización completa de la interfaz en componentes independientes (`sidebar`, `results_grid`, `search_bar`, `status_bar`).
* **Lógica de Layout Atómica**: Cálculos de dimensiones desacoplados de la lógica de renderizado para mayor estabilidad y fluidez.
* **Sistema de Comandos (Slash Commands)**: Soporte completo para comandos `/search`, `/export`, `/bookmark`, `/sessions`, `/patterns` y sus alias.
* **Navegación tipo Vim Avanzada**: Historial de autocompletado con `Tab`, teclas `j/k/h/l` para resultados/páginas, y Modo Visual `v`.

---

![GIT AST SEARCH](./assets/GIT_AST_SEARCH_V2.JPG)

---

## 🌟 ¿Por qué Git AST Search? (Propuesta de Valor Única)

### 🗺️ Matriz de Capacidades y Posibilidades de un Vistazo

| Capacidad / Pilar | ¿Qué hace? | Interfaz / Comando | Valor Clave para Desarrolladores e IAs |
|---|---|---|---|
| **⏳ Minería Histórica** | Analiza miles de commits sin realizar `git checkout` | TUI, `query`, MCP | Búsqueda profunda sin saturar disco ni tocar el working tree |
| **⚡ Deduplicación O(1)** | Lee blobs de Git directamente en RAM (`libgit2` + `DashSet`) | Automático en el core | Reduce tiempos de búsqueda de minutos a milisegundos |
| **🌳 Precisión AST** | Reconocimiento sintáctico exacto (`$$$`, `$VAR`) con Tree-Sitter | `/search`, `query` | Cero falsos positivos por formateo, espacios o saltos de línea |
| **🌐 Soporte Multipolíglota** | Detección automática de lenguaje por extensión en cada commit | Auto (`.rs`, `.js`, `.py`, `.go`...) | Auditorías homogéneas en monorepos y microservicios |
| **🏺 AST Blame Semántico** | Arqueología de bugs: rastrea cuándo un bloque mutó o se introdujo | `/evolution`, `evolution` | Revela la causa raíz de mutaciones lógicas ignorando ruido de `fmt` |
| **🛠️ Auto-Refactor Seguro** | Reescritura estructural con política `dry-run` por defecto | `/refactor`, `refactor --apply` | Modificaciones masivas seguras con previsualización de diffs |
| **🦙 Inteligencia Local** | Traduce lenguaje natural a AST y sintetiza reportes con Ollama | `/ai`, `/report`, `ai` | No requiere memorizar sintaxis AST; genera diagnósticos Markdown |
| **🔌 Servidor MCP Nativo** | Expone herramientas JSON-RPC para agentes (Cursor, Claude, IDE) | `mcp [REPO]` | Permite a las IAs invocar búsquedas y auditar repositorios |
| **🤝 Flujo Simbiótico** | Interfaz visual en TUI compartida en paralelo con el agente | TUI Vim (`j/k/h/l`) + MCP | Mental model compartido: humano e IA ven lo mismo sin cajas negras |

---

### 1. Colaboración Simbiótica Humano ↔ IA (Shared Mental Model)
* **El problema de las herramientas para agentes tradicionales:** Cuando un agente de IA ejecuta búsquedas en segundo plano en una herramienta invisible, el desarrollador humano pierde el contexto, la confianza y el control de lo que la IA está explorando.
* **La solución de Git-AST-Search:** El desarrollador mantiene la **TUI abierta en su terminal** explorando visualmente con controles Vim (`j/k/h/l`), mientras que en el IDE el agente de IA formula patrones complejos, ejecuta auditorías o sugiere refactors mediante el **servidor MCP**. Ambos comparten exactamente la misma visión del código.

### 2. Minería Histórica sin Checkout en O(1)
El motor no realiza *checkouts* de Git ni toca el disco duro. Lee directamente de la base de datos de objetos (Blobs) mediante `libgit2`. Además, implementa una **deduplicación concurrente agresiva**: si un archivo no ha cambiado entre 1,000 commits, el motor solo lo escanea *una vez*. Esto reduce tiempos de búsqueda de minutos a milisegundos en repositorios masivos.

### 3. Arqueología Semántica de Código (Git AST Blame)
A diferencia de `git blame` tradicional (que atribuye cambios de autoría por meros saltos de línea o formateo de `rustfmt`/`prettier`), el **AST Blame Semántico** rastrea la evolución estructural real:
* Identifica con precisión cuándo una función mutó de síncrona a `async`.
* Señala exactamente qué commit y autor introdujo un bloque `unsafe` o llamada a `unwrap()`.

### 4. Soporte Multipolíglota de Primera Clase
Detecta la extensión del archivo en cada commit y asigna el parser de Tree-Sitter correspondiente en tiempo real:
* 🦀 Rust (`.rs`)
* 🌐 JavaScript / TypeScript (`.js`, `.jsx`, `.ts`, `.tsx`)
* 🐹 Go (`.go`)
* 🐍 Python (`.py`)
* ☕ Java (`.java`)
* ⚙️ C / C++ (`.c`, `.cpp`, `.cc`, `.cxx`)

### 5. Navegación Profesional y Seguridad contra Modificaciones Accidentales
* **Separación estricta de Lectura vs Reescritura:** Los escaneos históricos son de solo lectura. El motor de refactorización (`/refactor` o `git-ast-search refactor`) opera bajo la filosofía de **`dry-run` por defecto**, mostrando previsualizaciones de diffs antes de permitir cualquier escritura en disco (`--apply`).

---

## 📁 Estructura del Proyecto v0.3.0

```
Git-AST-Search/
├── src/
│   ├── main.rs                   # Loop principal y orquestación
│   ├── ui/                        # Interfaz de Usuario Modular
│   │   ├── mod.rs                # Punto de entrada de UI
│   │   ├── app.rs                # Estado global y manejadores de App
│   │   ├── render.rs             # Funciones de dibujo (Frame orchestration)
│   │   ├── layout.rs             # Cálculos geométricos y constraints
│   │   ├── events.rs             # Bridge de eventos crossterm (WIP)
│   │   └── components/           # Widgets atómicos
│   │       ├── mod.rs            # Export de componentes
│   │       ├── sidebar.rs        # Panel lateral de historial
│   │       ├── results_grid.rs   # Grilla dinámica de resultados
│   │       ├── search_bar.rs     # Input de comandos/patrones
│   │       ├── status_bar.rs     # Información de modo y estado
│   │       └── help_overlay.rs   # Pantallas de ayuda y bienvenida
│   ├── modules/                   # Modelos de datos
│   │   ├── mod.rs
│   │   ├── search_result.rs       # Entidad SearchResult
│   │   ├── chat_entry.rs          # Modelo de historial de búsqueda
│   │   ├── session.rs             # Persistencia y gestión de sesiones
│   │   ├── bookmark.rs            # Marcadores de resultados
│   │   ├── filter.rs              # Lógica de filtrado
│   │   └── config.rs              # Configuración y Temas
│   ├── commands/                  # Engine de Comandos Slash
│   │   ├── mod.rs
│   │   ├── parser.rs              # Parseo de comandos /
│   │   ├── executor.rs            # Ejecutor de lógica de comandos
│   │   ├── registry.rs            # Registro central de comandos
│   │   ├── autocomplete.rs         # Sugerencias y autocompletado
│   │   └── commands/              # Implementaciones individuales
│   │       ├── mod.rs
│   │       ├── search.rs          # Comando /search
│   │       └── export.rs          # Comando /export
│   ├── navigation/               # Navegación y Modos (Vim-style)
│   │   ├── mod.rs                 # NavigationState
│   │   └── modes.rs               # NavMode enum
│   ├── languages/                 # Parsers y Detección Multipolíglota
│   │   ├── mod.rs
│   │   ├── registry.rs            # Registro de lenguajes y patrones
│   │   ├── detector.rs            # Detección automática por extensión
│   │   └── patterns.rs            # Definición de patrones AST base
│   └── engine/                    # Core: Git2 + AST-Grep + Rayon
│       └── mod.rs                 # Motor de búsqueda e integración Git
├── assets/                        # Recursos visuales y screenshots
├── docs/                          # Documentación detallada
├── Cargo.toml                     # Dependencias y meta del proyecto
├── README.md                      # Documentación en Español
└── README_ENG.md                  # English Documentation
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

## 🛠️ Guía de Uso: Apuntar a Cualquier Repositorio

`Git-AST-Search` es completamente portable e independiente de la carpeta donde se encuentre compilado. Puede auditar **cualquier repositorio Git en tu máquina** (local o en otra unidad de disco).

### 1. En la TUI Interactiva
Pasa la ruta del repositorio como primer argumento posicional (por defecto usa `.`, el directorio donde abres la terminal):

```bash
# Auditar el repositorio actual en la terminal
git-ast-search .

# Auditar otro proyecto por ruta relativa o absoluta
git-ast-search /home/user/dev/linux
git-ast-search "D:\proyectos\backend-api"
```

### 2. En el Modo Headless (Línea de Comandos)
Utiliza el flag opcional `--repo <RUTA>` para dirigir consultas a cualquier proyecto:

```bash
# Búsqueda estructural directa en otro repositorio
git-ast-search query "$A.unwrap()" --repo "D:\proyectos\mi-app" --lang rust --format json

# Arqueología de bugs / AST Blame en otro repositorio
git-ast-search evolution "std::thread::spawn($$$)" --repo "D:\proyectos\mi-app" --format text

# Refactorización estructural segura en otro repositorio
git-ast-search refactor "$A.unwrap()" "$A.unwrap_or_default()" --repo "D:\proyectos\mi-app"
```

### 3. En el Servidor MCP (Para Agentes de IA en el IDE)
Puedes iniciar el servidor MCP apuntando a un repositorio por defecto:
```bash
git-ast-search mcp "D:\proyectos\mi-app"
```
Además, en cada invocación de herramienta (`git_ast_search`, `git_ast_refactor`, `git_ast_evolution`), el agente de IA puede enviar el parámetro `"repo_path": "/otra/ruta"` para inspeccionar repositorios dinámicamente sin reiniciar el servidor.

---

## 🎮 Controles de la TUI

### Modo Insert (Default)

| Atajo | Acción |
|-------|--------|
| `Cualquiera` | Escribir patrón AST |
| `Enter` | Iniciar búsqueda |
| `Esc` | Volver al Modo Normal |

### Modo Normal

| Atajo | Acción |
|-------|--------|
| `Tab` | Alternar foco entre Resultados y Panel de Historial |
| `i` / `a` | Entrar en Modo Insert |
| `j` / `↓` | Siguiente resultado (o siguiente consulta en Historial) |
| `k` / `↑` | Resultado anterior (o consulta anterior en Historial) |
| `Enter` / `r` | Re-ejecutar consulta histórica (en Historial) o Iniciar búsqueda |
| `e` / `i` | Cargar consulta histórica al editor de texto (en Historial) |
| `h` / `←` | Página anterior / Saltar a Historial |
| `l` / `→` | Página siguiente / Saltar a Resultados |
| `Esc` | Limpiar selección / Volver a Resultados / Salir |
| `Ctrl+B` | Toggle sidebar lateral |
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
| `/history` o `/hist` | Enfocar y activar panel de Historial interactivo |
| `/replay [N]` o `/rerun` | Re-ejecutar consulta histórica por índice (default: última) |
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

### 🤖 Comandos de Inteligencia Artificial (Ollama)

| Comando | Descripción |
|---------|-------------|
| `/ai <pregunta>` | Traduce lenguaje natural a un patrón AST mediante Ollama y ejecuta la búsqueda |
| `/report [tema]` | Genera un informe de auditoría ejecutiva en `audit_report.md` con los resultados |
| `/refactor <P> -> <R>` | Previsualización segura de reescritura estructural (dry-run por defecto; `--apply` para escribir) |
| `/evolution <patrón>` | Arqueología de bugs: Traza la línea cronológica y mutaciones históricas del patrón |

---

## 🤖 Integración con AIs y Agentes (CLI & MCP Server)

`Git-AST-Search` está diseñado para ser utilizado directamente por modelos de lenguaje y entornos de desarrollo (como **Antigravity, Cursor, Claude y Windsurf**):

### 1. Servidor MCP (Model Context Protocol)
Inicia el servidor estándar sobre `stdio` para que tu asistente de IA lo invoque como una herramienta (tool) nativa:
```bash
git-ast-search mcp .
```
**Tools disponibles para la IA:**
* `git_ast_search`: Búsqueda AST estructural en el historial en milisegundos.
* `ai_query_ast`: Traducción de consultas en lenguaje natural a patrones AST usando Ollama.
* `generate_git_ast_report`: Diagnósticos y reportes de auditoría generados por IA.
* `list_ast_patterns`: Consulta del catálogo de patrones clasificados por lenguaje.
* `git_ast_refactor`: Reescritura estructural segura de código con previsualización dry-run.
* `git_ast_evolution`: Blame semántico para auditar cuándo mutó o se introdujo un bloque de código.

### 2. Modo Headless (CLI & CI/CD)
```bash
# Búsqueda estructural directa con salida JSON limpia
git-ast-search query '$A.unwrap()' --lang rust --format json

# Consulta en lenguaje natural vía Ollama por terminal
git-ast-search ai "encuentra bloques catch vacíos en typescript"

# Previsualización segura (dry-run) de refactorización estructural
git-ast-search refactor '$A.unwrap()' '$A.unwrap_or_default()' --lang rust

# Aplicar refactorización estructural directamente al disco
git-ast-search refactor '$A.unwrap()' '$A.unwrap_or_default()' --apply

# Arqueología de bugs: rastrear cómo evolucionó un bloque en la historia
git-ast-search evolution 'std::thread::spawn($$$)' --format text
```

---

## 🔍 Ejemplos de Búsqueda Semántica Multipolíglota

> **📖 Pro Tip:** ¡Revisa nuestro renovado [Catálogo de Exploración de Código](./code_exploration.md) que contiene **260 patrones AST listos para usar** (incluyendo 88 Fundamentales y 88 para Rust Profesional) para potenciar tus auditorías!

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

## ⚙️ Arquitectura de Sistemas (v0.3.0)

```
┌─────────────────────────────────────────────────────────────┐
│                          main.rs                            │
│           (Entry point ~50 líneas, Loop de Eventos)         │
└─────────────────────────────┬───────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐      ┌───────────────┐     ┌───────────────┐
│      ui/      │      │   commands/   │     │  navigation/  │
│ (Componentes) │      │   (Ingest)    │     │   (FSM)       │
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
│    (Modelos)  │      │  (TreeSitter) │
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
