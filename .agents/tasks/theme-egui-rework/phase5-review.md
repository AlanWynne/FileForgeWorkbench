# Phase 5 review -- CR-CH-056 egui-native Theme Rework (Task 33)

Phase 5 is the final, docs-heavy phase of the egui-native theme rework: it ships
an egui-native theme-authoring guide, records a test-migration audit, and sweeps
the TCR for Req 23/24/25 plus the reworded 13.2/18.3/20.5/20.11-20.13 rows. No
shipping `.rs` was modified this phase; the change set is `theme-authoring.md`
(new), the Task 33 markers in `tasks.md`, TER.6 in project-master, the TCR
CR-CH-056 section, and the change-log status line, plus the two task notes.

Watch for: the working-tree diff against HEAD is NOT Phase-5-only -- it also
carries the already-landed Phases 1-4 code (ff-theme, ff-theme-editor, ff-desktop)
and unrelated gate-authored docs (volume-model, nav-ladder, localization). I
scoped the review to the Task 33 deliverables the note enumerates and left the
rest alone. One non-blocking observation: pre-existing `design.md`/`requirements.md`
prose carries en-dashes/arrows/curly quotes, but those lines are gate-era (TER.1)
content, not Phase 5 additions, so they are out of this phase's scope.

**Verdict**: APPROVED

## High-level view

The authoring guide (`theme-authoring.md`) leads with egui `Style`/`Visuals`
vocabulary as the primary model and frames ISPF/Legacy as one retrofitted
instance, exactly as Req 23.4 demands. It documents the shipped shape: a chrome
layer configuring the full egui surface plus retained domain groups, the renamed
`gutter` group (formerly `chrome`), the versioned v2 TOML with flat authoring
groups authoritative and the chrome `Style` derived on load, the additive
embedded `[chrome_style]` snapshot, `base` resolution, the five built-ins, and
the derived editable surface in the Theme Editor. No stale `EditableToken` /
`palette.ui`-as-primary vocabulary survives.

The 33.2 test-migration audit concludes nothing was left to migrate because the
migration completed across Phases 1-4. The note backs each limb with grep
evidence: production chrome reads already go through `chrome_style.*` accessors,
the old `chrome` group is fully renamed to `gutter` (only the renamed-from doc
comment and the frozen `[chrome]` TOML header remain), and the `== 14`
editor-count assertions are already `> 14`. The retained `palette.ui.*` /
`palette.tab_bar.*` references assert the authoritative authoring groups (design
C56.2 keeps them authoritative, chrome derived from them), and the `.mode`/`.name`
THEME-command tests were correctly left unchanged. This is a confirmation audit,
consistent with "no shipping `.rs` changed this phase".

The TCR CR-CH-056 section sets Req 23.1-23.11, 24.1/24.2/24.3/24.5, 25.1-25.7,
and the reworded 13.2/18.3/20.5/20.11-20.13 rows to PASS, each citing a test file
and the owning phase. Req 24.4 (external-format import) is correctly left RED/red-circle
with an OUT-OF-SCOPE note -- no coverage was invented. Req 23.4 is PASS citing the
model tests plus the new authoring guide under the testing.md MANUAL-exception
rationale for terminology/doc coverage, which is the right call for a
terminology-honesty criterion.

Bookkeeping is complete and honest: Task 33 + 33.1/33.2/33.3 are `[x]` with DONE
notes, TER.6 is `[x]` marked code-complete-pending-gate, and the change-log
CR-CH-056 status reads Phases 1-4 DONE / Phase 5 code-complete pending the owner's
full gate. The verification note records scoped checks only (`cargo test -p
ff-theme -p ff-theme-editor`, scoped clippy/fmt, and a compile-only `cargo check
-p ff-desktop`) -- no `--workspace` or `ffwb-gate.ps1` run in-agent, matching the
hand-off protocol.

<details>
<summary>Issues (1)</summary>

1. **Pre-existing non-ASCII in theme design/requirements prose** (non-blocking,
   possible) -- `design.md`/`requirements.md` carry en-dashes, arrows, and curly
   quotes, but on gate-era (TER.1) lines, not Phase 5 additions. Out of Phase 5
   scope; worth a cleanup pass when those files are next touched, not a gate on
   this phase.

</details>

<details>
<summary>Details</summary>

### Docs honesty -- egui vocabulary primary, ISPF as retrofit (Req 23.4)

`theme-authoring.md` opens by stating FFWB is a Windows egui application, not an
ISPF terminal, and that the Legacy look is "just ONE retrofitted instance of the
general model, not the shape the model is built around." Section 1 defines a
Theme as a chrome layer (configuring the full `egui::Style`/`Visuals`/`WidgetVisuals`
surface) plus the retained domain groups egui does not model, and the glossary
table enumerates Theme / egui `Style` / `Visuals` / `WidgetVisuals` / chrome layer
/ domain groups / the `gutter` group renamed from `chrome`. Sections 2-7 document
the shipped model: the single `apply_theme` wholesale `apply_to_egui` seam with
Design_Tokens wired on and `dark_mode` matched; the versioned v2 TOML
(`version = 2`, `egui_version`, `name`, `mode`) with flat authoring groups
AUTHORITATIVE and the chrome `Style` DERIVED on load; the additive, tolerantly-read
`[chrome_style]` snapshot that never overrides the derived chrome; `base`
resolution with WARN-on-unresolvable and cycle detection; the five built-ins; and
the derived (not fixed-14) editable surface with Copy/Save/Save As (B081 fix) and
FFWB-native Import/Export with `THEME EXPORT`/`THEME IMPORT` command parity.

A grep over the file for `EditableToken` / `palette.ui as primary` / `primary
model` returns no matches, and a non-ASCII grep returns no matches. The guide
explicitly notes the authoritative wording lives in requirements/design and those
win on conflict, which keeps the doc from drifting ahead of the spec. This
satisfies Req 23.4 terminology honesty.

### Test-migration audit (33.2) -- nothing left, evidence supports it

The note's conclusion is a confirmation, not a change, and its four grep-backed
limbs are internally consistent with the shipped model documented in the guide:
chrome reads via `chrome_style.*`; the `chrome` -> `gutter` rename complete bar
the intentional renamed-from comment and the frozen `[chrome]` TOML header; the
`== 14` -> `> 14` editor-count migration already done; and the retained
`palette.ui.*`/`palette.tab_bar.*` references asserting the authoritative authoring
groups rather than removed fields. The `.mode`/`.name` THEME-command tests were
left unchanged per the task instruction, and the `focus_ring` tests are correctly
attributed to accessibility Req 3.1-3.3 rather than CR-CH-056. Because the diff
confirms no shipping `.rs` changed in Phase 5, an audit-only outcome is coherent;
re-running the suites was neither needed nor claimed.

### TCR correctness (33.3)

The CR-CH-056 TCR section (verified directly in `docs/quality/TCR.md`) uses the
allowed status emoji. Req 23.1-23.11, 24.1/24.2/24.3/24.5, 25.1-25.7, and reworded
13.2/18.3/20.5/20.11-20.13 are PASS, each with a cited test file and the phase that
landed it. Req 24.4 is the one deliberate RED row, annotated OUT OF SCOPE
(external-format import recorded as future) -- correctly not papered over with
invented coverage. Req 23.4 is PASS on the model tests plus the authoring guide,
with the MANUAL-exception rationale spelled out in the row, which is appropriate
for a terminology/user-facing-concept criterion that cannot be driven headlessly.

### Phases 1-4 untouched; scoped-only verification; bookkeeping

The task notes and TER.1-TER.5 markers show Phases 1-4 remain DONE/owner-confirmed
(Phase 4 = 9613 tests). Phase 5 added no shipping source and re-derived the audit
independently rather than redoing landed work. The verification note records only
scoped checks -- `cargo test -p ff-theme -p ff-theme-editor` (ff-theme 149 unit
tests pass), scoped clippy/fmt clean, and a compile-only `cargo check -p
ff-desktop` -- with an explicit hand-off for the owner's full `ffwb-gate.ps1`. No
`--workspace`/gate run was performed in-agent, matching the protocol. tasks.md
Task 33 + subtasks are `[x]`, project-master TER.6 is `[x]` (code-complete pending
gate), and change-log CR-CH-056 reads Phase 5 code-complete pending the owner's
full gate.

</details>

<details>
<summary>File map</summary>

Phase 5 change set (per the verification/implementation notes; isolated from the
larger uncommitted working tree):

- `docs/specs/theme-and-appearance/theme-authoring.md` -- NEW egui-native authoring
  guide (Req 23.4 terminology honesty).
- `docs/specs/theme-and-appearance/tasks.md` -- Task 33 + 33.1/33.2/33.3 -> `[x]`
  with DONE notes.
- `docs/project-management/project-master/tasks.md` -- TER.6 -> `[x]`,
  code-complete-pending-gate.
- `docs/quality/TCR.md` -- CR-CH-056 section: Req 23/24/25 + reworded rows PASS;
  Req 24.4 RED (out of scope).
- `docs/status/change-log.md` -- CR-CH-056 status -> Phase 5 code-complete pending
  owner full gate.
- `.agents/tasks/theme-egui-rework/phase5-note.md` + `phase5-verification.md` --
  implementation note + scoped-check record.

Out of Phase 5 scope but present in the working-tree diff: Phases 1-4 code
(ff-theme, ff-theme-editor, ff-desktop), and gate-authored docs for volume-model /
nav-ladder-semantics / localization. Not reviewed here.

</details>
