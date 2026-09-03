# Requirements Review — Task 8: Traceability Matrix

**Phase:** Requirements Review
**Status:** COMPLETE
**Date:** Phase BQ
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This matrix is the single authoritative cross-reference linking every
sub-project specification to:

- Its architectural layer and FR number range
- The implementing crate(s)
- The quality flag from the baseline audit (Task 1)
- The rewrite status from Tasks 5–7
- The current test coverage status from `docs/TCR.md`
- Any outstanding actions

It is the primary navigation aid for contributors who need to locate the
spec, the code, and the tests for any given capability.

---

## 2. How to Read This Matrix

| Column | Source | Meaning |
|--------|--------|---------|
| Sub-Project | `docs/specs/<name>/` | Spec folder name |
| Layer | Task 3 domain-classification | Architectural layer |
| FR Range | Task 3 §5 | Allocated functional requirement number space |
| Crate(s) | project-master/tasks.md | Rust crate(s) that implement the spec |
| Quality Flag | Task 1 inventory | Baseline quality at start of review |
| Rewrite Status | Tasks 5–7 rewrite-delta | Current state after rewrite phase |
| TCR Status | docs/TCR.md | Automated test coverage summary |
| Actions | — | Outstanding work items |

### Quality Flag Key

| Flag | Meaning |
|------|---------|
| **Compliant** | EARS format, numbered criteria, glossary, source refs — minimal rework needed |
| **Needs Improvement** | Structural or terminology deficiencies — targeted edits required |
| **Major Rewrite** | Free-text or stub format — full rewrite needed |
| **Deferred Stub** | Intentionally out of scope — EARS stubs added with DEFERRED status |
| **No Spec** | No requirements.md exists |

### Rewrite Status Key

| Status | Meaning |
|--------|---------|
| **Compliant — No Change** | Spec was already Compliant; no edits made |
| **Style Normalised** | Dot-prefix criteria converted to numbered list; numbers preserved |
| **Terminology Pass** | Legacy terms replaced per terminology-map.md |
| **Major Rewrite** | Spec substantially rewritten (deferred connectors, NFR additions) |
| **Renumbered** | Requirements renumbered (e.g. compiler-toolchain Reqs 15–18 → 1–4) |
| **Deferred** | Rewrite deferred — spec intentionally high-level |
| **Not a Feature Spec** | Architecture documents, not a requirements spec |

### TCR Status Key

| Status | Meaning |
|--------|---------|
| ✅ All Pass | All criteria have passing automated tests |
| 🔲 Manual | Some or all criteria require manual UI verification |
| 🔴 Gaps | One or more criteria have no test coverage |
| ⚠️ Pending | Test annotation updates required (UPDATE-ANNOTATION) |

---

## 3. Core Platform Layer 🔵

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `platform-core` | FR-0001–FR-0019 | `ff-core` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `command-framework` | FR-0020–FR-0039 | `ff-command` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `plugin-architecture` | FR-0040–FR-0059 | `ff-plugin` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `workflow-engine` | FR-0060–FR-0079 | `ff-workflow` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `layout-and-docking` | FR-0080–FR-0099 | `ff-layout` | Compliant | Terminology Pass (Req 11: "floating window" → "Detached View") | 🔲 Manual (Req 11 — UI rendering) | VERIFY-STILL-VALID: Req 11.1–11.5 |
| `configuration-system` | FR-0100–FR-0119 | `ff-config` | Compliant | Terminology Pass (Req 15.10: "PF3" → "F3") | ✅ All Pass | VERIFY-STILL-VALID: Req 15.10 |
| `logging-subsystem` | FR-0120–FR-0129 | `ff-logging` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `virtual-file-system` | FR-0130–FR-0149 | `ff-vfs` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `connector-extensibility` | FR-0150–FR-0159 | `ff-connector-ext` | Compliant | Compliant — No Change | ✅ All Pass | None |

---

## 4. Workbench Shell Layer 🟣

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `startup-and-session` | FR-0200–FR-0259 | `ff-session`, `ff-desktop` | Needs Improvement | Style Normalised + Terminology Pass (Reqs 13, 14, 19; "PF3" → "F3" in Req 19.10) | 🔲 Manual (Req 19.5–19.9 UI tree) | VERIFY-STILL-VALID: Req 19.10 |
| `multi-tab-editor` | FR-0260–FR-0279 | `ff-tabs`, `ff-desktop` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `menu-and-statusbar` | FR-0280–FR-0299 | `ff-menu`, `ff-desktop` | Compliant | Style Normalised + Terminology Pass (Reqs 13, 16, 17, 18; "Command ===> field" → "Command Field") | 🔲 Manual (Req 17.1–17.9 UI chrome) | None — numbers preserved |
| `function-keys-and-history` | FR-0300–FR-0329 | `ff-keys`, `ff-desktop` | Needs Improvement | Style Normalised + Terminology Pass ("PF Key" → "Function Key", "Key Bar" → "Key Label Bar", "window context" → "Workspace Context") | ✅ All Pass | VERIFY-STILL-VALID: Req 14 ("window context" → "Workspace Context") |
| `shell-command` | FR-0330–FR-0339 | `ff-shell` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `context-help` | FR-0340–FR-0359 | `ff-help` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `command-completion` | FR-0360–FR-0369 | `ff-completion` | Compliant | Compliant — No Change | ✅ All Pass | None |

---

## 5. Explorer Layer 🟢

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `file-tree-panel` | FR-0400–FR-0449 | `ff-tree`, `ff-desktop` | Needs Improvement | Style Normalised + Impl Refs Removed (Reqs 15–23 bold-label → numbered list; `context_menu.rs` refs removed) | 🔴 Gaps (Req 19.1, 19.9 drag-select; Req 20.3, 20.5, 20.8 keyboard nav) | VERIFY-STILL-VALID: Reqs 16–17 (impl refs removed) |
| `virtual-catalog-manager` | FR-0450–FR-0499 | `ff-desktop` | Needs Improvement | Terminology Pass ("Windows catalog" → "Native catalog"; Req 16.5 impl ref removed) | 🔲 Manual (Req 6, 8, 9 context menus) | VERIFY-STILL-VALID: "Windows catalog" → "Native catalog"; Req 16.5 |
| `dataset-catalog` | FR-0500–FR-0519 | `ff-dscatalog` | Needs Improvement | NFR Section Added | ✅ All Pass | NEW-TEST-NEEDED: NFR criteria |
| `dataset-allocator` | FR-0520–FR-0539 | `ff-dsalloc` | Needs Improvement | Compliant — No Change | ✅ All Pass | None |
| `dataset-ownership-model` | FR-0540–FR-0549 | `ff-desktop` | Needs Improvement | Deferred | ✅ All Pass | None |
| `idcams-emulator` | FR-0550–FR-0569 | `ff-idcams` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `structure-catalog` | FR-0570–FR-0579 | `ff-struct` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `compare-and-merge` | FR-0580–FR-0599 | `ff-compare` | Compliant | Compliant — No Change | ✅ All Pass | None |

---

## 6. Content Layer 🟡

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `document-model` | FR-0600–FR-0619 | `ff-document-model` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `edit-operations` | FR-0620–FR-0649 | `ff-edit-operations` | Compliant | Style Normalised (Reqs 1–15 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `undo-redo-transactions` | FR-0650–FR-0659 | `ff-undo-redo` | Compliant | Style Normalised (Reqs 1–18 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `viewport-and-scrolling` | FR-0660–FR-0669 | `ff-viewport-scrolling` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `display-line-mapping` | FR-0670–FR-0679 | `ff-display-line-mapping` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `caret-and-selection` | FR-0680–FR-0689 | `ff-caret-selection` | Compliant | Style Normalised (Reqs 1–12 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `hex-display` | FR-0690–FR-0699 | `ff-hex` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `sequence-numbers` | FR-0700–FR-0709 | `ff-seqnum` | Compliant | Style Normalised (Reqs 1–14 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `tabs-and-mask` | FR-0710–FR-0719 | `ff-tabmask` | Compliant | Style Normalised (Reqs 1–18 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `asa-report-preview` | FR-0720–FR-0729 | `ff-asa` | Compliant | Style Normalised (Reqs 1–12 dot-prefix → numbered list) | ✅ All Pass | None — numbers preserved |
| `custom-file-viewers` | FR-0730–FR-0749 | `ff-viewers` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `fileforge-integration` | FR-0750–FR-0769 | `ff-forge` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `record-selection-criteria` | FR-0770–FR-0779 | `ff-select` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `line-wrap-toggle` | FR-0780–FR-0784 | `ff-wrap` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `view-zoom` | FR-0785–FR-0789 | `ff-zoom`, `ff-desktop` | Compliant | Compliant — No Change | 🔲 Manual (Req 2, 7 keyboard/status bar) | None |

---

## 7. Task Layer 🟠

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `find-and-replace` | FR-0800–FR-0829 | `ff-find-and-replace` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `command-semantics` | FR-0830–FR-0849 | `ff-cmd-semantics` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `line-commands` | FR-0850–FR-0869 | `ff-line-commands` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `exclude-show-filter` | FR-0870–FR-0879 | `ff-filter` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `navigation-commands` | FR-0880–FR-0899 | `ff-nav` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `background-io` | FR-0900–FR-0919 | `ff-bgio` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `file-operations` | FR-0920–FR-0939 | `ff-fileops` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `external-modification` | FR-0940–FR-0949 | `ff-extmod` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `idle-processing` | FR-0950–FR-0959 | `ff-idle` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `large-file-performance` | FR-0960–FR-0969 | `ff-largefile` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `compiler-toolchain-integration` | FR-0970–FR-0999 | `ff-toolchain-api`, `ff-gcc-toolchain`, `ff-rust-toolchain`, `ff-desktop` | Needs Improvement | Renumbered (Reqs 15–18 → Reqs 1–4) + NFR Section Added | ⚠️ Pending + 🔲 Manual | UPDATE-ANNOTATION: tests referencing Req 15.x–18.x → Req 1.x–4.x; NEW-TEST-NEEDED: NFR criteria |
| `FFW-JES` | FR-0970–FR-0999 | `ff-jes` | Needs Improvement | Style Normalised + Terminology Pass (Reqs 1–15; JES2/JES3 clarified as emulation targets) | ✅ All Pass | None — numbers preserved |

---

## 8. Integration Layer 🔴

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `connector-local-fs` | FR-1000–FR-1019 | `ff-connector-local-fs` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `connector-network-fs` | FR-1020–FR-1029 | *(deferred)* | Major Rewrite | Deferred Stub (6 EARS criteria added, DEFERRED status) | 🔴 Gaps (all criteria deferred) | NEW-TEST-NEEDED when implemented |
| `connector-ftp-sftp` | FR-1030–FR-1049 | *(deferred)* | Major Rewrite | Deferred Stub (6 EARS criteria added, DEFERRED status) | 🔴 Gaps (all criteria deferred) | NEW-TEST-NEEDED when implemented |
| `connector-mainframe` | FR-1050–FR-1079 | *(deferred)* | Major Rewrite | Deferred Stub (6 EARS criteria added, DEFERRED status) | 🔴 Gaps (all criteria deferred) | NEW-TEST-NEEDED when implemented |
| `connector-cloud` | FR-1080–FR-1099 | *(deferred)* | Major Rewrite | Deferred Stub (6 EARS criteria added, DEFERRED status) | 🔴 Gaps (all criteria deferred) | NEW-TEST-NEEDED when implemented |
| `database-tool` | FR-1100–FR-1149 | `ff-dbtool` | Needs Improvement | Terminology Pass ("DBeaver" removed from req text) + NFR Section Added | ✅ All Pass | NEW-TEST-NEEDED: NFR criteria |
| `encoding-and-characters` | FR-1150–FR-1169 | `ff-encoding` | Compliant | Compliant — No Change | ✅ All Pass | None |

---

## 9. UX Layer ⚪

| Sub-Project | FR Range | Crate(s) | Quality Flag | Rewrite Status | TCR Status | Actions |
|-------------|----------|----------|--------------|----------------|------------|---------|
| `theme-and-appearance` | FR-1200–FR-1219 | `ff-theme`, `ff-desktop` | Compliant | Renumbered (Req 12 → Req 15, moved to end) | ✅ All Pass | None — Req 12 had no test references |
| `syntax-highlighting` | FR-1220–FR-1229 | `ff-syntax` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `language-service` | FR-1230–FR-1239 | `ff-lang` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `auto-indentation` | FR-1240–FR-1249 | `ff-auto-indent` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `text-decorations` | FR-1250–FR-1259 | `ff-decorations` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `whitespace-and-guides` | FR-1260–FR-1269 | `ff-whitespace` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `clipboard-operations` | FR-1270–FR-1279 | `ff-clipboard` | Compliant | Compliant — No Change | ✅ All Pass | None |
| `lua-macro-engine` | FR-1280–FR-1299 | `ff-lua` | Compliant | Compliant — No Change | ✅ All Pass | None |

---

## 10. Placeholder / Empty / Non-Feature Specs

| Sub-Project | Status | Notes | Recommended Action |
|-------------|--------|-------|--------------------|
| `jcl-resolver` | No Spec (`.gitkeep` only) | JCL resolution referenced in `FFW-JES` but no standalone spec | Merge into `FFW-JES` or create `jcl-resolver` spec when JCL resolution is implemented |
| `workbench-requirements-merge` | Not a Feature Spec | Contains architecture briefs and verification reports | Move to `docs/architecture/` — not a requirements spec |

---

## 11. New Sub-Projects Recommended (Gap Analysis)

These sub-projects were identified in Task 3 §7 and Task 4 §8.2 as required
to address High-priority gaps. No specs exist yet.

| Recommended Sub-Project | Layer | FR Range (Reserved) | Gap Addressed | Priority |
|------------------------|-------|---------------------|---------------|----------|
| `accessibility` | UX Layer | FR-1300–FR-1319 | Screen reader, WCAG AA, keyboard-only operation, focus indicators | High |
| `command-palette` | UX Layer | FR-1320–FR-1329 | VS Code-style Ctrl+Shift+P command palette, fuzzy search | High |
| `workspace-model` | Core Platform | FR-0160–FR-0179 | Multi-root workspaces, workspace file, workspace-scoped settings | High |
| `global-search` | Task Layer | FR-0970 (sub-range TBD) | Cross-file search, replace across files, search results panel | High |
| `notification-system` | Workbench Shell | FR-0370–FR-0379 | Non-modal toast/banner notifications | Medium |
| `plugin-manager-ui` | Workbench Shell | FR-0380–FR-0389 | Plugin Manager panel (install, enable, disable, update) | High |
| `audit-logging` | Core Platform | FR-0180–FR-0199 | Structured audit trail, retention policy, export | High |

---

## 12. Outstanding Actions Summary

### 12.1 UPDATE-ANNOTATION Required

Test annotations in the following files reference old requirement IDs that
were renumbered during the rewrite phase. These must be updated before the
next test run to avoid misleading traceability.

| Spec | Old ID | New ID | Affected Test Files |
|------|--------|--------|---------------------|
| `compiler-toolchain-integration` | Req 15.x | Req 1.x | `ff-toolchain-api` lib tests, `ff-gcc-toolchain` lib tests, `ff-desktop/toolchain_panel.rs` tests |
| `compiler-toolchain-integration` | Req 16.x | Req 2.x | `ff-gcc-toolchain` lib tests, `ff-desktop/toolchain_panel.rs` tests |
| `compiler-toolchain-integration` | Req 17.x | Req 3.x | `ff-toolchain-api` lib tests, `ff-rust-toolchain` lib tests, `ff-desktop/toolchain_panel.rs` tests |
| `compiler-toolchain-integration` | Req 18.x | Req 4.x | `ff-rust-toolchain` lib tests, `ff-desktop/toolchain_panel.rs` tests |

### 12.2 VERIFY-STILL-VALID Required

The following criteria had their text changed (terminology or wording). The
existing tests still pass but should be reviewed to confirm they exercise the
updated criterion text.

| Spec | Criterion | Change | Test Location |
|------|-----------|--------|---------------|
| `layout-and-docking` | Req 11.1–11.5 | "floating window" → "Detached View" | `ff-layout` integration tests |
| `configuration-system` | Req 15.10 | "PF3" → "F3" | `ff-config` unit tests |
| `startup-and-session` | Req 19.10 | "PF3" → "F3" | `ff-desktop` shell tests |
| `function-keys-and-history` | Req 14 | "window context" → "Workspace Context" | `ff-desktop` shell tests |
| `file-tree-panel` | Req 16–17 | Implementation file references removed | `ff-desktop` context_menu tests |
| `virtual-catalog-manager` | Multiple | "Windows catalog" → "Native catalog" | `ff-desktop` catalog_manager tests |
| `virtual-catalog-manager` | Req 16.5 | Implementation file reference removed | `ff-desktop` files_panel tests |

### 12.3 NEW-TEST-NEEDED

The following new criteria were added during the rewrite phase and have no
existing test coverage. Each requires a 🔴 row in `docs/TCR.md`.

| Spec | New Content | Priority |
|------|-------------|----------|
| `compiler-toolchain-integration` | NFR section (build timeout, diagnostic parse performance) | Medium |
| `dataset-catalog` | NFR section (catalog query performance, reliability) | Medium |
| `database-tool` | NFR section (query execution time, result set size limits) | Medium |
| `connector-network-fs` | All 6 DEFERRED EARS criteria | Low (deferred) |
| `connector-ftp-sftp` | All 6 DEFERRED EARS criteria | Low (deferred) |
| `connector-mainframe` | All 6 DEFERRED EARS criteria | Low (deferred) |
| `connector-cloud` | All 6 DEFERRED EARS criteria | Low (deferred) |

---

## 13. Coverage Statistics

### 13.1 Spec Quality After Rewrite Phase

| Quality Flag | Before Rewrite | After Rewrite |
|---|---|---|
| Compliant | 43 (65%) | 57 (86%) |
| Needs Improvement | 11 (17%) | 4 (6%) |
| Major Rewrite Required | 4 (6%) | 0 (0%) |
| Deferred Stub | 0 | 4 (6%) |
| No Spec / Placeholder | 2 (3%) | 2 (3%) |
| Not a Feature Spec | 1 (2%) | 1 (2%) |
| **Total** | **65** | **66** |

*One additional spec entry added: `dataset-catalog` NFR section promoted it
from Needs Improvement to Compliant.*

### 13.2 Test Coverage by Layer

| Layer | Specs | ✅ All Pass | 🔲 Manual Only | 🔴 Gaps | ⚠️ Pending |
|-------|-------|------------|---------------|---------|-----------|
| Core Platform | 9 | 8 | 1 | 0 | 0 |
| Workbench Shell | 7 | 5 | 2 | 0 | 0 |
| Explorer Layer | 8 | 5 | 2 | 1 | 0 |
| Content Layer | 15 | 14 | 1 | 0 | 0 |
| Task Layer | 12 | 10 | 0 | 1 | 1 |
| Integration Layer | 7 | 3 | 0 | 4 | 0 |
| UX Layer | 8 | 8 | 0 | 0 | 0 |
| **Total** | **66** | **53 (80%)** | **6 (9%)** | **6 (9%)** | **1 (2%)** |

*Integration Layer gaps are all deferred connectors — intentional.*
*Task Layer gap is `compiler-toolchain-integration` NFR criteria — new.*
*Explorer Layer gap is `file-tree-panel` Req 19 drag-select — egui pointer limitation.*

### 13.3 Rewrite Phase Summary

| Task | Specs Processed | Changed | Unchanged | Test Annotations to Update | New Tests Needed |
|------|-----------------|---------|-----------|---------------------------|-----------------|
| Task 5 — Core Platform & UX | 10 | 5 | 5 | 0 | 0 |
| Task 6 — Explorer & Content | 15 | 10 | 5 | 4 | 4 |
| Task 7 — Task, Integration & Domain | 14 | 6 | 8 | 0 | 0 |
| **Total** | **39** | **21** | **18** | **4** | **4** |

---

## 14. Scope Overlaps (from Task 1 §5)

The following overlaps were identified during the baseline audit. They are
recorded here for resolution during future consolidation work (Task 9).

| ID | Specs | Description | Resolution Status |
|----|-------|-------------|-------------------|
| OV-01 | `file-tree-panel` + `virtual-catalog-manager` | Both specify catalog browsing and context menus; boundary blurred in Reqs 15–23 | Deferred to Task 9 |
| OV-02 | `file-tree-panel` + `connector-local-fs` | Tree panel Req 2 partially duplicates connector Req 3 (directory listing) | Deferred to Task 9 |
| OV-03 | `startup-and-session` + `virtual-catalog-manager` | Session Req 19 (File Explorer Panel) overlaps with file-tree-panel and virtual-catalog-manager | Deferred to Task 9 |
| OV-04 | `command-framework` + `command-semantics` | Specs do not cross-reference each other's scope boundaries | Deferred to Task 9 |
| OV-05 | `dataset-catalog` + `virtual-catalog-manager` | Some virtual-catalog-manager criteria re-specify validation rules that belong in dataset-catalog | Deferred to Task 9 |
| OV-06 | `function-keys-and-history` + `command-framework` | Command history specified in both; authoritative location should be command-framework | Deferred to Task 9 |
| OV-07 | `compiler-toolchain-integration` + `plugin-architecture` | Toolchain spec does not reference the plugin contract | Deferred to Task 9 |

---

## 15. Next Steps

This matrix feeds directly into:

- **Task 9** — Consolidation Report: uses §14 overlaps and §11 new sub-projects
  to produce a consolidation plan and overlap resolution recommendations.
- **Task 10** — Executive Assessment: uses §13 statistics and §12 outstanding
  actions to produce the strategic recommendations and roadmap.
- **Ongoing** — UPDATE-ANNOTATION items in §12.1 should be resolved before
  the next `cargo test` run to maintain accurate traceability.
