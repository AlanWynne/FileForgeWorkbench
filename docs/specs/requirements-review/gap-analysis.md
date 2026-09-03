# Requirements Review — Task 4: Gap Analysis

**Phase:** Requirements Review  
**Status:** COMPLETE  
**Date:** Phase BQ  
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This document evaluates coverage of capabilities commonly expected in a modern
workbench product. For each gap category from the brief, every item is assessed
as one of:

| Status | Meaning |
|--------|---------|
| **COVERED** | A requirement exists and is adequately specified |
| **PARTIAL** | A requirement exists but is incomplete or lacks acceptance criteria |
| **MISSING** | No requirement exists — a new spec or criterion is needed |
| **DEFERRED** | Intentionally out of scope for initial release |

Gaps are prioritised **High / Medium / Low** based on user impact and
architectural importance.

---

## 2. Explorer Capabilities

### 2.1 Multi-Root Workspaces

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Multiple root directories open simultaneously | PARTIAL | `file-tree-panel` Req 2 | Bookmarked roots exist but there is no concept of a named "workspace" that groups roots, persists as a file, and can be shared. No `workspace.toml` spec. | High |
| Workspace file (open/save/close workspace) | MISSING | — | No spec for a workspace file format or workspace-level commands (Open Workspace, Save Workspace, Close Workspace). | High |
| Workspace-scoped settings | PARTIAL | `configuration-system` Req 5 | Config layering mentions "workspace" layer but no spec defines what constitutes a workspace or how workspace config is discovered. | High |
| Per-workspace recent files | MISSING | — | Recent files list is global; no per-workspace MRU. | Low |

### 2.2 Favourites and Bookmarks

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Bookmark a file or directory | PARTIAL | `file-tree-panel` Req 2.8–2.10 | Bookmarked roots exist for directories. No bookmarking of individual files. | Medium |
| Named favourites with custom labels | MISSING | — | No spec for user-defined named favourites distinct from bookmarked roots. | Medium |
| Favourites panel / section in Explorer | MISSING | — | No dedicated Favourites section in the Navigation Pane. | Medium |
| Persist favourites across sessions | PARTIAL | `file-tree-panel` Req 2.10 | Bookmarked roots persist; individual file favourites do not exist. | Medium |

### 2.3 Recent Locations

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Recent files list | COVERED | `startup-and-session` Req 4, `file-operations` | Recent_Files_List specified with configurable depth. | — |
| Recent directories / locations | MISSING | — | No spec for recently visited directories in the Explorer. | Medium |
| Recent locations in path bar | MISSING | — | `file-tree-panel` Req 11 specifies a path bar but no history dropdown for recent locations. | Low |
| Clear recent files command | MISSING | — | No requirement for a "Clear Recent Files" command. | Low |

### 2.4 Breadcrumb Navigation

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Breadcrumb path bar in Explorer | COVERED | `file-tree-panel` Req 11, `virtual-catalog-manager` Req 10.5 | Path bar and breadcrumb specified. | — |
| Clickable breadcrumb segments | COVERED | `virtual-catalog-manager` Req 10.5 | Each segment clickable for navigation. | — |
| Breadcrumb in Content Editor title | MISSING | — | No requirement for a breadcrumb showing the open file's path above the editor. | Low |

### 2.5 Filters

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Filter tree by name | COVERED | `file-tree-panel` Req 9 | Tree_Search_Box with live filter and glob support. | — |
| Filter by file type / extension | MISSING | — | No requirement for filtering the Explorer by file extension or type category. | Medium |
| Filter by date modified | MISSING | — | No requirement for date-based filtering in the Explorer. | Low |
| Show/hide hidden files toggle | COVERED | `file-tree-panel` Req 4.7, Req 13 | `file_tree.show_hidden_files` config key specified. | — |

### 2.6 Saved Searches

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Save a search query for reuse | MISSING | — | No spec for saved searches. `find-and-replace` covers in-file search only. | Medium |
| Named saved search with scope | MISSING | — | No spec for cross-file saved searches with a named scope (directory, catalog, workspace). | Medium |

### 2.7 Virtual Folders

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Virtual folder grouping files by criteria | MISSING | — | No spec for virtual folders (smart folders that group files by type, date, tag, etc.). | Low |
| Tag-based virtual folders | MISSING | — | No tagging system for files. | Low |

---

## 3. Productivity Capabilities

### 3.1 Command Palette

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Command palette (Ctrl+Shift+P) | MISSING | — | No spec. Referenced in architecture brief as a future capability. All commands are accessible via the Command Field but there is no fuzzy-search palette. | High |
| Fuzzy search over all registered commands | MISSING | — | No spec for fuzzy matching over the command registry. | High |
| Recent commands in palette | MISSING | — | No spec for showing recently used commands at the top of the palette. | Medium |
| Keyboard shortcut hints in palette | MISSING | — | No spec for displaying bound shortcuts alongside commands in the palette. | Medium |

### 3.2 Quick Open

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Quick open file by name (Ctrl+P) | MISSING | — | No spec. Referenced in gap analysis docs. The Command Field accepts `EDIT <path>` but there is no fuzzy-search quick-open dialog. | High |
| Fuzzy file name matching | MISSING | — | No spec for fuzzy matching over all files in open catalogs/workspaces. | High |
| Quick open symbol in file | MISSING | — | No spec for Ctrl+Shift+O style symbol navigation within a file. | Low |

### 3.3 Workspace Search (Cross-File)

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Search across all open catalogs | MISSING | — | `find-and-replace` covers in-file search only. No cross-file search spec exists. | High |
| Search with include/exclude patterns | MISSING | — | No spec for glob-based include/exclude in cross-file search. | High |
| Replace across files | MISSING | — | No spec for cross-file replace. | High |
| Search results panel | MISSING | — | No spec for a dedicated search results panel showing matches across files. | High |

### 3.4 Session Recovery

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Crash recovery from undo state | COVERED | `startup-and-session` Req 10 | Recovery_Files and crash recovery offer specified. | — |
| Auto-save interval | PARTIAL | `startup-and-session` Req 4.3 | Periodic session save specified (default 5 min) but no per-document auto-save to a temp file. | Medium |
| Recovery file per document | PARTIAL | `undo-redo-transactions` | Recovery_File contract referenced but the write interval and format are not fully specified in `startup-and-session`. | Medium |

---

## 4. Power User Features

### 4.1 Multi-Selection

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Multi-select files in Explorer | COVERED | `file-tree-panel` Req 19 (drag-select), Req 20 (keyboard) | Drag-select, Shift+click, Ctrl+click all specified. | — |
| Multi-select in content area | COVERED | `virtual-catalog-manager` Req 10 | Content area sort and navigation specified. | — |
| Multi-caret editing | COVERED | `edit-operations` Req 8 | Full multi-caret model specified. | — |
| Rectangular selection | COVERED | `edit-operations` Req 9 | Column selection fully specified. | — |

### 4.2 Bulk Operations

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Bulk copy/move files | COVERED | `file-tree-panel` Req 21 | File copy/paste with progress and collision handling specified. | — |
| Bulk rename with pattern | MISSING | — | No spec for batch rename with a pattern (e.g. add prefix, replace substring across selected files). | Medium |
| Bulk delete with confirmation | COVERED | `file-tree-panel` Req 6.9 | Delete confirmation dialog specified. | — |
| Bulk encoding conversion | MISSING | — | No spec for converting multiple files to a different encoding in one operation. | Low |

### 4.3 Keyboard-First Workflows

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Full keyboard navigation in Explorer | COVERED | `file-tree-panel` Req 8, Req 20 | Arrow keys, Tab, Enter, F2, Delete all specified. | — |
| Tab cycle through all panels | COVERED | `menu-and-statusbar` | Focus cycle specified. | — |
| Keyboard shortcut for every command | PARTIAL | `command-framework` Req 5 | Shortcut registry specified but not all commands have default bindings defined. | Medium |
| Vim/Emacs key binding profiles | MISSING | — | No spec for alternative key binding profiles. | Low |

### 4.4 Automation

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Lua macro recording and playback | PARTIAL | `lua-macro-engine` | Lua scripting specified but no macro recorder (record keystrokes → generate Lua). | Medium |
| Scheduled macro execution | MISSING | — | No spec for time-based or event-based macro scheduling. | Low |
| Macro library management | MISSING | — | No spec for a macro library panel (list, edit, run, delete macros). | Medium |

### 4.5 Custom Commands

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| User-defined commands via Lua | COVERED | `lua-macro-engine` Req 2 | Lua scripts can register commands via the scripting bridge. | — |
| User-defined commands via TOML | MISSING | — | No spec for defining simple command aliases in TOML without writing Lua. | Low |
| Command aliases | MISSING | — | No spec for aliasing one command name to another. | Low |

---

## 5. Enterprise Features

### 5.1 Audit Logging

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Audit trail of file open/save/delete | MISSING | — | No spec. Referenced in architecture brief. The logging subsystem provides diagnostic logs but not a structured audit trail. | High |
| Audit log format (structured, queryable) | MISSING | — | No spec for a structured audit log distinct from the diagnostic log. | High |
| Audit log retention policy | MISSING | — | No spec for audit log rotation, retention period, or archival. | Medium |
| Audit log export | MISSING | — | No spec for exporting audit logs to CSV, JSON, or syslog. | Medium |

### 5.2 Security Controls

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Read-only mode for catalogs | COVERED | `virtual-catalog-manager` Req 3.5, Req 7.6 | Read-only flag on POSIX and Native catalogs specified. | — |
| File-level read-only enforcement | COVERED | `startup-and-session` Req 14.30 | Tab-level read-only mode specified. | — |
| Credential storage for connectors | MISSING | — | No spec for secure credential storage (keychain, vault) for remote connectors. | High |
| TLS/certificate validation for connectors | MISSING | — | No spec for TLS configuration in deferred connectors. | Medium |

### 5.3 Role-Based Permissions

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| User roles (admin, editor, viewer) | MISSING | — | No spec for role-based access control. | Medium |
| Permission enforcement on catalog operations | MISSING | — | No spec for restricting catalog create/delete to admin role. | Medium |
| Role assignment UI | MISSING | — | No spec for a role management panel. | Low |

### 5.4 Policy Enforcement

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Enforce naming conventions on datasets | PARTIAL | `dataset-catalog` | Naming rules specified for DSN validation but no policy engine for custom rules. | Medium |
| Prevent save to restricted paths | MISSING | — | No spec for path-based save restrictions. | Medium |
| Mandatory file header templates | MISSING | — | No spec for enforcing file header templates on new file creation. | Low |

### 5.5 Configuration Deployment

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Deploy settings to multiple users | MISSING | — | No spec for enterprise-wide settings deployment (system-layer config). | Medium |
| Lock settings from user override | MISSING | — | No spec for marking a config key as locked (admin-only). | Medium |
| Settings import/export | MISSING | — | No spec for exporting/importing the full settings profile. | Low |

---

## 6. Extensibility

### 6.1 Plugin Architecture

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Plugin trait and lifecycle | COVERED | `plugin-architecture` | Full plugin contract specified. | — |
| Plugin capability discovery | COVERED | `plugin-architecture` Req 4 | Capability advertisement specified. | — |
| Plugin hot-reload | COVERED | `platform-core` Req 8 | Hot-restart of individual plugins specified. | — |
| Plugin Manager UI | MISSING | — | No spec for a Plugin Manager panel (install, enable, disable, update). | High |
| Plugin marketplace / registry | MISSING | — | No spec for discovering and installing plugins from a remote registry. | Low |
| Plugin sandboxing / permissions | MISSING | — | No spec for restricting what a plugin can access (filesystem, network, commands). | Medium |

### 6.2 Scripting

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Lua scripting engine | COVERED | `lua-macro-engine` | Full Lua engine with editor API and event hooks specified. | — |
| Lua REPL / interactive console | MISSING | — | No spec for an interactive Lua console panel. | Low |
| Script debugging | MISSING | — | No spec for debugging Lua scripts (breakpoints, step, inspect). | Low |
| Additional scripting languages | MISSING | — | No spec for Python or JavaScript scripting beyond Lua. | Low |

### 6.3 Extensions

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Custom file viewers | COVERED | `custom-file-viewers` | Viewer registry and PREVIEW command specified. | — |
| Custom syntax definitions | COVERED | `language-service` | TOML language definitions specified. | — |
| Custom themes | COVERED | `theme-and-appearance` | User TOML themes and hot-reload specified. | — |
| Custom key maps | COVERED | `function-keys-and-history` Req 20 | Key Configuration Dialog specified. | — |

### 6.4 External Tool Integration

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| GCC toolchain integration | COVERED | `compiler-toolchain-integration` | GCC detection, build, diagnostics specified. | — |
| Rust toolchain integration | COVERED | `compiler-toolchain-integration` | Rust/cargo detection, build, diagnostics specified. | — |
| Generic toolchain plugin trait | PARTIAL | `compiler-toolchain-integration` | `ToolchainPlugin` trait exists but the spec does not define it as a general extension point for future toolchains (LLVM, GnuCOBOL, OpenJDK). | High |
| Git integration | PARTIAL | `file-tree-panel` Req 16.15 | Git submenu is present but greyed-out (deferred). No Git spec exists. | Medium |
| Build output panel | COVERED | `compiler-toolchain-integration` | Build output and clickable diagnostics specified. | — |

---

## 7. User Experience

### 7.1 Responsive Layouts

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Resizable panels | COVERED | `layout-and-docking` | Resizable panels with drag handles specified. | — |
| Collapsible panels | COVERED | `file-tree-panel` Req 1.5 | Collapse to icon strip specified. | — |
| Responsive to window resize | PARTIAL | `layout-and-docking` | Layout serialisation specified but no explicit requirement for minimum window size or responsive reflow. | Low |

### 7.2 Layout Persistence

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Save and restore panel layout | COVERED | `layout-and-docking`, `startup-and-session` Req 4 | Layout_State serialisation and restore specified. | — |
| Named layout presets (personas) | COVERED | `layout-and-docking` | Personas (programmer, analyst, mainframe) specified. | — |
| Export/import layout | MISSING | — | No spec for exporting a layout preset to share with other users. | Low |

### 7.3 Customisable Panels

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Dock/undock any panel | COVERED | `layout-and-docking` | Dockable panels with floating window support specified. | — |
| Show/hide panels | COVERED | `layout-and-docking` | Panel show/hide specified. | — |
| Custom panel order in tab bar | PARTIAL | `multi-tab-editor` | Tab reordering specified but no drag-to-reorder in the tab bar. | Low |

### 7.4 Dark/Light Themes

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Built-in dark theme | COVERED | `theme-and-appearance` | Dark theme specified. | — |
| Built-in light theme | COVERED | `theme-and-appearance` | Light theme specified. | — |
| High-contrast theme | COVERED | `theme-and-appearance` | High-contrast theme specified. | — |
| User-defined custom themes | COVERED | `theme-and-appearance` | TOML user themes with hot-reload specified. | — |
| System theme follow (OS dark/light) | MISSING | — | No spec for automatically following the OS dark/light mode preference. | Medium |

### 7.5 Accessibility

| Item | Status | Location | Gap Description | Priority |
|------|--------|----------|-----------------|----------|
| Screen reader support | MISSING | — | No cross-cutting accessibility spec. `file-tree-panel` Req 14 has tree-specific ARIA semantics but there is no workbench-wide accessibility requirement. | High |
| WCAG AA compliance | MISSING | — | No spec for WCAG AA colour contrast requirements across all panels. | High |
| Keyboard-only operation | PARTIAL | Multiple specs | Keyboard navigation specified per-panel but no cross-cutting requirement that every action is reachable without a mouse. | High |
| Focus indicators | PARTIAL | `menu-and-statusbar`, `file-tree-panel` | Focus rings specified in some panels but not mandated globally. | Medium |
| Font size scaling | PARTIAL | `view-zoom` | Zoom level specified but no requirement for system font size scaling (OS accessibility settings). | Medium |
| Reduced motion support | MISSING | — | No spec for disabling animations for users with vestibular disorders. | Low |

---

## 8. Summary of Gaps

### 8.1 Missing Capabilities by Priority

| Priority | Count | Key Items |
|----------|-------|-----------|
| **High** | 18 | Command palette, quick open, cross-file search, workspace model, audit logging, accessibility spec, WCAG compliance, Plugin Manager UI, credential storage, generic toolchain trait |
| **Medium** | 22 | Favourites, recent locations, saved searches, bulk rename, macro library, role-based permissions, policy enforcement, config deployment, OS theme follow, focus indicators |
| **Low** | 16 | Virtual folders, command aliases, Lua REPL, layout export, reduced motion, per-workspace recent files, clear recent files |

### 8.2 New Sub-Project Specs Required

The following new sub-project specs are required to address High-priority gaps.
These were also identified in Task 3 §7.

| New Sub-Project | Layer | Addresses Gaps |
|----------------|-------|----------------|
| `accessibility` | UX Layer | Screen reader, WCAG AA, keyboard-only, focus indicators |
| `command-palette` | UX Layer | Command palette, fuzzy search, recent commands |
| `workspace-model` | Core Platform | Multi-root workspaces, workspace file, workspace settings |
| `global-search` | Task Layer | Cross-file search, replace across files, search results panel |
| `notification-system` | Workbench Shell | Non-modal notifications (currently all feedback is status bar) |
| `plugin-manager-ui` | Workbench Shell | Plugin Manager panel |
| `audit-logging` | Core Platform | Structured audit trail, retention, export |

### 8.3 Existing Specs Requiring New Criteria

The following existing specs need new acceptance criteria added (not new specs):

| Spec | Missing Criteria |
|------|-----------------|
| `file-tree-panel` | Filter by file type/extension; recent locations in path bar |
| `startup-and-session` | Per-document auto-save interval; clear recent files command |
| `theme-and-appearance` | OS dark/light mode follow |
| `compiler-toolchain-integration` | Generic ToolchainPlugin trait for future toolchains |
| `function-keys-and-history` | Keyboard shortcut default bindings for all commands |
| `file-operations` | Bulk encoding conversion |
| `lua-macro-engine` | Macro library management panel |
| `configuration-system` | Settings export/import; locked (admin-only) keys |

---

## 9. Deferred Gaps (By Design)

The following gaps are intentionally out of scope for the initial release and
are already documented as deferred in the architecture brief:

| Gap | Deferred To |
|----|------------|
| FTP/SFTP remote file access | `connector-ftp-sftp` (future phase) |
| z/OS mainframe remote connectivity | `connector-mainframe` (future phase) |
| Cloud storage (SharePoint, OneDrive) | `connector-cloud` (future phase) |
| Network filesystem (SMB/NFS) | `connector-network-fs` (future phase) |
| Plugin marketplace / registry | Future phase after Plugin Manager UI |
| Git integration (full) | Future phase — greyed-out placeholder exists |
| Script debugging (Lua) | Future phase |
| Role-based permissions | Future phase — enterprise edition scope |
| Vim/Emacs key binding profiles | Future phase |

---

## 10. Next Steps

This gap analysis feeds directly into:

- **Tasks 5–7** — Requirement Rewrites (add missing criteria to existing specs;
  create stub specs for new sub-projects)
- **Task 9** — Consolidation Report (overlaps between existing specs that
  partially cover the same gap)
- **Task 10** — Strategic Recommendations (High-priority gaps drive the
  priority roadmap)
