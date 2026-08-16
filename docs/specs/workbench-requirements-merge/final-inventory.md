# FileForgeWorkbench — Final Sub-Project Inventory

> Generated as part of Task 18.6 — Workbench Requirements Merge

---

## 1. Complete Sub-Project Inventory

### Legend

- **Reqs** = Number of top-level `### Requirement N` sections
- **AC** = Number of individual acceptance criteria (numbered items)
- **Sources**: FFE = FileForgeEditor, SCI = Scintilla/Lexilla/SciTE, WB = Workbench Architecture Brief, DBV = DBeaver, DSC = Dataset Catalog Brief, NEW = novel to this project
- **Layer**: Grouping category for implementation ordering

---

### Foundation Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 1 | `logging-subsystem` | Structured file-based logging, rotation, diagnostics, thread safety | 10 | 57 | FFE, WB |

### Platform Architecture Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 2 | `platform-core` | GUI-independent workbench core, crate structure, layer separation | 9 | 52 | FFE, WB |
| 3 | `command-framework` | Command registry, dispatch, metadata, undo/redo integration, scripting bridge | 7 | 43 | FFE, SCI, WB |
| 4 | `plugin-architecture` | Trait-based plugins, registration, lifecycle, capability discovery | 7 | 41 | FFE, WB |
| 5 | `workflow-engine` | State machine workflows, step sequencing, cancellation, progress reporting | 7 | 49 | FFE, WB |
| 6 | `layout-and-docking` | Dockable panels, tab groups, floating windows, personas, serialisable layouts | 10 | 93 | FFE, WB |
| 7 | `configuration-system` | TOML-based config, hot-reload, user profiles, per-project overrides, EditorConfig | 9 | 49 | FFE, SCI, WB |

### Virtual File System Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 8 | `virtual-file-system` | VFS abstraction (FFW-ARCH-001), unified resource addressing, provider registry | 8 | 60 | FFE, WB, DSC |
| 9 | `connector-local-fs` | Local file system provider, file watching, path resolution, cross-platform | 7 | 69 | FFE, WB |
| 10 | `connector-extensibility` | Plugin trait for future connectors, registration, capability advertisement | 7 | 47 | FFE, WB, DSC |

### Core Editor Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 11 | `document-model` | Text storage, buffer management, line indexing, large-file streaming | 10 | 75 | FFE, SCI, WB |
| 12 | `edit-operations` | Edit mode, character insertion/deletion, selection, multi-caret | 15 | 145 | FFE, SCI, WB |
| 13 | `undo-redo-transactions` | Full transaction system, coalescing, recovery, undo selection history | 18 | 121 | FFE, SCI, WB |
| 14 | `viewport-and-scrolling` | Viewport management, scrollbar, smooth scrolling | 13 | 97 | FFE, SCI, WB |
| 15 | `display-line-mapping` | Document-to-display line tracking for folding, exclusion, wrapping | 10 | 73 | FFE, SCI, WB |

### Command Engine Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 16 | `command-semantics` | Full ISPF command engine, parser, dispatcher, error handling | 7 | 54 | FFE, WB |
| 17 | `find-and-replace` | FIND/RFIND/CHANGE/RCHANGE with all modifiers, Unicode case folding | 20 | 152 | FFE, SCI, WB |
| 18 | `line-commands` | All ISPF line commands, block pairing, pending state | 14 | 92 | FFE, WB |
| 19 | `exclude-show-filter` | Line visibility, EXCLUDE/SHOW/RESET, display integration | 10 | 75 | FFE, SCI, WB |
| 20 | `navigation-commands` | LOCATE, SORT, COLS, BOUNDS, paragraph/word navigation | 19 | 148 | FFE, SCI, WB |

### UI and Rendering Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 21 | `menu-and-statusbar` | Menu bar, status bar layout, mode indicators | 11 | 66 | FFE, SCI, WB |
| 22 | `theme-and-appearance` | Colours, fonts, TOML themes, dark/light/high-contrast | 12 | 75 | FFE, SCI, WB |
| 23 | `text-decorations` | Search highlighting, error underlines, change markers, bookmarks | 15 | 127 | FFE, SCI, WB |
| 24 | `whitespace-and-guides` | Whitespace visibility, indent guides, edge column indicator | 9 | 62 | FFE, SCI, WB |
| 25 | `caret-and-selection` | Caret appearance, selection display, virtual space | 12 | 87 | FFE, SCI, WB |

### Language and Highlighting Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 26 | `language-service` | Language detection, TOML definitions, multi-line state | 10 | 61 | FFE, WB |
| 27 | `syntax-highlighting` | Highlighting engine, incremental re-highlight, keyword matching | 15 | 109 | FFE, SCI, WB |
| 28 | `auto-indentation` | Language-aware indent, block-start/end patterns | 10 | 64 | FFE, WB |

### File I/O and Session Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 29 | `file-operations` | Open, Save, Save As, New, Revert, Recent Files (all via VFS) | 10 | 86 | FFE, SCI, WB |
| 30 | `background-io` | Async file loading/saving, progress, cancellation, large-file support | 8 | 57 | FFE, WB |
| 31 | `encoding-and-characters` | Unicode handling, BOM detection, encoding detection, word classification | 14 | 107 | FFE, SCI, WB |
| 32 | `external-modification` | File change detection, reload prompt, mtime tracking (via VFS watcher) | 10 | 66 | FFE, WB |
| 33 | `startup-and-session` | Startup sequence, config loading, session restore, graceful degradation | 12 | 85 | FFE, SCI, WB |
| 34 | `multi-tab-editor` | Tab collection, per-tab state, MRU, context menu | 14 | 117 | FFE, WB |

### Desktop Integration Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 35 | `clipboard-operations` | Copy/Cut/Paste, COPY command clipboard-paste mode, file-insert | 19 | 120 | FFE, SCI, WB |
| 36 | `function-keys-and-history` | Global/profile key maps, key label bar, RETRIEVE, command history | 11 | 60 | FFE, WB |
| 37 | `shell-command` | External command execution, terminal, output capture | 18 | 105 | FFE, WB |
| 38 | `context-help` | F1 help, Help Panel, navigation, content | 16 | 93 | FFE, WB |
| 39 | `view-zoom` | Zoom level, shortcuts, persistence | 9 | 51 | FFE, SCI, WB |
| 40 | `line-wrap-toggle` | Word wrap modes | 13 | 84 | FFE, SCI, WB |

### Extension and Macro Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 41 | `lua-macro-engine` | Lua scripting, editor API, event hooks, per-buffer state, auto-reload | 10 | 73 | FFE, WB |
| 42 | `command-completion` | Command-line auto-complete, popup positioning | 10 | 69 | FFE, SCI, WB |

### Display Mode Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 43 | `hex-display` | HEX mode, hex editing, hex search | 16 | 105 | FFE, WB |
| 44 | `sequence-numbers` | Auto-detect, strip, UNNUM, NUMBER | 14 | 100 | FFE, WB |
| 45 | `tabs-and-mask` | TABS/MASK display-helper commands | 18 | 111 | FFE, WB |

### FileForge Domain Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 46 | `fileforge-integration` | Flat-file processing, EBCDIC, COMP-3, VB binary, ASA detection | 16 | 131 | FFE, WB |
| 47 | `structure-catalog` | Catalog management, grid browse/edit | 15 | 123 | FFE, WB |
| 48 | `record-selection-criteria` | Criteria dialog, operators, filtering | 14 | 114 | FFE, WB |
| 49 | `asa-report-preview` | ASA carriage control rendering | 12 | 85 | FFE, WB |
| 50 | `custom-file-viewers` | Viewer registry, PREVIEW command | 10 | 57 | FFE, WB |

### Dataset Catalog Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 51 | `dataset-catalog` | Mainframe dataset catalog emulation, SQLite DB, PDS/PDSE/GDG types | 15 | 126 | FFE, WB, DSC |
| 52 | `dataset-allocator` | Dataset allocation engine: DSN resolution, disposition, symbolic substitution | 16 | 120 | FFE, WB, DSC |

### Job Entry Subsystem Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 65 | `ffw-jes` | JES/SDSF-style batch job queue, initiator pool, job monitor, dataset allocation | 15 | 88 | JES, WB, DSC |

### File Explorer Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 53 | `file-tree-panel` | Panel layout, directory tree, multi-source browsing (Local, Catalogs, Connections) | 14 | 95 | FFE, WB, DSC |
| 54 | `compare-and-merge` | COMPARE command, diff view, merge (VFS-aware) | 17 | 116 | FFE, SCI, WB |

### Performance Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 55 | `idle-processing` | Background incremental work, syntax re-highlighting, wrap calculation | 12 | 73 | FFE, SCI, WB |
| 56 | `large-file-performance` | Long-line handling, measurement caching, chunked rendering | 9 | 63 | FFE, SCI, WB |

### Database Tool Layer

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 57 | `database-tool` | Full Database IDE: connection panel, SQL editor, result grid, schema browser, ER diagrams | 17 | 196 | FFE, DBV |

### Deferred Connectivity (NOT in initial release)

| # | Sub-Project | Description | Reqs | AC | Sources | Status |
|---|-------------|-------------|------|-----|---------|--------|
| 58 | `connector-network-fs` | Network/UNC paths, SMB/CIFS, NFS, mapped drives | — | — | WB | ⚠️ DEFERRED |
| 59 | `connector-ftp-sftp` | FTP, FTPS, SFTP connectors | — | — | WB | ⚠️ DEFERRED |
| 60 | `connector-mainframe` | z/OS FTP, TN3270, z/OSMF, USS SSH | — | — | WB, DSC | ⚠️ DEFERRED |
| 61 | `connector-cloud` | SharePoint, OneDrive, OAuth | — | — | WB | ⚠️ DEFERRED |

### Meta / Orchestration

| # | Sub-Project | Description | Reqs | AC | Sources |
|---|-------------|-------------|------|-----|---------|
| 62 | `project-master` | Master orchestration — inventory, dependency graph, cross-cutting requirements | 10 | 29 | FFE, SCI, WB, DBV, DSC |

### Additional Directories (Research / Non-standard)

| # | Sub-Project | Description | Reqs | AC | Sources | Status |
|---|-------------|-------------|------|-----|---------|--------|
| 63 | `connectivity-core` | Connectivity core placeholder | — | — | WB | No requirements.md |
| 64 | `FFW-JES` | JES (Job Entry Subsystem) research area | — | — | WB | No requirements.md |

---

## 2. Summary Statistics

### Totals

| Metric | Count |
|--------|-------|
| Total sub-project directories | 65 |
| Sub-projects with requirements.md | 59 |
| Sub-projects in initial release scope | 58 |
| Deferred sub-projects (future phases) | 4 |
| Research/placeholder directories | 2 |
| **Total requirements (### Requirement sections)** | **725** |
| **Total acceptance criteria** | **~5,148** |

### Breakdown by Source

| Source | Abbreviation | Sub-Projects Drawing From | Description |
|--------|-------------|---------------------------|-------------|
| FileForgeEditor | FFE | 57 | Original ISPF-inspired editor specs (takes priority on conflicts) |
| Scintilla/Lexilla/SciTE | SCI | 30 | Adapted concepts for viewport, selection, highlighting, decorations |
| Workbench Architecture Brief | WB | 57 | Overriding architectural principles (GUI independence, VFS, plugins) |
| DBeaver Feature Analysis | DBV | 2 | Full Database IDE capabilities adapted to Rust/egui workbench |
| FFW-JES Requirements | JES | 1 | Job Entry Subsystem / SDSF-style batch processing emulator |
| Dataset Catalog Brief | DSC | 7 | VFS abstraction, mainframe filesystem emulation, catalog operations |

> Note: Most sub-projects draw from multiple sources. FFE and WB are nearly universal because the Architecture Brief provides cross-cutting principles that apply to every crate.

### Breakdown by Layer

| Layer | Sub-Projects | Requirements | AC |
|-------|-------------|-------------|-----|
| Foundation | 1 | 10 | 57 |
| Platform Architecture | 6 | 49 | 327 |
| Virtual File System | 3 | 22 | 176 |
| Core Editor | 5 | 66 | 511 |
| Command Engine | 5 | 70 | 521 |
| UI and Rendering | 5 | 59 | 417 |
| Language and Highlighting | 3 | 35 | 234 |
| File I/O and Session | 6 | 68 | 518 |
| Desktop Integration | 6 | 86 | 513 |
| Extension and Macro | 2 | 20 | 142 |
| Display Mode | 3 | 48 | 316 |
| FileForge Domain | 5 | 67 | 510 |
| Dataset Catalog | 2 | 31 | 246 |
| File Explorer | 2 | 31 | 211 |
| Performance | 2 | 21 | 136 |
| Database Tool | 1 | 17 | 196 |
| Meta/Orchestration | 1 | 10 | 29 |
| Deferred Connectivity | 4 | — | — |

---

## 3. Dependency Graph

### Runtime Dependency Edges

Each edge reads as "A depends on B" (A uses B at compile/runtime):

```
logging-subsystem          → (none — leaf dependency)
platform-core              → logging-subsystem
command-framework          → platform-core, logging-subsystem
plugin-architecture        → platform-core, logging-subsystem
workflow-engine            → platform-core, command-framework, logging-subsystem
layout-and-docking         → platform-core, command-framework, configuration-system
configuration-system       → platform-core, logging-subsystem

virtual-file-system        → platform-core, plugin-architecture, logging-subsystem
connector-local-fs         → virtual-file-system, logging-subsystem
connector-extensibility    → virtual-file-system, plugin-architecture, logging-subsystem

document-model             → virtual-file-system, logging-subsystem
edit-operations            → document-model, command-framework, undo-redo-transactions
undo-redo-transactions     → document-model, command-framework
viewport-and-scrolling     → document-model, display-line-mapping, configuration-system
display-line-mapping       → document-model

command-semantics          → command-framework, document-model, logging-subsystem
find-and-replace           → command-semantics, document-model, display-line-mapping
line-commands              → command-semantics, document-model, edit-operations
exclude-show-filter        → command-semantics, document-model, display-line-mapping
navigation-commands        → command-semantics, document-model, viewport-and-scrolling

menu-and-statusbar         → command-framework, configuration-system
theme-and-appearance       → configuration-system, logging-subsystem
text-decorations           → document-model, display-line-mapping
whitespace-and-guides      → document-model, configuration-system, display-line-mapping
caret-and-selection        → document-model, viewport-and-scrolling, configuration-system

language-service           → configuration-system, logging-subsystem
syntax-highlighting        → language-service, document-model, display-line-mapping
auto-indentation           → language-service, document-model, edit-operations

file-operations            → virtual-file-system, command-framework, undo-redo-transactions
background-io              → virtual-file-system, logging-subsystem
encoding-and-characters    → document-model, logging-subsystem
external-modification      → virtual-file-system, logging-subsystem
startup-and-session        → configuration-system, logging-subsystem, virtual-file-system
multi-tab-editor           → document-model, command-framework, layout-and-docking

clipboard-operations       → document-model, command-framework, edit-operations
function-keys-and-history  → command-framework, configuration-system
shell-command              → command-framework, logging-subsystem, workflow-engine
context-help               → command-framework, configuration-system, layout-and-docking
view-zoom                  → configuration-system, command-framework
line-wrap-toggle           → document-model, display-line-mapping, command-framework

lua-macro-engine           → command-framework, plugin-architecture, document-model
command-completion         → command-framework, command-semantics

hex-display                → document-model, command-framework, viewport-and-scrolling
sequence-numbers           → document-model, command-framework
tabs-and-mask              → document-model, command-framework, edit-operations

fileforge-integration      → document-model, encoding-and-characters, virtual-file-system
structure-catalog          → configuration-system, virtual-file-system, logging-subsystem
record-selection-criteria  → document-model, command-framework
asa-report-preview         → document-model, fileforge-integration
custom-file-viewers        → plugin-architecture, document-model, command-framework

dataset-catalog            → virtual-file-system, connector-extensibility, logging-subsystem
dataset-allocator           → dataset-catalog, virtual-file-system

file-tree-panel            → virtual-file-system, dataset-catalog, startup-and-session, layout-and-docking
compare-and-merge          → virtual-file-system, document-model, command-framework, workflow-engine

idle-processing            → syntax-highlighting, display-line-mapping, logging-subsystem
large-file-performance     → document-model, background-io

database-tool              → command-framework, plugin-architecture, layout-and-docking, workflow-engine, virtual-file-system

ffw-jes                    → command-framework, plugin-architecture, layout-and-docking, workflow-engine, virtual-file-system, dataset-catalog, dataset-allocator
```

### Layered Diagram (Build Order)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Layer 0: FOUNDATION                                                      │
│   logging-subsystem                                                      │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 1: PLATFORM ARCHITECTURE                                           │
│   platform-core → command-framework → plugin-architecture                │
│   workflow-engine    layout-and-docking    configuration-system           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 2: VIRTUAL FILE SYSTEM                                             │
│   virtual-file-system → connector-local-fs                               │
│                       → connector-extensibility                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 3: CORE EDITOR                                                     │
│   document-model → edit-operations → undo-redo-transactions              │
│                  → viewport-and-scrolling → display-line-mapping          │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 4: COMMAND ENGINE                                                  │
│   command-semantics → find-and-replace                                   │
│                     → line-commands                                       │
│                     → exclude-show-filter                                 │
│                     → navigation-commands                                 │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 5: UI + RENDERING                                                  │
│   menu-and-statusbar  theme-and-appearance  text-decorations             │
│   whitespace-and-guides  caret-and-selection                             │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 6: LANGUAGE + HIGHLIGHTING                                         │
│   language-service → syntax-highlighting → auto-indentation              │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 7: FILE I/O + SESSION                                              │
│   file-operations  background-io  encoding-and-characters                │
│   external-modification  startup-and-session  multi-tab-editor           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 8: DESKTOP INTEGRATION                                             │
│   clipboard-operations  function-keys-and-history  shell-command         │
│   context-help  view-zoom  line-wrap-toggle                              │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 9: EXTENSIONS + MACROS                                             │
│   lua-macro-engine  command-completion                                    │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 10: DISPLAY MODES                                                  │
│   hex-display  sequence-numbers  tabs-and-mask                           │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 11: FILEFORGE DOMAIN                                               │
│   fileforge-integration  structure-catalog  record-selection-criteria     │
│   asa-report-preview  custom-file-viewers                                │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 12: DATASET CATALOG                                                │
│   dataset-catalog → dataset-allocator                                     │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 13: FILE EXPLORER                                                  │
│   file-tree-panel  compare-and-merge                                     │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 14: PERFORMANCE                                                    │
│   idle-processing  large-file-performance                                │
├─────────────────────────────────────────────────────────────────────────┤
│ Layer 15: DATABASE TOOL                                                  │
│   database-tool                                                          │
├─────────────────────────────────────────────────────────────────────────┤
│ DEFERRED (future phases):                                                │
│   connector-network-fs  connector-ftp-sftp                               │
│   connector-mainframe   connector-cloud                                  │
└─────────────────────────────────────────────────────────────────────────┘
```


---

## 4. Implementation Priority Ordering

The suggested implementation order follows a bottom-up strategy: foundational crates first, then layers that depend on them.

### Phase 1 — Foundation & Infrastructure

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 1 | `logging-subsystem` | Every other crate depends on this. Zero external deps within project. |
| 2 | `platform-core` | GUI-independent core; defines the service bus all crates plug into. |
| 3 | `configuration-system` | Many crates need config reads; must be available early. |
| 4 | `command-framework` | Commands are the universal dispatch mechanism. |
| 5 | `plugin-architecture` | Trait definitions for all plugin-based features. |
| 6 | `workflow-engine` | State machines for complex operations (data transfer, compare). |
| 7 | `layout-and-docking` | Panel infrastructure for all UI content areas. |

### Phase 2 — VFS & Connectors

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 8 | `virtual-file-system` | FFW-ARCH-001: all file I/O goes through VFS. |
| 9 | `connector-local-fs` | First (and initially only) real VFS provider. |
| 10 | `connector-extensibility` | Trait that future connectors implement; needed by dataset-catalog. |

### Phase 3 — Core Editor

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 11 | `document-model` | Text buffer is the central data structure for everything else. |
| 12 | `display-line-mapping` | Needed by viewport, exclude/show, folding. |
| 13 | `edit-operations` | Character-level editing depends on document-model. |
| 14 | `undo-redo-transactions` | Depends on document-model and edit-operations. |
| 15 | `viewport-and-scrolling` | Visible region management, depends on display-line-mapping. |

### Phase 4 — Commands & Navigation

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 16 | `command-semantics` | ISPF command parser/dispatcher; used by all command features. |
| 17 | `find-and-replace` | Core editing feature, heavily used. |
| 18 | `line-commands` | ISPF line commands, block operations. |
| 19 | `exclude-show-filter` | Line visibility engine. |
| 20 | `navigation-commands` | LOCATE, SORT, BOUNDS, etc. |

### Phase 5 — UI, Rendering & Language

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 21 | `theme-and-appearance` | Colours/fonts needed before any visual rendering. |
| 22 | `caret-and-selection` | Caret display for editing. |
| 23 | `text-decorations` | Highlights, indicators, markers. |
| 24 | `whitespace-and-guides` | Visual aids for indentation. |
| 25 | `menu-and-statusbar` | Application chrome. |
| 26 | `language-service` | Language detection for syntax. |
| 27 | `syntax-highlighting` | Depends on language-service + document-model. |
| 28 | `auto-indentation` | Language-aware indent logic. |

### Phase 6 — File I/O & Session

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 29 | `encoding-and-characters` | Needed before file open/save for charset handling. |
| 30 | `file-operations` | Core open/save/new via VFS. |
| 31 | `background-io` | Async large-file support. |
| 32 | `external-modification` | Reload prompts on external change. |
| 33 | `multi-tab-editor` | Multiple documents open simultaneously. |
| 34 | `startup-and-session` | Application launch, session restore. |

### Phase 7 — Desktop Integration

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 35 | `clipboard-operations` | Copy/Cut/Paste. |
| 36 | `function-keys-and-history` | Key bindings and command history. |
| 37 | `view-zoom` | Zoom controls. |
| 38 | `line-wrap-toggle` | Wrap mode switching. |
| 39 | `context-help` | F1 help system. |
| 40 | `shell-command` | External process execution. |

### Phase 8 — Extensions & Display Modes

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 41 | `command-completion` | Command-line auto-complete. |
| 42 | `lua-macro-engine` | Scripting extension. |
| 43 | `hex-display` | Alternate display mode. |
| 44 | `sequence-numbers` | Sequence number display/management. |
| 45 | `tabs-and-mask` | TABS/MASK helper commands. |

### Phase 9 — FileForge Domain

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 46 | `fileforge-integration` | Flat-file/EBCDIC/COMP-3 processing (core domain). |
| 47 | `structure-catalog` | Structure definitions for record layouts. |
| 48 | `record-selection-criteria` | Filtering engine for structured data. |
| 49 | `asa-report-preview` | ASA carriage control rendering. |
| 50 | `custom-file-viewers` | Pluggable viewer framework. |

### Phase 10 — Dataset & Explorer

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 51 | `dataset-catalog` | Mainframe catalog emulation (depends on VFS + connector-extensibility). |
| 52 | `dataset-allocator` | Dataset allocation engine (depends on dataset-catalog). |
| 53 | `ffw-jes` | JES/SDSF batch subsystem (depends on dataset-catalog + workflow + VFS). |
| 54 | `file-tree-panel` | Unified explorer (depends on VFS + dataset-catalog). |
| 55 | `compare-and-merge` | File comparison (depends on VFS + document-model). |

### Phase 11 — Performance & Database

| Priority | Crate | Rationale |
|----------|-------|-----------|
| 56 | `idle-processing` | Background incremental work (depends on syntax + display-line-mapping). |
| 57 | `large-file-performance` | Chunked rendering (depends on document-model + background-io). |
| 58 | `database-tool` | Full Database IDE (depends on platform arch + VFS + workflow). Last major feature. |

### Deferred (Future Releases)

| Priority | Crate | Rationale |
|----------|-------|-----------|
| — | `connector-network-fs` | Network filesystem access — after initial release. |
| — | `connector-ftp-sftp` | FTP/SFTP — after initial release. |
| — | `connector-mainframe` | z/OS remote — after initial release. |
| — | `connector-cloud` | Cloud storage — after initial release. |

---

## 5. Notes

- **FFW-ARCH-001** (VFS Principle) is the overriding architectural constraint: all content access flows through VFS.
- **Initial release scope**: Local filesystem + dataset catalog emulation. Remote connectors deferred.
- **FileForgeEditor priority rule**: Where FFE and SCI/WB requirements conflict, FFE takes precedence.
- **Architecture-incompatible Scintilla items** (C++ ABI, platform rendering, message-passing) were excluded or adapted to Rust/egui equivalents.
- **Crate naming**: All crates follow the pattern `ff-{sub-project-id}` (e.g., `ff-logging`, `ff-vfs`, `ff-document-model`).
- **Cross-cutting requirements** (10 requirements, 29 AC) in `project-master` apply to all crates and are not duplicated in individual specs.

---

*Document generated for FileForgeWorkbench Requirements Merge — Task 18.6*
