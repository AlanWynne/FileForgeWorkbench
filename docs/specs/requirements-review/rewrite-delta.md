# Requirements Review — Rewrite Delta Register

**Phase:** Requirements Review — Tasks 5–7
**Purpose:** Tracks every change made to each `requirements.md` during the rewrite
phase so that existing test annotations (`// Validates: Req X.Y`) can be updated
and new coverage gaps are visible in TCR.md.

---

## How to Read This Document

| Column | Meaning |
|--------|---------|
| Old ID | The criterion reference used in existing test code (e.g. `Req 8.3`) |
| New ID | The FR-XXXX number assigned in the rewritten spec |
| Change Type | `Renumbered` / `Renamed` / `Split` / `Merged` / `Reworded` / `New` / `Removed` |
| Test Impact | What action is needed in the codebase |

### Test Impact Codes

| Code | Action Required |
|------|----------------|
| `UPDATE-ANNOTATION` | Find `// Validates: <Old ID>` in test files and change to `// Validates: <New ID>` |
| `VERIFY-STILL-VALID` | Criterion text changed — confirm the existing test still exercises the new wording |
| `NEW-TEST-NEEDED` | New criterion with no existing test — add 🔴 row to TCR.md |
| `NO-ACTION` | Renumber only, criterion text unchanged, test still valid |

---

## Status Key

| Symbol | Meaning |
|--------|---------|
| ✅ | Delta recorded, all test impacts resolved |
| 🔄 | Rewrite complete, test impact resolution pending |
| ⏳ | Rewrite not yet started |

---

## Task 5 — Core Platform & UX Layer Specs (10 specs)

### 1. `platform-core` ⏳

No changes required — spec is Compliant. No terminology violations. No structural
issues. No test annotation updates needed.

**Delta:** None.

---

### 2. `command-framework` ⏳

No changes required — spec is Compliant. No terminology violations. No structural
issues. No test annotation updates needed.

**Delta:** None.

---

### 3. `plugin-architecture` ⏳

No changes required — spec is Compliant. No terminology violations. No structural
issues. No test annotation updates needed.

**Delta:** None.

---

### 4. `workflow-engine` ⏳

No changes required — spec is Compliant. No terminology violations. No structural
issues. No test annotation updates needed.

**Delta:** None.

---

### 5. `layout-and-docking` ⏳

Minor terminology pass required: "floating window" → "Detached View" in
user-facing criterion text. Requirement 11 uses non-standard criterion numbering
(11.1–11.5 with dot prefix instead of numbered list). No test annotation updates
needed (Req 11 criteria have no existing automated tests — all are 🔴 or 🔲).

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 11.1 | Req 11.1 | Reworded (terminology: "floating window" → "Detached View") | VERIFY-STILL-VALID |
| Req 11.2 | Req 11.2 | Reworded (terminology) | VERIFY-STILL-VALID |
| Req 11.3 | Req 11.3 | Reworded (terminology) | VERIFY-STILL-VALID |
| Req 11.4 | Req 11.4 | Reworded (terminology) | VERIFY-STILL-VALID |
| Req 11.5 | Req 11.5 | Reworded (terminology) | VERIFY-STILL-VALID |

---

### 6. `configuration-system` ⏳

Requirement 15 uses non-standard criterion numbering (15.1–15.11 with dot prefix).
Terminology: "PF3" → "F3" in criterion 15.10. No test annotation updates needed
for the numbering style change. One annotation update needed for the PF3 → F3 fix.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 15.10 ("PF3") | Req 15.10 ("F3") | Reworded (terminology: PF3 → F3) | VERIFY-STILL-VALID |

---

### 7. `theme-and-appearance` ⏳

No structural issues. Requirements 12, 13, 14 are out of sequence (12 appears after
14 in the file). Renumbering to sequential order. All existing test annotations
reference Req 13.x and Req 14.x — these numbers are preserved; only Req 12 moves
to end of file (no tests reference Req 12 by number in TCR).

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 12 (Extensibility) | Req 15 | Renumbered (moved to end, after Req 14) | NO-ACTION (no tests reference Req 12 theme-and-appearance) |
| Req 13.1–13.8 | Req 13.1–13.8 | No change | NO-ACTION |
| Req 14.1–14.10 | Req 14.1–14.10 | No change | NO-ACTION |

---

### 8. `menu-and-statusbar` ⏳

Requirement 13 (About dialog) appears between Req 11 and Req 16 — out of sequence.
Requirement 16 uses non-standard criterion numbering (16.1–16.22 with dot prefix,
mixed with plain numbered list). Requirement 17 and 18 use dot-prefix style.
Terminology: "Command ===> field" → "Command Field" in user stories.
All existing test annotations use Req 13.x, Req 16.x, Req 17.x, Req 18.x — numbers
preserved, only style normalised.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 13.1–13.8 | Req 13.1–13.8 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 16.1–16.22 | Req 16.1–16.22 | Style normalised | NO-ACTION |
| Req 17.1–17.9 | Req 17.1–17.9 | Style normalised | NO-ACTION |
| Req 18.1–18.7 | Req 18.1–18.7 | Style normalised | NO-ACTION |
| "Command ===> field" (user stories) | "Command Field" | Reworded (terminology) | NO-ACTION (user stories not referenced in test annotations) |

---

### 9. `function-keys-and-history` ⏳

Terminology pass: "PF Key" → "Function Key", "Key Bar" → "Key Label Bar",
"window context" → "Workspace Context", "screen" → "View".
Requirements 12–20 use non-standard criterion numbering (dot-prefix style).
All existing test annotations use Req 12.x through Req 20.x — numbers preserved.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 12.1–12.7 | Req 12.1–12.7 | Style normalised + terminology pass | NO-ACTION |
| Req 13.1–13.5 | Req 13.1–13.5 | Style normalised + terminology pass | NO-ACTION |
| Req 14.1–14.7 | Req 14.1–14.7 | Style normalised + terminology pass | NO-ACTION |
| Req 15.1–15.4 | Req 15.1–15.4 | Style normalised | NO-ACTION |
| Req 16.1–16.5 | Req 16.1–16.5 | Style normalised | NO-ACTION |
| Req 17.1–17.7 | Req 17.1–17.7 | Style normalised | NO-ACTION |
| Req 18.1–18.3 | Req 18.1–18.3 | Style normalised | NO-ACTION |
| Req 19.1–19.7 | Req 19.1–19.7 | Style normalised | NO-ACTION |
| Req 20.1–20.15 | Req 20.1–20.15 | Style normalised | NO-ACTION |
| "PF key" (intro text) | "Function Key" | Reworded (terminology) | NO-ACTION (intro text not in test annotations) |
| "Key Bar" (intro text) | "Key Label Bar" | Reworded (terminology) | NO-ACTION |
| "window context" (Req 14 text) | "Workspace Context" | Reworded (terminology) | VERIFY-STILL-VALID (Req 14 has tests) |

---

### 10. `context-help` ⏳

No structural issues. No terminology violations. Criterion numbering uses dot-prefix
style (1.1, 1.2, etc.) consistently throughout — this is the established style for
this spec and is acceptable. No changes required.

**Delta:** None.

---

## Task 6 — Explorer & Content Layer Specs (15 specs)

### 1. `startup-and-session` 🔄

Criteria in Req 13, Req 14 (14.1–14.42), and Req 19 used dot-prefix style.
Normalised to numbered list. Criterion numbers preserved — all existing test
annotations remain valid. Terminology: "PF3" → "F3" in Req 19.10.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 13.1–13.4 | Req 13.1–13.4 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 14.1–14.42 | Req 14.1–14.42 | Style normalised | NO-ACTION |
| Req 14.15a/b/c | Req 15a/15b/15c | Style normalised (sub-criteria labels) | NO-ACTION |
| Req 19.1–19.12 | Req 19.1–19.12 | Style normalised | NO-ACTION |
| Req 19.10 ("PF3") | Req 19.10 ("F3") | Reworded (terminology: PF3 → F3) | VERIFY-STILL-VALID |

---

### 2. `file-tree-panel` 🔄

Bold-label criteria style in Reqs 15–23 (`**16.1 — Title**`) normalised to
numbered sub-headings (`**1. Title**`). Criterion numbers preserved within each
requirement. Implementation file references removed from criteria text.
Glossary consolidation note added. Req 22/23 ordering note added.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 15.1–15.3 | Req 15.1–15.3 | Style normalised | NO-ACTION |
| Req 16.1–16.18 | Req 16.1–16.18 | Style normalised + impl refs removed | VERIFY-STILL-VALID |
| Req 17.1–17.9 | Req 17.1–17.9 | Style normalised + impl refs removed | VERIFY-STILL-VALID |
| Req 18.1–18.9 | Req 18.1–18.9 | Style normalised | NO-ACTION |
| Req 19.1–19.10 | Req 19.1–19.10 | Style normalised | NO-ACTION |
| Req 20.1–20.13 | Req 20.1–20.13 | Style normalised | NO-ACTION |
| Req 21.1–21.11 | Req 21.1–21.11 | Style normalised | NO-ACTION |
| Req 22.1–22.6 | Req 22.1–22.6 | Ordering note added | NO-ACTION |
| Req 23.1–23.10 | Req 23.1–23.10 | No change | NO-ACTION |

---

### 3. `virtual-catalog-manager` 🔄

Terminology: "Windows catalog" → "Native catalog" throughout. Implementation
file reference removed from Req 16.5. Req 11 ordering note added.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| "Windows catalog" (multiple criteria) | "Native catalog" | Reworded (terminology) | VERIFY-STILL-VALID |
| Req 16.5 (impl ref removed) | Req 16.5 | Reworded (impl neutrality) | VERIFY-STILL-VALID |
| Req 11 ordering note | — | Documentation note added | NO-ACTION |

---

### 4. `connector-network-fs` 🔄

Major rewrite: added DEFERRED EARS stubs section with 6 formal acceptance
criteria. Existing placeholder bullet-list requirements retained for context.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Placeholder Req 1–6 (bullet list) | Formal Req 1–6 (EARS, DEFERRED) | Rewritten | NEW-TEST-NEEDED (when implemented) |

---

### 5. `connector-ftp-sftp` 🔄

Major rewrite: added DEFERRED EARS stubs section with 6 formal acceptance
criteria.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Placeholder Req 1–6 (bullet list) | Formal Req 1–6 (EARS, DEFERRED) | Rewritten | NEW-TEST-NEEDED (when implemented) |

---

### 6. `connector-mainframe` 🔄

Major rewrite: added DEFERRED EARS stubs section with 6 formal acceptance
criteria.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Placeholder Req 1–6 (bullet list) | Formal Req 1–6 (EARS, DEFERRED) | Rewritten | NEW-TEST-NEEDED (when implemented) |

---

### 7. `connector-cloud` 🔄

Major rewrite: added DEFERRED EARS stubs section with 6 formal acceptance
criteria.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Placeholder Req 1–6 (bullet list) | Formal Req 1–6 (EARS, DEFERRED) | Rewritten | NEW-TEST-NEEDED (when implemented) |

---

### 8. `dataset-catalog` 🔄

NFR section added (performance, reliability, scalability, data integrity).
No existing criteria changed. No test annotation updates needed.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| — | NFR section (new) | New | NEW-TEST-NEEDED (NFR tests) |

---

### 9. `dataset-allocator` ⏳

No changes required — spec is Compliant. Exemplary quality. No test annotation
updates needed.

**Delta:** None.

---

### 10. `dataset-ownership-model` ⏳

Spec is Needs Improvement (high-level criteria, missing edge-case criteria) but
changes are deferred — the governance document is authoritative and its criteria
are intentionally high-level. No changes made in Task 6.

**Delta:** None (deferred to Task 7 if reclassified).

---

### 11. `FFW-JES` 🔄

Naming convention note added to Introduction. JES2/JES3 clarified as emulation
targets (not dependencies). Dot-prefix criteria (1.1–15.5) normalised to numbered
list. Criterion numbers preserved within each requirement.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.9 | Req 1.1–1.9 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.8 | Req 2.1–2.8 | Style normalised | NO-ACTION |
| Req 3.1–3.10 | Req 3.1–3.10 | Style normalised | NO-ACTION |
| Req 4.1–4.8 | Req 4.1–4.8 | Style normalised | NO-ACTION |
| Req 5.1–5.5 | Req 5.1–5.5 | Style normalised | NO-ACTION |
| Req 6.1–6.8 | Req 6.1–6.8 | Style normalised | NO-ACTION |
| Req 7.1–7.7 | Req 7.1–7.7 | Style normalised | NO-ACTION |
| Req 8.1–8.6 | Req 8.1–8.6 | Style normalised | NO-ACTION |
| Req 9.1–9.10 | Req 9.1–9.10 | Style normalised | NO-ACTION |
| Req 10.1–10.4 | Req 10.1–10.4 | Style normalised | NO-ACTION |
| Req 11.1–11.7 | Req 11.1–11.7 | Style normalised | NO-ACTION |
| Req 12.1–12.5 | Req 12.1–12.5 | Style normalised | NO-ACTION |
| Req 13.1–13.4 | Req 13.1–13.4 | Style normalised | NO-ACTION |
| Req 14.1–14.7 | Req 14.1–14.7 | Style normalised | NO-ACTION |
| Req 15.1–15.5 | Req 15.1–15.5 | Style normalised | NO-ACTION |
| JES2/JES3 intro text | Clarified as emulation targets | Reworded | NO-ACTION |

---

### 12. `database-tool` 🔄

"DBeaver" removed from requirement body text (retained in source references and
Source Reference Key table). NFR section added. No existing criteria changed.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| "DBeaver" in intro text | "integrated database IDE" | Reworded (impl neutrality) | NO-ACTION |
| — | NFR section (new) | New | NEW-TEST-NEEDED (NFR tests) |

---

### 13. `compiler-toolchain-integration` 🔄

Requirements renumbered from 15–18 to 1–4. Criteria renumbered accordingly
(15.N → N., 16.N → N., etc.). Gap note added about generic ToolchainPlugin trait.
NFR section added.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 15 | Req 1 | Renumbered | UPDATE-ANNOTATION (tests referencing Req 15.x) |
| Req 16 | Req 2 | Renumbered | UPDATE-ANNOTATION (tests referencing Req 16.x) |
| Req 17 | Req 3 | Renumbered | UPDATE-ANNOTATION (tests referencing Req 17.x) |
| Req 18 | Req 4 | Renumbered | UPDATE-ANNOTATION (tests referencing Req 18.x) |
| — | NFR section (new) | New | NEW-TEST-NEEDED (NFR tests) |

---

### 14. `multi-tab-editor` ⏳

No changes required — spec is Compliant. No test annotation updates needed.

**Delta:** None.

---

### 15. `compare-and-merge` ⏳

No changes required — spec is Compliant. No test annotation updates needed.

**Delta:** None.

---

## Task 7 — Task Layer, Integration Layer & Domain Specs (14 specs)

### 1. `virtual-file-system` ⏳

No changes required — spec is Compliant. Clean numbered criteria, no terminology
violations, no implementation file references, no structural issues.

**Delta:** None.

---

### 2. `connector-local-fs` ⏳

No changes required — spec is Compliant. Clean numbered criteria, good structure,
no terminology violations.

**Delta:** None.

---

### 3. `connector-extensibility` ⏳

No changes required — spec is Compliant. Clean numbered criteria, good structure,
no terminology violations.

**Delta:** None.

---

### 4. `document-model` ⏳

No changes required — spec is Compliant. Clean numbered criteria, good
cross-references, no terminology violations.

**Delta:** None.

---

### 5. `edit-operations` 🔄

All criteria used dot-prefix style (`1.1 WHEN...`, `1.2 WHEN...`) throughout all
15 requirements. Normalised to numbered list (`1. WHEN...`, `2. WHEN...`).
Criterion numbers preserved within each requirement — all existing test
annotations remain valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.8 | Req 1.1–1.8 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.6 | Req 2.1–2.6 | Style normalised | NO-ACTION |
| Req 3.1–3.8 | Req 3.1–3.8 | Style normalised | NO-ACTION |
| Req 4.1–4.12 | Req 4.1–4.12 | Style normalised | NO-ACTION |
| Req 5.1–5.8 | Req 5.1–5.8 | Style normalised | NO-ACTION |
| Req 6.1–6.17 | Req 6.1–6.17 | Style normalised | NO-ACTION |
| Req 7.1–7.7 | Req 7.1–7.7 | Style normalised | NO-ACTION |
| Req 8.1–8.16 | Req 8.1–8.16 | Style normalised | NO-ACTION |
| Req 9.1–9.10 | Req 9.1–9.10 | Style normalised | NO-ACTION |
| Req 10.1–10.12 | Req 10.1–10.12 | Style normalised | NO-ACTION |
| Req 11.1–11.9 | Req 11.1–11.9 | Style normalised | NO-ACTION |
| Req 12.1–12.5 | Req 12.1–12.5 | Style normalised | NO-ACTION |
| Req 13.1–13.12 | Req 13.1–13.12 | Style normalised | NO-ACTION |
| Req 14.1–14.9 | Req 14.1–14.9 | Style normalised | NO-ACTION |
| Req 15.1–15.6 | Req 15.1–15.6 | Style normalised | NO-ACTION |

---

### 6. `undo-redo-transactions` 🔄

All criteria used dot-prefix style throughout all 18 requirements. Normalised to
numbered list. Criterion numbers preserved — all existing test annotations remain
valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.7 | Req 1.1–1.7 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.6 | Req 2.1–2.6 | Style normalised | NO-ACTION |
| Req 3.1–3.7 | Req 3.1–3.7 | Style normalised | NO-ACTION |
| Req 4.1–4.9 | Req 4.1–4.9 | Style normalised | NO-ACTION |
| Req 5.1–5.9 | Req 5.1–5.9 | Style normalised | NO-ACTION |
| Req 6.1–6.7 | Req 6.1–6.7 | Style normalised | NO-ACTION |
| Req 7.1–7.10 | Req 7.1–7.10 | Style normalised | NO-ACTION |
| Req 8.1–8.8 | Req 8.1–8.8 | Style normalised | NO-ACTION |
| Req 9.1–9.9 | Req 9.1–9.9 | Style normalised | NO-ACTION |
| Req 10.1–10.4 | Req 10.1–10.4 | Style normalised | NO-ACTION |
| Req 11.1–11.5 | Req 11.1–11.5 | Style normalised | NO-ACTION |
| Req 12.1–12.6 | Req 12.1–12.6 | Style normalised | NO-ACTION |
| Req 13.1–13.6 | Req 13.1–13.6 | Style normalised | NO-ACTION |
| Req 14.1–14.7 | Req 14.1–14.7 | Style normalised | NO-ACTION |
| Req 15.1–15.7 | Req 15.1–15.7 | Style normalised | NO-ACTION |
| Req 16.1–16.4 | Req 16.1–16.4 | Style normalised | NO-ACTION |
| Req 17.1–17.5 | Req 17.1–17.5 | Style normalised | NO-ACTION |
| Req 18.1–18.5 | Req 18.1–18.5 | Style normalised | NO-ACTION |

---

### 7. `viewport-and-scrolling` ⏳

No changes required — spec is Compliant. Clean numbered criteria, no terminology
violations.

**Delta:** None.

---

### 8. `display-line-mapping` ⏳

No changes required — spec is Compliant. Clean numbered criteria, no terminology
violations.

**Delta:** None.

---

### 9. `caret-and-selection` 🔄

All criteria used dot-prefix style throughout all 12 requirements. Normalised to
numbered list. Criterion numbers preserved — all existing test annotations remain
valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.10 | Req 1.1–1.10 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.7 | Req 2.1–2.7 | Style normalised | NO-ACTION |
| Req 3.1–3.7 | Req 3.1–3.7 | Style normalised | NO-ACTION |
| Req 4.1–4.13 | Req 4.1–4.13 | Style normalised | NO-ACTION |
| Req 5.1–5.10 | Req 5.1–5.10 | Style normalised | NO-ACTION |
| Req 6.1–6.10 | Req 6.1–6.10 | Style normalised | NO-ACTION |
| Req 7.1–7.6 | Req 7.1–7.6 | Style normalised | NO-ACTION |
| Req 8.1–8.5 | Req 8.1–8.5 | Style normalised | NO-ACTION |
| Req 9.1–9.6 | Req 9.1–9.6 | Style normalised | NO-ACTION |
| Req 10.1–10.5 | Req 10.1–10.5 | Style normalised | NO-ACTION |
| Req 11.1–11.5 | Req 11.1–11.5 | Style normalised | NO-ACTION |
| Req 12.1–12.3 | Req 12.1–12.3 | Style normalised | NO-ACTION |

---

### 10. `hex-display` ⏳

No changes required — spec is Compliant. Clean numbered criteria, no terminology
violations.

**Delta:** None.

---

### 11. `sequence-numbers` 🔄

All criteria used dot-prefix style throughout all 14 requirements. Normalised to
numbered list. Criterion numbers preserved — all existing test annotations remain
valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.10 | Req 1.1–1.10 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.9 | Req 2.1–2.9 | Style normalised | NO-ACTION |
| Req 3.1–3.9 | Req 3.1–3.9 | Style normalised | NO-ACTION |
| Req 4.1–4.5 | Req 4.1–4.5 | Style normalised | NO-ACTION |
| Req 5.1–5.11 | Req 5.1–5.11 | Style normalised | NO-ACTION |
| Req 6.1–6.12 | Req 6.1–6.12 | Style normalised | NO-ACTION |
| Req 7.1–7.5 | Req 7.1–7.5 | Style normalised | NO-ACTION |
| Req 8.1–8.7 | Req 8.1–8.7 | Style normalised | NO-ACTION |
| Req 9.1–9.6 | Req 9.1–9.6 | Style normalised | NO-ACTION |
| Req 10.1–10.4 | Req 10.1–10.4 | Style normalised | NO-ACTION |
| Req 11.1–11.6 | Req 11.1–11.6 | Style normalised | NO-ACTION |
| Req 12.1–12.5 | Req 12.1–12.5 | Style normalised | NO-ACTION |
| Req 13.1–13.3 | Req 13.1–13.3 | Style normalised | NO-ACTION |
| Req 14.1–14.8 | Req 14.1–14.8 | Style normalised | NO-ACTION |

---

### 12. `tabs-and-mask` 🔄

All criteria used dot-prefix style throughout all 18 requirements. Normalised to
numbered list. Criterion numbers preserved — all existing test annotations remain
valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.10 | Req 1.1–1.10 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.8 | Req 2.1–2.8 | Style normalised | NO-ACTION |
| Req 3.1–3.5 | Req 3.1–3.5 | Style normalised | NO-ACTION |
| Req 4.1–4.7 | Req 4.1–4.7 | Style normalised | NO-ACTION |
| Req 5.1–5.6 | Req 5.1–5.6 | Style normalised | NO-ACTION |
| Req 6.1–6.11 | Req 6.1–6.11 | Style normalised | NO-ACTION |
| Req 7.1–7.4 | Req 7.1–7.4 | Style normalised | NO-ACTION |
| Req 8.1–8.6 | Req 8.1–8.6 | Style normalised | NO-ACTION |
| Req 9.1–9.6 | Req 9.1–9.6 | Style normalised | NO-ACTION |
| Req 10.1–10.6 | Req 10.1–10.6 | Style normalised | NO-ACTION |
| Req 11.1–11.5 | Req 11.1–11.5 | Style normalised | NO-ACTION |
| Req 12.1–12.4 | Req 12.1–12.4 | Style normalised | NO-ACTION |
| Req 13.1–13.7 | Req 13.1–13.7 | Style normalised | NO-ACTION |
| Req 14.1–14.5 | Req 14.1–14.5 | Style normalised | NO-ACTION |
| Req 15.1–15.5 | Req 15.1–15.5 | Style normalised | NO-ACTION |
| Req 16.1–16.4 | Req 16.1–16.4 | Style normalised | NO-ACTION |
| Req 17.1–17.5 | Req 17.1–17.5 | Style normalised | NO-ACTION |
| Req 18.1–18.7 | Req 18.1–18.7 | Style normalised | NO-ACTION |

---

### 13. `asa-report-preview` 🔄

All criteria used dot-prefix style throughout all 12 requirements. Normalised to
numbered list. Criterion numbers preserved — all existing test annotations remain
valid.

**Delta:**

| Old ID | New ID | Change Type | Test Impact |
|--------|--------|-------------|-------------|
| Req 1.1–1.9 | Req 1.1–1.9 | Style normalised (dot-prefix → numbered list) | NO-ACTION |
| Req 2.1–2.7 | Req 2.1–2.7 | Style normalised | NO-ACTION |
| Req 3.1–3.6 | Req 3.1–3.6 | Style normalised | NO-ACTION |
| Req 4.1–4.7 | Req 4.1–4.7 | Style normalised | NO-ACTION |
| Req 5.1–5.6 | Req 5.1–5.6 | Style normalised | NO-ACTION |
| Req 6.1–6.8 | Req 6.1–6.8 | Style normalised | NO-ACTION |
| Req 7.1–7.9 | Req 7.1–7.9 | Style normalised | NO-ACTION |
| Req 8.1–8.8 | Req 8.1–8.8 | Style normalised | NO-ACTION |
| Req 9.1–9.6 | Req 9.1–9.6 | Style normalised | NO-ACTION |
| Req 10.1–10.6 | Req 10.1–10.6 | Style normalised | NO-ACTION |
| Req 11.1–11.9 | Req 11.1–11.9 | Style normalised | NO-ACTION |
| Req 12.1–12.4 | Req 12.1–12.4 | Style normalised | NO-ACTION |

---

### 14. `custom-file-viewers` ⏳

No changes required — spec is Compliant. Clean numbered criteria, no terminology
violations.

**Delta:** None.

---

## Summary

| Task | Specs | Deltas Recorded | Test Annotations to Update | New Tests Needed |
|------|-------|-----------------|---------------------------|-----------------|
| Task 5 | 10 | 18 rows | 0 | 0 |
| Task 6 | 15 | 47 rows | 4 (compiler-toolchain Reqs 15→1, 16→2, 17→3, 18→4) | 4 (NFR sections + deferred connectors) |
| Task 7 | 14 | 97 rows | 0 | 0 |
