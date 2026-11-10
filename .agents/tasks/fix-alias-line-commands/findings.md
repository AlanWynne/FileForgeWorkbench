# Findings: Alias editor R and I line commands not working

Diagnosis against main commit `1532a5f` (Group B -- shared ISPF table editor +
Alias editor rebuild, CR-CH-075). READ-ONLY: no source changed.

## RECONCILIATION (follow-up on branch `fix-alias-line-commands`, after cc1a4bc)

This note supersedes the original "Root cause -- R" and "latent batch-only
hazard" sections below where they say those two defects were still LIVE on main
`1532a5f`. Re-verification against the ACTUAL tree at `1532a5f` shows both were
ALREADY fixed in the Group B merge itself:

- R is reinstated as a clone-after Copy alias. `LineCommandSet` has a `repeat`
  field, `LineCommandSet::alias()` sets `repeat: true`, `command_enabled` has a
  `Repeat | RepeatCount(_) => set.repeat` arm, and the apply pass handles
  `Repeat`/`RepeatCount` as an immediate clone-after. `AliasEditorState::line_command_set`
  returns `LineCommandSet::alias()`, so the alias host enables R. The
  `ff_line_commands` parser maps `R -> Repeat`, `Rn -> RepeatCount(n)` and
  `classify(Repeat) == Immediate`, so the Repeat arm is live, not dead.
- The batch index-shift hazard is closed. The apply pass rebuilds the row list
  by a single stable walk over ORIGINAL indices (per-row before/after/removed
  plan, payloads snapshotted up front), so a C/A batched with an I/D/R lands on
  the right rows regardless of batching.
- Tests already present and PASSING at `1532a5f`: `repeat_clones_row_after_on_non_empty_store`,
  `repeat_clones_row_after_on_single_row_store`, `repeat_count_clones_row_n_times`,
  `batch_copy_after_plus_insert_resolves_indices`, `batch_copy_after_plus_delete_resolves_indices`,
  `batch_copy_before_plus_repeat_resolves_indices`, plus `alias_line_command_set_reports_enabled_letters`.
  Scoped run on the branch: `cargo test -p ff-desktop ispf_table_editor` -> 22
  passed, 0 failed.

So the ONLY genuine user-visible defect this branch needed to add was the missing
in-cell Enter (the real "I/R line command not working" symptom): pressing Enter
with a row's Act field focused now applies the pending batch, instead of only the
`Command ===>` primary line doing so. That fix shipped in commit `cc1a4bc`
(rows.rs / host.rs / render.rs + the full-shell test
`full_shell_alias_editor_enter_in_act_cell_applies_line_command`). The earlier
conclusion that the command-line-only apply path was "sound" was correct for the
command line; what was missing was the in-cell Enter trigger, now added.

Net: R + batch were NOT missing on main and did NOT need to be re-landed; the
follow-up adds nothing to them. The original diagnosis below is retained verbatim
for history but is OUTDATED on the R/batch root-cause claims.

## Verdict (short)

- **R fails as a SINGLE command.** It is a genuine, letter-specific regression in
  the shared apply pass: `R` is parsed and classified fine, but the host's enabled
  Line_Command_Set has no Repeat entry and the shared `command_enabled` gate has no
  `Repeat` arm, so a typed `R` is treated as "not enabled", its Act cell is cleared,
  and the row list is left unchanged (plus a non-blocking "not enabled" notice).
  This is deliberate-removal collateral: the Group B rebuild dropped R and replaced
  it with C (Copy), but Req 7.6 / the owner still expect R as a clone-after alias.
- **I does NOT fail as a single command.** Every tested path for I passes: the
  shared apply-pass unit tests, the Alias-level unit tests
  (`insert_adds_blank_row_after_source`), and the full-shell empty-store test
  `full_shell_alias_editor_empty_store_i_insert_shows_editable_row` all pass. The
  apply pass inserts a blank row after the I row and clears the Act cell correctly,
  on both the empty-store scratch row and a non-empty row. I is therefore working
  at the apply-pass and live-UI levels that are covered.
- A **latent batch-only hazard exists for I/D** (NOT a single-command failure):
  when a `C`/`A` copy is batched in the SAME Enter as an immediate `I` or `D`, the
  immediate's stored ORIGINAL index is applied AFTER the copy insert has already
  grown the list, so a mixed batch can act on the wrong row. This matches the
  non-blocking finding #1 from the 1532a5f semantic review. It can make an I "look
  broken" only when combined with a C in one batch; a bare I is unaffected.

So: the reproducible, single-command break is **R**. "I not working" is not
reproducible as a single command in any covered path; the realistic I-related
defect is the batch index-shift when I/D is mixed with C/A.

## Root cause -- R (primary, single-command failure)

File: `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`
Function: `command_enabled(kind, set)` -- lines 222-233 (the `_ => false` fall-through
is line 230).

```rust
fn command_enabled(kind: &LineCommandKind, set: &LineCommandSet) -> bool {
    match kind {
        LineCommandKind::Insert => set.insert,
        LineCommandKind::Delete => set.delete,
        LineCommandKind::Copy => set.copy,
        LineCommandKind::Move => set.move_,
        LineCommandKind::After | LineCommandKind::Before => set.copy || set.move_,
        _ => false,   // <-- LineCommandKind::Repeat lands here
    }
}
```

`R` is parsed by `ff_line_commands::LineCommandParser::parse` to
`LineCommandKind::Repeat` (parser.rs, single-char arm `"R" => Repeat`) and
`ff_line_commands::command::classify` returns `LineCommandCategory::Immediate`
for it. But `command_enabled` has no `Repeat` arm, so it returns `false`.

Trace in `apply_line_commands` (same file, the "Parse + classify every non-empty
Act cell" loop, approx lines 73-110): for the R cell,
`command_enabled(&Repeat, &set)` is false, so the code pushes
`"ignored line command 'R' (not enabled)"` and adds the index to `clear`. The
clear loop then calls `host.clear_act` on that row. Net effect: the R Act cell is
blanked and NO structural change happens -- exactly the "R does nothing" symptom.

Contributing structural cause (same regression, two files):

1. `crates/ff-desktop/src/shell/ispf_table_editor/state.rs` -- `struct
   LineCommandSet` (approx lines 150-165) has fields `insert / delete / copy /
   move_ / sort` but NO `repeat` field, and `LineCommandSet::alias()` (approx
   lines 168-178) does not enable repeat (it cannot -- the field does not exist).
2. `crates/ff-desktop/src/alias_editor_panel/host.rs` -- `impl IspfTableHost for
   AliasEditorState::line_command_set` returns `LineCommandSet::alias()` (approx
   lines 62-64), i.e. I / D / C / SORT only. The doc comment and the Act-cell
   hint in `rows.rs` (`render_data_row`, "I / R / D") still advertise R, so the UI
   invites a command the apply pass rejects.

The retirement of R is spelled out in
`crates/ff-desktop/src/shell/alias_editor_line_commands.rs` (test comment approx
lines 176-179): "This supersedes the former `R` single-row repeat: the alias set
is I / D / C + SORT (CR-CH-075), and Copy is the primary duplicate command." So R
was intentionally dropped in the rebuild; the bug is that Req 7.6 and the owner
still expect R as a clone-after alias, and nothing maps it.

## Why I is NOT a single-command failure (evidence)

- Parser: `"I" => LineCommandKind::Insert`; classify -> `Immediate`
  (ff-line-commands parser.rs / command.rs).
- `command_enabled(&Insert, &set)` returns `set.insert` = true for the alias set.
- Apply pass Immediate arm (`apply_line_commands`, "Apply Immediate I/D by
  DESCENDING index", approx lines 190-215): `Insert` clears the Act cell and does
  `rows.insert(entry.index + 1, host.blank_row())`. Correct for a single I.
- Tests that currently PASS and cover this (ran `cargo test -p ff-desktop --bins
  alias_editor_line_commands` -> 7 passed):
  - `shell::alias_editor_line_commands::tests::insert_adds_blank_row_after_source`
    (single I on a non-empty row).
  - `shell::ispf_table_editor::line_commands::tests::insert_adds_blank_after_and_clears_act`.
  - full-shell `full_shell_alias_editor_empty_store_i_insert_shows_editable_row`
    (tests_focus.rs approx line 1205) drives the REAL shell: focus the scratch Act
    field, type `I`, Enter on the `Command ===>` line, and asserts a visible
    editable data row appears. This passes, so the live empty-store I path works.

What the existing I test does NOT cover: I on a row of a NON-empty store that also
has another pending line command (a mixed batch). That is the batch hazard below,
not a bare-I failure.

## Latent batch-only hazard for I/D (semantic-review finding #1)

File: `crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`,
`apply_line_commands`. Order of operations:
1. Copy/Move inserts are applied first (step 3a), growing `rows` and shifting
   indices above each insertion point.
2. Immediate I/D are applied second (step 3b) using the `entry.index` captured
   against the ORIGINAL (pre-insert) list.

The step-3b comment claims "a C/M never overlaps an I/D cell (one Act letter per
row)", which is true per-cell, but it does not account for a C/A INSERT shifting
the backing index of an unrelated I/D row that sits AFTER the insertion point. So
a batch that mixes a C/A copy with an I or D can apply the I/D to the wrong row
(or, with the `entry.index >= rows.len()` guard, silently skip it). This only
bites a MULTI-command batch; a single I or single D is immune (no insert precedes
it). No test exercises a C+I or C+D batch, which is why it is latent.

## How the Act command reaches the apply pass (for the fixer's context)

Act letters are applied only when the `Command ===>` primary line receives Enter,
not when Enter is pressed inside an Act cell. Chain (all on commit 1532a5f):
`ispf_table_editor/render.rs::render_ispf_table` -> on command-line `lost_focus()`
+ Enter yields `TableAction::ApplyLineCommands` / `CommandLine` ->
`alias_editor_panel/render.rs::render` maps it to `AliasEditorAction` ->
`shell/alias_editor.rs::apply_alias_editor_action` ->
`shell/alias_editor_line_commands.rs::apply_pending_line_commands` ->
`ispf_table_editor::apply_line_commands`. This path is sound for I/D/C; it is the
`command_enabled` gate (R) and the step-3a/3b ordering (mixed batch) that are at
fault.

## Suggested fix direction (NOT applied here)

- Reinstate R as clone-after (Req 7.6): add a `repeat` field to `LineCommandSet`,
  enable it in `LineCommandSet::alias()`, add a `LineCommandKind::Repeat` (and
  `RepeatCount`) arm to `command_enabled`, and handle `Repeat` in the apply pass as
  an immediate clone-after (insert a `clone_row` of the row at `index + 1`, Act
  cleared) -- mirroring Insert but copying field values instead of a blank row.
  Decide R-count (`Rn`) scope per Req 7.6.
- Close the batch hazard: apply immediates against indices that are adjusted for
  the copy/move inserts (or apply I/D before the inserts and translate target
  indices), so a C/A batched with an I/D lands on the right rows. Add a C+I and a
  C+D batch test.
- Per TDD, add a failing test reproducing single-R no-op (Validates: Requirement
  7.4 / 7.6) before the fix.

## Commands run during diagnosis

- `cargo test -p ff-desktop --bins alias_editor_line_commands` -> 7 passed, 0
  failed (log: `_ignore/logs/diag-alias-lc2.txt`). Confirms I/D/C apply-pass paths
  pass and there is no R test.
- `git log --oneline -5` -> confirmed HEAD at `1532a5f` (CR-CH-075 Group B merge).
