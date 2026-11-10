# Design Review -- l10n-ascii-gate (DOCS-ONLY requirements gate)

Reviewer: design-review subagent (fresh context).
Scope: the two separable requirements-gate amendments authored for the
FileForgeWorkbench localization ASCII-scope feature.
Iteration: first review of this gate (no prior design-review.json).

## What was reviewed

- Authoring summary: `.agents/tasks/l10n-ascii-gate/amendments-summary.md`
  (including its Revision 1 note).
- Amended spec: `docs/specs/context-help/requirements.md` (Req 17.9 + 17.9a/b/c).
- Amended spec: `docs/specs/localization/requirements.md` (Req 11.2, 11.6 + 11.11-11.13).
- Tasks: `docs/specs/localization/tasks.md` (Phases 5/6/7) and
  `docs/specs/context-help/tasks.md` (Task 23).
- Coverage record: `docs/quality/TCR.md` (context-help Req 17.9 rows,
  localization Req 11.x rows).
- Change log: `docs/status/change-log.md` (CR-CH-064, CR-CH-065).
- Cross-check source: `crates/ff-help/tests/shipped_content_tests.rs`
  (the `shipped_content_files_are_ascii` test and its `// Validates` annotation).
- Working-tree state via `git status --porcelain`.

## Verification against each required check

### 1. Correct ASCII criterion id (verified against the test, not the guess)

I read `crates/ff-help/tests/shipped_content_tests.rs` directly. The test
`shipped_content_files_are_ascii` carries the annotation:

```
// Validates: Requirement 17.9 -- shipped content files are ASCII.
```

The amendment targets context-help **Requirement 17.9** in place, and adds
17.9a/17.9b/17.9c. This MATCHES the real criterion the test validates. The
sibling criterion 17.8 (`shipped_content_has_no_dangling_cross_references`,
annotated `Requirement 17.8`) was correctly left untouched. The investigation's
"17.8/17.9" guess is resolved to the right one (17.9). PASS.

### 2. Amendments 1 and 2 are cleanly separable

- Amendment 1 (ASCII scope) lives in `docs/specs/context-help/requirements.md`
  (Req 17.9/a/b/c) and is logged as **CR-CH-064**.
- Amendment 2 (diacritic-insensitive resolver) lives in
  `docs/specs/localization/requirements.md` (Req 11.2/11.6/11.11-11.13) and is
  logged as **CR-CH-065**.
- The two change-log entries each carry an explicit separability line
  ("Separable from CR-CH-065 ... approve/defer/reject each independently" and the
  mirror). They touch disjoint requirement sets, disjoint spec files, and
  disjoint task phases (Phase 6 vs Phase 5/7). The owner can approve either
  without the other. PASS.

### 3. EARS form

The amended/added criteria use the WHEN/WHERE ... THE ... SHALL/MAY structure:
- 17.9 "THE shipped ENGLISH help content base ... SHALL contain only ..." (state-driven). OK.
- 17.9a "WHEN a per-locale translated help directory ... THE translated files MAY contain ..." (event + permission). OK.
- 17.9b "THE per-locale message catalogues ... MAY likewise contain ...". OK.
- 11.11 "THE resolver SHALL compare an entered verb ... under a ... FOLD defined as ...". OK.
- 11.12 "WHERE two DISTINCT surface forms ... fold to the SAME value ... THE resolver SHALL PREFER an exact match ...". OK.
- 11.13 "THE DISPLAY surface ... SHALL retain its full correct diacritics ...". OK.

The permission criteria (17.9a/b) legitimately use MAY, which is valid EARS for
an optional/permitted behaviour. 17.9c is written as a NOTE ("NOTE (follow-on,
NOT this gate): ... SHALL narrow that test ...") -- it still carries a SHALL and
is an acceptable deferred-action criterion. PASS.

### 4. Spec prose is plain ASCII

Ran a non-ASCII scan over the four amended spec/tasks files
(`context-help/requirements.md`, `localization/requirements.md`,
`localization/tasks.md`, `context-help/tasks.md`); the scan log is EMPTY -> all
amended spec prose is plain ASCII, including the examples that DESCRIBE allowing
non-ASCII localization content (they describe it, they do not contain it; e.g.
"LEERS written with a circumflex" is spelled out in ASCII). The change-log scan
surfaced only PRE-EXISTING emoji (status glyphs) elsewhere in the file; the new
CR-CH-064/CR-CH-065 entries (lines 428-445) contain no non-ASCII. PASS.

### 5. No source / data / help content changed by this gate

`git status --porcelain` shows the gate's changes confined to `docs/` only. No
`.rs`, no `help/` content, no `i18n/` DATA is modified. The context-help and
localization requirements/tasks, TCR, and change-log are the only gate-relevant
edits. (Other modified docs in the worktree -- global-search, menu-workspace,
bugs.md, etc. -- are from separate concurrent work, and are still docs, not
source/data/help.) The amendment itself repeatedly states it makes NO source or
test change and defers that to a later gated task. PASS.

### 6. Follow-on tasks recorded for both amendments

- Amendment 1 (1a) narrow the ff-help ASCII test to the English base:
  localization Phase 6 **Task 13 / 13.1** and context-help **Task 23 / 23.1**.
  Both carry `[ ]` markers, titles, and cross-references (`Satisfies context-help
  Req 17.9, 17.9a, 17.9c` / `Validates: Requirement 17.9 (amended), 17.9a,
  17.9c`).
- Amendment 1 (1b) note af/fr/de already satisfy: localization Phase 6
  **Task 14** ("Confirm the existing per-locale help files satisfy the amended
  rule (context-help Req 17.9a)"), a no-code NOTE task, valid `[ ]`, titled,
  cross-referenced.
- Amendment 2 (2a) diacritic-insensitive resolver: localization Phase 5
  **Task 12 (12.1-12.5)**, all `[ ]`, titled, each `Satisfies Req 11.x`.
- Amendment 2 (2b) af alias DATA + catalogue keys: localization Phase 7
  **Task 15 (15.1-15.3)**, all `[ ]`, titled, `Satisfies Req 11.5, 11.7, 11.8,
  11.13`.

All follow-on task lines use only `[ ]`/`[x]` markers, have titles, and
cross-reference the amended criteria. PASS.

The Revision-1 fix (misplaced context-help Task "16"/"16.1" collision) is
confirmed resolved: the follow-on is now Task 23/23.1 at the end of the list in a
dedicated CR-NR-103/CR-NR-109 section with its own coverage-mapping table, and
the original CR-NR-097 Task 16 (content authoring) and its `AC 17.1-17.9 | Task
16` rows are intact. No numbering collision remains, and the `AC 17.9 (amended) |
Task 23` mapping is unambiguous.

### 7. In-place edits noted; contradiction surfaced not resolved

- 17.9 carries an "AMENDED by CR-NR-103 / CR-NR-109" note explaining the
  narrowing and stating no decision about the English base is reversed.
- 11.2 and 11.6 each carry explicit "AMENDED by CR-NR-109" notes; 11.2 adds a
  "FLAG: this is a BEHAVIOUR CHANGE to the resolver ... realised by a LATER GATED
  IMPLEMENTATION TASK" line.
- The already-shipped CR-NR-109 Phase 4 resolver (Req 11.2 exact-match) is
  surfaced as a contradiction in the authoring summary and the change-log
  CR-CH-065 entry, handled conservatively: the existing exact-match TCR rows stay
  PASS (exact match remains a subset of folded match), and the new folded
  behaviour is carried as NOT COVERED rows (11.11-11.13, marked with the red
  glyph) pending the owner-approved Phase 5 Task 12. The decision is flagged for
  the owner, not unilaterally rewritten. PASS.

## Findings

No HIGH or MEDIUM findings. The following NITs do not block.

1. NIT -- `docs/quality/TCR.md`, context-help Req 17.9 ASCII row: the row marks
   the test row with the pass glyph while its own NOTE states the test "currently
   also walks per-locale subdirs and so is inconsistent with the amended
   17.9/17.9a." The row is accurate (the test as written still passes today
   because no non-ASCII per-locale file has yet tripped it, and Task 23 will
   narrow it), but a reader skimming only the glyph could miss that the amended
   17.9a is not yet enforced. Concrete fix (optional): append "(enforced for the
   English base; per-locale UTF-8 enforcement lands with Task 23)" to the glyph
   row so the status is unambiguous at a glance. Non-blocking.

2. NIT -- Amendment 1 scope note on `documentation-charset.md`: the authoring
   summary states the global `documentation-charset.md` steering file is not
   present in this workspace checkout and was therefore not edited, relying on
   its existing "deliberate runtime UI strings are OUT of scope" exception.
   That reasoning is sound for a docs-only gate, but it means the ASCII-scope
   relaxation is currently expressed only in the context-help spec, not in the
   house-rule file itself. Concrete fix (optional, owner-directed): if the owner
   wants the carve-out visible in the steering rule, open a separate user-space
   edit where that file actually lives -- out of scope for this workspace gate.
   Non-blocking.

## Verified Assumptions

- The test `shipped_content_files_are_ascii` validates Requirement 17.9 (read
  from the `// Validates` annotation in the actual test file). Confirmed.
- The ASCII test currently recurses into per-locale subdirectories (the
  `visit_help_files` helper recurses unconditionally and matches any
  `*.help.md`), which is exactly the inconsistency Task 23 / Phase 6 Task 13 is
  written to fix. Confirmed by reading the test source.
- CR-CH-064 and CR-CH-065 are distinct change-log entries with explicit
  mutual-separability statements. Confirmed.
- The four amended spec/tasks files contain no non-ASCII characters (empty scan
  log). Confirmed.
- No `.rs`, `help/`, or `i18n/` file is modified in the working tree (git status).
  Confirmed.
- Follow-on tasks 12, 13, 14, 15 (localization) and 23 (context-help) exist with
  valid `[ ]` markers, titles, and criterion cross-references. Confirmed by
  reading both tasks files.
- The TCR carries 17.9 (amended, annotated), 17.9a/b/c rows, a 11.2 annotated
  row, and 11.11/11.12/11.13 NOT-COVERED rows. Confirmed by reading the rows.

## Unverified / Wrong Assumptions

- None material. The investigation's original "17.8/17.9" uncertainty about the
  criterion id was checked against the test and resolved correctly to 17.9; the
  amendment did not rely on the wrong id.

## Verdict

No HIGH or MEDIUM findings (2 NITs, non-blocking). The gate correctly amends the
right criterion in place, keeps the two amendments separable, preserves EARS
form, keeps all spec prose ASCII, changes no source/data/help, records the
follow-on tasks with valid markers and cross-references, and surfaces the
already-shipped-resolver contradiction for owner review rather than resolving it
unilaterally.

APPROVED.
