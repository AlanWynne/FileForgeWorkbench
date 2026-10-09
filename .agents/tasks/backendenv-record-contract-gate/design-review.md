# Design Review -- BackendEnvironment record-aware store contract gate (CR-CH-060)

Reviewer: design-review subagent (fresh context). Scope: DOCS-ONLY, owner-directed
FRAMEWORK-CHANGE requirements gate. No build/test/cargo was run (there is no code
in this change). Verdict is mechanical: HIGH + MEDIUM findings > 0 => CHANGES_REQUESTED;
0 => APPROVED.

## What was reviewed

- Authoring notes: `.agents/tasks/backendenv-record-contract-gate/author-notes.md`.
- Grounding: `.agents/tasks/crch058-rcb8-overlap/report.md` (sections E/F),
  `.agents/tasks/windowed-record-foundation/FOUNDATION-DESIGN.md`,
  `.agents/tasks/rcb8-dataset-rationalisation/part2-mainframe-save-stop.md`.
- Gate artifacts:
  - `docs/specs/command-environments/requirements.md` (Req 18.1-18.8 + Glossary).
  - `docs/specs/command-environments/design.md` (CR-CH-060 section).
  - `docs/specs/command-environments/tasks.md` (tasks 23-25).
  - `docs/specs/document-model/requirements.md` (Req 13.1-13.6 + Cross-References).
  - `docs/specs/document-model/design.md` (Design Delta: SAVE-walk store-call selection).
  - Cross-reference notes in `virtual-file-system/requirements.md` (after Req 13.5),
    `dataset-catalog/requirements.md` (after Req 34.6),
    `idcams-emulator/requirements.md` (after Req 28.7).
  - `docs/project-management/project-master/tasks.md` (Phase (backendenv-record-contract),
    BRC.1-BRC.4 + Summary row).
  - `docs/quality/TCR.md` (CR-CH-060 section, 14 NOT COVERED rows).
  - `docs/status/change-log.md` (CR-CH-060 entry).
  - `.kiro/steering/framework-conformance.md` (BackendEnvironment store-contract note).

## Review against each required criterion

### 1. Faithfulness to overlap report section E/F (one contract serving BOTH consumers)
PASS. The overlap report section E recommends authoring ONE framework-change gate for
the `BackendEnvironment` store contract satisfying both CR-CH-058's SAVE walk and
RC.B.8 Part 2 prereq (a), keeping byte `save` for host CEs and adding a record-aware
entry. command-environments Req 18 and its design section implement exactly this, name
both consumers explicitly, and state "designed ONCE so the SAVE seam is not reshaped
twice." Section F's sequencing (contract first, mainframe CE downstream, registry wiring
as prereq (b)-(d)) is reflected in tasks 24/25 and BRC.3/BRC.4 downstream caveats.

### 2. Byte save(path,bytes) KEPT for host CEs with explicit byte-identical criterion
PASS. command-environments Req 18.1 retains `save(&self, path: &Path, bytes: &[u8]) ->
io::Result<()>` unchanged and asserts native SAVE is byte-identical, cross-referencing
Req 16.5 and document-model Req 12.12. document-model Req 13.2 restates the byte-identical
guarantee for the Delimited/host branch. Both the no-native-regression criterion and the
retained-signature requirement are present.

### 3. Record-aware store entry correctly specified (target + records + attrs; mainframe CE over DatasetAccess)
PASS. Req 18.2 specifies the three carried elements: (a) Store_Target by DSN / catalog
identity / owning-environment (NOT a host path), (b) an object-safe record stream, (c)
record attributes RECFM/LRECL/encoding. Req 18.7 has the mainframe CE (ff-idcams)
implement the entry over `ff_dscatalog::DatasetAccess` with open -> put -> close so the
RECFM codec frames bytes on close. This matches the overlap report section C/E and the
part2 stop file prereq (a)/(e).

### 4. CE supplies RecordFormat at open as single source of truth; save does not re-derive framing
PASS. Req 18.5 and document-model Req 13.4 both require the byte-vs-record selection be
driven by the owning CE's advertised RecordFormat "supplied at OPEN (document-model Req
11.2)" and explicitly state the save SHALL NOT re-derive framing. The design dataflow in
both design.md files repeats the single-source-of-truth statement. Consistent with
FOUNDATION-DESIGN section 6 and document-model Req 11.2.

### 5. Single owning-CE SAVE seam preserved; no parallel path/dispatcher; registry opening noted as prereq (b) wiring
PASS. Req 18.6 and document-model Req 13.5 require the record-aware call to ride the
single CR-CH-053 Task 20/21 FFEDIT->owning-CE seam via `dispatch_to_environment`, with no
parallel save path and no second dispatcher. The command-environments design section
"Registry opening is prereq wiring, not a new mechanism" correctly scopes opening the
closed `RegisteredEnv` enum / binding owning_env / registering the mainframe provider as
RC.B.8 Part 2 prereq (b)-(d) wiring on the EXISTING seam, NOT a CR-CH-060 criterion.

### 6. Object-safety of the trait preserved under the chosen shape
PASS. Req 18.4 requires `BackendEnvironment` to remain object-safe (`dyn
BackendEnvironment` in the registry) with the record-aware entry taking `&self` and
non-generic, non-`Self`-returning parameters (object-safe `&dyn` record stream). The
design justifies Shape 2 as "object-safety trivially preserved" with a provided default
keeping `dyn` dispatch valid and host impls compiling untouched. Task 23.1 writes an
object-safety test first.

### 7. BOTH shapes presented with tradeoffs and ONE recommended with justification
PASS. command-environments design.md presents Shape 1 (change `save` to a single
record-aware form) and Shape 2 (add a record-aware method alongside), each with
object-safety, native-byte-identical, migration-cost, and consumption tradeoffs. Shape 2
is recommended and justified on all four axes required by the brief: object-safety,
native-byte-identical by construction, migration cost across
ff-ce-host-fs/ff-ce-ntfs/ff-ce-posix (host CEs inherit the default, zero migration), and
clean consumption by both the CR-CH-058 SAVE walk and the mainframe CE. The owner retains
the final pick at approval; Shape 1 remains documented. This matches the overlap report
section E lean.

### 8. Gate completeness per workflow.md
PASS. Checked each required artifact:
- EARS criteria sequentially numbered per file: command-environments Req 18 continues from
  Req 17 (18.1-18.8); document-model Req 13 continues from Req 12 (13.1-13.6). No existing
  criterion renumbered or deleted (Req 16.5 / Req 12.12 referenced only). EARS form
  (WHEN/THE/SHALL) used.
- tasks.md `[ ]` only and titled: command-environments tasks 23/24/25 (and sub-tasks
  23.1/23.2/24.1/25.1) are all `[ ]`, each has a descriptive title and a `Validates:`
  cross-reference. No `[x]` pre-marked. document-model carries no new tasks (correctly --
  its consumption is noted; the SAVE-walk store-call lands in the CR-CH-058 F-series work,
  stated in its design delta).
- master-task Phase row + Summary: `Phase (backendenv-record-contract) -- CR-CH-060` with
  BRC.1-BRC.4 (`[ ]` only) and a Summary count row are present in
  project-master/tasks.md.
- one TCR NOT COVERED row per new criterion: 14 red rows present (command-environments
  Req 18.1-18.8 = 8; document-model Req 13.1-13.6 = 6), crates ff-vfs / ff-document-model /
  ff-idcams as appropriate.
- new CR-CH id in change-log cross-referencing CR-CH-058 and CR-CH-059: CR-CH-060 entry
  present under `## Change Requests`, highest prior unprefixed CR-CH is CR-CH-059 so 060 is
  the correct next id; it names CR-CH-058 and CR-CH-059 RC.B.8 Part 2 as the two consumers,
  cites the overlap report sections E/F and the part2 stop file, and is flagged FRAMEWORK
  CHANGE + owner-directed + PENDING GATE (direction confirmed; SHAPE pending approval).
- steering edit present: framework-conformance.md "What counts as a framework change"
  BackendEnvironment bullet is updated with the OWNER-CONFIRMED CR-CH-060 note (record-aware,
  byte `save` retained, additive Shape 2, object-safe, serves the mainframe CE).

### 9. ASCII-only (documentation.md); no contradiction with existing criteria
PASS (with a correctly-scoped pre-existing-only note). All CR-CH-060-authored content is
ASCII-clean: command-environments requirements.md / design.md / tasks.md and
framework-conformance.md return zero non-ASCII matches; the master-task row, TCR rows, and
change-log entry use only ASCII (plus the TCR status emoji, which documentation.md allows
in TCR tables). document-model design.md's only non-ASCII is box-drawing in a pre-existing
architecture diagram (ALLOWED by documentation.md). document-model requirements.md's only
non-ASCII is pre-existing en-dashes in the Introduction and Req 1-11 prose (lines ~11, ~210
onward) that predate this gate and sit OUTSIDE the Req 13 block (lines 319-371, which is
ASCII-clean). These are unrelated-section pre-existing issues the author-notes correctly
flagged as out-of-scope for this docs-only contract gate; they do not touch any CR-CH-060
content. No contradiction with existing criteria was found: Req 18 is additive to Req 16/17
and references (does not alter) Req 16.5; Req 13 is additive after Req 12 and references
(does not alter) Req 12.12 / Req 11.2.

## Spot-checks of specific articulable doubts

- Did the CR-CH-060 record-aware store entry invent a numbering collision with the
  pre-existing "Req 18"/"Req 13" rows elsewhere in TCR? No -- the earlier TCR grep hits for
  "Req 13"/"Req 18" are from unrelated sub-projects (editor_panel, about_dialog, theme,
  rust-toolchain). The CR-CH-060 TCR rows are in a dedicated `Phase
  (backendenv-record-contract)` section and are tagged with the owning sub-project
  ("command-environments Req 18.x", "document-model Req 13.x"), so there is no ambiguity.
- Is the DAG claim (`ff-idcams -> ff-dscatalog -> ff-volume -> ff-vfs`) consistent with the
  overlap report and RC.B.8? Yes -- the overlap report and the change-log CR-CH-059 entry
  describe the same acyclic chain; Req 18.7 and both design sections repeat it. The
  record-aware trait stays in ff-vfs; the mainframe CE in ff-idcams implements it.
- Does document-model carry no new tasks correctly? Yes -- the SAVE-walk store-call
  selection is a consumption that lands within the CR-CH-058 F-series SAVE-walk work
  (design delta "Where it is wired"); authoring duplicate tasks in document-model would
  risk a double-build. The command-environments task 24.1 owns the selection task and is
  cross-referenced from master BRC.3.

## Verified Assumptions

- Overlap report section E recommends ONE contract, byte save retained + record-aware entry
  added -- VERIFIED by reading report.md section E; the gate's Shape 2 matches it.
- CR-CH-059 is the highest prior unprefixed CR-CH, so CR-CH-060 is the correct next id --
  VERIFIED: change-log.md shows CR-CH-060 immediately above CR-CH-059; no CR-CH-061+ exists.
- `BackendEnvironment::save(path, bytes)` is today's byte-only signature and the trait doc
  comment acknowledges a mainframe backend "would instead pack records per RECFM/LRECL" --
  VERIFIED against the part2 stop file evidence point 1 and the design.md "Today" block
  (the review relied on the grounding docs' quoted code, which is consistent across sources;
  no .rs was read, per the docs-only mandate).
- DatasetAccess open/put/close with RECFM codec framing on close -- VERIFIED consistent
  across the overlap report section C evidence index and Req 18.7 / both design sections.
- 14 TCR NOT COVERED rows (8 + 6), one per new criterion -- VERIFIED by reading the TCR
  CR-CH-060 section.
- Cross-reference notes in virtual-file-system / dataset-catalog / idcams-emulator add NO
  new criteria -- VERIFIED: each note is explicitly tagged "no new criterion here."
- All CR-CH-060-authored content is ASCII-clean -- VERIFIED by scoped non-ASCII grep per
  file; the only hits are documentation.md-allowed box-drawing and pre-existing
  unrelated-section en-dashes.

## Unverified / Wrong Assumptions

- Exact live signature and doc comment of `ff-vfs::BackendEnvironment` were NOT
  independently read from the `.rs` source (the review mandate forbids reading/building
  code; this is a docs-only gate). The contract-shape claims were cross-checked across three
  independent grounding docs (overlap report, part2 stop file, FOUNDATION-DESIGN) which
  agree, so the risk is low, but the signature's current exact form is taken on the
  grounding docs' word rather than first-hand code inspection. This is a scope limit of the
  review, not a defect in the gate.
- No wrong assumptions were found in the authored documents. The author-notes' iteration-2
  claims (artifacts present, EARS-numbered, ASCII-clean in authored content, pre-existing
  non-ASCII left untouched) all held up against the files on disk.

## Findings

No HIGH or MEDIUM findings. No NIT findings that affect the verdict.

(One observation, non-blocking and already handled by the author: document-model
requirements.md contains pre-existing non-ASCII en-dashes in sections unrelated to
CR-CH-060. These predate this gate, sit outside the Req 13 block, and are correctly out of
scope for a docs-only contract gate. This is noted for the owner's awareness, not raised as
a finding against this change.)

## Verdict

APPROVED. The gate package faithfully implements the overlap report section E/F
recommendation: one record-aware store contract serving both CR-CH-058 and RC.B.8 Part 2,
byte `save` retained with an explicit byte-identical criterion, the record-aware entry
correctly carrying target + records + attrs with the mainframe CE implementing over
DatasetAccess, RecordFormat supplied at open as the single source of truth, the single
Task 20/21 SAVE seam preserved, object-safety preserved, both shapes presented with Shape 2
recommended and justified, and full gate completeness (EARS criteria, titled `[ ]` tasks,
master Phase + Summary, 14 TCR rows, a correctly-numbered cross-referencing CR-CH-060
change-log entry, and the framework-conformance steering edit), all ASCII-clean in the
authored content.
