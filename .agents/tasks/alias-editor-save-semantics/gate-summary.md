# Gate Summary -- CR-CH-071 ISPF Save/End/Cancel/Return/Refresh for editable Contexts

DOCS-ONLY requirements gate. NO source `.rs` files were edited. All authored docs
are plain ASCII (verified with a `[^\x00-\x7F]` search over every edited spec/doc;
the only non-ASCII in the repo is pre-existing emoji in unrelated TCR / change-log
entries, which documentation-charset.md permits for TCR status glyphs).

## Allocated CR-CH id and collision re-verification

**Allocated id: CR-CH-071.**

Re-verified against BOTH change-logs:
- `docs/status/change-log.md` (main, HEAD `15434d0`): the highest allocated id is
  CR-CH-069. CR-CH-070 and CR-CH-071 do NOT appear in main's change-log (a
  `CR-CH-07[01]` search over the whole main working tree returns only
  `.agents/tasks/...` findings/review prose, never a change-log row).
- The eqfiles-fix branch: CR-CH-070 (the `=FILES` root-token fix, Req 22) is
  authored on branch `eqfiles-fix` and is NOT merged to main. Its review and
  implementation notes live in `.agents/tasks/command-placement-audit/cr070-*.md`.
  So CR-CH-070 is reserved by that branch; the next FREE id above it is CR-CH-071.

CR-CH-071 therefore does not collide with the merged main (<= CR-CH-069) nor with
the unmerged eqfiles-fix branch (CR-CH-070).

## Documentation files produced (absolute paths)

1. `c:\workspace\apps\FileForgeWorkbench\docs\status\change-log.md`
   -- NEW change-request entry CR-CH-071 (inserted at the top of the Change
   Requests section).
2. `c:\workspace\apps\FileForgeWorkbench\docs\specs\command-configurator\requirements.md`
   -- Req 5.9 AMENDED inline (END/F3 now saves-then-returns); Req 5.14 AMENDED
   inline (ISPF SAVE/CANCEL/END semantics + private-interpreter retirement + REFRESH
   accepted; FILTER/RESET retained panel-local).
3. `c:\workspace\apps\FileForgeWorkbench\docs\specs\command-environments\requirements.md`
   -- NEW Requirement 20a (criteria 20a.1-20a.14): the editable-Context ISPF verb
   set, the FFCMD-base-delegating handler + shared persist/discard/reload seam, the
   FFEDIT active-wins SAVE override, CANCEL/REFRESH FFCMD-only in v1, REFRESH-not-UNDO
   naming, retirement of the private interpreter (one dispatch path), uniform
   application across editable non-buffer Contexts, the no-shadow consequence, and
   EXIT-stays-app-exit. Includes the Req 2a.5 / 11.1 reconciliation note.
4. `c:\workspace\apps\FileForgeWorkbench\docs\specs\command-environments\design.md`
   -- NEW design section for Req 20a: the problem (two parallel paths), the verb
   table, the `EditableContext` persist/discard/reload seam shape, the FFCMD-base
   handler + active-wins override dispatch, FFCMD root registration (CR-CH-067
   enumeration), retirement of `apply_alias_command_line`'s SAVE/CANCEL/END arms,
   the per-editor wiring plan (Alias Editor first, siblings fast-follow),
   REFRESH-vs-UNDO rationale, the open decisions, and "what is explicitly NOT
   changing".
5. `c:\workspace\apps\FileForgeWorkbench\docs\specs\command-environments\tasks.md`
   -- NEW Phase (alias-editor-save-semantics), Tasks 32-35 + an Acceptance Criteria
   Coverage table (task-format compliant: `[ ]`/`[x]` only, titled lines, Req
   cross-references). (Task 31 belongs to the eqfiles-fix branch's `=FILES` fix.)
6. `c:\workspace\apps\FileForgeWorkbench\docs\quality\TCR.md`
   -- NEW Phase (alias-editor-save-semantics) section with NOT-COVERED (red) rows,
   one per Req 20a criterion plus the two amended command-configurator criteria
   (5.9, 5.14).
7. `c:\workspace\apps\FileForgeWorkbench\docs\project-management\ROADMAP.md`
   -- NEW CR-CH-071 entry in the Section 6a command-environment cluster (after
   CR-CH-069), with the sequencing note and the related non-gated ghost-placeholder
   quick-fix cross-reference.
8. `c:\workspace\apps\FileForgeWorkbench\.agents\tasks\alias-editor-save-semantics\gate-summary.md`
   -- this file.

## Open decisions (recommended defaults; for the owner CONFIRM step)

1. **REFRESH vs REVERT verb name.** RECOMMENDED: **REFRESH**. Rationale: UNDO is the
   FFEDIT buffer single-step undo (undo-redo-transactions; command-environments
   Req 10.6 / CR-CH-054 Req 12.8), so naming the whole-Context discard-and-reload
   "UNDO" would collide. REFRESH is unambiguous; if REVERT is ever wanted it becomes
   an alias surface of canonical REFRESH, not a second verb.
2. **CANCEL / REFRESH FFCMD-only in v1, or FFEDIT also claims a buffer-revert now.**
   RECOMMENDED: **FFCMD-base-only in v1.** FFEDIT keeps its buffer CANCEL/RETURN
   (Req 10) and does NOT claim REFRESH; an FFEDIT buffer-level revert is a later
   additive task.
3. **EXIT on an editable Context -- save-then-leave vs app-exit.** RESOLVED in the
   design (not left open): **END (and F3), not EXIT, is the save-then-leave verb;
   EXIT/QUIT/LOGOFF remain the app-exit family unchanged** (Req 20a.13), so app-exit
   is not broken.
4. **Registration shape -- FFCMD-base-delegating handler vs per-Context-environment
   SAVE (Req 11 E10 option a).** RECOMMENDED: **the FFCMD-base-delegating handler**
   (one handler + one seam for all editable Contexts; matches the owner's "FFCMD's
   own save" phrasing). This reconciles with command-environments Req 2a.5 / 11.1
   (the base owns no working copy -- it delegates to the focused Context's seam); the
   OBSERVABLE behaviour is identical to the per-Context-environment shape, so the
   choice is a registration-site decision, flagged for CONFIRM.
5. **Scope of sibling editors in v1 vs follow-up.** RECOMMENDED: **design the uniform
   seam now; implement the Alias Editor first, the siblings (Settings/Menus/Theme/
   Keys/Kinds) as fast-follow tasks** (Tasks 34.1-34.5), not re-implemented per
   editor.

Related non-gated item (recorded, needs no gate): the empty Alias Editor's ECOB /
`FFWB.COBOL.MASTER(&1)` ghost placeholder is egui `hint_text` (not seeded data) in
`crates/ff-desktop/src/alias_editor_panel/rows.rs`; removing it is a cosmetic
quick-fix (findings Piece 1), tracked alongside CR-CH-071, with NO requirements
criterion added for it.

## SEQUENCING NOTE

Implementation of CR-CH-071 must land:
- **AFTER CR-CH-070 (`eqfiles-fix`) is merged** -- both touch the shell command /
  dispatch files and the FFCMD root surface (`BUILTIN_FFCMD_LADDER_VERB_ROOTS` /
  `builtin_ffcmd_roots`), so landing CR-CH-071 first would create a merge collision.
- **Coordinated with the pending Afrikaans alias-data task** -- that task also
  touches the alias overlay / FFCMD root surface (`af/aliases/*.toml`), and
  CR-CH-071's root-enumeration change (adding SAVE/CANCEL/REFRESH to the FFCMD root
  set) overlaps the same surface.

The gate docs are authored NOW (this gate); IMPLEMENTATION WAITS for the above and
for owner approval of the requirements/design and the open decisions.

## Framework-conformance determination

ADDITIVE ownership/placement change of the CR-CH-069 class (an editor verb's
environment placement) PLUS a gated BEHAVIOUR change (the ISPF semantics that amend
command-configurator Req 5.9/5.14). It builds ON the single front door and
active-wins: no second dispatcher, no second navigation stack, no new persistence
format, no reshaped public framework type. SAVE/CANCEL/REFRESH register in the
EXISTING FFCMD root sources and dispatch through the EXISTING ladder + active-wins;
the persist/discard/reload seam reuses the EXISTING `pending_action` /
`apply_<editor>_action` pattern; FFEDIT's buffer SAVE root and its
`dispatch_to_environment` store-addressing claim are byte-identical. By the
framework-conformance definition the ownership move is NOT a core framework change;
the behaviour/criteria change runs the normal gate and needs owner CONFIRM.
