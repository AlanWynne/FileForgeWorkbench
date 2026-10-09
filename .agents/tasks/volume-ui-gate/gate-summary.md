# Volume / Catalog UI Requirements Gate -- Verification Summary (CR-NR-105 / CR-CH-057, second gate)

DOCS-ONLY verification of the second (UI/flow) requirements gate for the first-class
Volume layer. No source was changed. No cargo build/test/gate was run.

## Verdict

PASS. The drafted docs are internally consistent, EARS-formed, sequentially numbered
with no collisions, do not contradict existing criteria, carry the correct IN PROGRESS
gate status, and are ASCII-clean in all gate-added content. No numbering or
contradiction fixes were needed; no character fixes were needed.

## Sub-projects touched

- `docs/specs/virtual-catalog-manager/` (requirements.md, design.md, tasks.md)
- `docs/specs/dataset-allocator/` (requirements.md, design.md, tasks.md)
- `docs/specs/idcams-emulator/` (requirements.md, design.md, tasks.md)
- `docs/project-management/project-master/tasks.md` (master task rows + Summary count)
- `docs/quality/TCR.md` (NOT COVERED rows)
- `docs/status/change-log.md` (CR-NR-105 / CR-CH-057 status)

## New requirements + criterion ranges

- virtual-catalog-manager Requirement 17: Volume Management Context (VTOC / Volume
  Report) -- criteria 17.1 through 17.10 (10 criteria). Sequential after the existing
  highest (Requirement 16); no collision.
- virtual-catalog-manager Requirement 18: Volume Picker in the Catalog-Creation Dialog
  -- criteria 18.1 through 18.6 (6 criteria).
- dataset-allocator Requirement 17: SPACE-Against-Volume Allocation Flow (Automatic
  Extent Charging) -- criteria 17.1 through 17.9 (9 criteria). Sequential after the
  existing highest (Requirement 16); no collision.
- dataset-allocator Requirement 18: Uncataloged Allocation and Resolution by VOL=SER +
  UNIT -- criteria 18.1 through 18.6 (6 criteria).
- idcams-emulator Requirement 27: DEFINE VOLUME Command Contract and VOLUMES() Binding
  -- criteria 27.1 through 27.8 (8 criteria). Sequential after the existing highest
  (Requirement 26); no collision.

All new criteria are in EARS form (WHEN ... THE ... SHALL ... / IF ... THEN THE ...
SHALL ...), numbered sequentially with no gaps or duplicates, and reference (not
restate) the owning volume-model criteria. No contradiction with existing criteria in
the same files or in volume-model / dataset-catalog was found.

## Design decisions encoded

- Volume management is a dedicated WorkspaceContext dispatched via
  `render_workspace_context` returning an `InteriorFocus` whose first interior control
  is a guaranteed-present filter field carrying a stable `egui::Id`
  (`volume_report_filter`); a full-shell first-Tab egui_kittest test is mandated. No
  bespoke render arm, no second navigation stack, no hand-wired focus ring.
- Every Volume admin action (DEFINE VOLUME, vary online/offline, set RW/RO, ALTER
  capacity) is a command routed through the single `resolve_target` /
  `dispatch_command_target` front door; state-changing intent is returned from render
  as a `VolumeAction` enum applied by `apply_volume_action` (pending_action pattern) --
  render never mutates the shell.
- Capacity is a hard cap set at DEFINE time, resizable later via ALTER VOLUME, with no
  over-commit (a requested cap below currently-used space is rejected, capacity
  unchanged).
- Two-level allocation model: DEFINE VOLUME is the admin act (VOLSER + host path +
  capacity + status); SPACE= on DISP=NEW charges extents automatically against Volume
  free capacity. Two distinct failures: dataset x37-style space-abend (Max_Extents) vs
  Volume_Full (no free space), never conflated.
- Catalog-creation dialog gains a Volume picker listing defined VOLSERs plus a
  "Define new volume..." entry that reuses the SAME Define_Volume_Dialog and the SAME
  DEFINE VOLUME command; the picker is the only Volume-choosing control (no inline
  Volume editor in the catalog dialog).
- VOL=SER + UNIT is an ADDITIONAL uncataloged resolution/allocation path, honouring
  Volume status/access gating; it does not replace the existing unresolved-DSN
  diagnostic.
- IDCAMS stays a thin orchestrator: it parses DEFINE VOLUME and delegates to a
  downstream `ff-volume`-backed Volume service; VOLUMES() binds DEFINE CLUSTER to real
  Volume entities via DatasetVolume; DefineVolumeCommand round-trips through the
  Pretty_Printer. Acyclic ownership DAG preserved (ff-dsalloc / ff-idcams depend on
  ff-volume, not via ff-dataset-catalog).

All built ON the existing WorkspaceContext + single-command-dispatch + per-tab
Navigation_Stack + WorkspaceDescriptor framework. No framework change.

## New master-task rows (project-master/tasks.md)

- VM.6 -- Volume management Context + volume picker (ff-desktop UI). Delivers
  virtual-catalog-manager Req 17-18 (tasks 28-32).
- VM.7 -- SPACE-against-volume allocation + VOL=SER/UNIT uncataloged (ff-dsalloc).
  Delivers dataset-allocator Req 17-18 (tasks 19-20).
- VM.8 -- DEFINE VOLUME command + VOLUMES() binding (ff-idcams). Delivers
  idcams-emulator Req 27 (task 28).

All three rows are `[ ]` (pending; none pre-marked). The Phase (volume-model) Summary
count row was updated to record the second (UI/flow) gate and the VM.1-VM.8 range, and
retains "Pending owner approval before any code."

## New TCR rows (one NOT COVERED red-circle per new criterion)

- ff-desktop section: 16 rows (virtual-catalog-manager Req 17.1-17.10 + 18.1-18.6).
- ff-dsalloc section: 15 rows (dataset-allocator Req 17.1-17.9 + 18.1-18.6).
- ff-idcams section: 8 rows (idcams-emulator Req 27.1-27.8).
- Total: 39 NOT COVERED rows. Each count exactly matches its requirement's criterion
  count (one row per criterion), in the correct crate section, all red-circle status.

## ASCII character check

PASS. An encoding-safe scan of the gate's actual added lines (git diff, added lines
only) for the prohibited set (em dash, en dash, curly quotes, ellipsis, arrows,
comparison/logic symbols) returned an EMPTY result -- no prohibited character was
introduced by this gate.

Note on method: a whole-file ripgrep scan reports many pre-existing non-ASCII matches
(en-dashes in older requirement ranges, arrows in older design diagrams, and mojibake
in change-log.md / TCR.md legend). None of those are in the gate-added content and are
out of scope for this verification. One apparent "violation" in the TCR status cell
turned out to be a PowerShell `Out-File` encoding artifact of the ALLOWED red-circle
emoji (allowed in TCR.md); read through the file tool the cell is the correct
red-circle. No character fixes were applied.

## Numbering / contradiction fixes

None required. All new requirement numbers (vcm 17-18, dsalloc 17-18, idcams 27) are
sequential after each file's existing highest, with no gaps introduced and no
duplicates. No criterion contradicts an existing one in the same file or in
volume-model / dataset-catalog.

## Gate status

- change-log.md CR-NR-105: Status IN PROGRESS -- "Pending owner approval before any
  code."
- change-log.md CR-CH-057: Status IN PROGRESS -- second (UI/flow) gate authored
  alongside CR-NR-105.
- All new spec tasks are `[ ]` (pending). No item self-approved; nothing marked DONE.

## OWNER APPROVAL REQUIRED

OWNER APPROVAL IS STILL REQUIRED before any code task begins. This gate is docs-only
and remains PENDING / IN PROGRESS awaiting the owner's explicit approval. No source may
change until the owner approves the drafted requirements, design, tasks, master rows,
and TCR rows above.
