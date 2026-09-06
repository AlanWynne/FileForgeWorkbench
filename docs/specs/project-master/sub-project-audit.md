# Sub-Project Audit Report

**Date:** Phase CO pre-gate
**Purpose:** Full cross-reference of spec folders, crates, and project-master inventory.
**Baseline:** 657 passing tests, 0 failures (Phase BT complete).

---

## 1. Audit Method

Three sources compared:

| Source | Count |
|--------|-------|
| `docs/specs/` sub-project folders | 74 folders |
| `crates/` workspace crates | 67 crates |
| `project-master/requirements.md` inventory | 62 entries (original) |

---

## 2. Spec Folders vs. project-master Inventory

### 2.1 Folders present in docs/specs/ but MISSING from requirements.md inventory

| Folder | Crate(s) | Phase Added | Status |
|--------|----------|-------------|--------|
| `automated-dialog-testing` | `ff-fftest` | Phase CK | COMPLETE |
| `bootstrap-scripts` | (scripts only) | Phase CJ | COMPLETE |
| `command-palette` | `ff-desktop` (inline) | Phase BS-B | COMPLETE |
| `compiler-toolchain-integration` | `ff-toolchain-api`, `ff-gcc-toolchain`, `ff-rust-toolchain` | Phase W | COMPLETE |
| `dataset-ownership-model` | `ff-governance-tests` | Phase M | COMPLETE |
| `ears-integration` | (workflow docs only) | Phase EI | COMPLETE -- docs only |
| `global-search` | `ff-global-search` | Phase BS-C / BT | COMPLETE |
| `idcams-emulator` | `ff-idcams` | Phase M | COMPLETE |
| `jes-emulator` | `ff-jes` | Phase N (renamed BR) | COMPLETE -- was FFW-JES |
| `jcl-resolver` | (stub, .gitkeep only) | -- | STUB -- no requirements yet |
| `virtual-catalog-manager` | `ff-desktop` (inline) | Phase AA | COMPLETE |
| `workbench-requirements-merge` | (architecture docs) | Phase A | DOCS ONLY |
| `workspace-model` | `ff-session` (workspace.rs) | Phase BS-A | COMPLETE |

### 2.2 Entries in requirements.md inventory that need updating

| Entry | Issue |
|-------|-------|
| `FFW-JES` (#62) | Renamed to `jes-emulator` in Phase BR |
| `dataset-allocator` (#51) | Crate is `ff-dsalloc`; spec folder is `dataset-allocator` -- OK |
| `dataset-catalog` (#50) | Crate is `ff-dscatalog`; spec folder is `dataset-catalog` -- OK |

---

## 3. Crates vs. Spec Folders

### 3.1 Crates with no dedicated spec folder

| Crate | Notes |
|-------|-------|
| `ff-vsam-services` | No spec folder exists. VSAM services are covered by `dataset-catalog` Req 21-23 (KSDS/RRDS/ESDS). No separate spec needed -- document as sub-module of dataset-catalog. |
| `ff-governance-tests` | Covered by `dataset-ownership-model` spec. |
| `ff-global-search` | Covered by `global-search` spec. |
| `ff-fftest` | Covered by `automated-dialog-testing` spec. |
| `ff-gcc-toolchain`, `ff-rust-toolchain`, `ff-toolchain-api` | All covered by `compiler-toolchain-integration` spec. |
| `egui-file-dialog` | Vendored third-party crate. No spec required. |

### 3.2 Spec folders with no crate (by design)

| Folder | Reason |
|--------|--------|
| `bootstrap-scripts` | Scripts only -- no Rust crate. |
| `ears-integration` | Workflow/planning docs only. |
| `workbench-requirements-merge` | Architecture docs only. |
| `jcl-resolver` | Stub -- no implementation yet. |
| `virtual-catalog-manager` | Implemented inline in `ff-desktop`. |
| `command-palette` | Implemented inline in `ff-desktop`. |
| `workspace-model` | Implemented as module in `ff-session`. |
| `connector-cloud`, `connector-ftp-sftp`, `connector-mainframe`, `connector-network-fs` | Deferred -- spec stubs only. |

---

## 4. Complete Sub-Project Status Table

| # | Spec Folder | Crate(s) | Status | Phase |
|---|-------------|----------|--------|-------|
| 1 | `platform-core` | `ff-core` | COMPLETE | Phase B |
| 2 | `command-framework` | `ff-command` | COMPLETE | Phase B |
| 3 | `plugin-architecture` | `ff-plugin` | COMPLETE | Phase B |
| 4 | `workflow-engine` | `ff-workflow` | COMPLETE | Phase B |
| 5 | `layout-and-docking` | `ff-layout` | COMPLETE | Phase B |
| 6 | `configuration-system` | `ff-config` | COMPLETE | Phase B |
| 7 | `virtual-file-system` | `ff-vfs` | COMPLETE | Phase C |
| 8 | `connector-local-fs` | `ff-connector-local-fs` | COMPLETE | Phase C |
| 9 | `connector-extensibility` | `ff-connector-extensibility` | COMPLETE | Phase C |
| 10 | `document-model` | `ff-document-model` | COMPLETE | Phase D |
| 11 | `edit-operations` | `ff-edit-operations` | COMPLETE | Phase D |
| 12 | `undo-redo-transactions` | `ff-undo-redo` | COMPLETE | Phase D |
| 13 | `viewport-and-scrolling` | `ff-viewport-scrolling` | COMPLETE | Phase D |
| 14 | `display-line-mapping` | `ff-display-line-mapping` | COMPLETE | Phase D |
| 15 | `command-semantics` | `ff-command-semantics` | COMPLETE | Phase E |
| 16 | `find-and-replace` | `ff-find-and-replace` | COMPLETE | Phase E |
| 17 | `line-commands` | `ff-line-commands` | COMPLETE | Phase E |
| 18 | `exclude-show-filter` | `ff-exclude-show-filter` | COMPLETE | Phase E |
| 19 | `navigation-commands` | `ff-navigation-commands` | COMPLETE | Phase E |
| 20 | `menu-and-statusbar` | `ff-menu` | COMPLETE | Phase F |
| 21 | `theme-and-appearance` | `ff-theme` | COMPLETE | Phase F |
| 22 | `text-decorations` | `ff-text-decorations` | COMPLETE | Phase F |
| 23 | `whitespace-and-guides` | `ff-whitespace-guides` | COMPLETE | Phase F |
| 24 | `caret-and-selection` | `ff-caret-selection` | COMPLETE | Phase F |
| 25 | `language-service` | `ff-language-service` | COMPLETE | Phase G |
| 26 | `syntax-highlighting` | `ff-syntax-highlighting` | COMPLETE | Phase G |
| 27 | `auto-indentation` | `ff-auto-indent` | COMPLETE | Phase G |
| 28 | `file-operations` | `ff-file-ops` | COMPLETE | Phase H |
| 29 | `background-io` | `ff-background-io` | COMPLETE | Phase H |
| 30 | `encoding-and-characters` | `ff-encoding` | COMPLETE | Phase H |
| 31 | `external-modification` | `ff-external-mod` | COMPLETE | Phase H |
| 32 | `startup-and-session` | `ff-session` | COMPLETE | Phase H |
| 33 | `multi-tab-editor` | `ff-tabs` | COMPLETE | Phase H |
| 34 | `clipboard-operations` | `ff-clipboard` | COMPLETE | Phase I |
| 35 | `function-keys-and-history` | `ff-keys` | COMPLETE | Phase I |
| 36 | `shell-command` | `ff-shell` | COMPLETE | Phase I |
| 37 | `context-help` | `ff-help` | COMPLETE | Phase I |
| 38 | `view-zoom` | `ff-zoom` | COMPLETE | Phase I |
| 39 | `line-wrap-toggle` | `ff-wrap` | COMPLETE | Phase I |
| 40 | `lua-macro-engine` | `ff-lua` | COMPLETE | Phase J |
| 41 | `command-completion` | `ff-completion` | COMPLETE | Phase J |
| 42 | `hex-display` | `ff-hex` | COMPLETE | Phase K |
| 43 | `sequence-numbers` | `ff-seqnum` | COMPLETE | Phase K |
| 44 | `tabs-and-mask` | `ff-tabmask` | COMPLETE | Phase K |
| 45 | `fileforge-integration` | `ff-forge` | COMPLETE | Phase L |
| 46 | `structure-catalog` | `ff-structure-catalog` | COMPLETE | Phase L |
| 47 | `record-selection-criteria` | `ff-select` | COMPLETE | Phase L |
| 48 | `asa-report-preview` | `ff-asa` | COMPLETE | Phase L |
| 49 | `custom-file-viewers` | `ff-viewers` | COMPLETE | Phase L |
| 50 | `dataset-catalog` | `ff-dscatalog` | COMPLETE | Phase M |
| 51 | `dataset-allocator` | `ff-dsalloc` | COMPLETE | Phase M |
| 52 | `idcams-emulator` | `ff-idcams` | COMPLETE | Phase M |
| 53 | `dataset-ownership-model` | `ff-governance-tests` | COMPLETE | Phase M |
| 54 | `jes-emulator` | `ff-jes` | COMPLETE | Phase N (renamed BR) |
| 55 | `file-tree-panel` | `ff-file-tree` | COMPLETE | Phase O |
| 56 | `compare-and-merge` | `ff-compare-merge` | COMPLETE | Phase O |
| 57 | `idle-processing` | `ff-idle-processing` | COMPLETE | Phase P |
| 58 | `large-file-performance` | `ff-large-file-performance` | COMPLETE | Phase P |
| 59 | `database-tool` | `ff-database-tool` | COMPLETE | Phase Q |
| 60 | `compiler-toolchain-integration` | `ff-toolchain-api`, `ff-gcc-toolchain`, `ff-rust-toolchain` | COMPLETE | Phase W |
| 61 | `virtual-catalog-manager` | `ff-desktop` (inline) | COMPLETE | Phase AA |
| 62 | `logging-subsystem` | `ff-logging` | COMPLETE | Phase A |
| 63 | `automated-dialog-testing` | `ff-fftest` | COMPLETE | Phase CK |
| 64 | `bootstrap-scripts` | (scripts) | COMPLETE | Phase CJ |
| 65 | `command-palette` | `ff-desktop` (inline) | COMPLETE | Phase BS-B |
| 66 | `global-search` | `ff-global-search` | COMPLETE | Phase BS-C / BT |
| 67 | `workspace-model` | `ff-session` (workspace.rs) | COMPLETE | Phase BS-A |
| 68 | `connector-cloud` | (stub) | DEFERRED | -- |
| 69 | `connector-ftp-sftp` | (stub) | DEFERRED | -- |
| 70 | `connector-mainframe` | (stub) | DEFERRED | -- |
| 71 | `connector-network-fs` | (stub) | DEFERRED | -- |
| 72 | `ears-integration` | (docs only) | COMPLETE -- docs | Phase EI |
| 73 | `workbench-requirements-merge` | (docs only) | COMPLETE -- docs | Phase A |
| 74 | `jcl-resolver` | (stub) | PENDING -- no requirements | -- |

---

## 5. Crate Count Summary

| Category | Count |
|----------|-------|
| Library crates (complete with tests) | 62 |
| Binary crate (`ff-desktop`) | 1 |
| Vendored third-party (`egui-file-dialog`) | 1 |
| Test-only crate (`ff-governance-tests`) | 1 |
| VSAM services sub-module (`ff-vsam-services`) | 1 |
| Toolchain crates (`ff-toolchain-api`, `ff-gcc-toolchain`, `ff-rust-toolchain`) | 3 |
| **Total workspace crates** | **67** |

---

## 6. Gaps and Actions Required

### 6.1 project-master/requirements.md -- actions

| Action | Detail |
|--------|--------|
| Add 13 missing sub-projects to inventory | See section 2.1 above |
| Rename FFW-JES to jes-emulator | Entry #62 |
| Update total count from 62 to 74 | Reflects all spec folders |
| Note ff-vsam-services as dataset-catalog sub-module | No separate spec needed |

### 6.2 project-master/tasks.md -- actions

| Action | Detail |
|--------|--------|
| Update final summary table | Test count is 657 (not 655); Phase BT complete |
| Add Phase CO as PENDING | Next phase: accessibility, plugin-manager-ui, notification-system |
| Correct stale "Active work" line | Currently says "Phase BT complete" -- correct |

### 6.3 .amazonq/rules/specs.md -- actions

| Action | Detail |
|--------|--------|
| Add `accessibility` | New sub-project (Phase CO) |
| Add `plugin-manager-ui` | New sub-project (Phase CO) |
| Add `notification-system` | New sub-project (Phase CO) |
| Verify `jcl-resolver` is listed | Stub sub-project |

---

## 7. No-Action Items

The following discrepancies are intentional and require no change:

- `ff-vsam-services` -- VSAM storage is a sub-module of `ff-dscatalog`; covered by dataset-catalog Req 21-23. No separate spec needed.
- `ears-integration` -- planning/workflow docs only; not a deliverable sub-project.
- `workbench-requirements-merge` -- architecture docs only.
- `jcl-resolver` -- stub folder with `.gitkeep`; no requirements written yet. Acceptable as placeholder.
- Deferred connectors (`connector-cloud`, `connector-ftp-sftp`, `connector-mainframe`, `connector-network-fs`) -- spec stubs exist; implementation deferred by design.
