# Analysis Record: syntax-highlighting (W1.10)

- **Wave**: 1 (Editor Core)
- **Backing crate**: `ff-syntax-highlighting`
- **Spec files**: requirements.md (332 lines, 16 requirements), tasks.md
  (22 top-level tasks / 185 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 1 re-baseline pass (post CR-NR-057 reset)

---

## 1. Split candidacy

WATCH -- not a forced spec split, but one file EXCEEDS the size cap (see below).

Spec-level split assessment against the 2-of-4 rule:

- Requirement volume: 16 requirements, 332 lines. Above the 12-req line. (1 signal)
- Responsibilities: cohesive lexing engine, but with several distinct internal
  layers: lexer trait+registry (Req 1,13), style buffer+incremental re-highlight
  (Req 2,3,4), keyword/WordList (Req 5), comment/multi-line state (Req 6),
  sub-styles (Req 7), fold-level assignment (Req 8), idle styling (Req 9),
  properties (Req 10), StyleContext helper (Req 14), HILITE command (Req 16).
  These are layers of one engine, already cleanly separated into submodules
  (engine/ fold/ keywords/ lexer/ state/ style/). Borderline on the
  responsibility count but they form a single tokenization pipeline. (partial)
- Crates: single crate. GUI-independent (Req 11 verified: 0 egui/eframe refs). No
  second crate embedded.
- Cohesion: high within the engine; HILITE (Req 16) is the one arguably-separable
  concern (command-layer, not lexing), but it is small and depends on the engine.

Net: 1 firm signal (req volume) + borderline responsibilities. Below a confident
2-of-4. NO spec split proposed. The module structure already does the internal
decomposition well.

### Source file size violation (PA-STD-014) -- ACTIONABLE

Non-test line counts (400-line cap is NON-TEST lines only):

| File | non-test | status |
|------|----------|--------|
| `engine/highlight_engine.rs` | 462 | **EXCEEDS 400 -- must split** |
| `hilite.rs` | 387 | WATCH (near cap) |
| `types.rs` | 226 | ok |
| `fold/context.rs` | 198 | ok |
| `style/sub_styles.rs` | 168 | ok |
| all others | <=150 | ok |

`highlight_engine.rs` at 462 non-test lines is over the hard cap. rust-standards.md
requires a split before adding more. Split by concern: e.g. style-buffer
management vs incremental re-highlight driver vs ensure_styled_to/demand-driven
vs lexer-binding lifecycle. REFACTOR (no gate; no observable behaviour change).
Recorded PA-STD-014. `hilite.rs` (387) is a WATCH -- if HILITE modes grow, split
command parsing from the LOGIC/PAREN scanners.

---

## 2. Cross-unit consistency

### Fold-level ownership boundary vs display-line-mapping -- CONFIRMED CLEAN

This was the open PA-WATCH from W1.4 (display-line-mapping Req 10.7 vs
syntax-highlighting Req 8). Resolved:

- syntax-highlighting OWNS fold-level COMPUTATION: `Lexer::fold_text` +
  `FoldContext::set_level`, stored per-line, exposed via `fold_level_at(line)` and
  `fold_level_range(start,end)` (Req 8.1-8.8, 15.6). `FoldData` store confirmed in
  `fold/store.rs`.
- display-line-mapping does NOT compute fold levels. Its 18 `fold_*` references are
  ALL `fold_text` = the collapsed-region DISPLAY LABEL string (the placeholder text
  shown when a fold is collapsed), a HashMap<line, String> in `contraction_state.rs`.
  No `FoldLevel` type usage, no `FoldContext`, no level computation.
- This exactly matches Req 15.1 ("display-line-mapping SHALL query fold levels
  exclusively from syntax-highlighting ... SHALL NOT compute fold levels
  independently") and display-line-mapping Req 10.7 (stores only
  visibility+expanded+label).

Clean single-owner boundary. No conflict. The W1.4 PA-WATCH on fold-level
ownership is now RESOLVED. Consistency matrix row updated from "confirm at W1.10"
to CONFIRMED.

Naming note: both specs use the identifier `fold_text` for DIFFERENT things --
syntax-highlighting `Lexer::fold_text()` (a method that COMPUTES levels) vs
display-line-mapping `fold_text` (a FIELD holding the collapsed label). Same
spelling, different layers, no code collision (different crates/types). Recorded
as a low-priority naming-clarity note, not a conflict.

### Other public types and ownership

- `StyleSlotIndex`, `LexerState`, `BytePosition`, `LineNumber`, `HighlightSpan`,
  `FoldFlags`, `FoldLevel`, `KeywordSetDescriptor`/`Index`, `PropertyDescriptor`,
  `PropertyType`, `SyntaxHighlighter` (trait), `Lexer` (trait), `LexerRegistry`,
  `StyleBuffer`, `StyleContext`, `WordList`, `SubStyleAllocator`/`SubStyleRange`,
  `HighlightEngine`, `FoldData`, `FoldContext`, `IdleStylingConfig`/`Result` --
  all sole-owned by `ff-syntax-highlighting`. No duplication found.
- Theme integration (Req 12): engine emits only `StyleSlotIndex` (u8); NO colour
  values (Req 12.1 verified -- 0 colour refs). theme-and-appearance resolves slots
  to attributes. Clean layering, consistent with theme ownership.
- text-decorations peer (Req 15.3-15.5): style data and indicator data are
  independently queryable; re-highlight does not touch decorations. Consistent
  (verify fully at text-decorations W1.15).
- HILITE command (Req 16) registered with command-framework; HILITE FIND extends
  find-and-replace highlight-all (PAR). Cross-crate command; consistent with
  command ownership. Note edit-operations also has a HILITE row (Req 16.12,
  TCR line 1256) that "parses and stores, delegates" -- so HILITE is split across
  edit-operations (parse/store) and syntax-highlighting (execute modes). Recorded
  as a WATCH (PA-WATCH) to confirm the delegation seam at Wave 3/4 (command wiring).

### Cross-reference integrity

All 8 declared cross-refs (language-service, document-model, theme-and-appearance,
display-line-mapping, text-decorations, idle-processing, configuration-system,
+ command-framework for HILITE) resolve to existing sub-projects. No dangling refs.

---

## 3. Completeness

Tracking: all 22 top-level / 185 sub-tasks `[x]`. Unlike an earlier concern, the
engine IS fully implemented -- the crate has 21 source files across submodules
(engine/, fold/, keywords/, lexer/, state/, style/). property_tests.rs exercises
the real modules (StyleBuffer length invariant, keyword case-fold lookup,
ensure_styled_to idempotency, sub-style non-overlap, fold-header rule, span
coverage). No PA-INCOMPLETE raised -- implementation matches tasks/reqs, no
false-positive test pattern (contrast background-io PA-INCOMPLETE-002).

### TCR gap (PA-TCR-004)

TCR.md enumerates ONLY Req 16 (HILITE, Phase CF: rows for Req 16.1-16.5) plus one
edit-operations HILITE row. Reqs 1-15 -- the entire engine (lexer, style buffer,
incremental re-highlight, keywords, comments, sub-styles, fold levels, idle
styling, properties, StyleContext) -- have NO TCR rows despite property_tests +
per-module unit tests existing. TCR RECORDING gap (not a test gap). Same pattern
as PA-TCR-002/003. Recorded PA-TCR-004: add per-requirement rows citing the
existing property/unit tests.

---

## 4. Logging audit

Scan of `crates/ff-syntax-highlighting/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0 (pure engine)

Zero-log is DEFENSIBLE for this GUI/IO-independent engine -- consistent with the
other pure Wave 0/1 models. BUT Req 10.7 SPECIFIES a mandated log:

- Req 10.7: unknown lexer property key -> "log a DEBUG-level message and store the
  value without error". Currently the code stores forward-compatibly but emits NO
  log record.

This is a minor spec-vs-impl gap and a natural CR-NR-058 dev-logging home
(DEBUG-level, exactly the dev/debug tier). Recorded PA-LOG-007 (LOW/dev-logging):
wire ff-logging for the Req 10.7 DEBUG record; incremental-rehighlight and
idle-styling progress are good additional dev-logging trace points under the
`dev-logging` gate once logging Req 13 (PA-CR058) lands.

---

## 5. Task revision proposals

- **PA-STD-014 (REFACTOR, ACTIONABLE)**: split `engine/highlight_engine.rs` (462
  non-test lines, over the 400 cap). Split by concern (buffer mgmt / incremental
  re-highlight / demand-driven ensure_styled_to / lexer lifecycle). No gate, no
  behaviour change. `hilite.rs` (387) is a WATCH within the same item.
- **PA-STD-015 (ASCII, minor)**: 17 non-ASCII bytes in `.rs` (em/en dashes in doc
  comments -- e.g. lib.rs "0-255", "GUI-independent -- it produces"). documentation.md
  mandates strict ASCII in `.rs`. Replace with `--`/`-`. REFACTOR, no gate.
- **PA-TCR-004**: enumerate per-requirement TCR rows for Reqs 1-15 (no code).
- **PA-LOG-007**: wire ff-logging for the mandated Req 10.7 DEBUG record + optional
  dev-logging trace points (owner/logging-Req-13 gated).
- **PA-WATCH (HILITE delegation)**: confirm the edit-operations(parse/store) ->
  syntax-highlighting(execute) HILITE seam at Wave 3/4 command wiring.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

syntax-highlighting is a complete, fully-tracked, GUI-independent lexing engine
with a well-decomposed module structure. The W1.4 open question -- fold-level
ownership -- is RESOLVED as a clean single-owner boundary (syntax-highlighting
computes levels; display-line-mapping only stores collapsed-label text and
consumes levels via the public API; the shared `fold_text` spelling is
coincidental across crates). Not a spec split candidate. One actionable
standards item: `highlight_engine.rs` exceeds the 400-line cap (PA-STD-014,
REFACTOR). Remaining items are minor: ASCII cleanup (PA-STD-015), TCR enumeration
(PA-TCR-004), a mandated DEBUG log (PA-LOG-007), and a HILITE delegation WATCH.
