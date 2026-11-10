# BUG: Alias editor R and I line commands not working

Reported: 2026-11-01 (owner), against main commit `1532a5f`
(Group B -- shared ISPF table editor + Alias editor rebuild, CR-CH-075).

## Symptom (owner report)
The `R` (repeat/clone) and `I` (insert) line commands in the Alias editor are
not working.

## Suspected area
The shared line-command apply pass introduced in this change:
`crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`, reached from
the Alias editor via `crates/ff-desktop/src/shell/alias_editor_line_commands.rs`
(`apply_pending_line_commands` now delegates to the shared pass) and the Alias
host `crates/ff-desktop/src/alias_editor_panel/host.rs`.

Possibly related: the semantic review of this change recorded a NON-BLOCKING
finding #1 -- a likely index-shift when a Copy is batched with an immediate D/I
in the same apply pass (immediate I/D entries collected against original indices
but applied after copy/move inserts grow the list). The "I not working" symptom
may share a root cause with that finding. "R not working" -- R was kept as a
clone-after alias (plan decision D4); confirm R is still mapped and routed
through the shared pass.

## Requirement
command-configurator Req 7.4 (Alias Line_Command_Set = I, D, C, SORT; R optional
clone-after per Req 7.6). I = insert blank row after; R = clone row after.

## To establish during diagnosis
- What happens on `I` / `R` + Enter: no-op, notice, wrong row, or command cleared?
- Does the single-command case fail, or only when batched with other commands?
- Are D and C still working? (isolates shared-pass vs letter-specific)

## Constraints for the fix
- Resolver UNCHANGED (environment.rs linear scan).
- Build ON the framework; no new dispatch path; ISPF explicit-commit save seam
  and the single render_workspace_context focus latch preserved.
- TDD: add a failing test reproducing the broken I/R behaviour (with a
  `// Validates: Requirement 7.4` annotation) before fixing.
- <=400 non-test lines/file; ASCII-only.
