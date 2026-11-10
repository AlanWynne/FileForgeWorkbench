# Findings: ISPF Alias Editor seeded-example + Save/Cancel/End semantics + FFCMD/FFEDIT Save ownership

Read-only investigation. Main working tree `c:\workspace\apps\FileForgeWorkbench`,
HEAD `15434d0` (CR-CH-069 merge). CR-CH-070 (`=FILES` fix) is on branch
`eqfiles-fix` and is NOT merged; this report reads main only. All claims are
grounded in live source at the cited file:line.

---

## Summary answers (read this first)

**(A) The "seeded ECOB example" is NOT seeded data. It is placeholder ghost
text.** There is no ECOB row anywhere in the default/empty alias store, the
reset-bare baseline, the live resolver seed, or any shipped TOML. What the owner
saw -- `FFCMD / <blank root> / en / ECOB / FFWB.COBOL.MASTER(&1)` -- is the single
EMPTY insertable scratch row (Req 5.17 / design 9.10.6) rendered with:
- CE column = `ffcmd` because `AliasRowEdit::default().environment` is
  `AliasEnvironment::Ffcmd` (a real selected value, not a hint);
- Root column = `--` (empty -> the "blank root" the owner noted);
- L18N / ALIAS / Parameters columns = egui `hint_text` ghost placeholders
  `"en"` / `"ECOB"` / `"FFWB.COBOL.MASTER(&1)"`, chosen during CR-CH-068 to
  illustrate the worked example.
The scratch row is never promoted into `rows` unless edited, so "empty store =
zero persisted rows" is still true. The fix is purely cosmetic: change the three
hint strings to generic placeholders (or remove them) in
`crates/ff-desktop/src/alias_editor_panel/rows.rs`. Nothing in reset-bare or any
`DEFAULT_*_TOML` references an alias example -- confirmed clean.

**(B) Current Save/End/Cancel semantics do NOT match the desired ISPF model, and
there are TWO parallel command interpreters.** The Alias Editor has its own
`Command ===>` field whose verbs are interpreted by
`WorkbenchShell::apply_alias_command_line`
(`crates/ff-desktop/src/shell/alias_editor_line_commands.rs:71-99`):
`SAVE` -> persist+stay (correct); `CANCEL` AND `END` -> BOTH call `nav_end()`,
i.e. leave WITHOUT saving and WITHOUT an explicit discard. There is NO `EXIT` and
NO `RETURN` handling in the panel field, and NO distinction between END and
CANCEL. Separately, a function-key F3 / globally-typed `END` / `RETURN` / `X` on
this Context does NOT reach the panel interpreter at all -- it goes through the
shell's FFCMD ladder (`try_commands_a` -> `nav_end`/`nav_return`/`nav_x`), none of
which save. So today: END never saves (gap vs desired), CANCEL does not discard
(gap), SAVE works but only from the panel command line (not a workbench verb).

**(C) Making SAVE/CANCEL FFCMD-owned with an FFEDIT override is FEASIBLE and
consistent with the command-environment model, and it is an ADDITIVE
placement/ownership change of the SAME class as CR-CH-069 (AUTONUM/NUM), NOT a
core framework change.** It requires no new dispatcher, no new navigation stack,
no reshape of a public framework type. The blast radius is moderate and bounded.
The recommended shaping keeps FFEDIT's existing buffer SAVE exactly as-is
(active-wins) and adds an FFCMD-base Save/Cancel that a focused editable
non-buffer Context (Alias Editor and the sibling config editors) handles by
persisting/discarding its own working copy. BUT the ISPF END=save / CANCEL=discard
/ SAVE=stay semantics directly CONTRADICT the current approved criteria 5.9 and
5.14, so that half is a GATED requirements change, while removing the ghost text
(A) is a trivial quick-fix needing no gate.

---

## (A) Source of the "seeded ECOB example" + the precise fix

### Trace: how the Alias Editor opens and renders the empty store

1. `ALIASES` verb -> classified as a `CustomWorkspace` target
   (`crates/ff-desktop/src/command_config/mod.rs:183,336`) through the single
   front door, which calls `open_alias_editor`.
2. `WorkbenchShell::open_alias_editor`
   (`crates/ff-desktop/src/shell/alias_editor.rs:33-50`) calls
   `load_user_alias_rows` then `self.alias_editor_panel.load_rows(rows)`.
3. `load_user_alias_rows` (`alias_editor.rs:54-78`): if the user store path is
   absent/unreadable/malformed it returns `Vec::new()` -- an EMPTY working copy.
   There is NO default/sample row injected here.
4. `AliasEditorState` (`crates/ff-desktop/src/alias_editor_panel/state.rs`) has
   `rows: Vec<AliasRowEdit>` defaulting to empty; `new()` is `Self::default()`.
   No seeded rows.
5. Render: `crates/ff-desktop/src/alias_editor_panel/render.rs:118-133`. WHEN
   `state.rows.is_empty()` it calls `render_scratch_row`
   (`render.rs:186-192`), which builds ONE `AliasRowEdit::default()`, renders it
   via `render_data_row`, and only pushes it into `rows` if it was edited
   (`scratch != AliasRowEdit::default()`). So the empty editor shows exactly one
   blank insertable row and persists nothing.

### The actual origin of each column the owner saw

`render_data_row` (`crates/ff-desktop/src/alias_editor_panel/rows.rs:82-133`)
renders the blank scratch row. For a DEFAULT `AliasRowEdit` this produces:

| Column | What renders for the blank scratch row | Source |
|--------|----------------------------------------|--------|
| Act | empty, hint `"_"` | `rows.rs:88` hint_text |
| CE | **`ffcmd`** (a real selected value) | `AliasRowEdit::default().environment == AliasEnvironment::Ffcmd` -- `user_aliases.rs:34-40` `#[default] Ffcmd`; combo shows `env.label()` = `"ffcmd"` (`rows.rs:95`, `render_ce_combo`) |
| Root | **`--`** (empty) | `render_root_combo` shows `"--"` when canonical is empty (`rows.rs:52-55`) -- the owner's "blank root" |
| L18N | ghost **`en`** | `rows.rs:110` `.hint_text("en")` |
| ALIAS | ghost **`ECOB`** | `rows.rs:117` `.hint_text("ECOB")` |
| Parameters | ghost **`FFWB.COBOL.MASTER(&1)`** | `rows.rs:129` `.hint_text("FFWB.COBOL.MASTER(&1)")` |

So the owner's reported row `FFCMD / <blank> / en / ECOB / FFWB.COBOL.MASTER(&1)`
is category (i) PARTIALLY -- the CE `ffcmd` is a real default value -- combined with
category (iii) -- the design's worked example embedded as egui ghost placeholder
text (`hint_text`), which only displays while the field is empty and is NEVER
written to the model or to disk.

This is the ONLY place `ECOB` / `FFWB.COBOL.MASTER(&1)` appear as a rendered row.
A repository-wide search for `ECOB` and `COBOL.MASTER` finds them ONLY in:
hint_text at `rows.rs:129`; doc-comment examples in `user_aliases.rs:80` and
`environment.rs`; test fixtures (`user_aliases.rs` tests, `tests_alias_editor.rs`,
`locale_resolver.rs` tests, `alias_reader.rs` tests); and change-log / spec prose.
None of these is production seed data.

### Confirming reset-bare / DEFAULT_*_TOML are clean

- The live resolver seed `LocaleAliasStore::seed`
  (`crates/ff-desktop/src/shell/alias_reader.rs:230-241`) seeds FFEDIT from
  `AliasTable::ffedit_english()` (identity verbs + X->EXCLUDE, INCLUDE->SHOW --
  `environment.rs:117-180`) and FFCMD from `identity_for(builtin_ffcmd_roots())`.
  No ECOB, no example row. Test
  `seeded_store_is_identity_only_without_overlay` (`alias_reader.rs`) asserts the
  seeded store is identity-only.
- `builtin_ffcmd_roots` (`alias_reader.rs:159-200`) derives roots from the
  code-only built-in menus + registered verb roots; it carries no alias example.
- The user store loader treats absent/empty/malformed as empty
  (`user_aliases.rs:149-160` `user_aliases_file`; `parse_user_alias_toml`
  `user_aliases.rs:166-170`; `alias_editor.rs:54-78`). A plain
  `pure_ffcmd_row_serialises_to_minimal_shape` test confirms the serialiser emits
  nothing by default.

### The fix (small, no gate)

In `crates/ff-desktop/src/alias_editor_panel/rows.rs`, change the three
illustrative `hint_text` strings on the DATA-row fields so the empty editor does
not read as a pre-filled ECOB example:
- `rows.rs:110` L18N `hint_text("en")`
- `rows.rs:117` ALIAS `hint_text("ECOB")`
- `rows.rs:129` Parameters `hint_text("FFWB.COBOL.MASTER(&1)")`

Recommended: use neutral placeholders (e.g. ALIAS `"name"`, Parameters
`"(&1)"`/empty, L18N empty) OR drop the hint entirely. The owner's remark that the
example "has no Root command" is explained by the blank Root dropdown (`--`) on a
default row; it is not a malformed data row, so no data fix is needed -- only the
placeholder cosmetics. Nothing in reset-bare / DEFAULT_*_TOML needs touching
(already clean). Optional belt-and-braces: a test asserting the freshly opened
empty editor has `rows.len() == 0` already exists
(`tests_focus.rs:1221`, "the user store is empty (zero persisted rows)").

---

## (B) Current vs desired SAVE / END / EXIT / CANCEL / RETURN behaviour

### Two separate command paths exist on the Alias Editor Context

1. **Panel `Command ===>` field** -> `AliasEditorAction::CommandLine(text)` ->
   `apply_alias_editor_action` (`alias_editor.rs:88-94`) ->
   `apply_alias_command_line` (`alias_editor_line_commands.rs:71-99`). This is the
   ONLY place SAVE/CANCEL/END are interpreted for the alias editor.
2. **Function keys / global dispatch** (F3, or a verb typed where the shell owns
   it) -> `dispatch_command_string` (`dispatch.rs`) -> FFCMD ladder. On the Alias
   Editor the active environment is FFCMD (the Context kind `AliasEditor` maps to
   FFCMD, not FFEDIT -- `environment_registry.rs` `active_name`; only editor-buffer
   kinds are FFEDIT), so FFEDIT never claims anything and END/RETURN/X resolve via
   `try_commands_a` (`commands_ladder_a.rs:215-262`).

The panel field and the global ladder are UNRELATED: F3 does NOT route into
`apply_alias_command_line`.

### Table: current vs desired (ISPF)

| Verb / key | Current behaviour (file:line) | Desired ISPF behaviour | Gap |
|------------|-------------------------------|------------------------|-----|
| `SAVE` (panel line) | `save_user_aliases()` -- validate (`validate_user_alias_rows` `alias_editor.rs:187-243`), write `user.toml`, refresh live store in place (`alias_editor.rs:104-170`). Stays in Context. (`alias_editor_line_commands.rs:83`) | persist + stay | **MATCHES** |
| `END` (panel line) | `nav_end()` -- pop Navigation_Stack, reconstruct parent, NO save (`alias_editor_line_commands.rs:84`; `nav_stack.rs:161-167`) | **save THEN leave** | **GAP: END does not save** |
| `END` / F3 (function key, global) | FFCMD ladder `commands_ladder_a.rs:215-237` -> `nav_end()`. NO save. | save THEN leave | **GAP + path divergence** (panel field is bypassed) |
| `CANCEL` (panel line) | `nav_end()` -- leaves, NO explicit discard (`alias_editor_line_commands.rs:84`) | leave WITHOUT saving (discard working changes) | **GAP: identical to END; no explicit discard** |
| `CANCEL` (function key / global) | NOT handled anywhere in the FFCMD ladder -- `CANCEL` is not an FFCMD verb (grep: only in the panel interpreter, the FFEDIT help topic list, and the navigation-commands delegation table). Falls through to the CommandEngine terminal ("unrecognised"). | leave WITHOUT saving | **GAP: no global CANCEL verb at all** |
| `EXIT` (panel line) | NOT handled -> `_ => no-op` arm (`alias_editor_line_commands.rs:94-96`). Stays in Context. | save THEN leave (ISPF EXIT == END) | **GAP: EXIT ignored in the editor** |
| `EXIT` (function key / global) | FFCMD `try_exit_family` (`commands_ladder_a.rs:32-44`) -- UNCONDITIONAL app exit (EXIT/QUIT/LOGOFF). NOT a save-then-leave-this-Context. | save THEN leave the Context | different concept (app exit) -- see note |
| `RETURN` (panel line) | NOT handled -> `_ => no-op`. Stays. | leave WITHOUT saving (ISPF RETURN == CANCEL, discard) | **GAP: RETURN ignored in the editor** |
| `RETURN` (function key / global) | FFCMD ladder `commands_ladder_a.rs:239-247` -> `nav_return()` -- collapse to tab visual root, NO save / NO discard (`nav_stack.rs:180-186`) | leave WITHOUT saving | GAP (no discard semantics) |

Notes:
- `nav_end` (`nav_stack.rs:161-167`) and `nav_return` (`nav_stack.rs:180-186`)
  perform pure navigation. Because `open_alias_editor` RELOADS rows from disk on
  every open (`alias_editor.rs:35-36`), unsaved edits are effectively dropped on
  the NEXT open, but there is no explicit discard at leave time and the stale rows
  linger in `self.alias_editor_panel.rows` until then. So "CANCEL discards" is
  only accidentally true and is not an explicit, immediate discard.
- The current spec matches the current code: Req 5.9 and Req 5.14
  (`docs/specs/command-configurator/requirements.md`) define END and CANCEL as
  BOTH "leave via the per-tab Navigation_Stack" with NO save and NO discard
  distinction. tasks.md 7.5.3 confirms "END/F3 returns via the Navigation_Stack".
  The owner's new ISPF model therefore CONTRADICTS the approved criteria and needs
  a gate (see Plan).

### Sibling editors (Settings / Menus / Theme / Keys / Kinds) -- comparison

All sibling configurator editors follow the SAME Menus-Editor pattern (the Alias
Editor was explicitly "modelled on `shell/menus_editor.rs`", tasks.md Task 7).
Their save is an explicit panel action (e.g. a SAVE command / Save affordance),
and END/F3 returns via the Navigation_Stack WITHOUT saving -- identical to the
Alias Editor. None of them implements ISPF END=save / CANCEL=discard today. This
is why the owner's proposal is naturally a WORKBENCH-LEVEL (FFCMD) concern: the
same Save/Cancel semantics should apply uniformly to every editable non-buffer
Context, not be re-implemented per editor. (Confirm the exact per-editor save
hooks during design; they live in the respective `shell/<x>_editor.rs` /
`apply_<x>_editor_action` methods, mirroring `apply_alias_editor_action`.)

---

## (C) SAVE / CANCEL command-environment ownership -- evaluation + recommendation

### How SAVE is owned today

- SAVE is an FFEDIT root: `AliasTable::ffedit_english()` includes `("SAVE","SAVE")`
  (`environment.rs:149`). FFEDIT classifies SAVE as the one STORE-AFFECTING verb
  (`is_store_affecting_verb` `environment_ffedit.rs:43-44`).
- `FfEditEnvironment::claim` (`environment_ffedit.rs:67-343`) handles SAVE by
  reading the tab's Owning_Environment and ADDRESSing SAVE to it via
  `dispatch_to_environment` (`environment_ffedit.rs:333-338`) -- the host-FS
  backend performs the dirty-aware buffer write (CR-CH-053 Task 20, Req
  14.4/14.5/14.6, which MODIFIES Req 10.1 routing).
- The claim only fires when the active environment is FFEDIT, i.e. an editor
  buffer Context (`commands.rs:155-198` `run_command_ladder` active-wins gate ->
  `is_ffedit_active`). On the Alias Editor (FFCMD-active) SAVE is NOT an FFCMD
  root (`builtin_ffcmd_roots` has no SAVE), so a globally-typed SAVE would fall
  through to the CommandEngine terminal. The alias editor's SAVE works ONLY
  because the panel's own `Command ===>` interpreter intercepts it
  (`alias_editor_line_commands.rs:83`) before any shell dispatch.
- There is NO CANCEL verb in FFCMD or FFEDIT today (grep confirms CANCEL exists
  only in the alias panel interpreter, the FFEDIT help topic list in
  `help.rs:462`, and the `ff-navigation-commands` delegation table which marks
  SAVE/CANCEL/END as non-undoable file-ops). It is not a dispatchable root.

### The command-environment model (CR-CH-053) the proposal must fit

- FFCMD is the always-present BASE and IS `resolve_target` (Req 1.2) -- not a
  `CommandEnvironment` trait object. Root enumeration for it comes from
  `builtin_ffcmd_roots()` via `EnvironmentRegistry::root_commands("FFCMD")`
  (`environment_registry.rs:270-318`).
- Active-wins (Req 5.1): the focused Context's kind supplies the active
  environment; that environment gets FIRST crack at a verb before the FFCMD base
  (`commands.rs` `run_command_ladder` gate). An editor buffer -> FFEDIT; every
  other Context (incl. the Alias Editor) -> FFCMD.
- CR-CH-069 precedent: AUTONUM/NUM were MOVED from the FFCMD ladder into FFEDIT
  roots so they are claimed only on an editor Context -- an additive placement
  change, gated as a normal CR, NOT a framework change
  (`environment.rs:151-179`, `environment_ffedit.rs` AUTONUM/NUM arms). The
  Save/Cancel ownership question is explicitly the SAME class.

### Is "FFCMD-owned Save/Cancel with FFEDIT override" feasible and consistent? YES.

- Making SAVE (and a new CANCEL) FFCMD roots means: add them to
  `builtin_ffcmd_roots()` (or the registered verb-root union) so
  `root_commands("FFCMD")` / `contains_root` enumerate them (CR-CH-067), and give
  the FFCMD base a handler that routes Save/Cancel to the FOCUSED editable
  Context's working-copy persist/discard. For the Alias Editor this is exactly the
  existing `save_user_aliases` (persist) and a new discard (reload-from-disk or
  clear working copy) -- reachable without the panel's private interpreter.
- FFEDIT keeps its SAVE root and its store-addressing claim unchanged. Because of
  active-wins, on an editor buffer FFEDIT's SAVE is reached FIRST and overrides
  the FFCMD base (active-wins IS the override mechanism -- no new machinery). On a
  non-buffer editable Context FFCMD's Save handles it. This is precisely the
  owner's "FFEDIT may reuse or override FFCMD's Save/Cancel": reuse = fall through
  to FFCMD base when FFEDIT does not claim; override = FFEDIT claims SAVE first
  when active. CANCEL can be left FFCMD-only initially (FFEDIT declines -> falls
  to base) or claimed by FFEDIT later for a buffer-revert, additively.
- This adds NO second dispatcher, NO new navigation stack, NO new persistence
  format, and reshapes NO public framework type. By the framework-conformance
  definition it is an ADDITIVE placement/ownership change (like CR-CH-069), NOT a
  framework change. It should still run the normal requirements gate because it
  changes observable behaviour and CONTRADICTS existing criteria (5.9/5.14).

### Blast radius

1. **FFEDIT SAVE + its tests** -- UNCHANGED if FFEDIT keeps its SAVE root and the
   store-addressing claim. The active-wins gate already orders FFEDIT ahead of the
   FFCMD base, so adding SAVE as an FFCMD base root does not shadow the buffer
   SAVE. Tests to re-run: `save_is_classified_store_affecting`,
   `in_buffer_verbs_are_not_store_affecting` (`environment_ffedit.rs` tests),
   and any host-FS save-addressing tests.
2. **Root enumeration (CR-CH-067)** -- SAVE (and CANCEL) would newly appear in
   `root_commands("FFCMD")`. That feeds the Alias Editor's own Root dropdowns and
   the save-side `contains_root` validation
   (`environment_registry.rs:270-335`; `validate_user_alias_rows`
   `alias_editor.rs:210-225`). Adding verbs here means a user could author an
   alias whose root is SAVE/CANCEL -- decide intentionally whether that is allowed
   (the no-shadow-built-in-root rule, Req 5.4/5.7, would then REJECT a user alias
   named SAVE/CANCEL, which is probably desirable). Both FFCMD and FFEDIT root
   lists must stay consistent so `contains_root` accepts SAVE under both.
3. **Editors relying on END/return** -- the Alias Editor and every sibling config
   editor (Settings/Menus/Theme/Keys/Kinds) currently leave via `nav_end`/
   `nav_return` without saving. Introducing END=save-then-leave changes behaviour
   for ALL of them uniformly (the point of FFCMD ownership). Each editor needs a
   persist hook and a discard hook wired to the FFCMD Save/Cancel/End handler;
   this is additive to each `apply_<x>_editor_action`.
4. **The Alias Editor's own save path** -- `save_user_aliases` is reused as the
   Alias Editor's persist hook; the panel's private `apply_alias_command_line`
   SAVE/CANCEL/END interpreter should be RETIRED in favour of the FFCMD verbs so
   there is one dispatch path (removing the current divergence in (B)).
5. **Spec criteria** -- Req 5.9 and 5.14 (and the sibling editors' equivalent
   return criteria) must be amended; the help topic list and
   `ff-navigation-commands` delegation entries for SAVE/CANCEL/END may need review.

### Recommended cleanest shaping

- KEEP FFEDIT's SAVE root and store-addressing claim exactly as-is (buffer SAVE
  is active-wins; zero change to the byte-identical host-FS write).
- ADD an FFCMD-base Save/Cancel/End semantic for FOCUSED editable non-buffer
  Contexts: FFCMD `SAVE` persists the focused Context's working copy; `CANCEL`
  discards it then leaves; `END`/`EXIT` persists then leaves; `RETURN` discards
  then leaves (ISPF). Implement as a small focused-Context "editable working copy"
  seam the FFCMD base calls (persist / discard), with the Alias Editor's
  `save_user_aliases` as the first implementor and a `discard` = reload-from-disk.
- Register SAVE/CANCEL as FFCMD roots in the enumeration so they are first-class
  verbs reachable from the command line and function keys, and RETIRE the alias
  panel's private SAVE/CANCEL/END interpreter so there is ONE path.
- Treat END/EXIT = save-then-leave and CANCEL/RETURN = discard-then-leave as the
  universal rule for editable Contexts, applied through the shared FFCMD handler
  (do NOT re-implement per editor).

This is an additive, framework-conformant ownership/placement change (CR-CH-069
class) PLUS a gated behaviour change to the ISPF semantics; it is NOT a core
framework change.

---

## PLAN OF ACTION (proposed; implement nothing yet)

Split into a trivial quick-fix and a gated design piece. BOTH should land AFTER
CR-CH-070 (`eqfiles-fix`) is merged to avoid a merge collision on the shell
command/dispatch files, and both must coordinate with the pending Afrikaans-data
task (the `af/aliases/*.toml` work referenced in CR-CH-069 / the change log),
since that task also touches the alias overlay/root enumeration surface.

### Piece 1 -- QUICK FIX (no gate; refactor-class, cosmetic)
- Scope: remove the illustrative ghost placeholders so the empty Alias Editor
  does not read as a pre-filled malformed ECOB example.
- Change: the three `hint_text` strings in
  `crates/ff-desktop/src/alias_editor_panel/rows.rs:110,117,129` to neutral
  placeholders (or empty). Optionally add a test asserting the freshly opened
  empty editor shows no populated row (the zero-rows assertion already exists at
  `tests_focus.rs:1221`).
- Size: XS (1 file, ~3 lines, + optional 1 test). No behaviour change, no gate
  (documentation-charset/ASCII still satisfied; no model change).
- Risk: negligible; purely display.

### Piece 2 -- GATED DESIGN (requirements gate required)
- Scope: the ISPF Save/End/Cancel/Return semantics (END/EXIT = save-then-leave;
  CANCEL/RETURN = discard-then-leave; SAVE = persist-and-stay) AND the FFCMD-owned
  Save/Cancel with FFEDIT override ownership decision.
- Why gated: it CONTRADICTS approved criteria 5.9 and 5.14
  (`docs/specs/command-configurator/requirements.md`) which define END and CANCEL
  as both "leave via the Navigation_Stack" with no save/discard distinction, and
  it changes observable behaviour across all sibling config editors. Per
  workflow.md this is a CHANGE REQUEST that must run the gate; per
  framework-conformance.md the ownership move itself is additive (CR-CH-069 class)
  and does NOT need the separate "framework change" owner-confirmation, but the
  behaviour/criteria change does need the normal gate + owner CONFIRM.
- Gate deliverables:
  1. Amend command-configurator Req 5.9/5.14 (and note the sibling editors'
     equivalent return criteria) to the ISPF model; add criteria for FFCMD
     Save/Cancel ownership + FFEDIT active-wins override.
  2. Design delta: the focused-editable-Context persist/discard seam the FFCMD
     base calls; SAVE/CANCEL added to FFCMD root enumeration
     (`builtin_ffcmd_roots` / `root_commands`); retire the alias panel's private
     SAVE/CANCEL/END interpreter; keep FFEDIT SAVE untouched.
  3. Tasks: wire the seam for the Alias Editor first (reusing `save_user_aliases`
     + a reload-based discard), then the sibling editors; update the no-shadow
     rule so a user alias cannot be named SAVE/CANCEL.
- Size: M (requirements + design + tasks + implementation across the FFCMD base
  handler, root enumeration, and each editor's persist/discard hook; plus tests:
  FFEDIT SAVE unchanged, FFCMD Save/Cancel on a non-buffer Context, END=save /
  CANCEL=discard behaviour, root-enumeration update).
- Risk: moderate -- touches dispatch/ownership and multiple editors; mitigated by
  keeping FFEDIT SAVE byte-identical and reusing active-wins as the override.

### Sequencing
1. Merge CR-CH-070 (`eqfiles-fix`) first.
2. Coordinate/merge the pending Afrikaans-data task (it touches
   `af/aliases/*.toml` and the alias overlay/root surface; Piece 2's root
   enumeration change overlaps).
3. Land Piece 1 (quick fix) any time after step 1 -- it is independent and safe.
4. Run the gate for Piece 2, then implement after owner CONFIRM.

---

## Evidence index (file:line)

- Empty-store scratch row: `crates/ff-desktop/src/alias_editor_panel/render.rs:118-133,186-192`
- Ghost placeholders (the "example"): `crates/ff-desktop/src/alias_editor_panel/rows.rs:110,117,129`; CE default `crates/ff-desktop/src/shell/user_aliases.rs:34-40`; Root `--` `rows.rs:52-55`
- Open/load empty: `crates/ff-desktop/src/shell/alias_editor.rs:33-78`
- Panel SAVE/CANCEL/END interpreter: `crates/ff-desktop/src/shell/alias_editor_line_commands.rs:71-99` (SAVE:83, CANCEL|END:84)
- Save path: `crates/ff-desktop/src/shell/alias_editor.rs:104-170,187-243`
- Nav (no save): `crates/ff-desktop/src/shell/nav_stack.rs:161-167 (nav_end), 180-186 (nav_return), 215-224 (nav_x)`
- FFCMD exit/END/RETURN/X ladder: `crates/ff-desktop/src/shell/commands_ladder_a.rs:32-44 (exit family), 215-262`
- Active-wins gate: `crates/ff-desktop/src/shell/commands.rs:80-86,155-198`
- FFEDIT SAVE claim + store-affecting classification: `crates/ff-desktop/src/shell/environment_ffedit.rs:43-44,319-338`; `crates/ff-desktop/src/shell/environment.rs:149`
- Front door dispatch: `crates/ff-desktop/src/shell/dispatch.rs:54-120`
- Root enumeration (CR-CH-067): `crates/ff-desktop/src/shell/environment_registry.rs:270-335`; `crates/ff-desktop/src/shell/alias_reader.rs:159-200`
- Live-store seed is identity-only: `crates/ff-desktop/src/shell/alias_reader.rs:230-241`
- Spec criteria 5.9/5.14 (current END/CANCEL = leave, no save): `docs/specs/command-configurator/requirements.md` Req 5.9 (line ~ "WHEN F3 or `END`") and Req 5.14
- CR-CH-070 unmerged: `git branch` shows `eqfiles-fix` present, not on `main` (HEAD `15434d0`)
