# Analysis Record: function-keys-and-history (W3.7)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-keys` (spec says `ff-function-keys`; actual dir is
  `ff-keys` -- naming drift)
- **Spec files**: requirements.md (524 lines, 21 requirements), tasks.md
  (271 sub-tasks, **247 done / 24 OPEN**), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

Borderline; not a forced split.

- Requirement volume: 21 reqs, 524 lines. ABOVE both thresholds (21 reqs, >350
  lines). (signal 1)
- Responsibilities: THREE ISPF capabilities -- (a) function/PF key maps + label bar
  (Req 1-4,12-16,20-21), (b) command history + RETRIEVE recall (Req 5-11,19), (c)
  END/RETURN navigation (Req 17-18). These are related (all keyboard/command-history
  ISPF workflows) but (a) key-mapping and (b) command-history are arguably separable
  concerns. (partial signal 2)
- Crates: single crate `ff-keys`; well-sized (function_key.rs 395, key_map.rs 375 --
  under cap).
- Cohesion: medium -- key maps and command history share the ISPF-keyboard theme but
  little code.

Borderline 2-of-4 (volume + a weak responsibility split). Recorded PA-SPLIT-009
(LOW, owner-gated): a POSSIBLE spec split into function-keys (key maps + label bar +
PFSHOW) vs command-history (history + RETRIEVE + dropdown), but the crate is
well-sized and the two share config/context plumbing -- low value, defer. No forced
split.

---

## 2. Completeness -- PA-INCOMPLETE-009 (Req 17 + Req 19 partial, honest tracking)

247/271 tasks done, **24 OPEN**. They map to two requirements plus bookkeeping:

REQ 17 (END and RETURN Navigation) -- Tasks 33-34: revise the END/RETURN handlers
in `ff-desktop/src/shell/commands.rs` so that on a POM they CLOSE the Workspace (or
`file.exit` when it is the only Workspace) instead of exiting. Substantial existing
code (91 END/RETURN/POM refs in the shell) -- these are REVISIONS to align with the
CR-NR-057 POM/Context navigation model (ties to menu-workspace W3.5 +
Context-Navigation-Stack PA-INCOMPLETE-003). Plus verification (35 Key_Label_Bar
blank-slot, 36 END/RETURN in Excluded_Command set).

REQ 19 (RETRIEVE Argument-Driven History Recall + Browser) -- Tasks DF.1-DF.6:
`recall_list` (dedup, most-recent-first), `recall_by_number` (1-based), RETRIEVE
LIST/numeric dispatch, RETRIEVE-not-recorded guard, and the ff-desktop
History_List overlay UI (DF.5). PARTIALLY built: `retrieve.rs` already has
`RetrieveResult::ShowList` with "LIST" tests -- so the LIST logic exists; the OPEN
work is the `recall_list`/`recall_by_number` helper API shape, the not-recorded
guard, and the ff-desktop numbered overlay (DF.5, UI).

BOOKKEEPING: 37 (TCR rows for Req 17.2/17.2a/17.4/4.3/8.2), 8.x.

So this is PARTIAL COMPLETION with HONEST `[ ]` tracking (not a false-positive):
Reqs 1-16, 18, 20-21 done; Req 17 (END/RETURN POM revision) + Req 19 (RETRIEVE
browser/overlay) are the open pieces. Recorded PA-INCOMPLETE-009 (MEDIUM): finish
Req 17 END/RETURN-from-POM handler revision (coordinate with menu-workspace
PA-INCOMPLETE-008 + Context Navigation Stack PA-INCOMPLETE-003 -- same POM/nav
model) and Req 19 RETRIEVE recall API + History_List overlay. Bookkeeping (37) +
verification (35/36) alongside.

---

## 3. Cross-unit consistency

### END/RETURN POM navigation -- ties to the CR-NR-057 nav model

The Req 17 END/RETURN-from-POM revision (Tasks 33-34) operates on the POM/Context
model that menu-workspace (W3.5) and command-framework Context Navigation Stack
(Req 10, PA-INCOMPLETE-003) define. END/RETURN closing a Workspace vs exiting is
Context-Navigation-Stack behaviour. So Req 17 is adjacent to the CR-NR-057
command-chaining/nav bundle (though it is END/RETURN semantics, not chaining
per se). Recorded as a coordination note within PA-INCOMPLETE-009 -- implement the
END/RETURN POM revision consistently with the nav-stack model.

### Command_Target / command-framework routing -- consistent

Function keys map to commands dispatched through command-framework (Req 3
Function Key Execution); the key map references Command_Targets (command-framework
Req 8), like the rest of the menu/command family (W3.4/W3.5/W3.6). RETRIEVE is a
command; excluded commands (Req 8) include RETRIEVE/UNDO/REDO/END/RETURN. Consistent.

### History store -- raw-fs (PA-WATCH-019 family)

`history_store.rs` does 1 `std::fs` block for the command-history TOML file, with
proper error types (history-load/parse/save in error.rs). Another local raw-store
in the PA-WATCH-019 family (menus/ + commands.toml + criteria + recent-files +
now history). Low concern (error-typed). Folded into PA-WATCH-019.

### History dropdown / RETRIEVE overlay vs command-completion popup

Req 10 (History Dropdown on the primary command field) + Req 19 (RETRIEVE
History_List overlay) are command-field overlays -- adjacent to command-completion's
popup (W3.2) and command-palette (W3.3). Different content (history vs completions
vs all-commands) but all are command-field overlay UIs. Recorded PA-WATCH-021 (LOW):
at Wave 4 shell, confirm the history-dropdown / RETRIEVE-list / completion-popup /
palette overlays share consistent positioning/keyboard-nav infrastructure (or note
they are intentionally separate). Not a conflict; a UI-consistency watch.

### Public types and ownership

- FunctionKey/KeyMap/KeyLabelBar/KeyMapResolver, CommandHistory/HistoryStore,
  RetrieveState, PFSHOW -- sole-owned by `ff-keys`. No duplication.
- Deps: ff-command + ff-config + ff-logging. GUI-independent model (label bar +
  overlay RENDER in the shell). Clean.

### Cross-reference integrity

Cross-refs resolve. No dangling refs.

---

## 4. Logging audit

Scan of `crates/ff-keys/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 1 (history_store.rs -- error-typed)

The error.rs has rich error variants (key-parse, key-map-entry, history-load/parse/
save, config, dispatch) but NONE are logged -- returned as `KeysError`. The
history-save/load failures (Req 6 persistence) and config coercion (Req 11) are
natural WARN sites. Function-key DISPATCH flows through command-framework (PA-W0.2
covers execution). Recorded PA-LOG-024 (LOW-MED): wire ff-logging for history
load/save/parse failures + config coercion + key-map-entry errors; dev-logging on
key dispatch + RETRIEVE. Dead-dep resolution.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-009 (MEDIUM)**: finish Req 17 (END/RETURN-from-POM handler revision,
  Tasks 33-34, coordinate with menu-workspace PA-INCOMPLETE-008 + Context Navigation
  Stack PA-INCOMPLETE-003) and Req 19 (RETRIEVE recall_list/recall_by_number API +
  not-recorded guard + ff-desktop History_List overlay, DF.1-6). Verification (35/36)
  + TCR (37) alongside. Honest `[ ]` -- partial completion.
- **PA-SPLIT-009 (LOW, owner-gated)**: possible spec split function-keys vs
  command-history; low value (crate well-sized, shared plumbing). Defer.
- **PA-STD-037 (ASCII, runtime strings)**: 119 non-ASCII bytes (11 non-comment) --
  em-dashes inside runtime `#[error]` strings (error.rs lines 11-90, ~6 messages) +
  a test comment. Non-ASCII in user-facing output. Replace with `--`. REFACTOR.
- **PA-LOG-024 (LOW-MED)**: wire ff-logging for history/config/key-map errors +
  dev-logging on dispatch/RETRIEVE (dead dep).
- **PA-WATCH-021 (LOW)**: confirm command-field overlay consistency (history dropdown
  / RETRIEVE list / completion popup / palette) at Wave 4 shell.
- **PA-TCR (none)**: TCR is GOOD (33 rows); no gap raised. Task 37 fills the Req
  17/19 rows once implemented.
- **PA-DOC (naming)**: spec crate `ff-function-keys`; actual `ff-keys`.

No requirement CHANGE proposed; Reqs 17/19 are correctly-gated pending work.

---

## Summary

function-keys-and-history (`ff-keys`, spec says `ff-function-keys`) covers PF-key
maps + label bar, command history + RETRIEVE, and END/RETURN navigation -- 21 reqs,
well-sized (no cap violation), GOOD TCR (33 rows), routing through command-framework
(Command_Target, consistent with the menu/command family). PA-INCOMPLETE-009
(MEDIUM): 24 open tasks = Req 17 (END/RETURN-from-POM handler revision, CR-NR-057
POM-nav-model adjacent -> coordinate with menu-workspace/nav-stack) + Req 19
(RETRIEVE recall API + History_List overlay, partially built -- LIST logic exists)
+ verification/bookkeeping; honest `[ ]` tracking. Other items: a LOW owner-gated
split possibility (PA-SPLIT-010, deferred), runtime-string ASCII (PA-STD-037), dead
ff-logging dep with error-typed history/config paths (PA-LOG-024), a LOW
command-field-overlay-consistency watch (PA-WATCH-021), the history-TOML raw-store
(PA-WATCH-019 family), and the ff-keys naming drift.
