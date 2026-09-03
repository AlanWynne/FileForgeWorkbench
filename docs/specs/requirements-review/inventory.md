# Requirements Review — Task 1: Inventory & Baseline Audit

**Phase:** Requirements Review  
**Status:** COMPLETE  
**Date:** Phase BP+  
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This document is the baseline audit of every sub-project specification under
`docs/specs/`. It records:

- Whether a `requirements.md` file exists and is substantive
- The approximate requirement count and format used
- A quality flag for each spec
- Identified scope overlaps and missing specs

This inventory drives all subsequent tasks (terminology standardisation,
domain classification, gap analysis, and requirement rewrites).

---

## 2. Quality Flag Definitions

| Flag | Meaning |
|------|---------|
| **Compliant** | EARS format throughout; numbered criteria; clear user stories; source references; glossary; cross-references present. Minimal rework needed. |
| **Needs Improvement** | Mostly correct structure but has deficiencies in one or more of: EARS format consistency, atomicity, testability, terminology, or missing cross-references. Targeted edits required. |
| **Major Rewrite Required** | Free-text, bullet-list, or stub format; missing user stories; criteria not testable; no source references; or the file is empty / a placeholder. Full rewrite needed. |

---

## 3. Spec Inventory

### 3.1 Platform Architecture

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 1 | `platform-core` | ✅ | 9 requirements, ~50 criteria | EARS, numbered, user stories, glossary, source refs | **Compliant** | Exemplary quality. All criteria testable. Cross-references present. |
| 2 | `command-framework` | ✅ | 7 requirements, ~50 criteria | EARS, numbered, user stories, glossary, source refs | **Compliant** | Exemplary quality. Scripting bridge and history well-specified. |
| 3 | `plugin-architecture` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Good quality. Lifecycle and capability discovery well-covered. |
| 4 | `workflow-engine` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Good quality. State machine model well-defined. |
| 5 | `layout-and-docking` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | Good quality. Persona and serialisation coverage present. |
| 6 | `configuration-system` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Good quality. Layered model and hot-reload well-specified. |

### 3.2 Virtual File System

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 7 | `virtual-file-system` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | VFS provider trait, URI scheme, and capability model well-specified. |
| 8 | `connector-local-fs` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | File watching, path resolution, cross-platform handling covered. |
| 9 | `connector-extensibility` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Plugin trait and registration lifecycle present. |
| 10 | `connector-network-fs` | ✅ | Stub only | Free-text stub | **Major Rewrite Required** | Deferred connector — no acceptance criteria. Needs FR stubs with DEFERRED status. |
| 11 | `connector-ftp-sftp` | ✅ | Stub only | Free-text stub | **Major Rewrite Required** | Deferred connector — no acceptance criteria. Needs FR stubs with DEFERRED status. |
| 12 | `connector-mainframe` | ✅ | Stub only | Free-text stub | **Major Rewrite Required** | Deferred connector — no acceptance criteria. Needs FR stubs with DEFERRED status. |
| 13 | `connector-cloud` | ✅ | Stub only | Free-text stub | **Major Rewrite Required** | Deferred connector — no acceptance criteria. Needs FR stubs with DEFERRED status. |

### 3.3 Core Editor

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 14 | `document-model` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Gap buffer, line index, large-file streaming well-specified. |
| 15 | `edit-operations` | ✅ | 15 requirements, ~100 criteria | EARS, numbered, user stories, glossary, source refs | **Compliant** | Exemplary quality. Multi-caret, rectangular selection, BOUNDS all covered. |
| 16 | `undo-redo-transactions` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Transaction coalescing and save points present. |
| 17 | `viewport-and-scrolling` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Smooth scrolling and viewport model well-defined. |
| 18 | `display-line-mapping` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Folding, exclusion, and wrap mapping covered. |

### 3.4 Command Engine

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 19 | `command-semantics` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | ISPF command parser and pipeline well-specified. |
| 20 | `find-and-replace` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | FIND/RFIND/CHANGE/RCHANGE with modifiers covered. |
| 21 | `line-commands` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Block pairing and pending state present. |
| 22 | `exclude-show-filter` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | EXCLUDE/SHOW/RESET and display integration covered. |
| 23 | `navigation-commands` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | LOCATE, SORT, COLS, BOUNDS, word nav covered. |

### 3.5 UI and Rendering

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 24 | `menu-and-statusbar` | ✅ | ~20 requirements | EARS, numbered | **Compliant** | Status bar layout, focus cycle, title line all present. |
| 25 | `theme-and-appearance` | ✅ | ~14 requirements | EARS, numbered | **Compliant** | Token system, hot-reload, high-contrast covered. |
| 26 | `text-decorations` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Search highlighting, change markers, bookmarks present. |
| 27 | `whitespace-and-guides` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Indent guides, edge column, wrap markers covered. |
| 28 | `caret-and-selection` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Caret appearance, selection display, virtual space covered. |

### 3.6 Language and Highlighting

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 29 | `language-service` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Language detection, TOML definitions, content-based detection covered. |
| 30 | `syntax-highlighting` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Incremental re-highlight and sub-styles present. |
| 31 | `auto-indentation` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Block-start/end patterns and language-aware indent covered. |

### 3.7 File I/O and Session

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 32 | `file-operations` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | Open, Save, Save As, Revert, Recent Files covered. |
| 33 | `background-io` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Async loading, progress, cancellation present. |
| 34 | `encoding-and-characters` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | BOM detection, encoding detection, word classification covered. |
| 35 | `external-modification` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | File change detection and reload prompt present. |
| 36 | `startup-and-session` | ✅ | ~25 requirements | EARS, numbered | **Needs Improvement** | Very large spec grown organically through many phases. Requirements 14–25 are well-formed; earlier requirements (1–13) are mixed quality. Terminology inconsistent (uses "Screen", "Module", "Window"). Requirement numbering has gaps and out-of-order additions. Needs consolidation and terminology pass. |
| 37 | `multi-tab-editor` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Tab collection, per-tab state, MRU, context menu covered. |

### 3.8 Desktop Integration

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 38 | `clipboard-operations` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Copy/Cut/Paste, COPY command, file-insert covered. |
| 39 | `function-keys-and-history` | ✅ | ~20 requirements | EARS, numbered | **Needs Improvement** | Well-structured but uses legacy terms ("PF key", "key bar"). Modifier binding section (Req 20) is detailed but uses inconsistent criterion numbering. Needs terminology pass and criterion renumbering. |
| 40 | `shell-command` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | External command execution and output capture covered. |
| 41 | `context-help` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | F1 help, Help Panel, navigation covered. |
| 42 | `view-zoom` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Zoom level, shortcuts, persistence covered. |
| 43 | `line-wrap-toggle` | ✅ | ~5 requirements | EARS, numbered | **Compliant** | Word wrap modes covered. |

### 3.9 Extensions and Macros

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 44 | `lua-macro-engine` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | Lua scripting, editor API, event hooks, auto-reload covered. |
| 45 | `command-completion` | ✅ | ~5 requirements | EARS, numbered | **Compliant** | Command-line auto-complete and popup positioning covered. |

### 3.10 Display Modes

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 46 | `hex-display` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | HEX mode, hex editing, hex search covered. |
| 47 | `sequence-numbers` | ✅ | ~5 requirements | EARS, numbered | **Compliant** | Auto-detect, strip, UNNUM, NUMBER covered. |
| 48 | `tabs-and-mask` | ✅ | ~5 requirements | EARS, numbered | **Compliant** | TABS/MASK commands covered. |

### 3.11 FileForge Domain

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 49 | `fileforge-integration` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | Flat-file processing, EBCDIC, COMP-3, VB binary covered. |
| 50 | `structure-catalog` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Catalog management and grid browse/edit covered. |
| 51 | `record-selection-criteria` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Criteria dialog, operators, filtering covered. |
| 52 | `asa-report-preview` | ✅ | ~5 requirements | EARS, numbered | **Compliant** | ASA carriage control rendering covered. |
| 53 | `custom-file-viewers` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Viewer registry and PREVIEW command covered. |

### 3.12 Dataset Catalog and Mainframe Emulation

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 54 | `dataset-catalog` | ✅ | ~12 requirements | EARS, numbered | **Needs Improvement** | Good coverage of SQLite catalog, dataset naming, PDS/PDSE/GDG. However, terminology mixes "dataset" and "file" inconsistently. Some criteria lack testable measurements. Missing explicit NFRs for catalog query performance. |
| 55 | `dataset-allocator` | ✅ | ~8 requirements | EARS, numbered | **Needs Improvement** | DSN resolution and disposition handling covered. ISPF-style fields well-specified. Missing explicit error-path criteria for all disposition combinations. Terminology uses "DYNALLOC" without glossary entry. |
| 56 | `dataset-ownership-model` | ✅ | ~6 requirements | EARS, numbered | **Needs Improvement** | Governance model present but criteria are high-level. Missing explicit acceptance criteria for ownership transfer and conflict resolution edge cases. |
| 57 | `idcams-emulator` | ✅ | ~10 requirements | EARS, numbered | **Compliant** | DEFINE, DELETE, LISTCAT, REPRO commands well-specified. |
| 58 | `virtual-catalog-manager` | ✅ | 16 requirements, ~80 criteria | EARS, numbered, user stories | **Needs Improvement** | Good coverage of catalog CRUD, POSIX provider, dataset allocation. However: (1) Requirement 11 appears out of order after Requirement 16 (numbering gap). (2) Terminology uses "Windows catalog" in some places vs "Native catalog" in others — inconsistent. (3) No explicit NFRs for dialog response time or catalog load performance. (4) Requirement 16 acceptance criteria 16.1 references `files_panel.rs` — implementation detail that violates implementation neutrality. |

### 3.13 Job Entry Subsystem

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 59 | `FFW-JES` | ✅ | ~15 requirements | EARS, numbered | **Needs Improvement** | Good coverage of job submission, queue management, SDSF-style monitor. However: (1) Sub-project folder name `FFW-JES` does not follow the `kebab-case` naming convention used by all other sub-projects. (2) Terminology uses "JES2/JES3" without clarifying these are emulation targets, not dependencies. (3) Some criteria reference specific z/OS return codes without explaining their desktop-emulation equivalents. (4) Missing explicit NFRs for job throughput and queue depth limits. |

### 3.14 File Explorer

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 60 | `file-tree-panel` | ✅ | 23 requirements, ~150 criteria | EARS, numbered, user stories, glossary | **Needs Improvement** | The most comprehensive spec in the corpus. Requirements 1–14 are well-formed and Compliant. Requirements 15–23 were added incrementally via change requests and show inconsistencies: (1) Requirements 15, 19, 20, 21 use a different sub-heading style (bold criterion labels like **19.1 —**) vs the numbered list style used in Reqs 1–14. (2) Requirement 22 appears after Requirement 23 (out of order). (3) Glossary additions are scattered across individual requirements rather than consolidated in the main Glossary section. (4) Some criteria in Reqs 16–21 reference implementation files (`context_menu.rs`, `files_panel.rs`) — violates implementation neutrality. (5) Terminology uses "File Explorer Panel" and "File Tree Panel" interchangeably. |
| 61 | `compare-and-merge` | ✅ | ~8 requirements | EARS, numbered | **Compliant** | COMPARE command, diff view, merge covered. |

### 3.15 Performance

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 62 | `idle-processing` | ✅ | ~6 requirements | EARS, numbered | **Compliant** | Background incremental work and syntax re-highlighting covered. |
| 63 | `large-file-performance` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Long-line handling, measurement caching, chunked rendering covered. |

### 3.16 Database Tool

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 64 | `database-tool` | ✅ | ~20 requirements | EARS, numbered | **Needs Improvement** | Comprehensive DBeaver-derived coverage. However: (1) Requirements are very long and not atomic — several requirements contain 15+ criteria that should be split into child requirements. (2) Terminology uses "DBeaver" as a reference point in requirement text — should be removed from the spec (it is a source reference, not a product term). (3) Missing explicit NFRs for query execution time and result set size limits. (4) No traceability to the DBeaver research documents in the same folder. |

### 3.17 Compiler Toolchain

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 65 | `compiler-toolchain-integration` | ✅ | ~10 requirements | EARS, numbered | **Needs Improvement** | GCC and Rust toolchain detection and build covered. However: (1) Requirements mix toolchain-specific details (GCC flags, rustup commands) with platform-level requirements — implementation details should be moved to design.md. (2) Missing explicit NFRs for build invocation timeout and diagnostic parse performance. (3) No requirement for a generic `ToolchainPlugin` trait that future toolchains (LLVM, GnuCOBOL) can implement — the spec is GCC/Rust-specific rather than extensible. |

### 3.18 Foundation

| # | Sub-Project | File Exists | Req Count (approx) | Format | Quality Flag | Notes |
|---|-------------|-------------|-------------------|--------|--------------|-------|
| 66 | `logging-subsystem` | ✅ | ~7 requirements | EARS, numbered | **Compliant** | Structured logging, rotation, diagnostics, thread safety covered. |

### 3.19 Placeholder / Empty Specs

| # | Sub-Project | File Exists | Status | Notes |
|---|-------------|-------------|--------|-------|
| 67 | `jcl-resolver` | `.gitkeep` only | **No spec** | No requirements.md. JCL resolution is referenced in FFW-JES but has no standalone spec. Needs creation or merger into FFW-JES. |
| 68 | `workbench-requirements-merge` | Architecture docs only | **No requirements.md** | Contains architecture-brief.md, gap analysis, and verification docs. Not a feature spec — should be reclassified as `docs/architecture/`. |

---

## 4. Quality Summary

| Quality Flag | Count | Percentage |
|---|---|---|
| **Compliant** | 43 | 65% |
| **Needs Improvement** | 11 | 17% |
| **Major Rewrite Required** | 4 | 6% |
| **No spec / Placeholder** | 2 | 3% |
| **Deferred stubs** | 4 | 6% |
| **Not a feature spec** | 1 | 2% |
| **Total** | 65 | 100% |

---

## 5. Identified Scope Overlaps

The following pairs of sub-projects have overlapping scope that should be
addressed during the rewrite phase:

| Overlap | Sub-Projects | Description |
|---------|-------------|-------------|
| OV-01 | `file-tree-panel` + `virtual-catalog-manager` | Both specify catalog browsing, context menus, and content area behaviour. `virtual-catalog-manager` owns the catalog CRUD and Files Panel; `file-tree-panel` owns the tree rendering and navigation. The boundary is blurred in Requirements 15–23 of `file-tree-panel` which re-specify catalog-type-specific behaviour already in `virtual-catalog-manager`. |
| OV-02 | `file-tree-panel` + `connector-local-fs` | `file-tree-panel` Req 2 specifies bookmarked roots and local file browsing that partially duplicates `connector-local-fs` Req 3 (directory listing, file watching). The tree panel should reference the connector spec rather than re-specify its behaviour. |
| OV-03 | `startup-and-session` + `virtual-catalog-manager` | `startup-and-session` Req 19 specifies the File Explorer Panel (POM option 2) which overlaps with `file-tree-panel` and `virtual-catalog-manager`. The session spec should own only the routing and persistence; the panel specs should own the rendering. |
| OV-04 | `command-framework` + `command-semantics` | `command-framework` specifies the generic command registry and dispatch; `command-semantics` specifies the ISPF-specific command parser. The boundary is clear in design but the specs do not explicitly cross-reference each other's scope boundaries. |
| OV-05 | `dataset-catalog` + `virtual-catalog-manager` | `dataset-catalog` specifies the SQLite catalog DB and dataset CRUD; `virtual-catalog-manager` specifies the UI dialogs that invoke those operations. Some acceptance criteria in `virtual-catalog-manager` re-specify validation rules that belong in `dataset-catalog`. |
| OV-06 | `function-keys-and-history` + `command-framework` | Command history is specified in both `command-framework` Req 7 and `function-keys-and-history`. The authoritative location should be `command-framework`; `function-keys-and-history` should reference it. |
| OV-07 | `compiler-toolchain-integration` + `plugin-architecture` | The compiler toolchain is implemented as a plugin but `compiler-toolchain-integration` does not reference the `plugin-architecture` plugin contract. The toolchain spec should explicitly state it implements `ToolchainPlugin` (a sub-trait of `FileForgePlugin`). |

---

## 6. Missing Specs (Gap Candidates)

The following capabilities are referenced in existing specs or the architecture
brief but have no dedicated sub-project spec:

| Gap | Description | Referenced In |
|-----|-------------|---------------|
| G-01 | **Command Palette** | No spec for a VS Code-style command palette (Ctrl+Shift+P). Referenced in architecture brief as a future capability. | `architecture-brief.md` |
| G-02 | **Quick Open** | No spec for quick file open by name (Ctrl+P). Referenced in gap analysis docs. | `workbench-requirements-merge/verification-gap-analysis.md` |
| G-03 | **Workspace / Project Model** | No spec for multi-root workspaces, project files, or workspace-scoped settings. The architecture brief mentions "project overrides" in the config system but no spec owns the workspace concept. | `configuration-system`, `architecture-brief.md` |
| G-04 | **Favourites and Bookmarks** | No spec for user-defined favourites or bookmarks in the file explorer. `file-tree-panel` Req 2 mentions bookmarked roots but does not specify a full favourites system. | `file-tree-panel` |
| G-05 | **Recent Locations / Recent Files** | `file-operations` mentions Recent Files but there is no spec for a unified Recent Locations panel or MRU list across all resource types. | `file-operations`, `startup-and-session` |
| G-06 | **Audit Logging** | No spec for enterprise audit logging (who opened what file, when, from which catalog). Referenced as a future enterprise feature. | `architecture-brief.md` |
| G-07 | **Accessibility** | No dedicated spec for accessibility (screen reader support, WCAG compliance, keyboard-only operation). `file-tree-panel` Req 14 has accessibility criteria but they are not cross-cutting. | `file-tree-panel` Req 14 |
| G-08 | **JCL Resolver** | `jcl-resolver` folder exists with only a `.gitkeep`. JCL resolution is referenced in `FFW-JES` but has no spec. | `FFW-JES` |
| G-09 | **Notification / Toast System** | No spec for a non-modal notification system (toasts, banners). Currently all feedback goes to the status bar, which is a single-message channel. | `menu-and-statusbar` |
| G-10 | **Plugin Manager UI** | `plugin-architecture` specifies the plugin contract but there is no spec for a Plugin Manager panel (install, enable, disable, update plugins from a UI). | `plugin-architecture` |
| G-11 | **Session Recovery / Crash Recovery** | `startup-and-session` covers normal session restore but there is no spec for crash recovery (restoring unsaved changes after an abnormal exit). | `startup-and-session`, `platform-core` Req 7 |
| G-12 | **Global / Workspace Search** | `find-and-replace` covers in-file search. There is no spec for cross-file search (search across all open catalogs or a directory tree). | `find-and-replace`, `workbench-requirements-merge/verification-gap-analysis.md` |

---

## 7. Naming Convention Violations

The following sub-project folder names do not follow the `kebab-case` convention
used by all other sub-projects:

| Folder | Issue | Recommended Name |
|--------|-------|-----------------|
| `FFW-JES` | Uses uppercase and project prefix | `jes-emulator` |
| `workbench-requirements-merge` | Not a feature spec — is an architecture document collection | Move to `docs/architecture/` |

---

## 8. Structural Issues Across the Corpus

| Issue | Affected Specs | Recommendation |
|-------|---------------|----------------|
| **Implementation details in requirements** | `file-tree-panel` (Reqs 16–21), `virtual-catalog-manager` (Req 16.5), `compiler-toolchain-integration` | Remove references to specific source files (`context_menu.rs`, `files_panel.rs`). Move to design.md. |
| **Out-of-order requirement numbering** | `virtual-catalog-manager` (Req 11 after Req 16), `file-tree-panel` (Req 22 after Req 23) | Renumber requirements sequentially during rewrite. |
| **Scattered glossary additions** | `file-tree-panel` (glossary additions in Reqs 16, 17, 19, 20, 21) | Consolidate all glossary terms into the main Glossary section. |
| **Inconsistent criterion numbering style** | `file-tree-panel` (Reqs 1–14 use numbered lists; Reqs 15–23 use bold labels like **19.1 —**) | Standardise to numbered list format throughout. |
| **Missing NFRs** | `dataset-catalog`, `dataset-allocator`, `database-tool`, `compiler-toolchain-integration`, `FFW-JES` | Add NFR sections covering performance, scalability, and reliability for each spec. |
| **Source reference inconsistency** | Some specs use `[WB]`, `[FFE]`, `[DSC]`; others use `[FFE-TREE]`, `[ISPF-POM]`, `[DSC]` | Standardise source reference tags across all specs. |
| **"DBeaver" in requirement text** | `database-tool` | Remove product names from requirement text. Use "the database IDE tool" or "the integrated database tool". |

---

## 9. Next Steps

This inventory feeds directly into:

- **Task 2** — Terminology Standardisation (uses the legacy term scan from §3 notes)
- **Task 3** — Architectural Domain Classification (uses the spec list from §3)
- **Task 4** — Gap Analysis (uses §6 Missing Specs and §5 Overlaps)
- **Tasks 5–7** — Requirement Rewrites (prioritised by quality flag from §3)

Recommended rewrite priority order based on quality flags and architectural
importance:

1. Deferred connector stubs (4 specs) — quick wins, establish DEFERRED FR pattern
2. `startup-and-session` — high-traffic spec, terminology issues affect many downstream specs
3. `file-tree-panel` — largest spec, structural inconsistencies most visible to contributors
4. `virtual-catalog-manager` — out-of-order numbering and implementation detail leakage
5. `FFW-JES` — naming convention and terminology issues
6. `database-tool` — atomicity and DBeaver reference issues
7. `compiler-toolchain-integration` — extensibility gap
8. `dataset-catalog`, `dataset-allocator`, `dataset-ownership-model` — missing NFRs
9. `function-keys-and-history` — terminology and numbering pass
