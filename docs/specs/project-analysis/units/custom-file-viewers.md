# Analysis Record: custom-file-viewers (W5.10)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-viewers` (custom file-viewer FRAMEWORK: viewer registry,
  `FileViewer` trait, `PREVIEW` command, built-in viewers for ASA/hex/CSV/image,
  plugin-provided viewers, dockable viewer panel, read-only constraint, refresh)
- **Spec files**: requirements.md (224 lines, 10 requirements), tasks.md
  (79 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

NOT a split candidate. 10 reqs, 224 lines, 17 files, no file over 350. Cohesive
framework. No split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-019 (framework vs point-solutions; orphaned framework + PREVIEW-command + ASA/hex DUPLICATION)

`ff-viewers` is a general viewer FRAMEWORK (registry + `FileViewer` trait + PREVIEW +
built-in viewers + plugin bridge). Three overlapping conflicts:

1. **ORPHAN framework (SIXTH orphan)**: `ff-viewers` is referenced by NO crate (0
   external `ff_viewers`/`FileViewer`/`ViewerRegistry` refs). Meanwhile the POINT
   SOLUTION `ff-asa` (W5.7) IS wired (173 refs). So the wired preview path is the
   standalone ASA crate; the general framework is orphaned.

2. **ASA + hex viewer DUPLICATION**: ff-viewers ships its OWN `asa_report.rs`, `hex.rs`,
   `csv_table.rs`, `image.rs` built-in viewers (keys "asa-report", "hex" via its
   `ViewerKey` registry) -- SEPARATE implementations from `ff-asa` (the wired ASA preview
   crate) and `ff-hex` (hex-display, W4.6). So there are TWO ASA rendering paths (ff-asa +
   ff-viewers/asa_report.rs) and TWO hex paths (ff-hex + ff-viewers/hex.rs). ff-viewers
   does NOT depend on ff-asa or ff-hex (0) -- it reimplements rather than wrapping them.

3. **PREVIEW command DOUBLE-OWNER**: ff-viewers defines a PREVIEW command (Req 3, 105
   PREVIEW refs + its own command.rs / PreviewCommandAction) AND ff-asa defines a PREVIEW
   command (Req 3, W5.7). Two owners of the same verb -- a command-ownership conflict
   (cousin of the shell/configurator + TSO-verb ambiguities).

Recorded PA-CONFLICT-019 (owner-gated, MEDIUM-HIGH): decide the viewer ARCHITECTURE --
either (a) adopt ff-viewers as THE framework and make its ASA/hex viewers WRAP ff-asa +
ff-hex (framework owns registry + PREVIEW + trait; point crates provide the rendering),
then wire ff-viewers into the shell and retire ff-asa's separate PREVIEW ownership --
PREFERRED (the extensible design is the framework); or (b) keep the standalone point
crates (ff-asa wired) and delete/absorb the orphaned ff-viewers framework + reconcile the
duplicate PREVIEW. Either way, ONE PREVIEW command owner + ONE ASA path + ONE hex path.
Pairs with the orphan cluster (PA-W4.1) and the ff-asa PREVIEW (W5.7). Code + owner
decision.

### Public types and ownership -- CONTESTED

- `FileViewer` trait, ViewerRegistry, ViewerKey, PREVIEW command, built-in ASA/hex/CSV/
  image viewers, dockable viewer panel -- nominally owned by `ff-viewers`, but the ASA +
  hex rendering + PREVIEW verb OVERLAP with ff-asa + ff-hex (PA-CONFLICT-019).

### Cross-reference integrity

ff-viewers does NOT cross-reference ff-asa / ff-hex despite duplicating their function
(PA-CONFLICT-019). Its plugin_bridge (Req 5) is the extensibility seam (consistent with
the plugin family) -- but unwired.

---

## 3. Completeness

Tracking: all 79 sub-tasks `[x]`. The framework is complete + tested AS A CRATE (registry,
trait, PREVIEW, built-in viewers, panel, read-only, refresh, config). FUNCTIONALLY
complete -- but orphaned + duplicative (PA-CONFLICT-019). No PA-INCOMPLETE for the crate;
the gap is architectural (unwired + overlapping).

### TCR gap (PA-TCR-032)

TCR.md has 1 row for ff-viewers across 10 reqs -- thin. Recorded PA-TCR-032.

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `std::fs`: 0.

Viewer registry + selection + refresh is a modest logging site (which viewer selected for
a file, registration conflicts, refresh triggers). ff-logging is a DEAD dep. Recorded
PA-LOG-048 (LOW): resolve the dead dep + add dev-logging on viewer selection / registration
/ refresh under the `dev-logging` gate (most valuable once wired, PA-CONFLICT-019).

---

## 5. Task revision proposals

- **PA-CONFLICT-019 (owner-gated, MEDIUM-HIGH)**: resolve viewer architecture -- adopt
  ff-viewers as the framework wrapping ff-asa + ff-hex + single PREVIEW owner [PREFERRED],
  or keep point crates + delete/absorb the orphan framework + reconcile PREVIEW. One
  PREVIEW owner, one ASA path, one hex path. Code + owner decision.
- **PA-STD-065 (ASCII, runtime strings)**: 20 non-comment non-ASCII -- em-dashes (U+2014)
  in runtime display/config-error strings (command.rs viewer-list formatting; config.rs
  invalid-value messages). Replace with `--`. REFACTOR, no gate.
- **PA-LOG-048 (LOW)**: resolve dead ff-logging + viewer-selection/registration/refresh
  dev-logging.
- **PA-TCR-032**: add TCR rows (1 for 10 reqs). No code.

No split (no file over 350). No PA-INCOMPLETE (complete as a crate). No raw-fs.

---

## Summary

custom-file-viewers (`ff-viewers`) is a complete, tested viewer FRAMEWORK (registry,
FileViewer trait, PREVIEW command, built-in ASA/hex/CSV/image viewers, plugin bridge,
dockable panel, read-only, refresh; 79/79, 17 files, no cap issue). The headline finding
is PA-CONFLICT-019 (MEDIUM-HIGH): a three-way framework-vs-point-solution conflict. (1) It
is the SIXTH orphan (0 external refs) while the point solution ff-asa is wired (173 refs).
(2) It DUPLICATES ASA + hex rendering -- its own asa_report.rs / hex.rs / csv_table.rs /
image.rs, with NO dep on ff-asa or ff-hex, so TWO ASA paths and TWO hex paths exist. (3)
The PREVIEW command has TWO owners (ff-viewers Req 3 + ff-asa Req 3). Resolve to one
architecture: adopt ff-viewers as the framework wrapping ff-asa + ff-hex with a single
PREVIEW owner (preferred -- the extensible design), or keep the point crates and
delete/absorb the orphan. Hygiene: 20 em-dash runtime strings (PA-STD-065), dead ff-logging
(PA-LOG-048), thin TCR (PA-TCR-032). The crate is sound; the gap is architectural (an
orphaned framework duplicating wired point crates and contesting their command).
