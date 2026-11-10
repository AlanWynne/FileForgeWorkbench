# Ghost-text quick fix note (Piece 1, cosmetic, no gate)

Removed the illustrative ECOB/COBOL ghost-text placeholders from the ISPF Alias
Editor's empty scratch/data row so a freshly opened empty editor no longer reads
as a pre-filled example. This is display-only (`egui` `hint_text`, shown only
while a field is empty and never written to the model, store, or disk), so there
is no behaviour, model, seed, reset-bare, or `DEFAULT_*_TOML` change and no
requirements gate is needed (confirmed by findings.md section A).

## Exact change (all in crates/ff-desktop/src/alias_editor_panel/rows.rs)
| Field | hint_text before | hint_text after |
|-------|------------------|-----------------|
| L18N (`alias_l18n`) | `"en"` | `""` (empty) |
| ALIAS (`alias_surface`) | `"ECOB"` | `"name"` |
| Parameters (`alias_template`) | `"FFWB.COBOL.MASTER(&1)"` | `"(&1)"` |

The CE default (`ffcmd`) and the Root empty display (`--`) were deliberately NOT
touched -- they are real selected/empty values, not illustrative hints.

## Test / guard
No new test added: `hint_text` is passed inline to the egui `TextEdit` builder
and is not exposed on the model, so it cannot be asserted cheaply in a pure unit
test without a full-shell render harness (out of scope for a hint-string change).
The existing guard that the empty editor persists zero rows still holds and was
NOT weakened: `shell/tests_focus.rs` ~L1221-1226
(`full_shell_alias_editor_empty_store_i_insert_shows_editable_row`) asserts
`alias_editor_panel.rows.len() == 0` for an empty store; that assertion is
independent of the hint strings. Relying on it per the task brief.

## Files changed / staging list
- crates/ff-desktop/src/alias_editor_panel/rows.rs  (the 3 hint strings)

(TERMINAL_DEFECTS.md and this note / tools/logs are NOT part of the code commit;
tools/logs is git-ignored.)

## Commands run + log results
First attempt (dead-terminal episode): `cargo check -p ff-desktop` streamed
partial compile output (reached `Checking mio`) then the shell wrapper went DEAD;
all later wrapped commands produced NO log file. Recorded in TERMINAL_DEFECTS.md.

The terminal RECOVERED on a later turn; verification, commit, AND push were then
performed successfully (not left to the owner). Authoritative scoped gate, all
clean, logs read back by effect:
- `cargo check -p ff-desktop *> tools\logs\ghosttext-check-final.log` -- clean;
  `ff-desktop` checked, `Finished dev profile ... in 4.08s`.
- `cargo clippy -p ff-desktop -- -D warnings *> tools\logs\ghosttext-clippy-final.log`
  -- clean; `Finished dev profile ... in 9.15s`, no warnings (would have aborted
  under -D warnings).
- `cargo fmt -p ff-desktop -- --check *> tools\logs\ghosttext-fmt-final.log` --
  empty log = no formatting diffs = clean.
- Static cross-check: the change is three `&str` literal substitutions inside
  `.hint_text(...)` calls; cannot alter compile/clippy/fmt results. Source ASCII.

## Commit + push (performed by the agent after terminal recovery)
- Staged by name: `git -C <worktree> add crates/ff-desktop/src/alias_editor_panel/rows.rs`
- Commit `80a75d8` on branch `alias-ghost-text`: "refactor(alias-editor): neutral
  ghost-text on empty scratch row (no ECOB example)" -- 1 file changed,
  3 insertions(+), 3 deletions(-).
- Pushed: `git -C <worktree> push -u origin alias-ghost-text`. Verified by effect
  -- `origin/alias-ghost-text` points at `80a75d8`; `git status -sb` shows
  `alias-ghost-text...origin/alias-ghost-text` with no ahead/behind marker (in
  sync). No merge, no PR, worktree left in place.

## Change-log
Repo convention: trivial cosmetic hint-string refactors do not get a CR id or a
change-log entry (the change log tracks CRs/features/bug fixes). This note is the
record. No change-log line added.
