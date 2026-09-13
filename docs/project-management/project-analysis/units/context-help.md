# Analysis Record: context-help (W4.10)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-help` (context-sensitive help: dockable Help Panel, F1
  context detection, searchable topic library, ISPF Tutorial model)
- **Spec files**: requirements.md (445 lines, 16 requirements), tasks.md
  (167 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 16 reqs, 445 lines. One cohesive concern (help panel +
F1 context detection + topic library + search + navigation + plugin help). 16
files, no file over 400 (well-decomposed). No split.

---

## 2. Cross-unit consistency -- CLEAN

### HELP command / F1 boundary -- clean delegation (command-semantics W3.1)

The `HELP` primary command (command-semantics Req 7, W3.1) ROUTES THROUGH to
ff-help: command-semantics owns the command DISPATCH; ff-help owns the panel +
F1 activation + content. `ff-help/src/commands.rs` handles "HELP command handler
and F1 activation logic ... context detection -> topic resolution -> panel
open/toggle", and per Req 1.10/13.10 HELP + F1 are never recorded (non-undoable,
consistent with the display-only command family: line-wrap W1.14, zoom W4.5).
Clean owner(command-semantics)/executor(ff-help) split -- same pattern as HILITE
(PA-WATCH-009) and the menu/command family Command_Target routing.
(`ff-command-semantics/src/help.rs` is the command-side stub; ff-help is the panel.)

### Plugin help -- ff-plugin extensibility

`plugin_help.rs` (ff-plugin dep) lets plugins contribute help topics -- consistent
with the plugin-architecture extensibility pattern (W0.13), like completion
providers (W3.2) + status segments (W3.6). (`ff-jes/src/sdsf_help.rs` is a
plugin-side help contributor, Wave 5.) Consistent.

### Help panel docking -- ff-layout

The Help Panel is a dockable panel via ff-layout (W4.1). Consistent (ff-layout owns
docking). Topic library uses 3 fs calls (help content files) -- legitimate (ff-help
owns its topic content). Config (panel width Req config) via ff-config.

### Public types and ownership

- HelpTopicRegistry, HelpPanel, context detector (F1), topic search/navigation,
  plugin-help registration -- sole-owned by `ff-help`. No duplication.
- Deps: ff-command + ff-layout + ff-config + ff-plugin + ff-logging -- all
  appropriate (command dispatch, docking, config, plugin topics). Clean.

### Cross-reference integrity

Cross-refs (command-semantics HELP, layout-and-docking, configuration-system,
plugin-architecture) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 167 sub-tasks `[x]`. Implementation present across all 16 reqs
(help panel, F1 context detection, topic library, search, navigation hierarchy,
plugin help, non-undoable HELP/F1, config). Tests in-file. No PA-INCOMPLETE.
Complete.

### TCR (adequate)

8 TCR rows for 16 reqs -- adequate. No PA-TCR raised (though could be fuller for a
16-req crate; not flagged given the coverage).

---

## 4. Logging audit

Scan of `crates/ff-help/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 3 (help topic content loading -- legitimate)

The error.rs has a rich error type (topic-not-found, content-file-not-found,
parse errors, registry lock-poisoned, search-too-short, navigation-empty, config)
-- all returned as `HelpError`, none logged. Topic-content load/parse failures
(Req topic library) are natural WARN sites. Recorded PA-LOG-035 (LOW): wire
ff-logging for topic-load/parse WARN + config coercion; dev-logging on HELP/F1
topic resolution. Resolve dead dep. Low priority (help errors surface to the user).

---

## 5. Task revision proposals

- **PA-STD-050 (ASCII, ACTIONABLE -- runtime strings)**: 199 non-ASCII bytes
  (12 non-comment). em/en-dashes inside runtime `#[error]` strings in error.rs
  (8+ messages: lookup/content/parse/registry/search/navigation/config) + config.rs
  range en-dash ("0.2-0.5"). Non-ASCII in user-facing output is a genuine defect
  (larger cluster -- the help error type is verbose). Replace with `--`/`-`.
  REFACTOR, no gate.
- **PA-LOG-035 (LOW)**: wire ff-logging for topic-load/parse WARN + config coercion;
  dev-logging on HELP/F1 topic resolution. Resolve dead dep.

No PA-STD size item (no file over 400). No conflict. No requirement CHANGE proposed.

---

## Summary

context-help (`ff-help`) is a complete, well-decomposed context-sensitive help
system (dockable Help Panel, F1 context detection, searchable topic library, plugin
help, ISPF Tutorial model). Clean cross-unit story: the HELP command +
F1 route through from command-semantics (Req 7, W3.1) with ff-help as the
executor (owner/executor split like HILITE + the menu/command family, non-undoable);
plugin help via ff-plugin extensibility; docking via ff-layout. 167/167 tracked,
no cap violation, adequate TCR (8). Findings are the familiar mechanical pair:
runtime-string ASCII (PA-STD-050 -- a larger em/en-dash cluster in the verbose
HelpError type) and a dead ff-logging dep with unlogged topic-load/parse errors
(PA-LOG-035, LOW). No conflict, no split, no size issue.
