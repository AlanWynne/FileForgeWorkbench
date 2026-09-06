# Project Master Specification

## Introduction

This is the **master orchestration document** for the FileForgeWorkbench project. It provides:

1. A high-level summary of the project's purpose and scope
2. A complete inventory of all sub-project specifications
3. A dependency graph showing relationships between sub-projects
4. A wave-based execution plan defining the logical order of implementation
5. Cross-cutting architectural requirements that span multiple sub-projects

This document does NOT redefine acceptance criteria that belong in individual sub-project specs. It references them and defines their relationships.

---

## Project Summary

FileForgeWorkbench is a **Rust Workbench Platform** that evolves FileForgeEditor into a GUI-independent, plugin-capable, workflow-driven desktop application. It combines the ISPF/PDF-inspired editing model of FileForgeEditor with a modern workbench architecture, a Virtual File System abstraction, a full database IDE tool, and mainframe dataset catalog emulation on the local desktop.

### Key Architectural Principles

- **GUI-independent platform-core** (FFW-ARCH-001): All content access goes through the VFS abstraction layer. The core operates without any GUI framework dependency; GUI shells are replaceable.
- **Plugin architecture** with trait-based extensibility: All optional features are implementable as plugins. Core provides traits, plugins implement them.
- **Command-driven architecture** with undo/redo integration: All user-facing operations are routable through the command framework. No direct state mutation from UI.
- **Workflow state machines** for complex operations: Multi-step processes (data transfer, import/export, compare-merge) are modelled as cancellable, resumable workflows.
- **Multi-crate Cargo workspace structure**: One crate per sub-project for independent compilation and testing.
- **Async I/O principle**: All file operations that may block use async I/O to avoid blocking the GUI thread.
- **Initial release scope**: Local filesystem + dataset catalog emulation; remote connectors (FTP, SFTP, z/OS, cloud) are deferred to future phases via the connector-extensibility trait.

### Source References

- **FFE** = FileForgeEditor specifications (24 sub-projects, ~267 requirements — takes priority on conflicts)
- **SCI** = Scintilla/Lexilla/SciTE extracted concepts (adapted to Rust/egui; architecture-incompatible items excluded)
- **WB** = Workbench Platform Architecture Brief (overriding architectural principles)
- **DSC** = Dataset Catalog Brief (VFS abstraction, mainframe filesystem emulation on local desktop)
- **DBV** = DBeaver feature analysis (adapted as integrated full Database IDE tool)

---

## Sub-Project Inventory

### Platform Architecture (NEW — from WB)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 1 | `platform-core` | Platform Core | GUI-independent workbench core, crate structure, layer separation |
| 2 | `command-framework` | Command Framework | Command registry, dispatch, metadata, undo/redo integration, scripting bridge |
| 3 | `plugin-architecture` | Plugin Architecture | Trait-based plugins, registration, lifecycle, capability discovery, versioning |
| 4 | `workflow-engine` | Workflow Engine | State machine workflows, step sequencing, cancellation, progress reporting |
| 5 | `layout-and-docking` | Layout & Docking | Dockable panels, tab groups, floating windows, multi-monitor, personas, serialisable layouts |
| 6 | `configuration-system` | Configuration System | TOML-based config, hot-reload, user profiles, per-project overrides, EditorConfig |

### Virtual File System (from WB + DSC)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 7 | `virtual-file-system` | Virtual File System | VFS abstraction layer (FFW-ARCH-001), unified resource addressing, provider registry, resource URI scheme |
| 8 | `connector-local-fs` | Local FS Connector | Local file system provider, file watching, path resolution, cross-platform path handling |
| 9 | `connector-extensibility` | Connector Extensibility | Plugin trait for future connectors, registration, capability advertisement, provider lifecycle |

### Core Editor (from FFE + SCI)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 10 | `document-model` | Document Model | Text storage, buffer management, line indexing, large-file streaming |
| 11 | `edit-operations` | Edit Operations | Edit mode, character insertion/deletion, selection, multi-caret concepts |
| 12 | `undo-redo-transactions` | Undo/Redo Transactions | Full transaction system, coalescing, recovery, undo selection history |
| 13 | `viewport-and-scrolling` | Viewport & Scrolling | Viewport management, scrollbar, smooth scrolling |
| 14 | `display-line-mapping` | Display Line Mapping | Document-to-display line tracking for folding, exclusion, and wrapping |

### Command Engine (from FFE + SCI)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 15 | `command-semantics` | Command Semantics | Full ISPF command engine, parser, dispatcher, error handling |
| 16 | `find-and-replace` | Find & Replace | FIND/RFIND/CHANGE/RCHANGE with all modifiers, Unicode case folding, incremental search |
| 17 | `line-commands` | Line Commands | All ISPF line commands, block pairing, pending state |
| 18 | `exclude-show-filter` | Exclude/Show Filter | Line visibility, EXCLUDE/SHOW/RESET, display integration |
| 19 | `navigation-commands` | Navigation Commands | LOCATE, SORT, COLS, BOUNDS, paragraph nav, word nav |

### UI and Rendering (from FFE + SCI)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 20 | `menu-and-statusbar` | Menu & Status Bar | Menu bar, status bar layout, mode indicators |
| 21 | `theme-and-appearance` | Theme & Appearance | Colours, fonts, TOML themes, design system, dark/light/high-contrast |
| 22 | `text-decorations` | Text Decorations | Search highlighting, error underlines, change markers, bookmarks, indicators |
| 23 | `whitespace-and-guides` | Whitespace & Guides | Whitespace visibility, indent guides, edge column indicator, wrap markers |
| 24 | `caret-and-selection` | Caret & Selection | Caret appearance, selection display, virtual space |

### Language and Highlighting (from FFE + Lexilla)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 25 | `language-service` | Language Service | Language detection, TOML definitions, multi-line state, content-based detection |
| 26 | `syntax-highlighting` | Syntax Highlighting | Highlighting engine, incremental re-highlight, keyword matching, sub-styles |
| 27 | `auto-indentation` | Auto-Indentation | Language-aware indent, block-start/end patterns |

### File I/O and Session (from FFE + SciTE — VFS-aware)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 28 | `file-operations` | File Operations | Open, Save, Save As, New, Revert, Recent Files, atomic rename (all via VFS) |
| 29 | `background-io` | Background I/O | Async file loading/saving, progress, cancellation, large-file support |
| 30 | `encoding-and-characters` | Encoding & Characters | Unicode handling, BOM detection, encoding detection, word classification |
| 31 | `external-modification` | External Modification | File change detection, reload prompt, mtime tracking (via VFS file-watcher) |
| 32 | `startup-and-session` | Startup & Session | Startup sequence, config loading, session restore, graceful degradation |
| 33 | `multi-tab-editor` | Multi-Tab Editor | Tab collection, per-tab state, MRU, context menu |

### Desktop Integration (from FFE)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 34 | `clipboard-operations` | Clipboard Operations | Copy/Cut/Paste, COPY command clipboard-paste mode, file-insert |
| 35 | `function-keys-and-history` | Function Keys & History | Global/profile key maps, key label bar, RETRIEVE, command history |
| 36 | `shell-command` | Shell Command | External command execution, terminal, output capture |
| 37 | `context-help` | Context Help | F1 help, Help Panel, navigation, content |
| 38 | `view-zoom` | View Zoom | Zoom level, shortcuts, persistence |
| 39 | `line-wrap-toggle` | Line Wrap Toggle | Word wrap modes |

### Extensions and Macros (from FFE + SciTE)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 40 | `lua-macro-engine` | Lua Macro Engine | Lua scripting, editor API, event hooks, OnChar/OnKey, per-buffer state, auto-reload |
| 41 | `command-completion` | Command Completion | Command-line auto-complete, popup positioning |

### Display Modes (from FFE)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 42 | `hex-display` | Hex Display | HEX mode, hex editing, hex search |
| 43 | `sequence-numbers` | Sequence Numbers | Auto-detect, strip, UNNUM, NUMBER |
| 44 | `tabs-and-mask` | TABS & MASK | TABS/MASK commands |

### FileForge Domain (from FFE)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 45 | `fileforge-integration` | FileForge Integration | Flat-file processing, EBCDIC, COMP-3, VB binary, ASA detection |
| 46 | `structure-catalog` | Structure Catalog | Catalog management, grid browse/edit |
| 47 | `record-selection-criteria` | Record Selection Criteria | Criteria dialog, operators, filtering |
| 48 | `asa-report-preview` | ASA Report Preview | ASA carriage control rendering |
| 49 | `custom-file-viewers` | Custom File Viewers | Viewer registry, PREVIEW command |

### Dataset Catalog (from DSC — depends on VFS)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 50 | `dataset-catalog` | Dataset Catalog | Mainframe dataset catalog emulation on local desktop: SQLite catalog DB, dataset naming, PDS/PDSE/GDG types, catalog operations |
| 51 | `dataset-allocator` | Dataset Allocator | Dataset allocation engine: DSN resolution against mounted catalogs, disposition handling (NEW/OLD/SHR/MOD), symbolic substitution — desktop equivalent of DYNALLOC/SVC 99 |

### Job Entry Subsystem (from JES — depends on VFS + dataset-catalog + plugin-architecture + workflow-engine)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 62 | `jes-emulator` | Job Entry Subsystem | Cross-platform JES/SDSF-style emulator: job submission, queue management, initiator pool, scheduling, SDSF-style Job Monitor, job logs, SYSOUT, retention/purge, provider abstraction for future remote JES |

### File Explorer (depends on VFS + dataset-catalog)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 52 | `file-tree-panel` | File Tree Panel | Panel layout, directory tree, async loading, multi-source browsing (Local Files, Catalogs, Connections) |
| 53 | `compare-and-merge` | Compare & Merge | COMPARE command, diff view, merge (VFS-aware resource comparison) |

### Performance (from SCI gap analysis)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 54 | `idle-processing` | Idle Processing | Background incremental work, syntax re-highlighting, wrap calculation |
| 55 | `large-file-performance` | Large File Performance | Long-line handling, measurement caching, chunked rendering |

### Database Tool (from DBV — integrated full Database IDE)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 56 | `database-tool` | Database Tool | Full Database IDE: connection panel, SQL editor, result grid, schema browser, data transfer workflows, ER diagrams |

### Deferred Connectivity (NOT in initial release)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 57 | `connector-network-fs` | Network FS Connector | ⚠️ DEFERRED — Network/UNC paths, SMB/CIFS, NFS |
| 58 | `connector-ftp-sftp` | FTP/SFTP Connector | ⚠️ DEFERRED — FTP, FTPS, SFTP connectors |
| 59 | `connector-mainframe` | Mainframe Connector | ⚠️ DEFERRED — z/OS FTP, TN3270, z/OSMF, USS SSH |
| 60 | `connector-cloud` | Cloud Connector | ⚠️ DEFERRED — SharePoint, OneDrive, OAuth |

### Foundation

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 61 | `logging-subsystem` | Logging Subsystem | Structured logging, rotation, diagnostics, thread safety |

**Total: 74 sub-projects across 20 categories** (62 original + 12 added in Phases W, AA, BS, CJ, CK + 4 deferred connector stubs)

### Compiler Toolchain Integration (Phase W)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 63 | `compiler-toolchain-integration` | Compiler Toolchain Integration | GCC and Rust toolchain detection, install, build invocation, diagnostic parsing, Toolchain Panel UI; generic `ToolchainPlugin` trait for future toolchains (LLVM, GnuCOBOL, OpenJDK) |

### Virtual Catalog Manager (Phase AA -- implemented inline in ff-desktop)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 64 | `virtual-catalog-manager` | Virtual Catalog Manager | ISPF-style catalog management UI: create/edit/delete Mainframe/POSIX/Native/Cloud catalogs, dataset allocation dialog, Files Panel (POM option 1), catalog registry persistence, default Home catalog |

### Dataset Ownership Model (Phase M governance)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 65 | `dataset-ownership-model` | Dataset Ownership Model | Governance tests and ownership rules for the dataset catalog; cross-crate consistency validation |

### IDCAMS Emulator (Phase M)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 66 | `idcams-emulator` | IDCAMS Emulator | Emulation of IBM IDCAMS utility: DEFINE CLUSTER/ALIAS/PATH, DELETE, LISTCAT, REPRO, VERIFY commands against the local dataset catalog |

### Workspace Model (Phase BS-A)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 67 | `workspace-model` | Workspace Model | Named persistable workspace grouping root directories with workspace-scoped settings, per-workspace MRU list, `.ffwb-workspace` TOML format, WORKSPACE OPEN/SAVE/CLOSE commands |

### Command Palette (Phase BS-B)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 68 | `command-palette` | Command Palette | Ctrl+Shift+P modal fuzzy-search overlay over all registered commands; displays name, category, description, shortcut; executes via Command_Dispatch; persists recent commands in session |

### Global Search (Phase BS-C / BT)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 69 | `global-search` | Global Search | Cross-file search and replace: `ff-global-search` crate, Search Results panel, Ctrl+Shift+F activation, GSEARCH command, replace pipeline with preview, search history (last 20 queries) |

### Bootstrap Scripts (Phase CJ)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 70 | `bootstrap-scripts` | Bootstrap Scripts | Platform-specific scripts (Windows PowerShell, Linux bash, macOS bash) that install the Rust stable toolchain without admin rights and guide a new contributor from `git clone` to `cargo build` |

### Automated Dialog Testing (Phase CK)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 71 | `automated-dialog-testing` | Automated Dialog Testing | FFTest framework: automation ID infrastructure, FFTest script parser and runner, headless execution, HTML/JSON reporting, visual regression; `ff-fftest` crate |

### EARS Integration (Phase EI -- documentation only)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 72 | `ears-integration` | EARS Integration | Planning and workflow documents for the EARS requirements integration project (Phases BW-CI); no deliverable crate -- docs only |

### JCL Resolver (stub -- pending requirements)

| # | Spec ID | Name | Description |
|---|---------|------|-------------|
| 73 | `jcl-resolver` | JCL Resolver | PENDING -- stub placeholder for future JCL resolution and submission pipeline; no requirements written yet |

---

## Dependency Graph

Sub-projects are organized into waves. Each wave's tasks depend on all prior waves being defined (requirements written). The dependency arrows mean "requirements must be defined before or concurrently with."

### Wave Structure

```
Wave 0: Foundation (no dependencies)
├── logging-subsystem

Wave 1: Project Master (depends on Wave 0)
├── project-master (this document)

Wave 2: Platform Architecture (depends on Wave 1)
├── platform-core
├── command-framework
├── plugin-architecture
├── workflow-engine
├── layout-and-docking
├── configuration-system

Wave 3: Virtual File System (depends on Wave 2)
├── virtual-file-system
├── connector-local-fs
├── connector-extensibility

Wave 4: Core Editor (depends on Wave 3)
├── document-model
├── edit-operations
├── undo-redo-transactions
├── viewport-and-scrolling
├── display-line-mapping

Wave 5: Command Engine (depends on Wave 4)
├── command-semantics
├── find-and-replace
├── line-commands
├── exclude-show-filter
├── navigation-commands

Wave 6: UI and Rendering (depends on Wave 5)
├── menu-and-statusbar
├── theme-and-appearance
├── text-decorations
├── whitespace-and-guides
├── caret-and-selection

Wave 7: Language and Highlighting (depends on Wave 6)
├── language-service
├── syntax-highlighting
├── auto-indentation

Wave 8: File I/O and Session (depends on Wave 7)
├── file-operations
├── background-io
├── encoding-and-characters
├── external-modification
├── startup-and-session
├── multi-tab-editor

Wave 9: Desktop Integration (depends on Wave 8)
├── clipboard-operations
├── function-keys-and-history
├── shell-command
├── context-help
├── view-zoom
├── line-wrap-toggle

Wave 10: Extensions and Macros (depends on Wave 9)
├── lua-macro-engine
├── command-completion

Wave 11: Display Modes (depends on Wave 10)
├── hex-display
├── sequence-numbers
├── tabs-and-mask

Wave 12: FileForge Domain (depends on Wave 11)
├── fileforge-integration
├── structure-catalog
├── record-selection-criteria
├── asa-report-preview
├── custom-file-viewers

Wave 13: Dataset Catalog (depends on Wave 3 VFS + Wave 12 FileForge)
├── dataset-catalog
├── dataset-allocator

Wave 13.5: Job Entry Subsystem (depends on Wave 2 Plugin + Wave 3 VFS + Wave 4 Workflow + Wave 13 Dataset)
├── FFW-JES

Wave 14: File Explorer (depends on Wave 13 + Wave 8)
├── file-tree-panel
├── compare-and-merge

Wave 15: Performance (depends on Wave 7 + Wave 8)
├── idle-processing
├── large-file-performance

Wave 16: Database Tool Research (no code deps — can run any time)
├── DBeaver research tasks (16.1–16.7 in tasks.md)

Wave 17: Database Tool Synthesis (depends on Wave 2 Platform + Wave 16 Research)
├── database-tool

Wave 18: Final Validation (depends on all prior waves)
├── Cross-reference consistency check
├── FileForgeEditor requirements coverage verification
├── Gap analysis coverage verification
├── DBeaver requirements completeness verification
├── VFS principle compliance verification
├── Final sub-project inventory with requirement counts
```

### Key Cross-Wave Dependencies

| Sub-Project | Hard Dependencies |
|-------------|-------------------|
| `virtual-file-system` | platform-core, plugin-architecture |
| `connector-local-fs` | virtual-file-system |
| `document-model` | virtual-file-system (content access via VFS) |
| `file-operations` | virtual-file-system, command-framework, undo-redo-transactions |
| `background-io` | virtual-file-system (async provider interface) |
| `external-modification` | virtual-file-system (file-watcher) |
| `dataset-catalog` | virtual-file-system, connector-extensibility |
| `FFW-JES` | plugin-architecture, command-framework, workflow-engine, layout-and-docking, virtual-file-system, dataset-catalog, dataset-allocator |
| `file-tree-panel` | virtual-file-system, dataset-catalog, startup-and-session |
| `database-tool` | command-framework, plugin-architecture, layout-and-docking, workflow-engine, virtual-file-system |
| `theme-and-appearance` | configuration-system (theme loading) |
| `lua-macro-engine` | command-framework, plugin-architecture |
| `idle-processing` | syntax-highlighting, display-line-mapping |
| `large-file-performance` | document-model, background-io |

---

## Cross-Cutting Architectural Requirements

These requirements span multiple sub-projects and must be honoured throughout.

### Requirement 1: FFW-ARCH-001 — Virtual File System Principle

**User Story:** As a workbench developer, I want all content access to go through the VFS abstraction layer, so that providers (local FS, dataset catalog, future remote connectors) are interchangeable without modifying consuming code.

**Source:** WB Architecture Brief — overriding connectivity principle. [WB, DSC]

#### Acceptance Criteria

1. ALL sub-projects that open, read, write, browse, or search file content SHALL access that content exclusively through the `virtual-file-system` crate's provider interface — never via direct `std::fs` calls.
2. THE VFS layer SHALL define a resource URI scheme (`vfs://provider/path`) that uniquely identifies any resource regardless of its backing store.
3. WHEN a new content provider is needed (e.g., dataset-catalog, future FTP connector), THE provider SHALL implement the `VfsProvider` trait defined by the `virtual-file-system` crate and register itself with the provider registry — no changes to consuming code required.
4. THE `connector-extensibility` crate SHALL define the plugin trait that future remote connectors implement, ensuring the VFS principle extends to deferred connectivity without architectural changes.

---

### Requirement 2: GUI Independence

**User Story:** As a workbench architect, I want the platform-core to operate without any GUI framework dependency, so that the rendering shell (egui, future alternatives) is replaceable without rewriting business logic.

**Source:** WB Architecture Brief §3 Principle 1. [WB]

#### Acceptance Criteria

1. THE `platform-core` crate SHALL have zero dependencies on any GUI framework (egui, winit, wgpu, or any future rendering library).
2. ALL business logic (commands, file operations, document model, undo/redo, workflows, plugins) SHALL execute within the GUI-independent layer and communicate with the GUI shell through a defined messaging interface.
3. WHEN a GUI shell is replaced (e.g., egui swapped for another framework), THE platform-core, all plugins, and all business logic SHALL continue to function without recompilation, requiring only the shell adapter crate to be rewritten.
4. THE GUI shell crate SHALL depend on platform-core; platform-core SHALL NOT depend on the GUI shell crate (strict layering, no circular dependencies).

---

### Requirement 3: Plugin Architecture Principle

**User Story:** As a workbench developer, I want all optional features to be implementable as plugins, so that the core remains minimal and features can be independently developed, tested, and deployed.

**Source:** WB Architecture Brief §10. [WB]

#### Acceptance Criteria

1. ALL optional features (viewers, language services, connectors, macro engines, database tool) SHALL be implementable as plugins that implement traits defined by the core.
2. THE `plugin-architecture` crate SHALL define the `FileForgePlugin` trait with lifecycle methods: `initialize`, `activate`, `deactivate`, `shutdown`.
3. WHEN a plugin is loaded, THE plugin system SHALL provide a `PluginContext` through which the plugin obtains services (logging, commands, configuration, VFS access) without tight coupling to implementation crates.
4. THE plugin system SHALL support capability discovery — plugins advertise what they provide (commands, viewers, providers, language support) and the core queries capabilities at runtime.
5. IF a plugin fails to initialize, THEN THE platform-core SHALL log the failure and continue operating with reduced functionality — never crash the application.

---

### Requirement 4: Command-Driven Architecture

**User Story:** As a workbench developer, I want all user-facing operations to be routable through the command framework, so that keyboard shortcuts, menus, macros, and the command line all invoke the same execution path with consistent undo/redo integration.

**Source:** WB Architecture Brief §7. [WB]

#### Acceptance Criteria

1. ALL user-facing operations that modify state SHALL be registered as commands in the `command-framework` crate's command registry.
2. THE command framework SHALL provide a single dispatch entry point (`execute_command(id, params)`) that all input sources (keyboard, menu, command line, macro, plugin) use to invoke operations.
3. WHEN a command is executed, THE command framework SHALL integrate with the undo/redo system — every undoable command produces an undo record as part of its execution.
4. NO UI code SHALL directly mutate application state. All state changes SHALL flow through commands.
5. THE command framework SHALL support command metadata (display name, description, default shortcut, icon, category) for runtime inspection by menus, keybinding UI, and help systems.

---

### Requirement 5: Configuration Namespace

**User Story:** As an operator, I want all configuration to live in a consistent TOML-based system with no key conflicts, hot-reload capability, and layered profiles, so that I have a single coherent configuration experience.

**Source:** FFE Req 1 (adapted), WB Architecture Brief §8. [FFE, WB]

#### Acceptance Criteria

1. ALL configuration keys across all sub-projects SHALL be unique — no two specs may define the same key name with different semantics.
2. THE `configuration-system` crate SHALL own the configuration layer, providing typed access to settings for all platform-core subsystems and plugins.
3. THE configuration system SHALL support a layered model: defaults → system → user → project → workspace, with later layers overriding earlier ones.
4. WHEN a configuration file is modified on disk, THE configuration system SHALL detect the change and hot-reload affected settings without requiring application restart.
5. WHEN any configuration key contains an invalid value, THE configuration system SHALL apply the default for that key and emit a warning via the logging subsystem — never crash or refuse to start.
6. THE configuration system SHALL support namespace prefixes to group related settings (e.g., `[logging]`, `[editor]`, `[theme]`, `[plugins]`, `[vfs]`).
7. LANGUAGE profile configuration SHALL live in separate TOML files (`languages/*.toml`), not in the main configuration file.

---

### Requirement 6: Async I/O Principle

**User Story:** As a user, I want the GUI to remain responsive while the workbench performs file operations, so that large-file loading, saving, and network operations never freeze the interface.

**Source:** WB Architecture Brief §9. [WB]

#### Acceptance Criteria

1. ALL file operations that may block (open, read, write, directory listing, network I/O) SHALL use async I/O via Tokio-based background workers.
2. THE GUI render thread SHALL NOT be blocked for more than 16 milliseconds (one frame at 60 FPS) by any file or network operation.
3. WHEN an async operation is in progress, THE workbench SHALL provide progress reporting (determinate or indeterminate) and cancellation support through the workflow-engine.
4. THE VFS provider interface SHALL define async method signatures for all I/O operations, enabling providers to implement non-blocking behaviour uniformly.

---

### Requirement 7: Multi-Crate Workspace Structure

**User Story:** As a workbench developer, I want the project to use a Cargo workspace with one crate per sub-project, so that crates can be compiled, tested, and versioned independently.

**Source:** WB Architecture Brief §4. [WB]

#### Acceptance Criteria

1. THE project SHALL be structured as a Cargo workspace with a root `Cargo.toml` listing all member crates.
2. EACH sub-project in the inventory SHALL correspond to exactly one crate within the workspace (e.g., `crates/ff-logging`, `crates/ff-core`, `crates/ff-vfs`, `crates/ff-command`).
3. CRATE names SHALL follow the pattern `ff-{sub-project-id}` (e.g., `ff-logging`, `ff-document-model`, `ff-plugin`).
4. EACH crate SHALL be independently compilable and testable via `cargo test -p ff-{crate-name}`.
5. INTER-CRATE dependencies SHALL be minimized. Crates SHALL depend only on crates from earlier waves in the dependency graph; circular dependencies are forbidden.
6. THE root application executable SHALL be named `ffwb` (producing `ffwb.exe` on Windows, `ffwb` on Linux/macOS). The binary crate SHALL reside in `crates/ff-desktop` (Shell Layer) and SHALL serve as the sole entry point that bootstraps `WorkbenchApp` from `ff-core` and launches the GUI shell.

---

### Requirement 8: Error Message Standards

**User Story:** As an operator, I want all error messages across the workbench to follow a consistent format, so that I can quickly understand problems regardless of which subsystem produced the error.

**Source:** FFE Req 5 (adapted). [FFE]

#### Acceptance Criteria

1. ALL error messages SHALL be 200 characters or fewer.
2. ALL command error messages SHALL identify the command name that failed.
3. ALL error messages SHALL be displayed in the status/message area — never in modal dialogs during normal operation (startup warnings are deferred to the status area).
4. ALL file operation errors SHALL include the resource URI that failed.
5. ALL configuration warnings SHALL identify the key name and the default that was applied.
6. THE error format SHALL be consistent across all crates: `[subsystem] operation: description`.

---

### Requirement 9: Status Bar Layout

**User Story:** As an operator, I want the status bar to display information from multiple features simultaneously without overwriting, so that I always see the full workbench state.

**Source:** FFE Req 2 (adapted for workbench multi-panel architecture). [FFE]

#### Acceptance Criteria

1. THE status bar SHALL display the following elements in left-to-right order, each occupying a fixed or proportional region:
   - Mode (Browse/Edit/View)
   - Insert/Overstrike
   - Encoding
   - Zoom indicator (when not 100%)
   - Line and column numbers
   - Modified indicator
   - Total line count
   - Active indicators: HEX, ASA, SEQSHOW, Viewer name, Criteria name, Record filter, Type filter
   - Active panel name (when focus is in a non-editor panel)
   - Message area (right-aligned, expanding)
2. WHEN multiple indicators are active simultaneously, ALL indicators SHALL be visible — none may be hidden or overwritten by another.
3. THE status bar SHALL be a single row that is always visible regardless of which panels, modes, or docked views are active.
4. PLUGINS SHALL be able to register additional status bar indicators through the plugin-architecture capability system.

---

### Requirement 10: Keyboard Shortcut Registry

**User Story:** As a workbench developer, I want a single authoritative registry of keyboard shortcuts, so that new features never accidentally conflict with existing bindings and users can discover all available shortcuts.

**Source:** FFE Req 3 (adapted for workbench command framework). [FFE]

#### Acceptance Criteria

1. THE following keyboard shortcuts are reserved globally and SHALL NOT be overridden by the key map system or plugins:
   - F1 — Help (hard-coded, per `context-help` spec)
   - Ctrl+Plus / Ctrl+Minus / Ctrl+0 — Zoom (per `view-zoom` spec)
   - Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z — Undo/Redo
   - Ctrl+C / Ctrl+X / Ctrl+V / Ctrl+A — Clipboard
   - Ctrl+S — Save
   - Ctrl+F — Focus command line with FIND
   - Ctrl+H — Focus command line with CHANGE
   - Ctrl+G — Go to line
   - Ctrl+Tab / Ctrl+Shift+Tab — Tab switching (per `multi-tab-editor`)
   - Ctrl+W — Close tab
   - Ctrl+N — New tab
   - Ctrl+Shift+D — Dock/undock panel (per `layout-and-docking`)
   - Ctrl+Shift+T — Undock/redock tab (per `layout-and-docking`)
2. FUNCTION keys F2–F24 SHALL be user-configurable via the key map system managed by `function-keys-and-history`.
3. WHEN a sub-project spec or plugin defines a new keyboard shortcut, IT SHALL be registered with the command framework's shortcut registry to prevent conflicts.
4. THE command framework SHALL detect and report shortcut conflicts at registration time, rejecting duplicate bindings with a warning via the logging subsystem.
5. PLUGINS SHALL be able to register keyboard shortcuts for their commands, but SHALL NOT override reserved global shortcuts.

---

## Initial Release Connectivity Scope

The initial release of FileForgeWorkbench provides:

1. **VFS abstraction layer** — the architectural foundation for all content access
2. **Local filesystem provider** (`connector-local-fs`) — full read/write/watch support for local files
3. **Dataset catalog emulation** (`dataset-catalog`) — mainframe filesystem behaviour on local desktop via SQLite catalog DB

Remote connectivity is deferred to future releases:
- Network FS (SMB/NFS) — via `connector-network-fs`
- FTP/SFTP — via `connector-ftp-sftp`
- Mainframe remote (z/OS FTP, TN3270, z/OSMF) — via `connector-mainframe`
- Cloud (SharePoint, OneDrive) — via `connector-cloud`

The `connector-extensibility` crate provides the trait-based hook for all future connectors, ensuring they can be added without architectural changes.

---

## Notes

- Each sub-project's detailed acceptance criteria live in `.kiro/specs/{sub-project}/requirements.md`
- Design documents and implementation task lists will be created in subsequent phases
- FileForgeEditor requirements always take priority where conflicts exist
- Architecture-incompatible Scintilla concepts are adapted to Rust/egui equivalents
- All acceptance criteria use EARS format (WHEN/THEN/SHALL, THE system SHALL, IF/THEN)
- DBeaver-derived database tool requirements are adapted as a workbench-integrated full Database IDE
- Database connectivity leverages VFS and connector-extensibility sub-projects
