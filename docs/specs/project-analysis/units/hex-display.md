# Analysis Record: hex-display (W4.6)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-hex` (spec says `ff-hex-display`; actual dir is `ff-hex` --
  naming drift)
- **Spec files**: requirements.md (342 lines, 16 requirements), tasks.md
  (169 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 16 reqs, 342 lines. One cohesive concern (hex view/edit:
three-pane layout, editing, search, cursor sync, undo, scrolling, export, goto,
session, compatibility). 15 files. No split.

### Source file size violation (PA-STD-048)

`controller.rs` = 417 non-test lines, just over the 400 cap (the hex-mode
controller orchestrating layout/editing/cursor-sync/scroll). Split by concern
(view/layout vs editing vs cursor+scroll). REFACTOR, no gate. Recorded PA-STD-048.
(Only file over cap.)

---

## 2. Cross-unit consistency

### PA-CONFLICT-002 (transaction model) -- CLEAN SEAM (trait-decoupled)

Req 4 (Hex Editing overwrite) + Req 7 (Undo/Redo in Hex Mode): hex editing mutates
bytes and must integrate with undo. It does so WITHOUT binding to a concrete
transaction type:
- `controller.rs` defines a `ByteReader` trait for read-only byte access (injection
  seam -- the document buffer is supplied).
- `editing.rs` "produces edit actions for the undo/redo system", each storing the
  previous byte value (for undo).
So hex-display RETURNS edit-action data (old/new byte); the CALLER wraps it in the
transaction (EditorTransaction or undo-redo Transaction). Same clean-seam pattern as
sequence-numbers (W1.9) / auto-indentation (W1.13) -- it stays OUT of PA-CONFLICT-002.
Deps: ff-logging ONLY (minimal-dep + injection, like ff-tabmask/ff-select). Recorded
as a CLEAN row.

### FIND X'...' hex search -- consumes find-and-replace

Req 5 (Hex Search Integration): FIND X'hh...' hex-literal search. find-and-replace
(W1.8) owns the FIND engine + hex-literal tokenisation (X'...' is a Command_Token in
command-semantics W3.1 Req 3). hex-display integrates the results into the hex view.
Consumer relationship; consistent.

### FileForge structured files (Req 14) -- coexists with ff-forge

Req 14 (Hex Mode with FileForge Structured Files): hex view of the raw bytes behind
a structured/record file. hex-display shows raw bytes; ff-forge (fileforge-integration)
owns the record model. `editing.rs:122` warns "Hex editing on EBCDIC files modifies
raw bytes directly" -- a correct boundary note (hex bypasses the record codec).
Coexists cleanly; confirm the hex-vs-grid mode switch at fileforge-integration (Wave 5).

### Public types and ownership

- Hex controller, ByteReader trait, hex edit actions, layout (offset/hex/ASCII
  panes), dump export, goto-offset, session state -- sole-owned by `ff-hex`. No
  duplication. 0 fs (operates on the byte buffer via ByteReader; no file I/O).
- HEX command (Req 1) + Goto Offset (Req 12) via command-framework (Command_Target).
  Session state (Req 15) via the ff-session contract. Consistent display-mode-command
  pattern (like line-wrap W1.14 / view-zoom W4.5).

### Cross-reference integrity

Cross-refs (document-model, find-and-replace, undo-redo, command-framework,
fileforge-integration, startup-and-session) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 169 sub-tasks `[x]`. Implementation present across all 16 reqs
(HEX commands, layout, bytes-per-row, editing, hex search, cursor sync, undo,
modified-byte indicators, scrolling, binary/text, dump export, goto, uppercase/
lowercase, FileForge structured, session, compatibility). Tests in-file. No
PA-INCOMPLETE. Complete.

### TCR gap (PA-TCR-022)

TCR.md has 1 row for ff-hex against 16 reqs / ~110 criteria. Thin. Recorded
PA-TCR-022.

---

## 4. Logging audit

Scan of `crates/ff-hex/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT (sole dep) -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 0

Pure byte-view/edit model -- zero-log largely defensible (invalid hex input +
goto-out-of-range surface via error/status). Recorded PA-LOG-032 (LOW): resolve the
dead dep; optional dev-logging on hex edit / goto / dump-export under the
`dev-logging` gate.

---

## 5. Task revision proposals

- **PA-STD-048 (REFACTOR)**: split `controller.rs` (417 non-test, just over cap) by
  concern. No gate.
- **PA-STD-049 (ASCII, ACTIONABLE -- box-drawing + em-dash in RUNTIME strings)**:
  59 non-ASCII bytes (3 non-comment). NOTABLE: `layout.rs:9` + `dump.rs:82` use a
  box-drawing vertical bar (U+2502) as a runtime SEPARATOR constant (`SEPARATOR =
  " | "` shown with U+2502) in the hex-dump output -- box-drawing is ALLOWED in
  `.md` but PROHIBITED in `.rs` per documentation.md, and this one is in RUNTIME
  output (the hex-dump pane separator). Plus an em-dash in a runtime warning string
  (editing.rs:122). Replace the box-drawing separator with an ASCII `|` and the
  em-dash with `--`. REFACTOR, no gate. (Design choice: the hex-dump pane separator
  currently renders a box-drawing bar; ASCII `|` is the doc.md-compliant substitute.)
- **PA-LOG-032 (LOW)**: resolve dead ff-logging dep; optional dev-logging on hex
  edit/goto/export.
- **PA-TCR-022**: enumerate per-requirement TCR rows (1 row for 16 reqs). No code.
- **PA-DOC (naming)**: spec crate `ff-hex-display`; actual `ff-hex`.

No requirement CHANGE proposed. Confirm the hex-vs-grid mode switch with ff-forge at
Wave 5 (fileforge-integration).

---

## Summary

hex-display (`ff-hex`, spec says `ff-hex-display`) is a cohesive, complete hex
view/edit subsystem (three-pane offset/hex/ASCII, editing, search, cursor sync,
undo, export, goto). It stays OUT of PA-CONFLICT-002 via the clean-seam pattern:
a `ByteReader` injection trait + editing that RETURNS undo edit-action data (old
byte) for the caller to wrap -- deps are ff-logging only (like sequence-numbers/
auto-indent). Consumes find-and-replace for FIND X'...' hex search; coexists with
ff-forge for structured-file raw-byte view (correct raw-bytes-bypass-codec boundary
note). Findings: controller.rs just over the cap (PA-STD-048); a NOTABLE ASCII item
(PA-STD-049) -- a box-drawing bar U+2502 used as the RUNTIME hex-dump pane separator
(prohibited in `.rs`) plus an em-dash in a runtime warning; dead ff-logging sole-dep
(PA-LOG-032, LOW); thin TCR (PA-TCR-022); and the ff-hex/ff-hex-display naming drift.
No conflict, no split.
