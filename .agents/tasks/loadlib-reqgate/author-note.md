# Requirements-gate author note -- CR-NR-108 (load libraries / RECFM=U / program search)

Docs-only gate for the owner's NEW REQUIREMENT: add first-class dataset-model
support for z/OS load libraries and program objects, the RECFM=U/LRECL=0 record
format that underpins them, and the MODEL foundation for STEPLIB program
resolution. No source outside `docs/` was touched. First iteration (no prior
review.json existed).

## New change-request id

- **CR-NR-108** (mainline, unprefixed; next after the existing highest CR-NR-107).

## Files edited (all under docs/)

1. `docs/status/change-log.md` -- appended `### CR-NR-108` under `## New
   Requirements` (newest-first position). Status: PENDING GATE. Linked spec:
   dataset-catalog; cross-refs jes-emulator + compiler-toolchain-integration.
2. `docs/specs/dataset-catalog/requirements.md`
   - Glossary: added Load_Library, Load_Module, Program_Object, Object_Module,
     Library_Kind, Member_Content_Type, Program_Search, STEPLIB, JOBLIB,
     Program_Search_Order.
   - Cross-References table: added `jes-emulator` (future consumer) and
     `compiler-toolchain-integration` (future consumer) rows.
   - Appended Requirements 37, 38, 39 (EARS), continuing from the real last
     Requirement 36. (Confirmed by reading the file end: Req 36 / criteria
     36.1-36.8 is the last; the "Requirement 34:" / "Requirement 36:" heading
     text in that block is historical label drift but the criteria numbering is
     sequential and the new reqs correctly continue at 37.)
3. `docs/specs/dataset-catalog/design.md` -- appended "Design Delta -- CR-NR-108"
   with the additive enum/field/validation design and the program-search API
   shape; states ADR-001 ownership and what is deferred to jes-emulator /
   compiler-toolchain-integration.
4. `docs/specs/dataset-catalog/tasks.md` -- appended Phase (loadlib-model) Tasks
   44, 45, 46 (continuing from the last Task 43), each `[ ]` with criteria
   cross-refs, plus an Acceptance Criteria Coverage table.
5. `docs/project-management/project-master/tasks.md` -- added Phase
   (loadlib-model) with deliverables LLM.1-LLM.3 and a per-phase `| Status |
   Count |` table (matching the CR-CH-060/061 per-phase pattern; the file has no
   maintained grand-total to bump).
6. `docs/quality/TCR.md` -- added a Phase (loadlib-model) section with one NOT
   COVERED (red-circle) row per new criterion (37.1-37.7, 38.1-38.8, 39.1-39.7 =
   22 rows), all in the `ff-dscatalog` crate section.

Acceptance test plan (`core-acceptance-test-plan.md`): NOT amended -- see owner
decision point 3 below.

## New requirements -- one line each

- **Req 37 (RECFM=U / LRECL=0), 37.1-37.7**: RECFM=U treats LRECL as
  not-applicable; `validate` accepts LRECL 0/None for U (rejects for non-U);
  BLKSIZE stays meaningful for U; `with_defaults` does not inject FB/80 for U; a
  load library defaults to RECFM=U; displays render U's LRECL as "not applicable".
- **Req 38 (load-library content model), 38.1-38.8**: additive `#[non_exhaustive]`
  `LibraryKind` (DataLibrary/ObjectLibrary/LoadLibrary) + `MemberContentType`
  (RecordData/ObjectModule/Executable); optional fields on
  `AllocParams`/`DatasetRecord` (default record-data); executable members are
  opaque binary (not text-editable); classic=PDS+RECFM=U, program-object=PDSE
  (both permitted); purely additive, existing datasets unchanged.
- **Req 39 (searchable load library), 39.1-39.7**: read-only `resolve_program`
  over an ordered Load_Library list (STEPLIB ++ JOBLIB ++ default), first member
  match wins; deterministic not-found; skip-with-diagnostic for unavailable
  libraries; only LoadLibrary datasets match; caller assembles the order; no
  byte load/execute; single- and multi-library searches via the same primitive.

## Key design decisions

- RECFM=U "LRECL not-applicable" uses the EXISTING `Option<u32>` lrecl (`None`
  preferred, `Some(0)` tolerated) -- NO new `Recfm` variant, NO tri-state field.
  Alternative `Recfm::U0` / `lrecl_not_applicable` flag was considered and
  REJECTED (duplicates state already carried by `Recfm::U`).
- `validate` becomes RECFM-aware: the "LRECL 1..=32760" and "BLKSIZE >= LRECL"
  checks apply only to F/FB/V/VB. Requirement 7's LRECL rule is AMENDED IN SCOPE
  (narrowed to non-U) rather than contradicted -- flagged in the Req 37 framework
  note. (See owner decision 1 about the existing Req 7 wording.)
- Content model is two additive `#[non_exhaustive]` enums + optional fields;
  orthogonal to `PartitionedSubtype` (PDS/PDSE kept). PDS/PDSE pairing is a
  RECOMMENDED default, not a hard model constraint, so a PDSE load library is
  allowed.
- Program search is a read-only primitive owned by ff-dscatalog; the caller
  (future jes-emulator) assembles the STEPLIB/JOBLIB/default order. Chosen
  behaviour for an unavailable earlier library: skip-with-diagnostic (continue to
  later libraries, accumulate conditions), so one bad library cannot mask a
  later match.

## Deferred (cross-referenced, OUT OF SCOPE here)

- **jes-emulator (ff-jes)**: parsing `//STEPLIB`/`//JOBLIB` concatenations,
  wiring `EXEC PGM=` (`ExecTarget::Program`, today only parsed) to call
  `resolve_program`, and executing the resolved program.
- **compiler-toolchain-integration**: the COMPILE step (source -> object module
  library) and the LINK/bind step (object -> load module / program object in a
  Load_Library). Added as Cross-References rows, not duplicated scope.

## Master tasks added

- LLM.1 (dataset-catalog Task 44) RECFM=U/LRECL=0 model.
- LLM.2 (dataset-catalog Task 45) load-library/program-object content model.
- LLM.3 (dataset-catalog Task 46) searchable load library -- resolve_program.

## TCR rows added

22 NOT COVERED (red-circle) rows, all `ff-dscatalog`, one per criterion
(37.1-37.7, 38.1-38.8, 39.1-39.7), under a new "Phase (loadlib-model)" section.

## Conformance

- Additive-only to ff-dscatalog types (new `#[non_exhaustive]` enums, optional
  fields, a relaxed validation branch, a read-only resolution primitive). No
  existing `Recfm`/`Dsorg`/`PartitionedSubtype` variant is added/removed/renamed;
  no public type reshaped; no new crate DAG edge; `ff-vfs::BackendEnvironment`
  store contract untouched. This is NOT a framework change and needs no special
  owner confirmation on that axis.

## Points needing an owner DECISION

1. **Existing Requirement 7 AC 10 wording.** That criterion still says "LRECL
   must be > 0 and <= 32760 ... BLKSIZE must be >= LRECL" unconditionally (and
   it uses the prohibited Unicode `<=`/`>=` glyphs that predate this task). Req
   37 narrows that rule to non-U formats via a framework note rather than editing
   AC 10 in place, to avoid collateral edits to an existing criterion. Owner to
   confirm whether AC 10 should be edited in place (and its Unicode `<=`/`>=`
   replaced with ASCII) at implementation time, or left as-is with the Req 37
   override standing. The source `AllocParams::validate` already rejects
   `lrecl == 0`, so the implementation MUST relax it for U -- the only question
   is the documentation form of AC 10.
2. **LRECL not-applicable representation.** Design chose `Option<u32>` with
   `None`/`Some(0)` meaning not-applicable for U. If the owner prefers an
   explicit model marker (newtype/enum), that is a small design change to confirm
   before implementation.
3. **Acceptance test plan.** `core-acceptance-test-plan.md` currently covers
   UI/command behaviour (Files Explorer, catalog navigation) and has NO rows for
   dataset-model internals (RECFM validation, allocation params, program search).
   CR-NR-108 is a model/metadata foundation; the only user-observable surface
   (Properties_Panel showing RECFM=U LRECL as "not applicable", and
   LibraryKind/MemberContentType display) is a LATER dataset-catalog UI task, not
   core shell behaviour walked in the sequential test pass. Per workflow.md 1a I
   did NOT force speculative rows in. Owner to decide whether a Properties_Panel
   row (e.g. "allocate a RECFM=U load library and view its properties -> LRECL
   shows not-applicable, kind shows Load Library") should be added when the UI
   task is scheduled.

## Verification performed

- Confirmed real highest existing ids by reading file ends: Requirement 36 (last
  in dataset-catalog), Task 43 (last in dataset-catalog tasks), CR-NR-107 (last
  change-log). New ids continue sequentially: Req 37-39, Task 44-46, CR-NR-108.
- EARS phrasing on every new criterion (WHEN ... THE ... SHALL ... / THE ...
  SHALL ...).
- No contradiction with existing criteria: Req 37 narrows Req 7's LRECL rule in
  scope (noted), does not delete it; Req 38/39 are additive.
- Plain-ASCII check: my new content uses only `--`, `->`, `>=`, straight quotes.
  A repo scan flagged only PRE-EXISTING lines (en-dashes in `1-8`, `->`/`<=`/`>=`
  glyphs, ellipsis in menu labels) that I did not author; see owner decision 1
  for the one existing AC that carries prohibited glyphs.
- Task markers are `[ ]` only with descriptive titles.
- No file outside `docs/` was created or modified; no cargo run; no code written.
