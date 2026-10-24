# Gate Review -- CR-NR-110 FIND vs SEARCH verb split (DOCS-ONLY)

Reviewer: wf-review (gating reviewer, fresh-context)
Scope: the CR-NR-110 docs-only requirements gate authored by wf-design.
Checkout under review: worktree `c:\workspace\apps\FileForgeWorkbench\.worktrees\crnr109-locale-alias`
(the main checkout path resolves to this active worktree; the drafted docs live here).

Verdict: APPROVED (no HIGH or MEDIUM findings).

## What was reviewed

Working-tree changes relevant to CR-NR-110 (read, not edited):

- docs/specs/global-search/requirements.md (new Req 7, 8, 9; Intro scope note; Glossary; Out of Scope)
- docs/specs/global-search/design.md (SEARCH front-door dispatch, GREP via AliasTable, results routing, Deferred, framework-conformance)
- docs/specs/global-search/tasks.md (new Tasks 6, 7)
- docs/specs/find-and-replace/requirements.md (new Req 22; Intro scope note)
- docs/specs/command-semantics/requirements.md (new Req 12 -- verb-split decision record)
- docs/project-management/project-master/tasks.md (new Phase CR-NR-110, NR110.1-NR110.6)
- docs/project-management/ROADMAP.md (CR-NR-109 note; not CR-NR-110 specific)
- docs/quality/TCR.md (new CR-NR-110 phase table + CR-NR-109 rows)
- docs/status/change-log.md (new CR-NR-110 entry + CR-NR-109 entry)

Other working-tree changes (localization, logging-subsystem, notification-system,
bugs.md, and the CR-NR-109 rows/entry) were confirmed to carry NO CR-NR-110 content
and belong to separate work in this shared worktree; they are out of this gate's scope.

## Owner-decision conformance (all five FINAL decisions)

1. FIND = FFEDIT in-buffer verb: SATISFIED. find-and-replace Req 22.1/22.2 record FIND as
   the FFEDIT editor-buffer verb; Req 22 explicitly FORMALISES existing behaviour (FIND
   already in `AliasTable::ffedit_english`) and its behaviour home stays Req 1-5 of
   find-and-replace. command-semantics Req 12.1 agrees.
2. SEARCH = FFCMD canonical root cross-backend verb in the existing Search Results Context,
   not a raw shell-out: SATISFIED. global-search Req 7.1-7.3, 7.5 and Req 9.1-9.2;
   design "Dispatch: the single front door" + "Results route to the existing Search
   Results Context". Req 9.1 forbids raw shell-out as the primary mechanism.
3. GREP = alias surface resolving to canonical SEARCH via the EXISTING FFCMD AliasTable:
   SATISFIED. global-search Req 8.1-8.4; design "The GREP alias through the existing FFCMD
   AliasTable"; command-semantics Req 12.3.
4. Engine + mainframe traversal DEFERRED with the named dependency (CR-NR-105 volume-model
   + dataset-catalog) in BOTH requirements and design: SATISFIED. global-search
   requirements Out of Scope + Req 9.4/9.5; design "Deferred: engine + mainframe traversal".
5. Verb-split decision recorded in command-semantics (and cross-referenced from
   command-environments), not duplicated: SATISFIED. command-semantics Req 12 is the
   decision record; it cross-references command-environments Req 2a / 6a rather than
   duplicating them; find-and-replace Req 22 and global-search Req 7-9 reference back.

## House-rule conformance

- DOCS-ONLY: SATISFIED. `git status` shows only `.md` files modified/added; no `.rs`
  change and no cargo invocation. No BLOCKING source change.
- EARS form: SATISFIED. All new criteria use `WHEN ... THE ... SHALL ...` or unconditional
  `THE ... SHALL ...`, numbered sequentially per requirement (global-search 7.1-7.5,
  8.1-8.4, 9.1-9.5; find-and-replace 22.1-22.4; command-semantics 12.1-12.6).
- ASCII-only: SATISFIED for touched content. The new global-search docs and the new
  command-semantics Req 12 are pure ASCII. The only non-ASCII hits in command-semantics
  (lines 25, 66, 79, 175, 219) are in PRE-EXISTING, untouched lines, outside this gate's
  diff. The find-and-replace diff adds only ASCII (the pre-existing `ΓÇô` mojibake lines
  are untouched context). TCR status markers are the established red-circle / check-mark
  emoji convention (the documentation-charset TCR exception), used consistently in the
  new rows; new-row prose is ASCII.
- Task format: SATISFIED. New tasks use only `[ ]`; every line titled; DEFERRED/status
  notes are on indented sub-bullets (global-search task 7.3, project-master NR110.5);
  `(Req X.Y)` / `Satisfies:` cross-references present.
- Framework conformance: SATISFIED. The design builds ON the single front door
  (`resolve_target` / `dispatch_command_target`, SEARCH as a `Function` target), the
  existing FFCMD AliasTable (GREP -> canonical SEARCH), and the existing Search Results
  Context (`TabKind::SearchResults`). It explicitly rejects a bespoke command-ladder
  intercept, a second dispatcher, a parallel nav stack, a new persistence format, and a
  raw shell-out. The change-log states "No framework change is required by this gate".
- CR id consistency: SATISFIED. CR-NR-110 is used consistently across global-search,
  find-and-replace, command-semantics, project-master, TCR, and change-log. The change-log
  entry Status is "GATE AUTHORED / PENDING OWNER APPROVAL" under the New Requirements
  section.
- TCR rows: SATISFIED. Exactly one NOT-COVERED row per new criterion -- 14 for global-search
  (7.x=5, 8.x=4, 9.x=5), 4 for find-and-replace Req 22, 6 for command-semantics Req 12.
  Crate placement is correct: SEARCH dispatch/alias rows on `ff-desktop`, cross-backend
  scope/engine rows on `ff-global-search`, FIND rows on `ff-find-and-replace`, decision-
  record rows on `ff-cmd-semantics`. Column format matches existing TCR rows.
- Internal consistency: SATISFIED. Cross-references resolve -- command-environments Req 2a
  (per-environment ownership), Req 6 criterion 1 and 2a (FFEDIT owns FIND; claim step),
  Req 6a and 6a.2 (AliasTable; canonical recorded/persisted) all exist as cited.
  project-master / TCR / change-log agree with the specs.

## Findings

No HIGH or MEDIUM findings. The NITs below do not affect the verdict and need not block;
they are offered for optional polish by the author.

1. NIT -- docs/specs/global-search/requirements.md, Introduction scope note: the text says
   "Requirements 1 to 6 above define ..." while that sentence physically precedes Req 1-9
   in the file (the note sits in the Introduction, above the Requirements section). The
   word "above" is slightly loose. Optional fix: change "Requirements 1 to 6 above define"
   to "Requirements 1 to 6 define" and "Requirements 7 to 9 (added by CR-NR-110)" reads
   fine as-is.

2. NIT -- docs/specs/global-search/requirements.md Req 7.5 vs Req 1.2: Req 7.5 and Req 1.2
   both describe the `GSEARCH`/`SEARCH` activation seam. The relationship is stated
   ("Req 1.2 is the activation seam; CR-NR-110 makes SEARCH the canonical identifier"),
   so there is no contradiction, but a one-line forward-reference comment on Req 1.2
   pointing to Req 7 would make the layering obvious to a future reader. Optional.

3. NIT -- docs/specs/find-and-replace/requirements.md Req 22.2 cites "command-environments
   Requirement 6.1/6.2a". The citation is correct (Req 6 criterion 1 and Req 6 criterion
   2a both exist), but the "6.2a" shorthand could be misread as a top-level "Req 6.2a".
   Optional fix: write "Requirement 6 (criteria 1 and 2a)" for clarity.

## Verified assumptions

- No `.rs` file was created or modified and no build was run (git status: only `.md`
  paths; no cargo output). Confirmed.
- command-environments Req 2a exists (per-environment verb ownership). Confirmed by read.
- command-environments Req 6 criterion 1 names FIND among FFEDIT-owned verbs; criterion 2a
  defines the active-environment claim step. Confirmed by read.
- command-environments Req 6a exists (alias table) and criterion 6a.2 states the canonical
  verb is recorded/persisted. Confirmed by read.
- The CR-NR-110 TCR rows number exactly one per new criterion (24 total: 14 + 4 + 6).
  Confirmed by counting against the requirements files.
- The change-log CR-NR-110 Status is "GATE AUTHORED / PENDING OWNER APPROVAL". Confirmed.
- The other working-tree doc changes (localization, logging, notification, bugs, CR-NR-109)
  contain no CR-NR-110 content. Confirmed (empty CR-NR-110 grep over those files).

## Unverified / wrong assumptions

- None material. The design asserts the FFCMD alias seed will be added in
  `shell/environment.rs` alongside `ffedit_english`; this is an implementation detail
  deliberately left to the engine gate (design "Note on placement") and is not a gate
  deliverable, so it is correctly not verified against source here.
- The internal search engine and mainframe dataset traversal are explicitly DEFERRED, so
  their feasibility is out of scope for this gate and intentionally not assessed.
