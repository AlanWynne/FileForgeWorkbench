# Design Review -- l10n-ascii-gate CORRECTED Amendment 1 (r2)

Reviewer: design-review subagent (fresh context).
Subject: DOCS-ONLY requirements gate, CORRECTED Amendment 1 (CR-CH-064), plus
Amendment 2 (CR-CH-065) separability check.
Correction summary reviewed: `.agents/tasks/l10n-ascii-gate/amendments-summary-r2.md`.

## Verdict

APPROVED. No HIGH or MEDIUM findings. Two NITs recorded below; neither blocks.

The r2 iteration addressed the prior MEDIUM (code-ahead-of-criteria) by surfacing
it to the owner, who chose Option B (accept the landed test relaxation and
reconcile the docs). The docs now match the realised code state, every required
amendment is present, EARS-form, and ASCII-clean, and Amendment 2 remains cleanly
separable.

---

## Checklist results

### 1. Correct ASCII criterion id, amended IN PLACE, edit NOTED -- PASS
- Confirmed criterion id is context-help **Requirement 17.9**, cross-checked
  against `crates/ff-help/tests/shipped_content_tests.rs`: the relevant test
  carries `// Validates: Requirement 17.9 (amended, CR-NR-103 / CR-NR-109)`.
  17.8 is the no-dangling-xref criterion (distinct), confirmed by the adjacent
  test `shipped_content_has_no_dangling_cross_references` annotated 17.8.
- Req 17.9 was amended IN PLACE (not a new id) and the edit is NOTED: the sub-note
  under 17.9 explicitly records that the original all-ASCII wording AND the earlier
  narrow "English base stays ASCII" framing are BOTH discarded, and names the
  amending change ids (CR-NR-103 / CR-NR-109).

### 2. Whole help/ tree exempt, English base INCLUDED, no lingering English-base clause -- PASS
- Req 17.9 reads: "the English help base directly in the `help/` root AND every
  per-locale `help/<locale>/` subdirectory ... MAY contain non-ASCII UTF-8 ... and
  SHALL be valid UTF-8. The project documentation character-set (ASCII) rule SHALL
  NOT apply to any file under the `help/` tree."
- The sub-note states "No English-base-stays-ASCII clause is retained." I searched
  the amended criterion and its sub-clauses (17.9, 17.9a, 17.9b, 17.9c) and found
  no surviving clause keeping the English base in ASCII scope.
- Justification present and correct: help is RENDERED PRODUCT CONTENT for a planned
  dedicated help viewer, a DISTINCT ARTIFACT CLASS from project docs (docs/**) and
  Rust source.

### 3. 17.9a subsumed; 17.9b kept; 17.9c NOTE updated to remove/relax over the whole tree -- PASS
- 17.9a is marked "(SUBSUMED.)" -- the per-locale-only carve-out is collapsed into
  the whole-tree rule and carries no separate constraint. Correct.
- 17.9b is KEPT: per-locale `.ftl` DATA and `i18n/<locale>/aliases/<environment>.toml`
  rows MAY be non-ASCII; `i18n/en/` and the loading Rust source REMAIN ASCII.
  Matches the required 17.9b shape exactly.
- 17.9c NOTE is present and UPDATED: it states the former
  `shipped_content_files_are_ascii` test has been relaxed/renamed to
  `shipped_content_files_are_valid_utf8`, walking EVERY `.help.md` under the ENTIRE
  `help/` tree (English base in the root AND every `help/<locale>/` subdir), asserting
  valid UTF-8 with NO ASCII assertion retained -- i.e. for the entire tree, not
  narrowed to the English base. Annotation re-keyed to the amended Req 17.9.

### 4. design.md records the rendered-product-content exemption -- PASS
- `docs/specs/context-help/design.md` carries a dedicated note: "Help content is
  rendered product content, not project documentation (amended CR-NR-103 /
  CR-NR-109)" covering the WHOLE `help/` tree (English base in root AND per-locale
  subdirs), stating the ASCII rule does not apply and the remaining constraint is
  valid UTF-8 (Req 17.9). The D6 test bullet is updated to state the whole-tree
  valid-UTF-8 test has LANDED and the former ASCII test no longer exists.

### 5. ASCII rule still governs docs/**, steering, README/CHANGELOG, ALL Rust source -- PASS (with NIT 1)
- The criteria scope the exemption to ONLY the `help/` tree and per-locale i18n
  DATA; `i18n/en/` and all Rust source explicitly REMAIN ASCII (17.9b). The
  amendment prose nowhere widens the exemption to project docs/steering/source.
- Steering file verification: `.kiro/steering/documentation-charset.md` is a
  global/user-space injected rule and is NOT present in this workspace checkout
  (file_search returned no results), so a workspace-scoped gate cannot edit it.
  This is correctly surfaced to the owner (change-log CR-CH-064 "Affects" NOTE +
  the summary). The injected copy of that rule still shows only the
  runtime-UI-strings exception and no help/ exemption -- the governing rule does
  not yet reflect the amendment. Because the edit is outside this gate's editable
  scope and is correctly surfaced for owner action, this is a NIT (see NIT 1), not
  a blocker.

### 6. Amendment 2 (CR-CH-065) UNCHANGED and cleanly separable -- PASS
- localization Req 11.2/11.6 amended-in-place + new 11.11-11.13 are present and
  unchanged this iteration; the already-shipped-resolver contradiction is surfaced
  in the CR-CH-065 change-log entry (FLAG line), not resolved unilaterally.
- Separability holds: disjoint requirement sets (context-help Req 17 + localization
  Phase 6 vs localization Req 11 + Phases 5/7), disjoint spec files, disjoint task
  phases, and distinct change-log entries each carrying a mutual-separability
  statement ("Separable from CR-CH-064/065 ... owner may approve/defer/reject each
  independently"). No Amendment 2 cross-reference points at changed Amendment 1
  text.

### 7. Follow-on tasks recorded/updated with valid markers, titles, cross-refs; TCR rows match -- PASS
- context-help `tasks.md` Task 23/23.1: `[x]` DONE, titled, Validates Req 17.9
  (amended whole-tree) + 17.9c, cross-referenced to the localization Phase 6 twin.
  Task 16.6 NOTE, Task 16 "Covers" line, and the CR-NR-103/109 coverage-mapping row
  updated to "realised as the valid-UTF-8 test".
- localization `tasks.md` Phase 6 Task 13/13.1 `[x]` (test relaxed over the whole
  tree) and Task 14 `[x]` (af/fr/de help files already satisfy, proven by the
  passing test). Amendment 2 resolver Task 12 (Phase 5) and af alias DATA Task 15
  (Phase 7) remain present as `[ ]`.
- All inspected task lines use `[ ]` / `[x]` only, each carries a title, status
  notes sit on indented sub-bullets (not inside brackets).
- TCR: Req 17.9 row -> COVERED/PASS citing `shipped_content_files_are_valid_utf8`;
  17.9c -> realised/PASS; 17.9a SUBSUMED; 17.9b retained; Req 11.11-11.13 remain
  NOT-COVERED (correct -- Amendment 2 follow-on code not yet landed).

### 8. EARS-form and spec prose ASCII -- PASS
- Req 17.9 is EARS-form ("WHEN any `.help.md` file is shipped ... THE shipped help
  content set MAY ... and SHALL be valid UTF-8"); 17.9b is EARS-form; Req 11.11-11.13
  are EARS-form. 17.9a/17.9c are explanatory notes attached to EARS criteria, which
  is appropriate.
- Non-ASCII scan of the three pure-prose files (context-help requirements.md,
  context-help tasks.md, localization tasks.md) produced an EMPTY result log =
  clean ASCII. design.md/TCR.md/change-log.md non-ASCII is confined to permitted
  box-drawing in fences and status glyphs / pre-existing rows, none on amended lines.

### 9. No source/DATA/help content changed BY THIS GATE -- PASS (with NIT 2)
- The only `.rs` change is the owner-REQUESTED, already-LANDED ff-help test
  relaxation (`shipped_content_files_are_valid_utf8`). The gate itself edited only
  docs/specs, design.md, change-log, and the TCR. No `help/` content or `i18n/`
  DATA was changed. The r2 docs now accurately describe the landed test state (the
  prior "no source/test changed" wording was corrected under Option B), so docs and
  reality agree. See NIT 2 on the residual framing nuance.

### 10. Contradictions surfaced, not resolved unilaterally -- PASS
- The code-ahead-of-criteria contradiction (prior Finding 1) was surfaced to the
  owner; owner chose Option B; docs reconciled. The Amendment 2 already-shipped
  resolver contradiction is surfaced in CR-CH-065. The missing steering file is
  surfaced. None resolved unilaterally.

---

## Verified Assumptions

- The test symbol is `shipped_content_files_are_valid_utf8` (not
  `shipped_content_files_are_ascii`), asserts valid UTF-8 over the whole `help/`
  tree via recursive `visit_help_files`, retains NO ASCII assertion, and its
  `// Validates:` annotation keys to the amended Req 17.9. VERIFIED by reading
  `crates/ff-help/tests/shipped_content_tests.rs`.
- The no-dangling-xref criterion is 17.8 (distinct from the character-set 17.9).
  VERIFIED against both the requirements doc and the test annotations.
- Req 17.9 exempts the ENTIRE `help/` tree with the English base explicitly
  INCLUDED and no retained English-base-ASCII clause. VERIFIED in
  `docs/specs/context-help/requirements.md`.
- 17.9a SUBSUMED, 17.9b retained (per-locale DATA UTF-8; en/ + Rust source ASCII),
  17.9c realised-note. VERIFIED in requirements.md and TCR.md.
- design.md D6 rationale note + updated test bullet present. VERIFIED by grep +
  read of `docs/specs/context-help/design.md`.
- Both tasks files use valid `[ ]`/`[x]` markers with titles; Task 23/13/14 `[x]`,
  Task 12/15 `[ ]`. VERIFIED in both tasks.md files.
- change-log carries both CR-CH-064 and CR-CH-065 with mutual separability lines
  and the surfaced contradictions. VERIFIED in `docs/status/change-log.md`.
- The three pure-prose amended docs are ASCII-clean. VERIFIED by Select-String
  scan (empty result).
- `.kiro/steering/documentation-charset.md` is absent from the workspace checkout.
  VERIFIED by file_search (no results).

## Unverified / Wrong Assumptions

- The ASSERTION that the landed test "passes (fmt/clippy/build clean)" could NOT be
  independently re-verified here -- this is a DOCS-ONLY review and the owner's
  manual gate is authoritative for a green run (per the testing execution-split
  rule). The review relied on the committed source state of the test file, which is
  consistent with the docs. Not treated as a finding: the step is docs-scoped.
- The governing (user-space) `documentation-charset.md` Scope could not be read or
  confirmed to carry the help/ exemption, because the file is not in the workspace.
  Recorded as NIT 1.

---

## Findings

1. NIT -- Governing steering Scope not yet updated. `.kiro/steering/documentation-charset.md`
   is a user/global injected rule absent from this workspace checkout; its Scope
   still lists only the runtime-UI-strings exception and does not yet put the whole
   `help/` tree + per-locale i18n DATA out of ASCII scope. This is OUTSIDE the
   editable scope of a workspace gate and is correctly surfaced to the owner
   (CR-CH-064 "Affects" NOTE). Fix (owner, user-space, post-approval): add a Scope
   bullet exempting the entire `help/` tree and per-locale i18n DATA as "rendered
   product content," a class distinct from the existing runtime-UI-strings
   exception, with no English-base-in-scope wording. Non-blocking.

2. NIT -- "DOCS-ONLY gate" framing vs the one landed test. The gate is described as
   DOCS-ONLY, yet one `.rs` test change (owner-requested) has landed and the docs
   were reconciled to it under Option B. The docs are internally consistent and the
   change-log states this accurately, so there is no contradiction left to resolve;
   the only residual is terminological (a "docs-only" gate that also ratifies one
   pre-landed owner-requested test). Fix (optional): nothing required -- the
   change-log already scopes it correctly ("the single ff-help TEST relaxation has
   LANDED; rest docs-only"). Non-blocking.

(Count: HIGH = 0, MEDIUM = 0. Verdict mechanically APPROVED.)
