# Gate Review: Volume Model + Dataset-Catalog Split (CR-NR-105 / CR-CH-057)

Reviewer scope: a DOCUMENTATION-ONLY requirements-gate change. No code was to be
written. This review opened every deliverable listed in the brief, read the
authoring plan, the approved recommendation, and the source design document, and
checked each gate rule. No cargo/build/test was run (correctly -- there is no
code to run).

**Verdict: APPROVED** -- no blocking findings.

---

## Summary

The author delivered a clean, faithful execution of the authoring plan. The new
`volume-model` sub-project (requirements/design/tasks) is created; the
`dataset-catalog` spec is edited in place (glossary, Requirement 1 and 7
superseding notes, new Requirement 32, design CR-CH-057 section, tasks 33-37);
the `dataset-ownership-model` gains Requirement 21 and a new criterion 7.7 plus a
task 12; and all cross-cutting registrations (change-log CR-NR-105 + CR-CH-057,
project-master Phase (volume-model) VM.1-VM.5, TCR NOT COVERED rows, specs.md
line) are present and correct.

Every acceptance criterion traces to an owner decision (1-8) or a source-document
ADR; nothing is invented. The split-not-rename direction is enforced by
construction: the Catalog is a pure metadata locator (ADR-002), the Volume owns
bytes (ADR-001), the DatasetVolume association (sequence + opaque locator)
replaces `storage_path`, and the `ff-volume` crate ownership keeps the DAG
acyclic so a catalog owning bytes is unrepresentable. Both space-failure points
(dataset x37-style vs Volume_Full) are present and explicitly distinguished.
Tracks/cylinders/extents are metadata-only with the configurable geometry
(cylinder = 15 tracks, default max extents 16).

Watch for: the only non-ASCII and task-marker-corruption artefacts found are on
PRE-EXISTING, untouched lines of `dataset-catalog/requirements.md` and
`dataset-catalog/tasks.md` (confirmed, out of scope per the "review only the
change" rule). The newly authored content is 100% ASCII-clean. Modified files
exist under `crates/`, but they are confirmed unrelated in-flight work (ff-theme
CR-CH-056 and a command-ladder refactor), not products of this gate (confirmed).

---

## Detailed findings

### 1. Scope conformance (brief check 1) -- PASS

- **Catalog cannot own bytes by construction (ADR-002).** volume-model Req 1.5
  and 32.8 put the Volume type in `ff-volume`; ownership-model Req 21.3 forbids
  `ff-volume` depending on the catalog/allocator/idcams, and Req 7.7 fixes the
  chain `ff-idcams -> ff-dataset-allocator -> ff-dataset-catalog -> ff-volume ->
  storage providers`. Catalog depends on Volume, never the reverse -- the
  acyclic DAG makes "catalog owns bytes" unrepresentable. (confirmed)
- **Catalog as pure locator / Volume owns bytes (ADR-001/002).** dataset-catalog
  glossary redefines Catalog/Catalog_Database/Repository/Dataset_Resolution to
  the locator model with inline "Amended by CR-CH-057" notes; Req 32.4 and
  ownership Req 21.4 restate the invariant. (confirmed)
- **DatasetVolume replaces storage_path.** Req 32.2 and volume-model Req 9.1 make
  the (volume_id, sequence_number, is_primary, locator) tuple the authoritative
  location; `storage_path` is retained read-only only for the dual-read window
  (Req 32.5). (confirmed)
- **Multivolume + uncataloged resolution.** volume-model Req 9.2/9.4/9.5 and
  dataset-catalog Req 32.6/32.7 model multivolume (ordered sequence) and the
  uncataloged VOL=SER+UNIT path that needs no catalog row. (confirmed)
- **Two distinct space-failure points.** volume-model Req 5 (x37-style dataset
  capacity / Max_Extents), Req 7 (Volume_Full), with Req 5.5/6.4/7.3 explicitly
  asserting the two are distinct and separately reported; design.md maps both
  onto `StorageError` variants and gives the overflow decision flow. (confirmed)
- **Tracks/cylinders/extents are metadata-only with deterministic geometry.**
  volume-model Req 3 (configurable `bytes_per_track`, `tracks_per_cylinder`
  default 15), Req 3.4 (accounting only, no physical layout), Req 8.4/11.2
  (derived counters). Max extents default 16 (Req 6.1). (confirmed)
- **ff-volume specified not created; ff-dscatalog depends on it.** design.md
  "Crate Ownership" states "DO NOT create the `ff-volume` crate in this gate";
  task 35 adds the dependency at implementation time. No `ff-volume` directory
  exists under `crates/`. (confirmed)

### 2. Gate conformance (brief check 2) -- PASS

- **EARS format.** All volume-model criteria (1.1-11.3) and dataset-catalog
  32.1-32.8 are `THE ... SHALL ...` / `WHEN ... THE ... SHALL ...`. (confirmed)
- **Sequential, unique numbering.** volume-model Requirements 1-10 plus NFR
  Requirement 11; criteria X.Y with no collisions. dataset-catalog new
  Requirement 32 follows the existing highest (31); criteria 32.1-32.8 unique.
  dataset-ownership-model Requirement 21 follows 20; new criterion 7.7 appended
  to Req 7 without renumbering 7.1-7.6. (confirmed)
- **Edited in place with change noted.** dataset-catalog glossary entries carry
  "(Amended by CR-CH-057 ... was: ...)" / "(Added by CR-CH-057.)"; Req 1 and Req
  7 carry blockquote superseding notes rather than silent rewrites. (confirmed)
- **Change-log.** CR-NR-105 under "## New Requirements" and CR-CH-057 under
  "## Change Requests", both using the exact template fields, both set to
  IN PROGRESS. Highest prior ids were CR-NR-104 and CR-CH-056; no collision.
  (confirmed)
- **Master tasks.** Phase (volume-model) with VM.1-VM.5 (all `[ ]`) plus the
  trailing Status/Count table row. (confirmed)
- **TCR.** One 🔴 NOT COVERED row per new criterion: volume-model 1.1-11.3 under
  `ff-volume`; dataset-catalog 32.1-32.8 under `ff-dscatalog`; ownership-model
  7.7 and 21.1-21.5 under `ff-governance-tests`. Correct 🔴 marker only, no
  stray 🟢/🟡. (confirmed)
- **specs.md registration.** `volume-model` line inserted alphabetically between
  `virtual-file-system` and `whitespace-and-guides`. (confirmed)
- **dataset-ownership-model ADR stub.** Req 21 carries an explicit "ADR amendment
  stub (do NOT rewrite existing ADRs)" note; existing ADRs are not rewritten.
  (confirmed)

### 3. Task-file format (brief check 3) -- PASS

volume-model/tasks.md (tasks 1-10 + coverage table) and the appended
dataset-catalog tasks 33-37 and ownership-model task 12 use only `[ ]`, every
task line has a title, and none is pre-marked `[x]`. (The old `[x]` tasks 1-31 in
dataset-catalog/tasks.md are pre-existing completed work, not part of this
change.) No `[~]`/`[-]`/`[/]` markers in any authored line. (confirmed)

### 4. Character set (brief check 4) -- PASS for authored content

- `volume-model/*.md`: zero non-ASCII bytes (full-file grep clean). (confirmed)
- dataset-catalog authored lines (CR-CH-057 glossary notes, Req 1/7 notes, Req
  32, design CR-CH-057 section, tasks 33-37): ASCII only -- `--`, `->`, `<=`
  spelled as needed. (confirmed)
- dataset-catalog/requirements.md DOES contain non-ASCII (en-dash `1-8`, `<=`,
  `>=`, `->` arrows) and tasks.md contains `?` corruption artefacts, but these
  are all on PRE-EXISTING lines (Dataset_Name glossary, Req 2/7/9 criteria, old
  tasks 1-31) untouched by this change. Per the review rule "only review changes
  in the PR diff, not pre-existing issues," these are out of scope and NOT a
  finding against this gate. (confirmed, out of scope)
- Box-drawing appears only inside fenced code blocks (volume-model design ER
  diagram, dataset-catalog design schema block). (confirmed)

### 5. No contradiction (brief check 5) -- PASS

The dataset-catalog design CR-CH-057 section includes an explicit "Decision
touched (called out)" blockquote for Requirement 4 / Requirement 20 (the physical
layout owner moves from Catalog to Volume; the directory layout itself is
unchanged). No new criterion silently contradicts an existing dataset-catalog
criterion or an existing ADR; where a prior decision is touched it is called out.
(confirmed)

### 6. Docs-only guard (brief preamble) -- PASS

Modified files exist under `crates/` (ff-theme, ff-desktop shell command
ladders/nav_stack/render_theme, and tests), but a content grep for
`ff-volume|volume_model|dataset_volumes|VOLSER` in those files returns only
PRE-EXISTING `volser` fields (ff-jes SpoolVolume, ff-dataset-catalog traits,
ff-lua tso_builtins) that the recommendation itself cited as already present.
None of the modified crate files reference the volume-model work. They are
unrelated in-flight workspace changes (CR-CH-056 theme rework + command-ladder
refactor), NOT products of this gate. No `ff-volume` crate was created. This gate
touched only `docs/` and `.kiro/steering/specs.md`. (confirmed -- not a blocking
finding)

---

## Non-blocking observations (informational, no action required for this gate)

- The pre-existing non-ASCII in `dataset-catalog/requirements.md` (en-dashes,
  `<=`/`>=`/`->`) and the `?` artefacts in `dataset-catalog/tasks.md` predate
  this change. They are worth a future housekeeping pass under the
  documentation.md character rule but are explicitly out of scope here and do
  not block this gate.
- The design's worked geometry example fixes an illustrative `bytes_per_track =
  56664`; the spec correctly defers the shipped default to implementation. Good
  separation of a documented example from a binding default.
