# Implementation Plan: FileForgeWorkbench Requirements Merge

## Overview

Create the complete requirements specification for FileForgeWorkbench — a Rust Workbench Platform that evolves FileForgeEditor into a GUI-independent, plugin-capable, workflow-driven desktop application. Requirements are merged from four sources:

1. **FileForgeEditor** (priority — all 24 sub-projects, ~267 requirements)
2. **Scintilla/Lexilla/SciTE** (applicable concepts adapted to Rust/egui)
3. **Workbench Platform Architecture Brief** (GUI independence, plugin model, workflow state machines, command-driven architecture)
4. **Dataset Catalog Brief** (VFS abstraction, mainframe filesystem emulation on local desktop, catalog management)

FileForgeEditor requirements take precedence on conflicts. Architecture-incompatible Scintilla requirements (platform rendering, C++ ABI, message-passing) are excluded or adapted. Initial release connectivity is scoped to local filesystem + dataset catalog emulation; remote connectors are deferred.

## Tasks

- [x] 1. Project scaffolding, logging, and steering rules
  - [x] 1.1 Create .kiro directory structure with specs/ and steering/ folders
  - [x] 1.2 Copy and adapt steering rules from FileForgeEditor (TDD, EARS format, build commands, spec task format, test checklist)
  - [x] 1.3 Write requirements for `logging-subsystem` — Structured logging, rotation, diagnostics, thread safety (from FFE file-based-logging). Logging is a foundation dependency for all other crates.
  - [x] 1.4 Create initial project-master requirements.md with sub-project inventory, dependency graph, and cross-cutting architectural requirements

- [x] 2. Platform Architecture Requirements (NEW — from workbench brief)
  - [x] 2.1 Write requirements for `platform-core` — GUI-independent workbench core, crate structure, layer separation
  - [x] 2.2 Write requirements for `command-framework` — Command registry, dispatch, metadata, undo/redo integration, scripting bridge
  - [x] 2.3 Write requirements for `plugin-architecture` — Trait-based plugins, registration, lifecycle, capability discovery, versioning
  - [x] 2.4 Write requirements for `workflow-engine` — State machine workflows, step sequencing, cancellation, progress reporting
  - [x] 2.5 Write requirements for `layout-and-docking` — Dockable panels, tab groups, floating windows, multi-monitor, personas, serialisable layouts
  - [x] 2.6 Write requirements for `configuration-system` — TOML-based config, hot-reload, user profiles, per-project overrides, EditorConfig

- [x] 3. Virtual File System Foundation (from workbench brief + dataset-catalog-brief)
  - [x] 3.1 Write requirements for `virtual-file-system` — VFS abstraction layer (FFW-ARCH-001), unified resource addressing, provider registry, resource URI scheme, provider-agnostic open/save/browse/search
  - [x] 3.2 Write requirements for `connector-local-fs` — Local file system provider, file watching, path resolution, cross-platform path handling (Windows/Linux/macOS)
  - [x] 3.3 Write requirements for `connector-extensibility` — Plugin trait for future connectors (FTP, SFTP, z/OS, cloud), registration, capability advertisement, provider lifecycle

- [x] 4. Core Editor Requirements (from FileForgeEditor + Scintilla)
  - [x] 4.1 Write requirements for `document-model` — Text storage, buffer management, line indexing, large-file streaming (merges FFE mvp-implementation Reqs 1-2 + Scintilla document/cellbuffer concepts)
  - [x] 4.2 Write requirements for `edit-operations` — Edit mode, character insertion/deletion, selection, multi-caret concepts (merges FFE mvp-implementation Req 3 + Scintilla editor/selection)
  - [x] 4.3 Write requirements for `undo-redo-transactions` — Full transaction system, coalescing, recovery, undo selection history (merges FFE undo-redo-transactions + Scintilla UndoHistory)
  - [x] 4.4 Write requirements for `viewport-and-scrolling` — Viewport management, scrollbar, smooth scrolling (merges FFE scrollbar-full-file-range + Scintilla viewport concepts)
  - [x] 4.5 Write requirements for `display-line-mapping` — Document-to-display line tracking for folding, exclusion, and wrapping (NEW from Scintilla gap analysis)

- [x] 5. Command Engine Requirements (from FileForgeEditor + Scintilla)
  - [x] 5.1 Write requirements for `command-semantics` — Full ISPF command engine, parser, dispatcher, error handling (from FFE core-command-semantics Reqs 1-2, 36-38)
  - [x] 5.2 Write requirements for `find-and-replace` — FIND/RFIND/CHANGE/RCHANGE with all modifiers, Unicode case folding, incremental search (merges FFE core-command-semantics Reqs 3-9 + Scintilla RESearch)
  - [x] 5.3 Write requirements for `line-commands` — All ISPF line commands, block pairing, pending state (from FFE core-command-semantics Reqs 22-35)
  - [x] 5.4 Write requirements for `exclude-show-filter` — Line visibility, EXCLUDE/SHOW/RESET, display integration (from FFE core-command-semantics Reqs 7-8 + Scintilla ContractionState)
  - [x] 5.5 Write requirements for `navigation-commands` — LOCATE, SORT, COLS, BOUNDS, paragraph nav, word nav (merges FFE core-command-semantics Reqs 10-21 + Scintilla caret movement)

- [x] 6. UI and Rendering Requirements (from FileForgeEditor + Scintilla)
  - [x] 6.1 Write requirements for `menu-and-statusbar` — Menu bar, status bar layout, mode indicators (from FFE mvp-implementation Req 4)
  - [x] 6.2 Write requirements for `theme-and-appearance` — Colours, fonts, TOML themes, design system, dark/light/high-contrast (merges FFE theme-and-appearance + workbench design system). References configuration-system for theme loading.
  - [x] 6.3 Write requirements for `text-decorations` — Search highlighting, error underlines, change markers, bookmarks, indicators (NEW from Scintilla gap analysis)
  - [x] 6.4 Write requirements for `whitespace-and-guides` — Whitespace visibility, indent guides, edge column indicator, wrap markers (NEW from Scintilla gap analysis)
  - [x] 6.5 Write requirements for `caret-and-selection` — Caret appearance, selection display, virtual space (merges FFE mvp + Scintilla style/viewstyle)

- [x] 7. Language and Highlighting Requirements (from FileForgeEditor + Lexilla)
  - [x] 7.1 Write requirements for `language-service` — Language detection, TOML definitions, multi-line state, content-based detection (merges FFE mvp Req 6 + Lexilla concepts)
  - [x] 7.2 Write requirements for `syntax-highlighting` — Highlighting engine, incremental re-highlight, keyword matching, sub-styles (merges FFE + Lexilla lexer-support)
  - [x] 7.3 Write requirements for `auto-indentation` — Language-aware indent, block-start/end patterns (NEW from SciTE gap analysis)

- [x] 8. File I/O and Session Requirements (from FileForgeEditor + SciTE — VFS-aware)
  - [x] 8.1 Write requirements for `file-operations` — Open, Save, Save As, New, Revert, Recent Files, atomic rename. All operations go through VFS abstraction layer. (merges FFE file-menu-operations + SciTE I/O)
  - [x] 8.2 Write requirements for `background-io` — Async file loading/saving, progress, cancellation, large-file support. Uses VFS provider async interface. (NEW from SciTE gap analysis)
  - [x] 8.3 Write requirements for `encoding-and-characters` — Unicode handling, BOM detection, encoding detection, word classification (NEW from Scintilla gap analysis)
  - [x] 8.4 Write requirements for `external-modification` — File change detection, reload prompt, mtime tracking. Leverages VFS file-watcher. (NEW from SciTE gap analysis)
  - [x] 8.5 Write requirements for `startup-and-session` — Startup sequence, config loading, session restore, graceful degradation (from FFE startup-and-session)
  - [x] 8.6 Write requirements for `multi-tab-editor` — Tab collection, per-tab state, MRU, context menu (from FFE multi-tab-editor)

- [x] 9. Desktop Integration Requirements (from FileForgeEditor)
  - [x] 9.1 Write requirements for `clipboard-operations` — Copy/Cut/Paste, COPY command clipboard-paste mode, file-insert (from FFE copy-clipboard-paste + mvp Req 8)
  - [x] 9.2 Write requirements for `function-keys-and-history` — Global/profile key maps, key label bar, RETRIEVE, command history (from FFE function-keys-and-command-history)
  - [x] 9.3 Write requirements for `shell-command` — External command execution, terminal, output capture (from FFE shell-command)
  - [x] 9.4 Write requirements for `context-help` — F1 help, Help Panel, navigation, content (from FFE context-help)
  - [x] 9.5 Write requirements for `view-zoom` — Zoom level, shortcuts, persistence (from FFE view-zoom)
  - [x] 9.6 Write requirements for `line-wrap-toggle` — Word wrap modes (from FFE line-wrap-toggle)

- [x] 10. Extension and Macro Requirements (from FileForgeEditor + SciTE)
  - [x] 10.1 Write requirements for `lua-macro-engine` — Lua scripting, editor API, event hooks, OnChar/OnKey, per-buffer state, auto-reload (merges FFE mvp Req 7 + SciTE LuaExtension)
  - [x] 10.2 Write requirements for `command-completion` — Command-line auto-complete, popup positioning (NEW from Scintilla gap analysis)

- [x] 11. Display Mode Requirements (from FileForgeEditor)
  - [x] 11.1 Write requirements for `hex-display` — HEX mode, hex editing, hex search (from FFE hex-display)
  - [x] 11.2 Write requirements for `sequence-numbers` — Auto-detect, strip, UNNUM, NUMBER (from FFE sequence-numbers)
  - [x] 11.3 Write requirements for `tabs-and-mask` — TABS/MASK commands (from FFE tabs-and-mask)

- [x] 12. FileForge Domain Requirements (from FileForgeEditor)
  - [x] 12.1 Write requirements for `fileforge-integration` — Flat-file processing, EBCDIC, COMP-3, VB binary, ASA detection (from FFE fileforge-integration)
  - [x] 12.2 Write requirements for `structure-catalog` — Catalog management, grid browse/edit (from FFE structure-catalog)
  - [x] 12.3 Write requirements for `record-selection-criteria` — Criteria dialog, operators, filtering (from FFE record-selection-criteria)
  - [x] 12.4 Write requirements for `asa-report-preview` — ASA carriage control rendering (from FFE asa-report-preview)
  - [x] 12.5 Write requirements for `custom-file-viewers` — Viewer registry, PREVIEW command (from FFE custom-file-viewers)

- [x] 13. Dataset Catalog and Mainframe Emulation (from dataset-catalog-brief — depends on VFS)
  - [x] 13.1 Write requirements for `dataset-catalog` — Mainframe dataset catalog emulation on local desktop: SQLite catalog DB, dataset naming (HLQ.qualifier), sequential/PDS/PDSE/GDG types, repository layout, catalog mount/unmount/add/remove/export/import, dataset create/delete/rename/resolve/allocate, PDS member navigation, properties panel, context menus
  - [x] 13.2 Write requirements for `dataset-allocator` — Dataset allocation engine: DSN resolution against mounted catalogs, disposition handling, symbolic substitution (desktop equivalent of DYNALLOC/SVC 99)

- [x] 14. File Tree and Exploration (depends on VFS + dataset-catalog)
  - [x] 14.1 Write requirements for `file-tree-panel` — Panel layout, directory tree, async loading, multi-source browsing: Local Files node, Catalogs node (dataset tree), Connections node (future). Unified explorer rendering all VFS providers. (from FFE file-tree-panel + VFS + dataset-catalog)
  - [x] 14.2 Write requirements for `compare-and-merge` — COMPARE command, diff view, merge. VFS-aware resource comparison. (from FFE compare-and-merge)

- [x] 15. Background Processing and Performance (NEW from Scintilla gap analysis)
  - [x] 15.1 Write requirements for `idle-processing` — Background incremental work, syntax re-highlighting, wrap calculation
  - [x] 15.2 Write requirements for `large-file-performance` — Long-line handling, measurement caching, chunked rendering. Cross-references background-io for async large-file loading.

- [x] 16. Database Tool Requirements (from DBeaver — incorporated as full Database IDE)
  - [x] 16.1 Research and extract DBeaver core requirements — connection management, driver registry, multi-database support, credential storage, SSH tunnelling, connection pooling
  - [x] 16.2 Extract DBeaver SQL editor requirements — SQL editing, syntax highlighting per dialect, auto-complete, query execution, result set display, explain plan, parameter binding
  - [x] 16.3 Extract DBeaver data viewer requirements — grid/table view, cell editing, inline filtering, sorting, data export (CSV/JSON/SQL/XML), LOB handling, NULL display
  - [x] 16.4 Extract DBeaver schema browser requirements — object tree navigation, table/view/procedure/trigger inspection, DDL generation, dependency graph, search across objects
  - [x] 16.5 Extract DBeaver data transfer requirements — import/export wizards, bulk loading, cross-database transfer, column mapping, error handling, progress/cancellation
  - [x] 16.6 Extract DBeaver ER diagram requirements — visual schema diagrams, relationship display, auto-layout, export to image/PDF, notation styles
  - [x] 16.7 Extract DBeaver metadata and admin requirements — user/role management, session monitoring, lock inspection, storage/tablespace info, database statistics
  - [x] 16.8 Write requirements for `database-tool` — Adapt DBeaver capabilities as a FileForgeWorkbench integrated full Database IDE: connection panel, SQL editor panel, result grid panel, schema browser panel, data transfer workflows, ER diagram panel
    - Integrate with workbench command framework, plugin architecture, layout/docking, and VFS
    - Adapt to Rust/egui rendering and async I/O patterns
    - Support JDBC-equivalent Rust database drivers (sqlx, tokio-postgres, tiberius, rusqlite, etc.)
    - Leverage workbench workflow-engine for data transfer and import/export operations

- [x] 17. Deferred Connectivity (NOT in initial release — documented for future phases)
  - [x] 17.1 *(DEFERRED)* Write requirements for `connector-network-fs` — Network/UNC paths, SMB/CIFS, NFS, mapped drives
    - ⚠️ DEFERRED: Not in initial release. Connector-extensibility trait provides future hook.
  - [x] 17.2 *(DEFERRED)* Write requirements for `connector-ftp-sftp` — FTP, FTPS, SFTP connectors
    - ⚠️ DEFERRED: Not in initial release.
  - [x] 17.3 *(DEFERRED)* Write requirements for `connector-mainframe` — z/OS FTP, TN3270, z/OSMF, USS SSH
    - ⚠️ DEFERRED: Not in initial release. Dataset-catalog provides local emulation first.
  - [x] 17.4 *(DEFERRED)* Write requirements for `connector-cloud` — SharePoint, OneDrive, OAuth
    - ⚠️ DEFERRED: Not in initial release.

- [x] 18. Final validation
  - [x] 18.1 Cross-reference all requirements for consistency — no conflicts, no duplicates, all cross-cutting requirements honoured
  - [x] 18.2 Verify all FileForgeEditor requirements are represented (none lost in translation)
  - [x] 18.3 Verify gap analysis HIGH/MEDIUM items are all addressed
  - [x] 18.4 Verify DBeaver-derived database tool requirements are complete and integrated with workbench architecture
  - [x] 18.5 Verify VFS principle (FFW-ARCH-001) is honoured across all file-related specs
  - [x] 18.6 Produce final sub-project inventory with requirement counts and dependency graph

## Notes

- Each task produces a `requirements.md` file in `.kiro/specs/{sub-project}/`
- Design.md and tasks.md for each sub-project will be created in a LATER phase
- FileForgeEditor requirements always take priority where conflicts exist
- Architecture-incompatible Scintilla concepts are adapted to Rust/egui equivalents
- The workbench platform architecture brief provides overriding architectural principles
- All acceptance criteria use EARS format (WHEN/THEN/SHALL, THE system SHALL, etc.)
- Source references: FFE = FileForgeEditor spec, SCI = Scintilla extracted spec, WB = Workbench architecture brief, DBV = DBeaver feature analysis, DSC = Dataset catalog brief
- DBeaver requirements are extracted from public documentation and source analysis, then adapted as a workbench-integrated full Database IDE tool
- Database connectivity leverages the workbench `virtual-file-system` and `connector-extensibility` sub-projects
- **Initial release connectivity scope:** VFS abstraction + local filesystem provider + dataset catalog emulation (mainframe filesystem on local desktop)
- **Deferred connectivity:** FTP/SFTP, network FS, mainframe remote (z/OS), cloud — extensibility trait provides the hook for future phases
- FFW-ARCH-001: All content accessed through Virtual File System abstraction — this is the overriding connectivity principle
- DBeaver research tasks (16.1–16.7) have no code dependencies and can conceptually run any time; placed late because synthesis (16.8) needs platform architecture defined first

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2", "1.3"] },
    { "id": 1, "tasks": ["1.4"] },
    { "id": 2, "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6"] },
    { "id": 3, "tasks": ["3.1", "3.2", "3.3"] },
    { "id": 4, "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5"] },
    { "id": 5, "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5"] },
    { "id": 6, "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5"] },
    { "id": 7, "tasks": ["7.1", "7.2", "7.3"] },
    { "id": 8, "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6"] },
    { "id": 9, "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6"] },
    { "id": 10, "tasks": ["10.1", "10.2"] },
    { "id": 11, "tasks": ["11.1", "11.2", "11.3"] },
    { "id": 12, "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5"] },
    { "id": 13, "tasks": ["13.1", "13.2"] },
    { "id": 14, "tasks": ["14.1", "14.2"] },
    { "id": 15, "tasks": ["15.1", "15.2"] },
    { "id": 16, "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7"] },
    { "id": 17, "tasks": ["16.8"] },
    { "id": 18, "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5", "18.6"] }
  ]
}
```
