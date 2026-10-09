# Author notes -- BackendEnvironment record-aware store contract gate (CR-CH-060)

Owner-directed, docs-only FRAMEWORK-CHANGE requirements gate authored on branch
`main` (no worktree, no branch, no commit, no `.rs` change, no build/test). This
file is the authoritative record for the reviewer and the summary step.

## Allocated CR id

**CR-CH-060** -- the next unprefixed mainline CR-CH. Confirmed from
`docs/status/change-log.md`: the highest existing unprefixed CR-CH is CR-CH-059
(mainframe dataset stack rationalisation). CR-CH-060 is allocated here.

CR-CH-060 is a FRAMEWORK CHANGE serving TWO existing consumers:
- **CR-CH-058** (universal windowed record-oriented document model / piece-table):
  its SAVE walk needs to deliver re-framed records (not a flat byte buffer) to
  the owning Command Environment.
- **CR-CH-059 RC.B.8 Part 2** (record-aware MAINFRAME editor SAVE): its
  prerequisite (a) is exactly this contract reshape; (b)-(e) are downstream wiring.

Owner direction (verbatim): "We need to move forward with both CR-CH-058 and
RC.B.8 Part 2", "design the contract once", final message "A and B". The DIRECTION
is owner-approved; the drafted contract SHAPE still pauses for explicit owner
approval before any code (a later step).

## The decision surfaced in the gate (owner picks at approval)

Two contract shapes, both presented in `command-environments/design.md`; the gate
RECOMMENDS Shape 2 and the overlap report (section E) concurs.

- **Shape 1 -- change the `save` signature to a record-aware form** (one method
  that carries records-or-bytes + target + attrs).
  - Object-safety: achievable but the single method must still express a plain
    byte write for host CEs, so the parameter object grows a bytes-or-records
    union -- more complex at the one call site that only ever writes bytes.
  - Native byte-identical: at RISK -- every host write now travels through the
    record-aware parameter shape; the "nothing changed for native" guarantee is
    harder to prove and the regression surface is larger.
  - Migration cost: HIGH -- every existing `impl BackendEnvironment`
    (`ff-ce-host-fs` decider, `ff-ce-ntfs`, `ff-ce-posix`) and the one call site
    (`host_fs_save` -> `save_active_tab_via_backend`) must change at once.
  - Consumption: CR-CH-058's Delimited path and the mainframe path share one
    method but must branch inside it; muddier.

- **Shape 2 -- ADD a record-aware store method ALONGSIDE the byte `save`** (two
  methods; host CEs keep `save`; the record method has a default that declines or
  is overridden only by record-aware CEs). RECOMMENDED.
  - Object-safety: trivially preserved -- both methods take `&self` and concrete
    (non-generic, non-`Self`-returning) parameters; `dyn BackendEnvironment`
    stays valid. A `provided` default on the record method keeps existing host
    impls compiling unchanged.
  - Native byte-identical: PRESERVED by construction -- `save(path, bytes)` is
    untouched, so native/Delimited SAVE is literally the same code path and the
    same bytes (command-environments Req 16.5, document-model Req 12.12).
  - Migration cost: LOW -- host CEs need NO change (they inherit the default);
    only the mainframe CE (housed in `ff-idcams`) and the editor SAVE walk opt in.
  - Consumption: cleanest -- the SAVE walk picks the method by the owning CE's
    advertised RecordFormat (Delimited/host -> `save`; Fixed/Variable -> the
    record method); the mainframe CE implements the record method over
    `ff_dscatalog::DatasetAccess::put` (+ open/close so the RECFM codec frames
    bytes on close).

RECOMMENDATION: **Shape 2** -- additive record-aware store method alongside the
byte save. It is the only shape that makes native-byte-identical a construction
guarantee (not a re-proof), keeps the trait object-safe with zero host-CE
migration, and lets each consumer opt in cleanly. Shape 1 is documented as the
considered alternative with its tradeoffs so the owner can still choose it.

Illustrative Shape 2 surface (NAMES ARE ILLUSTRATIVE, finalised at approval):

```
trait BackendEnvironment: Send + Sync {
    fn name(&self) -> &str;
    fn is_case_sensitive(&self) -> bool;
    fn save(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;      // UNCHANGED (host byte write)
    fn save_records(&self, target: &StoreTarget, records: &dyn RecordSource,
                    attrs: &RecordStoreAttrs) -> BackendStoreResult {   // NEW, provided default = NotRecordCapable
        BackendStoreResult::NotRecordCapable
    }
    fn record_capable(&self) -> bool { false }                          // host CEs: false; mainframe CE overrides true
}
```
- `StoreTarget` carries dataset identity (DSN / catalog identity / owning-env) --
  NOT a host path.
- `RecordSource` is an object-safe record stream the editor SAVE walk fills from
  the re-framed Piece_List (object-safe = `&dyn`, no generics on the method).
- `RecordStoreAttrs` carries RECFM / LRECL / encoding (sourced from the catalog
  attributes, the SAME values the CE used to build the RecordFormat at open).
- `BackendStoreResult` mirrors `BackendOutcome` (rc-carrying) so the addressing
  caller maps it to a status / macro RC.

## The records -> store dataflow (designed ONCE for both consumers)

```
editor SAVE (FFEDIT verb, CR-CH-053 Task 20/21 owning-CE SAVE seam)
  -> CR-CH-058 SAVE walk: walk Piece_List, re-frame each piece per the owning
     CE's RecordFormat (Delimited -> bytes; Fixed/Variable -> records)
  -> if owning CE advertises Delimited/host  : BackendEnvironment::save(path, bytes)   [byte-identical, unchanged]
     if owning CE advertises Fixed/Variable  : BackendEnvironment::save_records(target, records, attrs)
  -> mainframe CE (ff-idcams) implements save_records over
       ff_dscatalog::DatasetAccess::open -> put(record)* -> close
     (the RECFM codec frames bytes on close -- FixedCodec/VariableCodec/BinaryCodec)
```
Single source of truth: the owning CE supplies the RecordFormat at OPEN
(document-model Req 11.2); the SAVE path MUST NOT re-derive framing independently.
Single SAVE-addressing seam preserved (CR-CH-053 Task 20/21 FFEDIT -> owning CE);
NO parallel save path, NO second dispatcher.

Registry opening (closed `RegisteredEnv` enum -> named-backend map) is part of
the picture but is RC.B.8 Part 2 prereq (b) wiring, NOT a new framework mechanism
-- noted in the design, not a CR-CH-060 criterion.

DAG unchanged: ff-idcams -> ff-dscatalog -> ff-volume -> ff-vfs (the record-aware
trait stays in ff-vfs; the mainframe CE in ff-idcams implements it over
DatasetAccess).

## Files edited

### command-environments (primary -- the BackendEnvironment contract)
- `requirements.md`: NEW **Requirement 18** (criteria 18.1-18.8) -- the
  record-aware store contract. Continues from the file's existing highest
  (Req 17). No existing criterion changed (Req 16.5 byte-identical is referenced,
  not altered).
- `design.md`: appended section "Record-aware BackendEnvironment store contract
  (CR-CH-060)" -- both shapes, Shape 2 recommended, the dataflow, object-safety,
  native-byte-identical preservation, the registry-opening note, and the
  "designed ONCE for CR-CH-058 + RC.B.8 Part 2" statement with the unchanged DAG.
- Glossary: added `Record_Store_Contract` and `Store_Target` terms.

### document-model (editor SAVE-walk side)
- `requirements.md`: NEW **Requirement 13** (criteria 13.1-13.6) -- the SAVE walk
  selects byte-vs-record by the owning CE's advertised RecordFormat and calls the
  CR-CH-060 contract; Delimited stays byte-identical. Continues from Req 12.
- `design.md`: appended section "SAVE-walk store-call selection (CR-CH-060)" tying
  the existing "SAVE = re-baseline" delta to the two-method contract.

### Cross-referenced sub-projects (requirements cross-ref notes only; no new criteria unless noted)
- `virtual-file-system`: cross-reference note (ff-vfs owns the record-aware trait;
  additive, object-safe). Design: "No design changes required" note appended.
- `dataset-catalog`: cross-reference note (DatasetAccess::put is the mainframe
  store target). No new criteria -- Req 34 already covers put/open/close.
- `idcams-emulator`: cross-reference note (mainframe CE home implements the
  record-aware method). No new criteria beyond existing Req 28/task 29 which
  RC.B.8 already carries.

### Cross-cutting
- `docs/project-management/project-master/tasks.md`: NEW phase section
  "Phase (backendenv-record-contract) -- CR-CH-060" with tasks BRC.1-BRC.4
  ([ ] only) + Summary row. Summary counts note updated.
- `docs/quality/TCR.md`: NEW section "Phase (backendenv-record-contract) --
  CR-CH-060" with one NOT COVERED (red) row per new criterion
  (command-environments Req 18.1-18.8, document-model Req 13.1-13.6).
- `docs/status/change-log.md`: NEW `### CR-CH-060` entry under `## Change
  Requests`, cross-referencing CR-CH-058 and CR-CH-059 RC.B.8 Part 2, flagged
  FRAMEWORK CHANGE + owner-directed + PENDING GATE.

### Steering
- `.kiro/steering/framework-conformance.md`: mechanism 1-7 list already names
  BackendEnvironment indirectly via the dispatch seam; added a note under "What
  counts as a framework change" that the BackendEnvironment store contract is now
  record-aware (CR-CH-060) and that the record-aware method is the additive Shape
  2 (byte `save` retained). ASCII-only.

## New requirement/criterion ranges per sub-project
- command-environments: Requirement 18, criteria 18.1-18.8 (NEW).
- document-model: Requirement 13, criteria 13.1-13.6 (NEW).
- No criteria renumbered or deleted anywhere; Req 16.5 / Req 12.12 referenced only.

## Master-task rows
Phase (backendenv-record-contract) -- CR-CH-060: BRC.1 (gate -- this authoring),
BRC.2 (the contract reshape in ff-vfs + host CEs inherit default), BRC.3
(document-model SAVE-walk store-call selection), BRC.4 (mainframe CE in ff-idcams
implements the record method over DatasetAccess -- downstream, needs RC.B.8 (b)-(e)).

## TCR rows
14 NOT COVERED rows: command-environments Req 18.1-18.8 (8) + document-model
Req 13.1-13.6 (6), all in a new CR-CH-060 section, crate `ff-vfs` / `ff-document-model`
/ `ff-idcams` as appropriate.

## Iteration 2 -- verification pass (no design-review.json present; FIRST iteration)

Re-read all authored artifacts and the overlap report sections E/F. Verified
every claim in these notes against the files on disk:

- command-environments/requirements.md: Requirement 18 (18.1-18.8) present,
  EARS-numbered, ASCII-clean; Glossary carries Record_Store_Contract + Store_Target
  (both tagged [CR-CH-060]). Faithful to the owner direction (byte `save` retained,
  additive record entry, object-safe, single Task 20/21 seam, CE supplies format at
  open, DAG acyclic).
- command-environments/design.md: "Record-aware BackendEnvironment store contract
  (CR-CH-060)" section present with BOTH shapes, Shape 2 recommended, dataflow,
  object-safety, native-byte-identical construction guarantee, registry-opening
  prereq note, designed-once + unchanged-DAG statement. ASCII-clean.
- command-environments/tasks.md: tasks 23-25 present, `[ ]` only, titled, each
  cross-referencing criteria; downstream/approval caveats stated.
- document-model/requirements.md: Requirement 13 (13.1-13.6) present, EARS-numbered,
  ASCII-clean, correctly scoped as the editor SIDE (does NOT reshape the trait).
  Cross-References note added pointing the store contract to command-environments
  Req 18.
- document-model/design.md: "Design Delta: SAVE-walk store-call selection
  (CR-CH-060)" section present and consistent.
- Cross-referenced sub-projects: virtual-file-system, dataset-catalog (Req 34.6),
  idcams-emulator each carry a "Cross-reference (CR-CH-060, no new criterion here)"
  note. No new criteria added there (correct).
- project-master/tasks.md: Phase (backendenv-record-contract) with BRC.1-BRC.4
  + Summary row present.
- TCR.md: 14 NOT COVERED (red) rows present (8 + 6) in the new CR-CH-060 section.
- change-log.md: CR-CH-060 entry present under Change Requests, cross-referencing
  CR-CH-058 and CR-CH-059 RC.B.8 Part 2, flagged FRAMEWORK CHANGE + owner-directed
  + PENDING GATE (direction confirmed; SHAPE pending explicit approval).
- framework-conformance.md: "What counts as a framework change" updated with the
  record-aware store contract note (Shape 2, byte save retained, object-safe).

ASCII check: the authored CR-CH-060 content in every file is ASCII-clean. NOTE:
document-model/requirements.md + design.md contain PRE-EXISTING non-ASCII in
UNRELATED sections (en dashes in the Introduction/ranges, and math symbols in the
correctness-property blocks). These predate this gate and are NOT in any
CR-CH-060-authored section; fixing them is out of scope for this docs-only
contract gate (unrelated churn) and was deliberately left untouched.

Outcome: NO CHANGES NEEDED. All artifacts are internally consistent, EARS-numbered,
ASCII-clean in the authored content, and faithful to overlap report sections E/F
and the owner direction. Left the docs as authored.
