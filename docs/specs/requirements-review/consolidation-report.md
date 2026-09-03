# Requirements Review — Task 9: Consolidation Report

**Phase:** Requirements Review
**Status:** COMPLETE
**Date:** Phase BQ
**Reviewer:** Amazon Q Developer (Senior Requirements Engineer role)

---

## 1. Purpose

This report consolidates the findings from Tasks 1–8 into a set of concrete,
actionable recommendations for the requirements corpus. It addresses:

1. Scope overlaps between existing specs — with a recommended resolution for each
2. Boundary clarifications for multi-layer sub-projects
3. New sub-project stubs required to close High-priority gaps
4. Structural housekeeping (naming violations, misplaced documents)
5. A prioritised action backlog for the next phase of requirements work

This report does **not** change any source code. All recommendations are
documentation-level changes to `docs/specs/`.

---

## 2. Scope Overlap Resolutions

Seven overlaps were identified in Task 1 §5 and carried forward to the
traceability matrix §14. Each is assessed and resolved below.

---

### OV-01 — `file-tree-panel` + `virtual-catalog-manager`

**Description:** Both specs specify catalog browsing, context menus, and
content area behaviour. Requirements 15–23 of `file-tree-panel` re-specify
catalog-type-specific behaviour that is already owned by `virtual-catalog-manager`.

**Root Cause:** `file-tree-panel` grew organically through Phases AZ–BE,
adding catalog-specific context menus and content rendering directly into the
tree panel spec rather than referencing the catalog manager spec.

**Resolution: Boundary Clarification — Reference, Don't Duplicate**

| Spec | Owns | References |
|------|------|-----------|
| `virtual-catalog-manager` | Catalog CRUD, catalog type definitions, dataset allocation, VFS provider contract, content area columns and sort | `file-tree-panel` for tree rendering |
| `file-tree-panel` | Tree rendering, node expand/collapse, keyboard navigation, multi-select, drag-select, context menu structure | `virtual-catalog-manager` for catalog-type-specific menu items and content rules |

**Action:** In `file-tree-panel` Reqs 15–23, replace catalog-type-specific
acceptance criteria with cross-references of the form:
> "WHEN the user right-clicks a Mainframe node THE system SHALL display the
> context menu defined in `virtual-catalog-manager` Req 6."

This eliminates duplication without changing any implemented behaviour.

**Priority:** Medium — no code change; documentation edit only.

---

### OV-02 — `file-tree-panel` + `connector-local-fs`

**Description:** `file-tree-panel` Req 2 specifies bookmarked roots and local
file browsing that partially duplicates `connector-local-fs` Req 3 (directory
listing, file watching).

**Root Cause:** The tree panel spec was written before the connector spec was
fully developed. Req 2 describes what the connector must provide rather than
what the panel must display.

**Resolution: Ownership Transfer**

| Spec | Owns |
|------|------|
| `connector-local-fs` | Directory listing, file watching, path normalisation, cross-platform path handling |
| `file-tree-panel` | Displaying the listing returned by the connector; bookmarked root management (which roots are registered, not how they are read) |

**Action:** In `file-tree-panel` Req 2, replace directory-listing acceptance
criteria with a single cross-reference:
> "THE system SHALL use the `connector-local-fs` VFS provider to enumerate
> directory contents (see `connector-local-fs` Req 3)."

Retain only the bookmarked-root registration and persistence criteria in
`file-tree-panel`.

**Priority:** Low — the connector is fully implemented; this is a spec
alignment only.

---

### OV-03 — `startup-and-session` + `virtual-catalog-manager` + `file-tree-panel`

**Description:** `startup-and-session` Req 19 specifies the File Explorer
Panel (POM option 2), which overlaps with both `file-tree-panel` (tree
rendering) and `virtual-catalog-manager` (catalog management).

**Root Cause:** The File Explorer Panel was added as a session-level feature
(Phase AS) before the tree panel and catalog manager specs were updated to
own their respective parts of the panel.

**Resolution: Partition by Concern**

| Spec | Owns |
|------|------|
| `startup-and-session` | Routing (`=2`, `=FILES`, `FILES` commands), tab kind (`FileExplorerPanel`), session persistence of the tab, title bar text `[FILES]` |
| `file-tree-panel` | All tree rendering, node types, expand/collapse, keyboard navigation, context menus |
| `virtual-catalog-manager` | Catalog sidebar, catalog selection, content pane per catalog type |

**Action:** In `startup-and-session` Req 19, retain only the routing and
persistence criteria (Req 19.1–19.4, 19.10–19.12). Replace Req 19.5–19.9
(tree rendering) with cross-references to `file-tree-panel` Reqs 1–5 and
`virtual-catalog-manager` Req 23.

**Priority:** Medium — clarifies ownership for future feature work on the
File Explorer Panel.

---

### OV-04 — `command-framework` + `command-semantics`

**Description:** Both specs cover command dispatch but do not cross-reference
each other's scope boundaries. `command-framework` owns the generic registry
and dispatch; `command-semantics` owns the ISPF-specific parser and pipeline.

**Root Cause:** The two specs were written independently. The boundary is
clear in the implementation but not stated in the specs.

**Resolution: Add Explicit Scope Boundary Statements**

**Action:** Add a "Scope Boundary" section to each spec:

- In `command-framework` Introduction: "This spec owns the generic command
  registry, dispatch pipeline, and history store. ISPF-specific command
  parsing is owned by `command-semantics`."
- In `command-semantics` Introduction: "This spec owns the ISPF command
  parser and semantic validation pipeline. The generic dispatch mechanism
  is owned by `command-framework`."

No criteria need to change — this is a documentation addition only.

**Priority:** Low — no ambiguity in practice; clarification for new contributors.

---

### OV-05 — `dataset-catalog` + `virtual-catalog-manager`

**Description:** Some `virtual-catalog-manager` acceptance criteria re-specify
dataset naming validation rules that belong in `dataset-catalog`.

**Root Cause:** The catalog manager dialog was specified with inline validation
rules (DSN format, HLQ length, character set) that duplicate the authoritative
rules in `dataset-catalog`.

**Resolution: Reference the Authoritative Spec**

**Action:** In `virtual-catalog-manager` Req 5 (Dataset Allocation Dialog),
replace inline DSN validation criteria with:
> "THE system SHALL validate the Dataset Name field according to the naming
> rules defined in `dataset-catalog` Req 2."

Retain only the UI-level criteria (field layout, error display, confirm/cancel
behaviour) in `virtual-catalog-manager`.

**Priority:** Medium — prevents future divergence if naming rules change.

---

### OV-06 — `function-keys-and-history` + `command-framework`

**Description:** Command history is specified in both `command-framework`
Req 7 and `function-keys-and-history`. The authoritative location should be
`command-framework`; `function-keys-and-history` should reference it.

**Root Cause:** The RETRIEVE/LIST history feature was added to
`function-keys-and-history` (Phase AM) without checking that `command-framework`
already owned the history store contract.

**Resolution: Ownership Clarification**

| Spec | Owns |
|------|------|
| `command-framework` | Command history store: append, retrieve, depth limit, persistence |
| `function-keys-and-history` | RETRIEVE command UI: LIST overlay, selection populates Command Field, Escape clears |

**Action:** In `function-keys-and-history` Req 19, replace history-store
criteria with a cross-reference to `command-framework` Req 7. Retain only
the UI interaction criteria (LIST overlay, selection, Escape).

**Priority:** Low — both specs are Compliant; this is a precision improvement.

---

### OV-07 — `compiler-toolchain-integration` + `plugin-architecture`

**Description:** The compiler toolchain is implemented as a plugin but
`compiler-toolchain-integration` does not reference the `plugin-architecture`
plugin contract. The toolchain spec is GCC/Rust-specific rather than
extensible.

**Root Cause:** The toolchain spec was written before the generic
`ToolchainPlugin` trait was identified as a gap (Task 4 §6.4).

**Resolution: Add Generic Trait Requirement**

**Action:** Add a new requirement to `compiler-toolchain-integration`:
> "FR-0971: Generic Toolchain Plugin Trait — THE system SHALL define a
> `ToolchainPlugin` trait that extends `FileForgePlugin` (see
> `plugin-architecture` Req 2) and provides the standard interface for
> toolchain detection, installation, build invocation, and diagnostic
> parsing. GCC and Rust toolchain plugins SHALL implement this trait."

This closes the gap identified in Task 4 §6.4 and makes the spec extensible
for future toolchains (LLVM, GnuCOBOL, OpenJDK).

**Priority:** High — required before any new toolchain plugin is implemented.

---

## 3. Boundary Clarifications for Multi-Layer Sub-Projects

Task 3 §4 identified seven sub-projects that span two or more architectural
layers. The following clarifications are recommended to make the layer
ownership explicit in each spec without splitting the crate.

| Sub-Project | Recommended Addition |
|-------------|---------------------|
| `startup-and-session` | Add a "Layer Partition" note: Reqs 1–11 (session model, persistence) → Core Platform section; Reqs 13–19 (POM, tab container, shell interactions) → Workbench Shell section |
| `file-tree-panel` | Add a "Layer Partition" note: Reqs 1–14 (tree panel, VFS browsing) → Explorer Layer; Reqs 15–23 (catalog-specific UI) → Explorer Layer with Integration Layer cross-refs |
| `virtual-catalog-manager` | Add a "Layer Partition" note: Reqs 1–11 (catalog UI, dialogs) → Explorer Layer; Reqs 7, 12–16 (VFS provider, path resolution) → Integration Layer |
| `database-tool` | Add a "Layer Partition" note: connection management, schema browser → Integration Layer; SQL editor, result grid → Content Layer |
| `layout-and-docking` | Add a "Layer Partition" note: layout model, serialisation → Core Platform; panel rendering, drag-drop → Workbench Shell |
| `function-keys-and-history` | Add a "Layer Partition" note: key map resolver, history store → Core Platform; key label bar rendering → Workbench Shell |
| `compiler-toolchain-integration` | Add a "Layer Partition" note: build invocation, diagnostic parsing → Task Layer; toolchain detection, install → Integration Layer |

These notes are added to the Introduction section of each spec. No criteria
need to change.

---

## 4. New Sub-Project Stubs Required

Seven new sub-project specs were recommended by the gap analysis (Task 4 §8.2)
and confirmed by the domain classification (Task 3 §7). The following table
defines the minimum content for each stub spec.

Each stub must be created as `docs/specs/<name>/requirements.md` with:
- Introduction and Glossary sections
- At minimum one DEFERRED EARS requirement per identified gap
- Source references
- A "Status: STUB — Not yet implemented" banner

| Sub-Project | Layer | FR Range | Minimum Requirements | Priority |
|-------------|-------|----------|---------------------|----------|
| `accessibility` | UX Layer | FR-1300–FR-1319 | Req 1: Screen reader support; Req 2: WCAG AA colour contrast; Req 3: Keyboard-only operation; Req 4: Focus indicators; Req 5: Font size scaling | High |
| `command-palette` | UX Layer | FR-1320–FR-1329 | Req 1: Ctrl+Shift+P opens palette; Req 2: Fuzzy search over command registry; Req 3: Recent commands at top; Req 4: Keyboard shortcut hints | High |
| `workspace-model` | Core Platform | FR-0160–FR-0179 | Req 1: Multi-root workspace definition; Req 2: Workspace file format (TOML); Req 3: Open/Save/Close workspace commands; Req 4: Workspace-scoped settings layer | High |
| `global-search` | Task Layer | FR-0970 sub-range | Req 1: Cross-file search with include/exclude patterns; Req 2: Search results panel; Req 3: Replace across files; Req 4: Search scope (catalog, directory, workspace) | High |
| `notification-system` | Workbench Shell | FR-0370–FR-0379 | Req 1: Non-modal toast notification; Req 2: Notification queue; Req 3: Dismiss on timeout or click; Req 4: Error/warning/info severity levels | Medium |
| `plugin-manager-ui` | Workbench Shell | FR-0380–FR-0389 | Req 1: Plugin Manager panel (list installed plugins); Req 2: Enable/disable plugin; Req 3: Plugin detail view; Req 4: Install from local path | High |
| `audit-logging` | Core Platform | FR-0180–FR-0199 | Req 1: Structured audit trail (file open/save/delete); Req 2: Audit log format (JSON/CSV); Req 3: Retention policy; Req 4: Audit log export | High |

---

## 5. Structural Housekeeping

### 5.1 Naming Convention Violations

Two sub-project folder names violate the `kebab-case` convention:

| Current Name | Issue | Recommended Action |
|-------------|-------|--------------------|
| `FFW-JES` | Uppercase and project prefix | Rename folder to `jes-emulator`; update all cross-references in other specs and `docs/specs/project-master/tasks.md` |
| `workbench-requirements-merge` | Not a feature spec | Move contents to `docs/architecture/`; remove from `docs/specs/` |

**Note:** The `FFW-JES` rename is a documentation-only change. The crate name
`ff-jes` already follows the correct convention and does not need to change.

### 5.2 Empty Placeholder

| Folder | Current State | Recommended Action |
|--------|--------------|-------------------|
| `jcl-resolver` | `.gitkeep` only | Either: (a) merge JCL resolution requirements into `jes-emulator` as a sub-section, or (b) create a minimal stub spec when JCL resolution is scheduled for implementation |

### 5.3 Source Reference Tag Standardisation

The following source reference tags are used inconsistently across specs.
A single canonical set should be adopted:

| Canonical Tag | Meaning | Replaces |
|--------------|---------|---------|
| `[WB]` | FileForge Workbench architecture brief | `[FFW]`, `[FFWB]` |
| `[ISPF]` | IBM ISPF Reference Summary | `[ISPF-POM]`, `[ISPF-REF]` |
| `[DSC]` | Dataset Catalog design document | `[DSC]` (already consistent) |
| `[FFE]` | FileForge Enterprise specification | `[FFE]`, `[FFE-TREE]` |
| `[EGUI]` | egui documentation | `[EGUI]`, `[EGUI-API]` |

**Action:** Apply the canonical tags during the next terminology pass on any
spec that uses non-standard variants.

---

## 6. Prioritised Action Backlog

The following table consolidates all recommended actions from this report,
the traceability matrix §12, and the gap analysis §8.3. Items are ordered
by priority within each category.

### 6.1 Immediate Actions (before next implementation phase)

| ID | Action | Spec(s) | Type | Effort |
|----|--------|---------|------|--------|
| CA-01 | UPDATE-ANNOTATION: change `Req 15.x–18.x` → `Req 1.x–4.x` in all compiler toolchain test files | `compiler-toolchain-integration` | Test annotation | Low |
| CA-02 | Add `FR-0971: Generic Toolchain Plugin Trait` requirement | `compiler-toolchain-integration` | New criterion | Low |
| CA-03 | Add scope boundary statements to Introduction sections | `command-framework`, `command-semantics` | Documentation | Low |

### 6.2 High-Priority Documentation Actions

| ID | Action | Spec(s) | Type | Effort |
|----|--------|---------|------|--------|
| CA-04 | Create `accessibility` stub spec | New spec | Stub creation | Medium |
| CA-05 | Create `command-palette` stub spec | New spec | Stub creation | Medium |
| CA-06 | Create `workspace-model` stub spec | New spec | Stub creation | Medium |
| CA-07 | Create `global-search` stub spec | New spec | Stub creation | Medium |
| CA-08 | Create `plugin-manager-ui` stub spec | New spec | Stub creation | Low |
| CA-09 | Create `audit-logging` stub spec | New spec | Stub creation | Low |
| CA-10 | Resolve OV-05: replace inline DSN validation in `virtual-catalog-manager` Req 5 with cross-reference to `dataset-catalog` Req 2 | `virtual-catalog-manager` | Criterion edit | Low |
| CA-11 | Resolve OV-07: add `ToolchainPlugin` trait requirement (CA-02 above) | `compiler-toolchain-integration` | New criterion | Low |

### 6.3 Medium-Priority Documentation Actions

| ID | Action | Spec(s) | Type | Effort |
|----|--------|---------|------|--------|
| CA-12 | Resolve OV-01: replace duplicate catalog-browsing criteria in `file-tree-panel` Reqs 15–23 with cross-references to `virtual-catalog-manager` | `file-tree-panel` | Criterion edit | Medium |
| CA-13 | Resolve OV-03: partition `startup-and-session` Req 19 — retain routing/persistence, cross-reference tree and catalog specs | `startup-and-session` | Criterion edit | Medium |
| CA-14 | Add Layer Partition notes to all 7 multi-layer sub-projects | Multiple | Documentation | Low |
| CA-15 | Create `notification-system` stub spec | New spec | Stub creation | Low |
| CA-16 | Add NFR test coverage for `compiler-toolchain-integration`, `dataset-catalog`, `database-tool` | Multiple | New tests | Medium |
| CA-17 | VERIFY-STILL-VALID: review 7 criteria with terminology changes against existing tests | Multiple | Test review | Low |

### 6.4 Low-Priority Housekeeping

| ID | Action | Spec(s) | Type | Effort |
|----|--------|---------|------|--------|
| CA-18 | Rename `FFW-JES` folder to `jes-emulator`; update all cross-references | `FFW-JES` | Rename | Low |
| CA-19 | Move `workbench-requirements-merge` to `docs/architecture/` | `workbench-requirements-merge` | Move | Low |
| CA-20 | Resolve `jcl-resolver` placeholder — merge into `jes-emulator` or create stub | `jcl-resolver` | Decision | Low |
| CA-21 | Resolve OV-02: align `file-tree-panel` Req 2 with `connector-local-fs` Req 3 | `file-tree-panel` | Criterion edit | Low |
| CA-22 | Resolve OV-04: add scope boundary statements to `command-framework` and `command-semantics` | Both | Documentation | Low |
| CA-23 | Resolve OV-06: move history-store criteria from `function-keys-and-history` Req 19 to `command-framework` Req 7 | Both | Criterion move | Low |
| CA-24 | Standardise source reference tags across all specs | Multiple | Terminology | Low |

---

## 7. Consolidation Summary

### 7.1 Overlaps

| ID | Status | Resolution |
|----|--------|-----------|
| OV-01 | Deferred to CA-12 | Boundary clarification via cross-references |
| OV-02 | Deferred to CA-21 | Ownership transfer to `connector-local-fs` |
| OV-03 | Deferred to CA-13 | Partition `startup-and-session` Req 19 |
| OV-04 | Deferred to CA-22 | Scope boundary statements |
| OV-05 | Deferred to CA-10 | Cross-reference to `dataset-catalog` |
| OV-06 | Deferred to CA-23 | Ownership transfer to `command-framework` |
| OV-07 | Immediate — CA-02/CA-11 | Add `ToolchainPlugin` trait requirement |

### 7.2 New Sub-Projects

| Sub-Project | Priority | Action |
|-------------|----------|--------|
| `accessibility` | High | CA-04 |
| `command-palette` | High | CA-05 |
| `workspace-model` | High | CA-06 |
| `global-search` | High | CA-07 |
| `plugin-manager-ui` | High | CA-08 |
| `audit-logging` | High | CA-09 |
| `notification-system` | Medium | CA-15 |

### 7.3 Corpus Health After Phase BQ

| Metric | Value |
|--------|-------|
| Total specs | 66 |
| Compliant (post-rewrite) | 57 (86%) |
| Needs Improvement (remaining) | 4 (6%) |
| Deferred stubs | 4 (6%) |
| No spec / placeholder | 2 (3%) |
| Overlaps identified | 7 |
| Overlaps with immediate resolution | 1 (OV-07) |
| Overlaps deferred | 6 |
| New sub-projects recommended | 7 |
| Outstanding test annotation updates | 4 |
| Outstanding new tests needed | 7 |

---

## 8. Next Steps

This consolidation report feeds directly into:

- **Task 10** — Executive Assessment: uses §7.3 corpus health metrics,
  §6 action backlog priorities, and §4 new sub-project list to produce
  the strategic roadmap and executive summary.
- **Phase BR (future)** — Implementation of CA-01 through CA-11 (immediate
  and high-priority actions) as the first deliverable of the next
  requirements maintenance phase.
