# l10n-ascii-gate -- Authoring Summary

Requirements-gate (DOCS ONLY) authoring for two separable amendments. First
iteration (no prior `design-review.json`). Plain ASCII throughout.

## Confirmed ASCII criterion id + test annotation

- The failing test is `shipped_content_files_are_ascii` in
  `crates/ff-help/tests/shipped_content_tests.rs`.
- Its `// Validates:` annotation reads:
  `// Validates: Requirement 17.9 -- shipped content files are ASCII.`
- Confirmed against `docs/specs/context-help/requirements.md`: Requirement 17
  ("Shipped Help Content Set", CR-NR-097), criterion 17.9 is the ASCII constraint.
  (The investigation's "17.8/17.9" guess was close: 17.8 is the no-dangling-xref
  criterion; 17.9 is the ASCII one. The amended criterion is 17.9.)

## Amendment 1 -- ASCII policy scope (LEAD, unblocks the red gate)

In-place amendment of context-help Req 17.9 (NOTED, not silently rewritten):

- 17.9 NARROWED to the ENGLISH `help/` base (which doubles as authored project
  docs) + Rust source stays ASCII; amendment note records why and that no
  decision about the English base is reversed.
- 17.9a ADDED: per-locale `help/<locale>/` translated `.help.md` MAY contain
  non-ASCII UTF-8 (explicitly permits existing `help/af|fr|de/index.help.md`).
- 17.9b ADDED: per-locale `.ftl` catalogues + `i18n/<locale>/aliases/*.toml` DATA
  MAY be non-ASCII UTF-8; `i18n/en/` and the loading Rust source stay ASCII.
- 17.9c ADDED: records the follow-on SCOPED code/test change (narrow the test to
  the English base, exclude per-locale subdirs) that this gate does NOT make.

## Amendment 2 -- Diacritic-insensitive command input (SEPARATE)

In-place amendment of localization Req 11.2 and Req 11.6 (both NOTED) + new
criteria:

- Req 11.2 amended: resolved localised string is the display surface AND an
  accepted typeable form (no longer the ONLY typeable form); flagged as a
  behaviour change realised by a later gated task.
- Req 11.6 amended: each lookup step performed under the diacritic-insensitive
  fold (Req 11.11); order + unknown-command fall-through unchanged; exact match a
  subset of folded match.
- Req 11.11 ADDED: the fold (NFD -> strip combining marks -> ASCII case fold),
  applied to both sides; covers every word with no extra data.
- Req 11.12 ADDED: fold-collision handling (prefer exact; ambiguous folded input
  -> diagnostic, no silent resolution).
- Req 11.13 ADDED: display keeps full diacritics (fold affects input only);
  optional explicit ASCII alias only where a genuinely different ASCII spelling
  is needed.

## Contradiction surfaced (NOT resolved unilaterally)

CR-NR-109 Phase 4 (localization Req 11.1-11.10) is ALREADY IMPLEMENTED and its
TCR rows are PASS, including the Req 11.2 EXACT-match resolver and the
"one source of truth / only typeable surface" wording. Amendment 2 is a BEHAVIOUR
CHANGE to that already-shipped resolver. Handled conservatively, for owner review:

- Req 11.2/11.6 amended with explicit NOTES rather than rewriting the shipped
  decision.
- The existing exact-match test stays valid (exact match still resolves; the fold
  is a superset), so no PASS row is invalidated.
- The new folded-input behaviour is a LATER GATED IMPLEMENTATION TASK (localization
  Phase 5 Task 12); its TCR rows (Req 11.11-11.13) are NOT COVERED.
- The Req 11.2 TCR row is annotated to explain the relaxation and point at the
  NOT-COVERED rows.

This is flagged for the owner (change-log CR-CH-065 "FLAG" line) rather than
treated as settled.

## documentation-charset.md note

The `documentation-charset.md` steering file is referenced by
`.amazonq/rules/steering-pointer.md` and `.kiro/steering/rust-standards.md`, but
it does NOT exist as a file in this workspace checkout (it is a global/user-space
steering rule, delivered as an injected rule, not a workspace file). It was
therefore NOT edited. This is acceptable: its EXISTING "deliberate runtime UI
strings are OUT of scope" exception already covers per-locale translated
localization content, which is the basis for Amendment 1. If the owner wants the
scope exception spelled out explicitly in that global steering file, that is a
user-space edit outside this workspace-scoped gate and should be made where the
file actually lives.

## Files touched (docs only)

| File | Change |
|------|--------|
| `docs/specs/context-help/requirements.md` | Req 17.9 amended in place; 17.9a/17.9b/17.9c added (Amendment 1) |
| `docs/specs/localization/requirements.md` | Req 11.2 + 11.6 amended in place; Req 11.11-11.13 added (Amendment 2) |
| `docs/specs/localization/tasks.md` | Phase 5 (Task 12 diacritic resolver), Phase 6 (Task 13-14 ASCII test narrow + af/fr/de help note), Phase 7 (Task 15 af alias DATA + catalogue keys) |
| `docs/specs/context-help/tasks.md` | Task 23 (narrow `shipped_content_files_are_ascii` to English base) + its own CR-NR-103/109 coverage-mapping rows -- see Revision 1 below |
| `docs/quality/TCR.md` | Req 17.9 row annotated + 17.9a/17.9b/17.9c rows; Req 11.2 row annotated + Req 11.11-11.13 NOT COVERED rows |
| `docs/status/change-log.md` | CR-CH-064 (ASCII policy scope) + CR-CH-065 (diacritic-insensitive input) added under Change Requests |

NOT edited (correctly): any `.rs` source, `help/` content, per-locale alias/
catalogue DATA, and `documentation-charset.md` (not in this workspace).

## Follow-on tasks recorded (NOT done here)

- Amendment 1 (1a): localization Phase 6 Task 13 / context-help Task 23 -- narrow
  `shipped_content_files_are_ascii` to the English `help/` root, exclude per-locale
  subdirs, add a valid-UTF-8 assertion. Resolves B085 / the red gate.
- Amendment 1 (1b): localization Phase 6 Task 14 -- NOTE that `help/af|fr|de/
  index.help.md` already satisfy the amended rule; no edit needed.
- Amendment 2 (2a): localization Phase 5 Task 12 -- implement the diacritic-
  insensitive resolver fold + fold-collision diagnostic.
- Amendment 2 (2b): localization Phase 7 Task 15 -- author the `af` menu-option
  alias DATA + the `verb-*` catalogue keys (en identity + af surfaces).

## Verification

- Scanned the four amended spec/tasks files for non-ASCII (`[^\x00-\x7F]`): the
  log `ascii-scan.txt` is EMPTY -> all amended spec prose is plain ASCII.
- TCR additions use the file's pre-existing permitted status emoji convention
  plus ASCII prose.
- No source, test, help content, or locale DATA was changed.

## Revision 1 (design-review CHANGES_REQUESTED -- 1 MEDIUM)

Finding: the Amendment-1 follow-on task had been inserted into
`docs/specs/context-help/tasks.md` as a SECOND top-level task `16` (with `16.1`),
colliding with the existing CR-NR-097 content-authoring Task 16 / 16.1 and sitting
out of numeric order between tasks 13 and 14. This made the `AC 17.9 | Task 16`
coverage mapping ambiguous.

Fix applied (docs only):

- Removed the misplaced `16`/`16.1` block from between tasks 13 and 14 in
  `docs/specs/context-help/tasks.md`.
- Re-added it as top-level task `23`/`23.1` in a new
  "CR-NR-103 / CR-NR-109" section at the END of the task list (after task 22),
  preserving numeric order (1-22, then 23).
- Added a DISTINCT CR-NR-103/109 coverage-mapping table pointing AC 17.9 (amended),
  17.9a, and 17.9c at Task 23 -- separate from the existing `AC 17.9 | Task 16`
  content-authoring row, so traceability is unambiguous.
- Updated the dependent pointers from "Task 16" to "Task 23":
  `docs/quality/TCR.md` (the 17.9 annotation row and the 17.9a row) and
  `docs/status/change-log.md` CR-CH-064 "Affects" line.
- Confirmed the localization Phase 6 Task 13 NOTE references context-help by
  criterion id (17.9c), not by task number, so no change was needed there.
- The original CR-NR-097 Task 16 ("Author the shipped help content set") and its
  `AC 17.1-17.9 | Task 16` coverage rows are left intact.
