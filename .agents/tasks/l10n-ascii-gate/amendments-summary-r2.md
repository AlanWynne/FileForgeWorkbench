# l10n-ascii-gate -- Correction Summary (Amendment 1, r2)

Requirements-gate CORRECTION of Amendment 1 only. Plain ASCII throughout.
Amendment 2 (CR-CH-065) was left AS-IS. Work done on the main checkout
`c:\workspace\apps\FileForgeWorkbench` (not a worktree).

## REVISION iteration (design-review-r2 findings addressed)

`design-review-r2.json` is PRESENT -- verdict CHANGES_REQUESTED, one MEDIUM
finding (Finding 1) and one NIT (Finding 2). Finding 1 was surfaced to the owner
(it is a code-vs-criteria contradiction that must not be resolved unilaterally),
the owner chose OPTION B, and the docs are now reconciled to match. The body
below records both the resolution of Finding 1 and the full first-correction
detail.

### Finding 1 (MEDIUM) -- code had moved ahead of the criteria

The r2 review found that `crates/ff-help/tests/shipped_content_tests.rs` had an
UNCOMMITTED working-tree modification (git status ` M`): the ASCII test
`shipped_content_files_are_ascii` had already been RELAXED and RENAMED to
`shipped_content_files_are_valid_utf8` (ASCII assertion removed, now a valid-UTF-8
assertion over the whole `help/` tree, `// Validates:` annotation rewritten to the
amended Req 17.9). Every gate document, by contrast, described that ASCII test as
still present and its removal as a PENDING follow-on (context-help Task 23 /
localization Task 13), and the prior summary stated "no source/test changed."

Git-verified facts (this iteration):
- Committed (HEAD `9dd4adb`, 2026-09-30): test is `shipped_content_files_are_ascii`,
  annotated `// Validates: Requirement 17.9 -- shipped content files are ASCII.`
- Working tree (uncommitted): test is `shipped_content_files_are_valid_utf8`,
  valid-UTF-8 only, annotation keyed to the amended whole-tree Req 17.9. The
  symbol `shipped_content_files_are_ascii` no longer exists anywhere under
  `crates/ff-help/`.

This was surfaced to the owner (severity warning) rather than resolved in code --
the two options (A: revert the `.rs` so docs stay accurate; B: accept the
relaxation landed and reconcile the docs) are materially different owner
decisions, and reverting a `.rs` is forbidden by this DOCS-ONLY step.

### Owner decision: OPTION B (the test relaxation is INTENDED and has landed)

The owner confirmed the ASCII-test fix was explicitly requested and made
(`shipped_content_files_are_valid_utf8` passes; fmt/clippy/build clean). The
whole premise of Amendment 1 is that help content is rendered product content
exempt from the ASCII rule, so the valid-UTF-8 test IS the realised end state,
not a pending follow-on. Per Option B, the docs were reconciled to reality
(DOCS ONLY -- no further `.rs` change made).

Reconciliation applied (docs only):
- `docs/specs/context-help/requirements.md` Req 17.9c: reworded from "follow-on,
  NOT this gate" to "realised" -- the former `shipped_content_files_are_ascii` is
  now `shipped_content_files_are_valid_utf8` (whole tree, valid-UTF-8 only, no
  ASCII assertion), annotation keyed to the amended Req 17.9; the removed symbol
  is no longer referenced as current. EARS wording of 17.9 itself unchanged.
- `docs/specs/context-help/design.md` D6 test bullet: changed "REMOVED/RELAXED ...
  (Done by the follow-on gated test task, not this gate.)" to state the whole-tree
  valid-UTF-8 test has LANDED and the former ASCII test no longer exists. The
  rationale note (help is rendered product content) was already correct and is
  unchanged.
- `docs/specs/context-help/tasks.md`: Task 23 and 23.1 marked `[x]` DONE and
  reworded to the landed `shipped_content_files_are_valid_utf8`; Task 16.6 NOTE,
  the Task 16 "Covers:" line, and the CR-NR-103/109 AC-17.9 coverage-mapping row
  updated from "see Task 23 (pending)" to "realised as the valid-UTF-8 test".
- `docs/specs/localization/tasks.md`: Phase 6 Task 13/13.1 marked `[x]` DONE and
  reworded to the landed symbol; Task 14 (af/fr/de help files satisfy the amended
  rule) marked `[x]` and now states this is literally proven by the passing
  valid-UTF-8 test.
- `docs/quality/TCR.md`: Req 17.9 row moved from NOT-COVERED to COVERED/PASS
  citing `tests/shipped_content_tests.rs` `shipped_content_files_are_valid_utf8`;
  17.9c row moved to realised/PASS. 17.9a stays SUBSUMED; 17.9b (per-locale i18n
  DATA UTF-8) left AS-IS per owner instruction.
- `docs/status/change-log.md` CR-CH-064: Description + Affects + Status corrected
  so the "no source/test changed" / "follow-on not this gate" wording is replaced
  by an accurate statement -- the single owner-requested ff-help TEST relaxation
  has LANDED (realising the Amendment 1 follow-on), while the gate's OTHER work
  remains docs-only. Not overclaimed beyond that one test change.

### Finding 2 (NIT) -- global steering file not in workspace

`.kiro/steering/documentation-charset.md` is a global/user-space injected rule,
not present in this workspace checkout, so its Scope section could not be edited
by this workspace-scoped gate. It remains SURFACED to the owner (CR-CH-064
"Affects" NOTE + this summary) as an outstanding user-space edit: the Scope must
put the WHOLE `help/` tree + per-locale i18n DATA OUT of ASCII scope as rendered
product content (a class distinct from the existing runtime-UI-strings
exception), with no English-base-in-scope wording. No action inside this gate.

---

## Confirmed ASCII criterion id + test annotation

- Confirmed criterion id: context-help **Requirement 17.9** (the shipped-help
  character-set constraint; 17.8 is the no-dangling-xref criterion). The
  amendment edited 17.9 IN PLACE.
- Committed test `// Validates:` annotation (HEAD): `// Validates: Requirement
  17.9 -- shipped content files are ASCII.`
- Current (working-tree) test `// Validates:` annotation: keyed to Requirement
  17.9 amended (CR-NR-103 / CR-NR-109) -- help is rendered product content EXEMPT
  from the ASCII rule; the remaining constraint is valid UTF-8. Test renamed to
  `shipped_content_files_are_valid_utf8`.

## The correction -- Amendment 1 exempts the WHOLE help/ tree (English base INCLUDED)

DISCARDED framing (earlier loop): "English help/ base stays ASCII, only per-locale
`help/<locale>/` subdirs get UTF-8."

CORRECTED decision (owner clarification): the ENTIRE `help/` tree is exempt from
the project ASCII rule, English base INCLUDED. Help files are RENDERED PRODUCT
CONTENT for a planned dedicated help viewer; they are a DISTINCT ARTIFACT CLASS
from project documentation (docs/**) and from Rust source, so the project ASCII
documentation rule does not apply to them. All project docs/steering,
README/CHANGELOG, the `i18n/en/` catalogue, and ALL Rust source STILL stay ASCII.
Only the `help/` tree and the per-locale i18n DATA are exempt.

## Files touched this iteration and what changed (docs only)

| File | Change (r2 reconciliation to Option B) |
|------|-----------------------------------------|
| `docs/specs/context-help/requirements.md` | Req 17.9c reworded from pending follow-on to REALISED (test already relaxed to `shipped_content_files_are_valid_utf8`, whole tree, valid-UTF-8 only). Req 17.9/17.9a/17.9b unchanged this iteration. |
| `docs/specs/context-help/design.md` | D6 test bullet updated to "the whole-tree UTF-8 test has LANDED"; rationale note unchanged. |
| `docs/specs/context-help/tasks.md` | Task 23/23.1 `[x]` DONE + reworded to landed symbol; Task 16.6 NOTE, Task 16 Covers, CR-NR-103/109 AC-17.9 coverage row updated to "realised". |
| `docs/specs/localization/tasks.md` | Phase 6 intro updated; Task 13/13.1 `[x]` DONE + landed symbol; Task 14 `[x]` (proven by passing test). |
| `docs/quality/TCR.md` | Req 17.9 row -> COVERED/PASS (`shipped_content_files_are_valid_utf8`); 17.9c row -> realised/PASS; 17.9a SUBSUMED, 17.9b retained AS-IS. |
| `docs/status/change-log.md` | CR-CH-064 Description/Affects/Status corrected: the single ff-help TEST relaxation has LANDED; rest of gate docs-only; "no source changed" wording removed. |

NOT edited (correctly): any `.rs` source (no further change beyond the
owner-requested test relaxation that already landed), `help/` content, per-locale
alias/catalogue DATA, Amendment 2 criteria/tasks/rows.

## design.md rationale note (requested item 4)

Present and unchanged: the shipped `.help.md` files under the WHOLE `help/` tree
(English base in the root AND every per-locale subdirectory) are RENDERED PRODUCT
CONTENT for the planned dedicated help viewer, a DISTINCT ARTIFACT CLASS from
project docs and Rust source; the project ASCII rule DOES NOT apply to any file
under `help/`; shipped help MAY contain non-ASCII UTF-8 and SHALL be valid UTF-8
(Req 17.9). This is the rationale for the amendment; the earlier narrow framing
is recorded as discarded.

## Updated follow-on tasks and TCR rows (now realised)

- context-help Task 23 / 23.1 == localization Phase 6 Task 13 / 13.1: `[x]` DONE
  -- `shipped_content_files_are_ascii` RELAXED to
  `shipped_content_files_are_valid_utf8` for the ENTIRE `help/` tree (valid-UTF-8
  only). Resolves B085 / the red gate.
- localization Phase 6 Task 14: `[x]` -- the existing `help/af|fr|de/index.help.md`
  satisfy the amended whole-tree rule, proven by the passing valid-UTF-8 test.
- TCR: Req 17.9 -> COVERED/PASS (valid-UTF-8 test); 17.9a SUBSUMED; 17.9b retained;
  17.9c realised/PASS.

## Amendment 2 (CR-CH-065) left AS-IS

Amendment 2 (diacritic-insensitive command input: Req 11.2/11.6 amended +
11.11-11.13 added + the already-shipped-resolver contradiction surfaced) was NOT
re-authored. No cross-reference in Amendment 2 points at changed Amendment 1 text
(its separability line names CR-CH-064 by id only, which is unchanged), so no
cross-reference adjustment was needed. The two amendments remain cleanly
separable: disjoint requirement sets, disjoint spec files, disjoint task phases
(context-help Req 17 + localization Phase 6 vs localization Req 11 + Phases 5/7),
and distinct change-log entries each carrying a mutual-separability statement.

## Contradictions surfaced for owner review (NOT resolved unilaterally)

1. The already-relaxed ff-help test (r2 Finding 1) -- surfaced via send_message;
   owner chose OPTION B; docs reconciled accordingly (above).
2. The global/user-space `documentation-charset.md` steering file is not in this
   workspace checkout; its Scope edit is surfaced to the owner (r2 Finding 2, NIT)
   and recorded in the CR-CH-064 "Affects" NOTE.
3. (From Amendment 2, left AS-IS) the already-shipped CR-NR-109 exact-match
   resolver that Amendment 2's behaviour change touches -- surfaced in the
   CR-CH-065 change-log entry; unchanged this iteration.

## Verification

- Confirmed the committed and working-tree test annotations by reading
  `crates/ff-help/tests/shipped_content_tests.rs` and `git show HEAD:` for the
  same file (logs under `tools/logs/`).
- `git status --porcelain` confirms the ff-help test `.rs` is the only modified
  source file; no `help/` or `i18n/` DATA file is modified by this gate.
- Non-ASCII scan of the five amended docs: the three pure-prose files
  (context-help requirements/tasks, localization tasks) scan clean; the only
  non-ASCII in design.md is box-drawing inside code fences (permitted) and in
  TCR.md/change-log.md are status glyphs + pre-existing entries -- none on any
  line this correction edited. All newly authored prose is plain ASCII.
- Scan logs written to `tools/logs/`.
