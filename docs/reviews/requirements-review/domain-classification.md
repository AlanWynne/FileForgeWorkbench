# Requirements Review — Task 3: Architectural Domain Classification

**Phase:** Requirements Review  
**Status:** COMPLETE  
**Date:** Phase BQ  
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This document maps every sub-project specification to one of the six
architectural layers defined in the product model, identifies sub-projects
that span multiple layers (split candidates), and produces the recommended
Capability → Feature hierarchy aligned to the architecture.

---

## 2. Architectural Layers

| Layer | Description | Colour Code |
|-------|-------------|-------------|
| **Core Platform** | GUI-independent foundation: runtime, commands, plugins, workflows, layout, configuration, VFS | 🔵 |
| **Workbench Shell** | The egui desktop shell: tab bar, menus, status bar, session, startup, ISPF POM | 🟣 |
| **Explorer Layer** | Resource navigation: file explorer, dataset explorer, catalog manager, compare | 🟢 |
| **Content Layer** | Editing and viewing: text editor, hex editor, structured viewer, compare viewer | 🟡 |
| **Task Layer** | Background operations: search, replace, build, sync, transfer, analysis | 🟠 |
| **Integration Layer** | External connectivity: VFS connectors, mainframe, cloud, Git, database | 🔴 |
| **UX Layer** | Cross-cutting UX: themes, accessibility, keyboard nav, command palette, context menus | ⚪ |

---

## 3. Sub-Project to Domain Mapping

### 3.1 Core Platform 🔵

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `platform-core` | Core Platform | — | Pure platform foundation |
| `command-framework` | Core Platform | — | Command registry and dispatch |
| `plugin-architecture` | Core Platform | — | Plugin contract and lifecycle |
| `workflow-engine` | Core Platform | — | State machine workflows |
| `layout-and-docking` | Core Platform | Workbench Shell | Layout model is core; rendering is shell |
| `configuration-system` | Core Platform | — | TOML config, hot-reload, profiles |
| `logging-subsystem` | Core Platform | — | Foundation layer |
| `virtual-file-system` | Core Platform | Integration Layer | VFS abstraction is core; providers are integration |
| `connector-extensibility` | Core Platform | Integration Layer | Plugin trait for connectors |

### 3.2 Workbench Shell 🟣

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `startup-and-session` | Workbench Shell | Core Platform | Session orchestration is shell; persistence model is core |
| `multi-tab-editor` | Workbench Shell | Content Layer | Tab container is shell; per-tab state is content |
| `menu-and-statusbar` | Workbench Shell | — | Menu bar, status bar, command field, title line |
| `function-keys-and-history` | Workbench Shell | Core Platform | Key label bar is shell; key map resolver is core |
| `shell-command` | Workbench Shell | Task Layer | Terminal launch is shell; output capture is task |

### 3.3 Explorer Layer 🟢

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `file-tree-panel` | Explorer Layer | Workbench Shell | Tree rendering is explorer; panel docking is shell |
| `virtual-catalog-manager` | Explorer Layer | Integration Layer | Catalog UI is explorer; VFS provider is integration |
| `compare-and-merge` | Explorer Layer | Content Layer | Compare trigger is explorer; diff view is content |
| `dataset-catalog` | Explorer Layer | Integration Layer | Catalog model is explorer; SQLite storage is integration |
| `dataset-allocator` | Explorer Layer | Integration Layer | Allocation UI is explorer; DSN resolution is integration |
| `dataset-ownership-model` | Explorer Layer | Core Platform | Governance model spans both |
| `idcams-emulator` | Explorer Layer | Integration Layer | IDCAMS commands are explorer-level; execution is integration |
| `structure-catalog` | Explorer Layer | Content Layer | Structure definitions are explorer; grid view is content |

### 3.4 Content Layer 🟡

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `document-model` | Content Layer | Core Platform | Document buffer is content; VFS access is core |
| `edit-operations` | Content Layer | — | Pure content layer |
| `undo-redo-transactions` | Content Layer | Core Platform | Transaction stack is content; command integration is core |
| `viewport-and-scrolling` | Content Layer | — | Pure content layer |
| `display-line-mapping` | Content Layer | — | Pure content layer |
| `caret-and-selection` | Content Layer | — | Pure content layer |
| `text-decorations` | Content Layer | UX Layer | Decorations are content; colour tokens are UX |
| `whitespace-and-guides` | Content Layer | UX Layer | Guide rendering is content; colour tokens are UX |
| `hex-display` | Content Layer | — | Hex view mode |
| `sequence-numbers` | Content Layer | — | Sequence number display mode |
| `tabs-and-mask` | Content Layer | — | TABS/MASK display mode |
| `asa-report-preview` | Content Layer | — | ASA carriage control rendering |
| `custom-file-viewers` | Content Layer | — | Viewer registry and PREVIEW command |
| `fileforge-integration` | Content Layer | Integration Layer | Flat-file processing is content; EBCDIC/COMP-3 is integration |
| `record-selection-criteria` | Content Layer | Task Layer | Criteria dialog is content; filtering execution is task |
| `line-wrap-toggle` | Content Layer | — | Word wrap mode |
| `view-zoom` | Content Layer | UX Layer | Zoom level is content; persistence is UX |

### 3.5 Task Layer 🟠

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `find-and-replace` | Task Layer | Content Layer | Search execution is task; result highlighting is content |
| `command-semantics` | Task Layer | Workbench Shell | ISPF command parser is task; dispatch is shell |
| `line-commands` | Task Layer | Content Layer | Line command execution is task; prefix area rendering is content |
| `exclude-show-filter` | Task Layer | Content Layer | Filter execution is task; visibility rendering is content |
| `navigation-commands` | Task Layer | Content Layer | Navigation execution is task; viewport update is content |
| `background-io` | Task Layer | Core Platform | Async I/O workers are task; Tokio runtime is core |
| `file-operations` | Task Layer | Core Platform | File open/save pipeline is task; VFS access is core |
| `external-modification` | Task Layer | Core Platform | File watching is task; VFS watcher is core |
| `idle-processing` | Task Layer | Content Layer | Background incremental work is task; syntax update is content |
| `large-file-performance` | Task Layer | Content Layer | Chunked rendering is task; document model is content |
| `compiler-toolchain-integration` | Task Layer | Integration Layer | Build invocation is task; toolchain detection is integration |
| `FFW-JES` | Task Layer | Integration Layer | Job submission is task; JES provider is integration |

### 3.6 Integration Layer 🔴

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `connector-local-fs` | Integration Layer | Core Platform | Local FS provider implements VFS trait |
| `connector-network-fs` | Integration Layer | — | Deferred — network FS connector |
| `connector-ftp-sftp` | Integration Layer | — | Deferred — FTP/SFTP connector |
| `connector-mainframe` | Integration Layer | — | Deferred — z/OS connector |
| `connector-cloud` | Integration Layer | — | Deferred — cloud storage connector |
| `database-tool` | Integration Layer | Content Layer | DB connection is integration; SQL editor and result grid are content |
| `encoding-and-characters` | Integration Layer | Content Layer | Encoding detection is integration; character classification is content |

### 3.7 UX Layer ⚪

| Sub-Project | Primary Layer | Spans | Notes |
|-------------|--------------|-------|-------|
| `theme-and-appearance` | UX Layer | Core Platform | Theme data is UX; token system is core |
| `context-help` | UX Layer | Workbench Shell | Help panel is UX; topic routing is shell |
| `clipboard-operations` | UX Layer | Content Layer | Clipboard access is UX; paste semantics are content |
| `auto-indentation` | UX Layer | Content Layer | Indent behaviour is UX; language detection is content |
| `syntax-highlighting` | UX Layer | Content Layer | Highlight rendering is UX; language engine is content |
| `language-service` | UX Layer | Content Layer | Language detection is UX; TOML definitions are content |
| `lua-macro-engine` | UX Layer | Core Platform | Macro execution is UX; scripting bridge is core |
| `command-completion` | UX Layer | Workbench Shell | Completion popup is UX; command registry is core |

---

## 4. Multi-Layer Sub-Projects (Split Candidates)

The following sub-projects span two or more layers significantly enough that
their requirements should be explicitly partitioned during the rewrite phase.
This does not require splitting the crate — only that the requirements document
clearly labels which layer each requirement belongs to.

| Sub-Project | Layers | Recommended Action |
|-------------|--------|--------------------|
| `startup-and-session` | Workbench Shell + Core Platform | Partition requirements: Reqs 1–11 (session model) → Core Platform section; Reqs 13–19 (POM, tab container, shell interactions) → Workbench Shell section |
| `file-tree-panel` | Explorer Layer + Workbench Shell | Partition: Reqs 1–14 (tree panel, VFS browsing) → Explorer Layer; Reqs 15–23 (catalog-specific UI, context menus) → Explorer Layer sub-section with Integration Layer cross-refs |
| `virtual-catalog-manager` | Explorer Layer + Integration Layer | Partition: Reqs 1–11 (catalog UI, dialogs) → Explorer Layer; Reqs 7, 12–16 (VFS provider, path resolution) → Integration Layer |
| `database-tool` | Integration Layer + Content Layer | Partition: connection management, schema browser → Integration Layer; SQL editor, result grid, ER diagram → Content Layer |
| `layout-and-docking` | Core Platform + Workbench Shell | Partition: layout model, serialisation → Core Platform; panel rendering, drag-drop → Workbench Shell |
| `function-keys-and-history` | Workbench Shell + Core Platform | Partition: key map resolver, history store → Core Platform; key label bar rendering → Workbench Shell |
| `compiler-toolchain-integration` | Task Layer + Integration Layer | Partition: build invocation, diagnostic parsing → Task Layer; toolchain detection, install → Integration Layer |

---

## 5. Recommended Capability → Feature Hierarchy

This is the canonical product hierarchy. Each Capability maps to an
architectural layer. Each Feature maps to one or more sub-project specs.

### Capability 1: Core Platform 🔵

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Platform Core | `platform-core` | FR-0001–FR-0019 |
| Command Framework | `command-framework` | FR-0020–FR-0039 |
| Plugin Architecture | `plugin-architecture` | FR-0040–FR-0059 |
| Workflow Engine | `workflow-engine` | FR-0060–FR-0079 |
| Layout Manager | `layout-and-docking` (core portion) | FR-0080–FR-0099 |
| Settings System | `configuration-system` | FR-0100–FR-0119 |
| Logging | `logging-subsystem` | FR-0120–FR-0129 |
| Virtual File System | `virtual-file-system` | FR-0130–FR-0149 |
| Connector Extensibility | `connector-extensibility` | FR-0150–FR-0159 |

### Capability 2: Workbench Shell 🟣

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Startup and Session | `startup-and-session` | FR-0200–FR-0229 |
| Workbench Home View | `startup-and-session` (POM section) | FR-0230–FR-0259 |
| Tab Container | `multi-tab-editor` | FR-0260–FR-0279 |
| Menu and Status Bar | `menu-and-statusbar` | FR-0280–FR-0299 |
| Function Keys | `function-keys-and-history` | FR-0300–FR-0329 |
| Shell Command | `shell-command` | FR-0330–FR-0339 |
| Context Help | `context-help` | FR-0340–FR-0359 |
| Command Completion | `command-completion` | FR-0360–FR-0369 |

### Capability 3: Explorer Layer 🟢

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| File Explorer | `file-tree-panel` | FR-0400–FR-0449 |
| Catalog Explorer | `virtual-catalog-manager` | FR-0450–FR-0499 |
| Dataset Catalog | `dataset-catalog` | FR-0500–FR-0519 |
| Dataset Allocator | `dataset-allocator` | FR-0520–FR-0539 |
| Dataset Ownership | `dataset-ownership-model` | FR-0540–FR-0549 |
| IDCAMS Emulator | `idcams-emulator` | FR-0550–FR-0569 |
| Structure Catalog | `structure-catalog` | FR-0570–FR-0579 |
| Compare and Merge | `compare-and-merge` | FR-0580–FR-0599 |

### Capability 4: Content Editor 🟡

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Document Model | `document-model` | FR-0600–FR-0619 |
| Edit Operations | `edit-operations` | FR-0620–FR-0649 |
| Undo and Redo | `undo-redo-transactions` | FR-0650–FR-0659 |
| Viewport and Scrolling | `viewport-and-scrolling` | FR-0660–FR-0669 |
| Display Line Mapping | `display-line-mapping` | FR-0670–FR-0679 |
| Caret and Selection | `caret-and-selection` | FR-0680–FR-0689 |
| Hex Display | `hex-display` | FR-0690–FR-0699 |
| Sequence Numbers | `sequence-numbers` | FR-0700–FR-0709 |
| Tabs and Mask | `tabs-and-mask` | FR-0710–FR-0719 |
| ASA Report Preview | `asa-report-preview` | FR-0720–FR-0729 |
| Custom Viewers | `custom-file-viewers` | FR-0730–FR-0749 |
| FileForge Integration | `fileforge-integration` | FR-0750–FR-0769 |
| Record Selection | `record-selection-criteria` | FR-0770–FR-0779 |
| Line Wrap | `line-wrap-toggle` | FR-0780–FR-0784 |
| View Zoom | `view-zoom` | FR-0785–FR-0789 |

### Capability 5: Task Layer 🟠

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Find and Replace | `find-and-replace` | FR-0800–FR-0829 |
| ISPF Command Engine | `command-semantics` | FR-0830–FR-0849 |
| Line Commands | `line-commands` | FR-0850–FR-0869 |
| Exclude and Filter | `exclude-show-filter` | FR-0870–FR-0879 |
| Navigation Commands | `navigation-commands` | FR-0880–FR-0899 |
| Background I/O | `background-io` | FR-0900–FR-0919 |
| File Operations | `file-operations` | FR-0920–FR-0939 |
| External Modification | `external-modification` | FR-0940–FR-0949 |
| Idle Processing | `idle-processing` | FR-0950–FR-0959 |
| Large File Performance | `large-file-performance` | FR-0960–FR-0969 |
| Compiler Toolchain | `compiler-toolchain-integration` | FR-0970–FR-0999 |
| JES Emulator | `FFW-JES` | FR-0970–FR-0999 |

### Capability 6: Integration Layer 🔴

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Local FS Connector | `connector-local-fs` | FR-1000–FR-1019 |
| Network FS Connector | `connector-network-fs` | FR-1020–FR-1029 |
| FTP/SFTP Connector | `connector-ftp-sftp` | FR-1030–FR-1049 |
| Mainframe Connector | `connector-mainframe` | FR-1050–FR-1079 |
| Cloud Connector | `connector-cloud` | FR-1080–FR-1099 |
| Database Tool | `database-tool` | FR-1100–FR-1149 |
| Encoding | `encoding-and-characters` | FR-1150–FR-1169 |

### Capability 7: UX Layer ⚪

| Feature | Sub-Project(s) | FR Range |
|---------|---------------|----------|
| Theme and Appearance | `theme-and-appearance` | FR-1200–FR-1219 |
| Syntax Highlighting | `syntax-highlighting` | FR-1220–FR-1229 |
| Language Service | `language-service` | FR-1230–FR-1239 |
| Auto-Indentation | `auto-indentation` | FR-1240–FR-1249 |
| Text Decorations | `text-decorations` | FR-1250–FR-1259 |
| Whitespace and Guides | `whitespace-and-guides` | FR-1260–FR-1269 |
| Clipboard Operations | `clipboard-operations` | FR-1270–FR-1279 |
| Lua Macro Engine | `lua-macro-engine` | FR-1280–FR-1299 |
| Accessibility | *(new spec needed)* | FR-1300–FR-1319 |
| Command Palette | *(new spec needed)* | FR-1320–FR-1329 |

---

## 6. Sub-Projects Requiring Reclassification

The following sub-projects are currently filed under a folder name that does
not reflect their architectural domain. These should be renamed or moved during
the documentation restructure (no code changes required).

| Current Name | Recommended Name | Reason |
|-------------|-----------------|--------|
| `FFW-JES` | `jes-emulator` | Naming convention violation; content is a Task Layer emulator |
| `workbench-requirements-merge` | Move to `docs/architecture/` | Not a feature spec — contains architecture briefs and verification reports |
| `jcl-resolver` | Merge into `jes-emulator` or create `jcl-resolver` spec | Currently empty; JCL resolution is a sub-feature of the JES emulator |

---

## 7. New Sub-Projects Recommended

Based on the gap analysis (Task 4 will detail these), the following new
sub-project specs are recommended to cover identified gaps:

| Recommended Sub-Project | Layer | Rationale |
|------------------------|-------|-----------|
| `accessibility` | UX Layer | No cross-cutting accessibility spec exists; `file-tree-panel` Req 14 is the only accessibility content |
| `command-palette` | UX Layer | VS Code-style Ctrl+Shift+P command palette — referenced in architecture brief but unspecified |
| `workspace-model` | Core Platform | Multi-root workspaces, project files, workspace-scoped settings — referenced but unspecified |
| `notification-system` | Workbench Shell | Non-modal toast/banner notifications — currently all feedback is single-line status bar |
| `plugin-manager-ui` | Workbench Shell | Plugin Manager panel (install, enable, disable, update) — plugin contract exists but no UI spec |
| `global-search` | Task Layer | Cross-file search across all open catalogs — `find-and-replace` covers in-file only |
| `audit-logging` | Core Platform | Enterprise audit trail — referenced in architecture brief but unspecified |

---

## 8. Next Steps

This classification feeds directly into:

- **Task 4** — Gap Analysis (uses §7 new sub-projects and the layer map to identify missing coverage)
- **Tasks 5–7** — Requirement Rewrites (use the FR range allocations from §5 to assign IDs)
- **Task 9** — Consolidation Report (uses the multi-layer split candidates from §4)
