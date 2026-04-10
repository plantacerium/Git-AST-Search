# 🦀 Git AST Search TUI v0.2.1

**Git AST Search** es una herramienta de terminal (TUI) de alto rendimiento diseñada para la minería de código histórica. A diferencia de las herramientas de búsqueda tradicionales basadas en texto plano o expresiones regulares (Regex), esta herramienta utiliza **Análisis de Árboles de Sintaxis Abstracta (AST)** para encontrar estructuras de código exactas, ignorando comentarios, espacios en blanco o saltos de línea, a través de *toda* la historia de un repositorio Git.

### ✨ Novedades en v0.2.1
* **UI Modular de Alto Nivel**: Refactorización completa de la interfaz en componentes independientes (`sidebar`, `results_grid`, `search_bar`, `status_bar`).
* **Lógica de Layout Atómica**: Cálculos de dimensiones desacoplados de la lógica de renderizado para mayor estabilidad y fluidez.
* **Sistema de Comandos (Slash Commands)**: Soporte completo para comandos `/search`, `/export`, `/bookmark`, `/sessions`, `/patterns` y sus alias.
* **Navegación tipo Vim Avanzada**: Historial de autocompletado con `Tab`, teclas `j/k/h/l` para resultados/páginas, y Modo Visual `v`.

---

![GIT AST SEARCH](./assets/GIT_AST_SEARCH_V2.JPG)

---
## ⚡ Support
<div align="center">

**Made with ❤️ and ☕ by the Plantacerium**

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/plantacerium)

⭐**Star us on GitHub**⭐
</div>
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

### 3. Perspectiva de Experiencia de Usuario (UI/UX) Modular
Diseñado para el "Flujo de trabajo Terminal-First". v0.2.1 introduce una arquitectura de UI basada en componentes atómicos. Esto permite una granularidad total:
* **Sidebar Histórico**: Mantén el contexto de todas tus investigaciones previas.
* **Search Contextual**: Input inteligente que cambia dinámicamente según el modo (Search vs Command).
* **Grilla de Alto Rendimiento**: Paginación optimizada para no saturar el buffer de la terminal incluso con miles de resultados.

### 4. Perspectiva de Navegación Profesional (Vim-style)
Controles familiares para desarrolladores. Implementa un sistema de estados (`NavMode`) que permite alternar entre navegación rápida, selección visual de bloques de código y ejecución de comandos sin levantar las manos del teclado.

---

## 📁 Estructura del Proyecto v0.2.1

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

## 🛠️ Guía de Uso

Inicia la herramienta pasando la ruta del repositorio Git como argumento (por defecto es el directorio actual):

```bash
./target/release/git-ast-search .
./target/release/git-ast-search /home/user/dev/linux
```

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
| `i` / `a` | Entrar en Modo Insert |
| `j` / `↓` | Siguiente resultado |
| `k` / `↑` | Resultado anterior |
| `h` / `←` | Página anterior |
| `l` / `→` | Página siguiente |
| `Esc` | Limpiar selección / Cerrar |
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

## ⚙️ Arquitectura de Sistemas (v0.2.1)

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
## ⚡ Support
<div align="center">

**Made with ❤️ and ☕ by the Plantacerium**

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/plantacerium)

⭐**Star us on GitHub**⭐
</div>

## 📄 Licencia

Este proyecto está distribuido bajo la Licencia MIT. Siéntete libre de usarlo, modificarlo y distribuirlo para empujar los límites de la minería de código estática.
