# Gate Summary -- Record-aware ff-vfs::BackendEnvironment store contract

**Status: DOCS-ONLY gate. Design-review APPROVED. ASCII-clean. OWNER APPROVED --
CONTRACT SHAPE = SHAPE 2. APPROVED-as-the-plan / PENDING IMPLEMENTATION.**

The owner confirmed the DIRECTION ("move forward with both CR-CH-058 and RC.B.8
Part 2", "design the contract once", "A and B") AND, at gate review, APPROVED the
gate and picked **SHAPE 2** (add a record-aware store method ALONGSIDE the byte
`save`, with a provided default that declines; only record-aware CEs override).
This finalises the DESIGN/CONTRACT SHAPE only. Code (BRC.2 contract reshape, BRC.3
SAVE-walk selection, BRC.4 mainframe CE) begins ONLY on a SEPARATE explicit TASK
instruction, SEQUENCED per the overlap report: CR-CH-058 F1 lands on main first
(the editor needs records to pack), THEN BRC.2 (the ff-vfs additive entry), with
BRC.4 gated on RC.B.8 Part 2 (b)-(d) wiring.

---

## 1. Allocated CR id

**CR-CH-060** -- the next unprefixed mainline CR-CH (highest prior was CR-CH-059).
Logged under `## Change Requests` in `docs/status/change-log.md`, flagged as an
owner-directed FRAMEWORK CHANGE, PENDING GATE (direction confirmed; shape pending
approval). It cross-references its two consumers CR-CH-058 and CR-CH-059 RC.B.8
Part 2, and cites the overlap report (sections E/F) and the part2 stop file.

## 2. Sub-projects touched + new requirement/criterion ranges

| Sub-project | What was added | New criteria |
|-------------|----------------|--------------|
| `command-environments` (PRIMARY -- owns the trait) | Requirement 18 "Record-aware BackendEnvironment store contract" + Glossary terms `Record_Store_Contract`, `Store_Target`; design.md section (both shapes, Shape 2 recommended, dataflow, object-safety, native-byte-identical, registry-opening note, designed-once + DAG); tasks 23-25 | **Req 18.1-18.8** (8), continues from Req 17 |
| `document-model` (editor SAVE-walk side) | Requirement 13 "SAVE-walk store-call selection over the record-aware store contract"; design delta "SAVE-walk store-call selection (CR-CH-060)"; Cross-References note | **Req 13.1-13.6** (6), continues from Req 12 |
| `virtual-file-system` | Cross-reference note only (ff-vfs owns the record-aware trait; additive, object-safe) | none (explicitly "no new criterion here") |
| `dataset-catalog` | Cross-reference note after Req 34.6 (DatasetAccess::put is the mainframe store target) | none |
| `idcams-emulator` | Cross-reference note after Req 28.7 (mainframe CE home implements the record method) | none |

No existing criterion was renumbered or deleted. Req 16.5 (command-environments)
and Req 12.12 / Req 11.2 (document-model) are REFERENCED, not altered. Total new
criteria: **14** (8 + 6).

## 3. The TWO contract shapes (owner picks ONE at approval)

### Shape 1 -- change the `save` signature to a single record-aware form
One method carries records-or-bytes + target + attrs.
- Object-safety: achievable, but the single method must still express a plain
  byte write, so its parameter grows a bytes-or-records union -- more complex at
  the one call site that only ever writes bytes.
- Native byte-identical: AT RISK -- every host write travels through the
  record-aware parameter shape; the "nothing changed for native" guarantee must
  be re-proven and the regression surface is larger.
- Migration cost: HIGH -- every `impl BackendEnvironment` (`ff-ce-host-fs`,
  `ff-ce-ntfs`, `ff-ce-posix`) and the one call site change at once.
- Consumption: the Delimited path and the mainframe path share one method but
  branch inside it -- muddier.

### Shape 2 -- ADD a record-aware store method ALONGSIDE the byte `save` (RECOMMENDED)
Two methods; host CEs keep `save`; the record method has a provided default that
declines (not-record-capable); only record-aware CEs override it.
- Object-safety: trivially preserved -- both methods take `&self` and concrete
  (non-generic, non-`Self`-returning) parameters; `dyn BackendEnvironment` stays
  valid. The provided default keeps existing host impls compiling unchanged.
- Native byte-identical: PRESERVED BY CONSTRUCTION -- `save(path, bytes)` is
  untouched, so native/Delimited SAVE is literally the same code path and the
  same bytes (command-environments Req 16.5, document-model Req 12.12).
- Migration cost: LOW -- host CEs need NO change (they inherit the default); only
  the mainframe CE (in `ff-idcams`) and the editor SAVE walk opt in.
- Consumption: cleanest -- the SAVE walk picks the method by the owning CE's
  advertised RecordFormat; the mainframe CE implements the record method over
  `ff_dscatalog::DatasetAccess`.

### Owner decision and justification
**OWNER APPROVED SHAPE 2 -- additive record-aware store method alongside the byte
save.** It is the only shape that makes native-byte-identical a CONSTRUCTION
guarantee rather than a re-proof, keeps the trait object-safe with ZERO host-CE
migration (`ff-ce-host-fs`/`ff-ce-posix`/`ff-ce-ntfs` inherit the declining
default), and lets each consumer (CR-CH-058 SAVE walk; CR-CH-059 RC.B.8 Part 2
mainframe CE) opt in cleanly. The overlap report section E concurs. Shape 1 is
REJECTED for this contract (re-routes every host write, native-byte-identical at
risk, higher migration cost) but remains fully documented in design.md as the
considered alternative. Illustrative trait/method/type names may be finalised at
design discretion during implementation, keeping the approved semantics
(Store_Target carrying dataset identity + owning-env; an object-safe record
stream; RECFM/LRECL/encoding attrs; a `record_capable()`-style capability so the
SAVE walk selects byte-vs-record by the CE's advertised RecordFormat supplied at
OPEN, never re-derived).

## 4. The records -> store design dataflow (designed ONCE for both consumers)

```
editor SAVE (FFEDIT verb, CR-CH-053 Task 20/21 owning-CE SAVE-addressing seam)
  -> CR-CH-058 SAVE walk: walk the Piece_List, re-frame each piece per the
     owning CE's RecordFormat (Delimited -> bytes; Fixed/Variable -> records)
  -> if owning CE advertises Delimited/host : BackendEnvironment::save(path, bytes)  [byte-identical, unchanged]
     if owning CE advertises Fixed/Variable : BackendEnvironment record-aware store entry
  -> mainframe CE (ff-idcams) implements the record entry over
       ff_dscatalog::DatasetAccess::open -> put(record)* -> close
     (the RECFM codec frames bytes on close: FixedCodec/VariableCodec/BinaryCodec)
```

Single source of truth: the owning CE supplies the RecordFormat at OPEN
(document-model Req 11.2); the SAVE path MUST NOT re-derive framing. Single
SAVE-addressing seam preserved (CR-CH-053 Task 20/21); NO parallel save path, NO
second dispatcher. DAG unchanged and acyclic: `ff-idcams -> ff-dscatalog ->
ff-volume -> ff-vfs` (the record-aware trait stays in ff-vfs; the mainframe CE in
ff-idcams implements it). Opening the closed `RegisteredEnv` registry is RC.B.8
Part 2 prereq (b) wiring on the EXISTING seam, NOT a CR-CH-060 criterion.

## 5. New master-task rows + TCR rows

### Master tasks (`docs/project-management/project-master/tasks.md`)
New section `## Phase (backendenv-record-contract) -- CR-CH-060`, all `[ ]`:
- **BRC.1** Requirements gate (this authoring).
- **BRC.2** The contract reshape in ff-vfs -- additive record-aware store entry
  alongside the retained byte `save`, provided default + `record_capable()`
  advertisement, `dyn` object-safe, host CEs inherit the default unchanged.
  (command-environments tasks 23.1-23.2.)
- **BRC.3** Editor SAVE-walk store-call selection -- FFEDIT SAVE picks byte vs
  record by the owning CE's advertised RecordFormat on the single Task 20/21 seam.
  (command-environments task 24.1; document-model Req 13.)
- **BRC.4** Mainframe CE record-aware store over DatasetAccess (DOWNSTREAM; needs
  RC.B.8 Part 2 (b)-(d) registry/binding/provider wiring).
Plus a Summary count row recording the gate as authored.

### TCR (`docs/quality/TCR.md`)
New section `### Phase (backendenv-record-contract) -- CR-CH-060` with **14 NOT
COVERED (red) rows**, one per new criterion:
- command-environments Req 18.1-18.8 -> crates `ff-vfs` (18.1-18.4),
  `ff-document-model` (18.5-18.6), `ff-idcams` (18.7-18.8).
- document-model Req 13.1-13.6 -> crate `ff-document-model`.

## 6. The two consumer CRs and exactly how each consumes the contract

- **CR-CH-058 (universal windowed record-oriented document model / piece-table).**
  Its F-phase SAVE walk re-frames the Piece_List per the owning CE's RecordFormat
  and calls the contract: the BYTE entry (`save(path, bytes)`) for a Delimited/host
  CE -- byte-identical to pre-CR-CH-058 behaviour (document-model Req 13.2 / Req
  12.12) -- and the RECORD-aware entry for a Fixed/Variable CE (document-model Req
  13.1/13.3). The selection uses the RecordFormat supplied at OPEN; the walk does
  NOT re-derive framing (Req 13.4). This is the document-model SIDE of CR-CH-060;
  it does NOT itself reshape the trait.

- **CR-CH-059 RC.B.8 Part 2 (record-aware MAINFRAME editor SAVE).** Its
  prerequisite (a) IS this contract reshape. Once the record-aware entry exists,
  the mainframe Command Environment (housed in `ff-idcams`) implements it over
  `ff_dscatalog::DatasetAccess` (open -> put each record -> close so the RECFM
  codec frames bytes), surfacing the store RC (incl. x37 space-full) through the
  outcome type (command-environments Req 18.7/18.8). Prerequisites (b)-(e) --
  opening the closed `RegisteredEnv` enum, binding the mainframe `owning_env`,
  registering the mainframe provider -- are downstream wiring on the EXISTING seam,
  not CR-CH-060 criteria (BRC.4 is marked downstream and gated on them).

Designing the contract ONCE means the SAVE seam is not reshaped twice and no
separate mainframe packer duplicates CR-CH-058's piece-list re-framing.

## 7. Steering edit made

`.kiro/steering/framework-conformance.md` -- the "What counts as a framework
change" BackendEnvironment bullet now records the OWNER-CONFIRMED CR-CH-060 note:
the store contract is record-aware; the byte `save` is RETAINED for host CEs
(native SAVE byte-identical); the record-aware entry is the additive Shape 2
(Store_Target + object-safe record stream + RECFM/LRECL/encoding, provided default
so host CEs are unchanged, kept object-safe for `dyn BackendEnvironment`); it
serves the mainframe CE over `ff_dscatalog::DatasetAccess`. Any FURTHER reshape of
this contract still needs express owner confirmation. ASCII-clean.

## 8. ASCII-check result

PASS for all gate-added content. The documentation enforcement check
(`rg '[^\x00-\x7F\u2500-\u257F]'`, allowing box-drawing + TCR status emoji) was run
via the clean non-interactive pwsh7 wrapper with output captured to
`tools/logs/`:
- `command-environments/requirements.md`, `design.md`, `tasks.md`: ZERO non-ASCII
  matches (`tools/logs/backendenv-gate-ce.txt` empty).
- `document-model` Req 13 block and the "SAVE-walk store-call selection" design
  delta: ASCII-clean (verified by reading the ranges). The only non-ASCII in those
  two files is PRE-EXISTING, in UNRELATED sections (en-dashes in the Introduction /
  Req 1-11 prose, box-drawing diagrams, and math symbols in correctness-property
  blocks) that predate this gate and sit outside any CR-CH-060 content -- correctly
  left untouched (out of scope for a docs-only contract gate).
- `project-master/tasks.md`, `TCR.md`, `change-log.md` CR-CH-060 rows: ASCII-clean
  (the whole-file grep hits are pre-existing mojibake in historical rows + the
  documentation.md-allowed TCR status emoji, none in CR-CH-060 content).
- `.kiro/steering/framework-conformance.md`: ZERO non-ASCII matches.

No prohibited characters were found in any gate-added content, so no fixes were
needed.

## 9. Gate completeness (per workflow.md)

- EARS criteria, sequentially numbered per file (command-environments Req 18.1-18.8
  after Req 17; document-model Req 13.1-13.6 after Req 12); no existing criterion
  altered.
- tasks `[ ]` only and titled, each with a `Validates:` cross-reference
  (command-environments tasks 23-25 and sub-tasks).
- master Phase section (BRC.1-BRC.4) + Summary row.
- 14 TCR NOT COVERED rows, one per new criterion.
- CR-CH-060 change-log entry cross-referencing CR-CH-058 and CR-CH-059 RC.B.8 Part 2.
- framework-conformance.md steering edit present.
- Design review verdict: **APPROVED** (0 HIGH, 0 MEDIUM findings) --
  `.agents/tasks/backendenv-record-contract-gate/design-review.json`.

---

## OWNER APPROVAL -- RECORDED

This gate is DOCS-ONLY. It is approved by design review, ASCII-clean, and the owner
has APPROVED it with **CONTRACT SHAPE = SHAPE 2**. The approval finalises the
DESIGN/CONTRACT SHAPE only -- it does NOT authorise implementation. Code (BRC.2,
BRC.3, BRC.4) begins only on a SEPARATE explicit TASK instruction, SEQUENCED per
the overlap report: CR-CH-058 F1 lands on main first, THEN BRC.2 (ff-vfs additive
entry), with BRC.4 gated on RC.B.8 Part 2 (b)-(d) wiring. The gate package is now
APPROVED-as-the-plan / PENDING IMPLEMENTATION; the docs record the approval and
Shape 2 across change-log, command-environments design.md, TCR, and the
project-master Summary row (framework-conformance.md already carries the
owner-confirmed CR-CH-060 Shape 2 note).
