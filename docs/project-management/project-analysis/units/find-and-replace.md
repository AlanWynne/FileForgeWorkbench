# Analysis Record: find-and-replace

- **Wave**: 1 (core editing/model)
- **Backing crate**: `ff-find-and-replace`
- **Spec folder**: `docs/specs/find-and-replace/`
- **Analysed**: Wave 1, task W1.8 (CR-NR-057/058 re-baseline)
- **Verdict**: COMPLETE tracking (185/185 tasks `[x]`) but with THIN TCR coverage
  (1 row for 20 reqs), a new CONFLICT (duplicate CaseFolder -- PA-CONFLICT-003),
  two large 400-cap refactors (regex.rs 993, engine.rs 838), and a SPLIT CANDIDATE
  (20 reqs, 437 lines -- regex engine is the natural split). Zero-log defensible.
- **CR-NR-057/058 impact**: NONE directly (find specs untouched).

---

## 1. Scope summary

`ff-find-and-replace` is the GUI-independent search/replace engine: ISPF FIND/
RFIND/CHANGE/RCHANGE with literal/regex/hex modes, direction (NEXT/PREV/FIRST/
LAST/ALL), scope (TAGGED/EXCLUDED/VISIBLE/NONTAGGED), column/bounds restriction,
Unicode case folding, whole-word/word-start, an NFA regex engine (group capture
\1-\9, backreferences, lazy/greedy), find-state persistence, incremental search,
highlight-all. 20 requirements, 437 req lines, single backing crate.

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 437 lines AND 20 reqs | YES (both) |
| 3+ distinct responsibilities | find/change command engine; NFA regex engine (Req 4,8,12); case folding (Req 10); incremental search + highlight + find-state (Req 13,14) | YES |
| 2+ crates | one crate | No |
| low-cohesion clusters | the NFA RegexEngine (regex.rs 993) is a large self-contained component separable from the FIND/CHANGE command engine | YES |
| file-size pressure | `regex.rs` 993 and `engine.rs` 838 far over the cap | YES |

4 criteria. **SPLIT CANDIDATE** (PA-SPLIT-007). The NFA RegexEngine (Req 12 +
regex.rs) is the clearest split: a self-contained `ff-regex` crate that
find-and-replace consumes. Owner-gated. (Also relates to the CaseFolder
duplication -- see PA-CONFLICT-003: case folding should come from ff-encoding, not
be re-implemented here.)

## 3. Consistency / conflict -- PA-CONFLICT-003 (NEW)

- Public types owned here (FindEngine, FindRequest, FindResult, SearchMode,
  SearchDirection, SearchScope, RegexEngine, CaptureGroup, FindState,
  SubstitutionTemplate) -- sole owner. Added to consistency-matrix.
- **PA-CONFLICT-003 (NEW -- duplicate CaseFolder)**: `ff-find-and-replace` defines
  its OWN `CaseFolder` (case_folder.rs:25) and does NOT depend on `ff-encoding`
  (Cargo grep 0; never imports `ff_encoding`/`ICaseConverter`, grep 0). BUT
  `ff-encoding` (W0.19) is the declared sole owner of `CaseFolder` +
  `ICaseConverter` trait + `case_convert`/`case_convert_string` (encoding Req 10),
  and this crate's OWN cross-reference says "encoding-and-characters -- Unicode
  case folding ... depend[s] on encoding awareness" and its glossary says
  CaseFolder "Replaces Scintilla's CaseFolder". So there are TWO independent,
  unbridged CaseFolder implementations. Owner decision: consume
  `ff-encoding::CaseFolder` (via the `ICaseConverter` trait -- exactly what that
  trait was made for) and delete the local one, OR document the local one as
  intentional. Same conflict pattern as PA-CONFLICT-001/002.
- Bounds (Req 2.5, 7.5): find-and-replace CONSUMES the active Bounds owned by
  navigation-commands (Req 5.15, single owner confirmed W1.7). Consistent.
- CHANGE undo (Req 7.7, 9.6): wrapped in an undoable Transaction -- relates to
  PA-CONFLICT-002 (which transaction model). Verify it uses `EditorTransaction` at
  task-revision (Wave 4 shell confirmation).
- Line visibility/tag scope (Req 2) reads the `excluded`/`tagged`/`visible` flags
  owned by display-line-mapping / exclude-show-filter. Consistent (single readers).
- Char classification for WORD/WORDSTART (Req 11.3) uses document-model /
  encoding-and-characters classification. Consistent.

## 4. Completeness -- TCR thin

- Tasks: 185 `[x]`, 0 `[ ]`. Tracked complete.
- TCR: only **1** `ff-find-and-replace` row ("Search, replace, scope filters,
  regex") for a 20-requirement crate. The crate HAS a property_tests.rs (cited in
  the row) + lib unit tests, so it is tested, but the TCR does not enumerate the
  20 requirements' criteria. TCR-coverage GAP (not a code gap). Logged PA-TCR-002.
- Completeness verdict: **feature-tracked complete; TCR under-enumerated.**

## 5. Logging audit

- Zero log call sites; `ff-logging` declared (wired, unused). DEFENSIBLE: pure
  search/replace engine; not-found / invalid-hex / invalid-regex surface via
  FindResult / Result + status messages (Req 1.7, 3.2/3.3, 4.11), not logs. No
  requirement mandates a log record.
- CR-NR-058 relevance: search execution + regex compile would be good
  `log_trace!`/`log_debug!` candidates under the `dev-logging` gate (especially the
  incremental-search time budget Req 14.1); optional, folds into PA-LOG-002 re-scope.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (defensible zero-log for a pure engine)**.

## 6. Findings logged

- **PA-CONFLICT-003** (CONSISTENCY -- duplicate CaseFolder): ff-find-and-replace
  defines its own `CaseFolder` and does not consume `ff-encoding::CaseFolder`
  (encoding Req 10 owner) despite its own cross-ref stating dependence on
  encoding-and-characters. Two unbridged implementations. Owner decision: consume
  `ff-encoding::CaseFolder` via the `ICaseConverter` trait (delete the local one)
  OR document the duplication as intentional. Code change + owner decision.
- **PA-SPLIT-007** (SPLIT PROPOSAL): 20 reqs / 437 lines / 4 criteria. Extract the
  NFA RegexEngine (Req 12 + regex.rs 993) into a self-contained `ff-regex` crate;
  find-and-replace consumes it. Owner-gated.
- **PA-STD-011** (REFACTOR -- 400-line cap): `regex.rs` (993) and `engine.rs` (838)
  are far over the cap -- the two largest offenders found so far. Split by concern
  (regex: compile / execute / classes; engine: find / change / state). REFACTOR,
  no gate (the regex split may fold into PA-SPLIT-007).
- **PA-TCR-002** (TCR-GAP -- coverage enumeration): only 1 TCR row for 20 reqs.
  The crate is tested (property_tests.rs + unit tests) but TCR does not enumerate
  the criteria. Add per-requirement TCR rows citing the existing tests. No code.
- **PA-CONFLICT-002 (relates)**: CHANGE undo (Req 7.7/9.6) uses the transaction
  API; verify `EditorTransaction` at task-revision.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-find-and-replace contributes
  matches (regex/case-fold examples, doc em-dashes). Note: some are legitimate
  Unicode test/regex-example literals -- the project-wide cleanup must exclude
  genuine test/pattern data. Rolled into PA-LOG-001 with this caveat.
