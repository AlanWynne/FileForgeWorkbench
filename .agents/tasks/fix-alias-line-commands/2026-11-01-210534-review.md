# Alias editor line commands follow-up: R reinstated, in-cell Enter, stable-index batch (pass 2)

This is a follow-up review of branch `fix-alias-line-commands` against `main`.
The branch carries one commit beyond its merge-base (`cc1a4bc`). Reviewed with
`git diff main..fix-alias-line-commands` plus the full `cc1a4bc` content, since
`main` has since advanced to `6797f5e` and already contains part of the work.

The change fixes the Alias editor's broken `R` and `I` line commands from three
angles: `R` is reinstated as a clone-after Copy alias (a `repeat` flag on
`LineCommandSet`, enabled in `alias()`, a `Repeat | RepeatCount` arm in
`command_enabled`, and a clone-after arm in the apply pass); the apply pass is
rebuilt around a per-row before/after/removed plan keyed to stable original
indices so a `C`/`A` copy batched with an immediate `I`/`D`/`R` lands on the
correct original rows; and the Act cell now reports its own `lost_focus()+Enter`
so the pending batch applies in-cell, not only from the `Command ===>` line.

Watch for (confirmed): the net merge delta is smaller than the task brief
implies. The brief frames `cc1a4bc` as the in-cell-Enter fix and expects a newer
commit to add the R-reinstatement and batch-index closure. The actual history is
the inverse: `main` (`6797f5e`) already contains the R reinstatement and the
rebuild-by-walk batch pass, so the net `main..` diff is only the in-cell-Enter
fix (host.rs, rows.rs, render.rs) plus the full-shell test. The R/batch logic is
present and correct in the branch tree, it simply does not appear in `main..`
because it is identical on both sides. findings.md documents this reconciliation
accurately (likely intentional given the brief's wording).

**Verdict**: APPROVED

## High-level view

The apply pass in `ispf_table_editor/line_commands.rs` builds, for each of the
`len` original rows, a `before` and `after` insert list and a `removed` flag, all
keyed to original index, then rebuilds the output by walking originals in order
and emitting `[before] + row? + [after]`. No list mutation happens until the
final walk, so the relative order of a copy insert and an immediate `I`/`D`/`R`
can no longer shift each other's target. This retires the latent index-shift
hazard findings.md flagged for mixed batches. Move is preserved as a copy op with
`removed[source]=true`; Move is not in the alias set, so it is unaffected here.

`R`/`Rn` is handled in the immediate loop as a single-row clone-after Copy
(`RepeatCount(n)` uses `n.max(1)`), gated by `set.repeat` so only opt-in hosts
accept it; `enabled_letters` now advertises `R` and `S`. No `ff-line-commands`
public API changed: the parser already produced `Repeat`/`RepeatCount`, and only
the downstream host gate and apply pass gained arms.

The in-cell Enter is threaded minimally: `render_data_row` returns its Act field's
`lost_focus()+Enter`, `render_row_cells` propagates the bool, and
`render_ispf_table` OR-folds it across every rendered row (including the promoted
empty-store scratch row) and raises `ApplyLineCommands` only when no command-line
action already claimed the frame, so there is no double-apply. The stable
`alias_act` egui id is untouched, so the focus-latch contract and the full-shell
focus tests are unaffected.

TDD is honored: every new test carries a `// Validates:` annotation (Req 7.4 /
7.6 / ispf-table-editor 4.3). Coverage spans the shared apply pass (single R,
single-row R, `Rn`, and C+I / C+D / C+R batches), the alias-level adapter (single
R and single I on empty/non-empty stores, mixed C+A+I batch), and a full-shell
egui_kittest test that focuses an Act cell, types I/R/D, presses Enter in the
cell, and asserts the structural result.

The two non-blocking items from the prior review pass are resolved or correctly
deferred: the findings/fix alignment gap is now reconciled in findings.md (a
RECONCILIATION section supersedes the earlier "I does not fail as a single
command" claim and records that the net new defect was the missing in-cell
Enter), and the authoritative full `cargo gate` before merge remains the owner's
step per gate-policy.md.

<details>
<summary>Issues (2)</summary>

1. **Brief-vs-history mismatch (confirmed, informational)** -- the task expects a
   commit newer than `cc1a4bc` carrying the R and batch fixes; in fact `main`
   already holds those, so `cc1a4bc` is the newest commit and the net `main..`
   delta is only the in-cell-Enter fix plus tests. Nothing to change in the code;
   reviewer should read the merge delta as the in-cell-Enter fix and treat the
   R/batch logic as already-landed-and-identical. Non-blocking.
2. **Owner full-gate still required** -- the coder recorded scoped checks and
   scoped tests only (check/clippy/fmt clean; ispf_table_editor 22, alias_editor
   52, tests_focus 94 passed), per the gate-policy.md execution split. The
   authoritative whole-suite `cargo gate` before merge to `main` is the owner's
   responsibility and has not been run. Non-blocking for this review.

</details>

<details>
<summary>Details</summary>

### Rebuild-by-walk closes the index-shift hazard

The former pass applied copy/move inserts first (growing the list and shifting
every index above each insertion point) and then applied immediate `I`/`D`
against indices captured on the original list. For `len` originals the new model
allocates `before: Vec<Vec<Row>>`, `after: Vec<Vec<Row>>`, and `removed:
Vec<bool>`, all indexed by original position: copy/move push a cleared clone onto
`before[target]`/`after[target]` and set `removed[source]` for a move; insert
pushes `blank_row()` onto `after[index]`; delete sets `removed[index]`; repeat
pushes `count` clones onto `after[index]`. The output is rebuilt by walking the
originals in order. Because every decision is keyed to an original index and no
mutation happens until the walk, the ordering of ops relative to each other no
longer matters. `batch_copy_after_plus_insert_resolves_indices`,
`batch_copy_after_plus_delete_resolves_indices`, and
`batch_copy_before_plus_repeat_resolves_indices` lock this behavior in, and the
alias-level `batch_insert_plus_copy_after_resolves_indices` proves it through the
real host adapter.

Insert payloads (copy clones, repeat clones, blank rows) are snapshotted via
`host.clone_row`/`host.blank_row` while the original indices are still valid, so
the final `std::mem::take(rows)` walk cannot read a stale row. A move's source
removal is independent of its target's before/after insert, so a move whose
target precedes its source rebuilds correctly; Move is not in the alias set, so
this path is exercised only by the shared-pass suite the owner's gate runs.

### R reinstated as a clone-after Copy alias

`R`/`Rn` clones the row, clears both the clone's and the source's Act cell, and
pushes `count` clones onto `after[index]`, mirroring a single-row Copy-after,
which matches Req 7.6's "single-row Copy alias" framing. `command_enabled` routes
`Repeat | RepeatCount(_)` through `set.repeat`, so non-alias hosts still clear a
typed `R` with the existing "not enabled" notice -- no behavior change for them.
`LineCommandSet::alias()` sets `repeat: true`, and `enabled_letters` advertises
`R` and `S`, so the UI hint matches the accepted set.

### In-cell Enter and the empty-store scratch row

`act_enter` is OR-folded across every rendered row and, notably, across the
promoted empty-store scratch row, so the first entry created via an Act command
on an empty store also triggers the batch after promotion. The `matches!(action,
TableAction::None)` guard keeps a command-line Enter (which already applies the
batch) from being doubled by an Act-cell Enter in the same frame. The Act
`TextEdit` keeps its `alias_act` id, so no focus test regresses and the single
`render_workspace_context` focus-latch path is untouched.

### Constraints verified

- `environment.rs` (the resolver) is absent from both the `cc1a4bc` stat and the
  net `main..` diff -- the resolver is untouched.
- No `ff-line-commands` public API change; the parser already emitted
  `Repeat`/`RepeatCount` and `classify(Repeat) == Immediate`.
- Touched source files are under 400 non-test lines: line_commands.rs 251,
  state.rs 273, render.rs 227, rows.rs 146, host.rs 165,
  alias_editor_line_commands.rs 146.
- All touched files are ASCII-clean.
- ISPF explicit-commit save seam, the single focus-latch path, and the stable
  `alias_act` id are preserved (no save-path or focus-anchor changes in the diff).
- `cc1a4bc` (the in-cell-Enter fix the brief says must be preserved) is intact; it
  is the branch HEAD and is the commit under review.

</details>

<details>
<summary>File map</summary>

- `alias_editor_panel/host.rs` -- `render_row_cells` returns the Act-cell Enter bool.
- `alias_editor_panel/rows.rs` -- `render_data_row` captures `lost_focus()+Enter` on
  the Act field and returns it; stable `alias_act` id preserved.
- `shell/alias_editor_line_commands.rs` -- doc/comment updates for R; alias-level
  tests (single R empty/non-empty, single I single-row, mixed C+A+I batch).
- `shell/ispf_table_editor/line_commands.rs` -- rebuild-by-walk apply pass, Repeat
  clone-after arm, `command_enabled` Repeat/RepeatCount arm; new apply-pass tests.
- `shell/ispf_table_editor/render.rs` -- `render_row_cells` returns bool; fold
  `act_enter` and raise `ApplyLineCommands` when no command-line action claimed the
  frame.
- `shell/ispf_table_editor/state.rs` -- `repeat` field, `alias()` enables it,
  `enabled_letters` advertises R and S; updated set test.
- `shell/tests_focus.rs` -- full-shell `full_shell_alias_editor_enter_in_act_cell_applies_line_command`.

Net merge delta (`main..`): host.rs, rows.rs, render.rs, tests_focus.rs. The
R/batch logic in line_commands.rs/state.rs is already present on `main` and is
identical on the branch.
</details>
