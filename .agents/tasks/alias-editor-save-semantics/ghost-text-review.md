# Review -- Alias Editor ghost-text cosmetic fix (branch alias-ghost-text)

## Scope reviewed
- Branch diff `main...alias-ghost-text` (captured to tools/logs/ghost-diff.txt + ghost-stat.txt)
- Coder note: .agents/tasks/alias-editor-save-semantics/ghost-text-fix-note.md
- Coder verification logs: ghosttext-check-final.log, ghosttext-clippy-final.log,
  ghosttext-fmt-final.log
- Spot-check: crates/ff-desktop/src/shell/tests_focus.rs ~L1221 (zero-rows assertion)

## Findings against the five required points

1. ONLY the three illustrative hint_text strings changed -- CONFIRMED.
   diff shows exactly three single-line edits in
   crates/ff-desktop/src/alias_editor_panel/rows.rs:
   - L18N (`alias_l18n`): `.hint_text("en")` -> `.hint_text("")`
   - ALIAS (`alias_surface`): `.hint_text("ECOB")` -> `.hint_text("name")`
   - Parameters (`alias_template`): `.hint_text("FFWB.COBOL.MASTER(&1)")` -> `.hint_text("(&1)")`
   diff --stat: 1 file changed, 3 insertions(+), 3 deletions(-).

2. NO data/seed/reset-bare/DEFAULT_*_TOML, CE default (ffcmd), Root `--`, or
   save/exit/cancel behaviour touched -- CONFIRMED. The diff stat lists only
   rows.rs; the changes are `&str` literals passed inline to egui `TextEdit`
   `hint_text`, which per egui semantics renders only while the field is empty
   and is never written to the model, store, or disk. No model, dispatch, or
   persistence code altered.

3. Empty editor no longer displays the ECOB / FFWB.COBOL.MASTER example --
   CONFIRMED. The illustrative strings are removed; replacements ("", "name",
   "(&1)") are neutral field-type placeholders, not a pre-filled example alias.

4. Existing empty-editor zero-rows assertion intact and NOT weakened -- CONFIRMED.
   tests_focus.rs is absent from the diff (unmodified). Spot-read at ~L1221 shows
   `assert_eq!(harness.state().alias_editor_panel.rows.len(), 0, ...)` still
   present. No new test was added; justified because hint_text is not exposed on
   the model and cannot be asserted without a full-shell render harness, which is
   out of scope for a hint-string change. Acceptable per the task brief.

5. Scope minimal; source ASCII; no unrelated changes; no secrets -- CONFIRMED.
   Single file, three literal substitutions, all ASCII. No credentials present.

## Verification evidence (read, not re-run)
- ghosttext-check-final.log: `ff-desktop` checked, `Finished dev profile ... in 4.08s` -- clean.
- ghosttext-clippy-final.log: `ff-desktop` compiled, `Finished ... in 9.15s`, no
  warnings emitted (would have aborted under `-D warnings`) -- clean.
- ghosttext-fmt-final.log: empty = no formatting diffs -- clean.
Evidence is present, scoped to ff-desktop, and consistent with a three-literal
change that cannot alter compile/lint/format results.

## Verdict
APPROVED. All five points hold with no blocking findings. The change is purely
cosmetic (egui hint_text), minimal, ASCII, verified clean, and leaves the
zero-rows guard and all save/behaviour code untouched.
