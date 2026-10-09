# Gate Conformance Review -- Mainframe Dataset Stack Rationalisation (CR-CH-059)

Reviewer role: GATE CONFORMANCE reviewer, docs-only requirements-gate package.
Scope: the gate package the previous step authored/edited across `docs/`, verified
against the authoritative plan `.agents/tasks/dataset-vision-fit/report.md`
(sections C/D/E) and the retirement procedure in
`.agents/tasks/dscatalog-duplicate/report.md`.

Method: read the report + the duplicate-crate report; diffed the whole `docs/`
tree (`git diff -- docs/`); read every gate-added spec section
(requirements/design/tasks) for dataset-ownership-model, virtual-file-system,
dataset-catalog, dataset-allocator, idcams-emulator, jes-emulator,
virtual-catalog-manager, volume-model; read the change-log, bugs.md, TCR.md,
project-master/tasks.md, and stream-prefixes.md gate edits. Ran a programmatic
scan of all ADDED `docs/` diff lines for prohibited non-ASCII (result: zero).
NO build/test was run (docs gate).

Verdict: **APPROVED** -- zero HIGH or MEDIUM findings. Details below.

---

## 1. Correctness vs the report (sections C / D / E)

PASS. The package encodes the report faithfully with no invented alternative
architecture.

- **Section D -- the `DatasetAccess` trait** is captured verbatim-in-intent in
  `dataset-catalog/requirements.md` Requirement 34 and `dataset-catalog/design.md`
  ("DatasetAccess -- the single JES/JCL access contract"). All seven methods are
  present and correctly scoped: `allocate`/`open`/`get`/`put`/`point`/`close`/
  `dispose` (Req 34.1), keyed by DD + DISP + DSN with VOL=SER (34.1), RECFM-aware
  get/put with no host text-line delimiter (34.2), resolving location through
  `ff-volume` (34.3, and volume-model Req 12.1), backed by `ff-vfs::StorageProvider`
  + the record codecs (34.3, virtual-file-system Req 13), owned by `ff-dscatalog`
  (34.1/34.6). Object-safety is required (34.8). This matches report section D
  exactly.
- **VSAM modelled as a cluster entity, not a Dsorg variant**: dataset-catalog
  Req 33.5 ("VSAM SHALL be modelled as a cluster entity ... NOT as a `Dsorg`
  variant -- the removal of the `Vsam` and `Da` `Dsorg` variants is deliberate")
  and the terminology map in design.md. Matches report section B/C.
- **The one `Dsorg {PS,PO,GDG}` / `Recfm {F,FB,V,VB,U}` collapse** is specified
  in dataset-catalog Req 33.4 (both `#[non_exhaustive]`, both owned by
  `ff-dscatalog`, the divergent `ff-dataset-catalog` `{Ps,Po,Da,Vsam}` /
  `{F,Fb,V,Vb,U}` set explicitly not carried forward). Matches report section B.
- **Section C -- KEEP/MERGE/RENAME/RETIRE + DAG**: the target DAG is reproduced in
  `dataset-catalog/design.md` ("Target dependency DAG") and
  `dataset-ownership-model/requirements.md` Req 22.6 as
  `ff-idcams -> ff-dsalloc -> ff-dscatalog -> ff-volume -> storage providers` with
  `ff-vfs` as universal infrastructure, asserted acyclic. RETIRE of
  `ff-dataset-catalog` + `ff-vsam-services` and the single-`posix`-registrant
  collapse are all encoded (Req 35, virtual-file-system Req 14).
- **Section E -- sequencing + the "must NOT build before consolidation" waste
  list** is reproduced as the "Retirement sequence (additive-first)" and the
  "DO NOT build before consolidation" block in `dataset-catalog/design.md`, as
  Req 35.5, and as project-master phase RC.C.11. The additive-first ordering
  (reconciled trait before any deletion) is stated repeatedly and consistently.

No alternative architecture was introduced; every delta references the existing
seams (`ff-vfs::StorageProvider`, `VfsProvider`, `render_workspace_context`,
`resolve_target`, `WorkspaceDescriptor`, `ff-volume`).

## 2. EARS + numbering

PASS.

- Every new acceptance criterion uses the `WHEN ... THE ... SHALL ...` /
  `THE ... SHALL ...` EARS form. Spot-verified across ownership Req 22,
  vfs Req 13-14, dataset-catalog Req 33-35, dataset-allocator Req 17-19,
  idcams Req 27-28, jes Req 19, vcm Req 17-18, volume-model Req 12.
- Numbering continues from each file's existing highest with no collision:
  - dataset-ownership-model: existing top Req 21 -> new Req 22.
  - virtual-file-system: existing top Req 12 -> new Req 13, 14.
  - dataset-catalog: existing top Req 32 -> new Req 33, 34, 35.
  - dataset-allocator: existing top Req 16 -> new Req 17, 18, 19.
  - idcams-emulator: existing top Req 26 -> new Req 27, 28.
  - jes-emulator: existing top Req 18 -> new Req 19.
  - virtual-catalog-manager: existing top Req 16 (Req 11 is the documented gap)
    -> new Req 17, 18 (matches the in-file note "next free Requirement number
    here is 17").
  - volume-model: existing top Req 11 -> new Req 12.
- No new criterion contradicts an existing one; each delta explicitly states it
  is additive and preserves prior decisions (e.g. ADR-002 "catalogs never own
  bytes" restated as preserved in ownership Req 22.7; vfs Req 13 builds ON the
  existing Req 9 StorageProvider).
- Touched existing criteria carry change notes: the jes-emulator Introduction,
  Req 1.7, and Req 11.1/11.3/11.5/12.5 crate-name corrections each carry
  "(Crate names corrected ... by CR-CH-059)"; the ADR-001 read-as remap is
  recorded in ownership Req 22.1 + the design delta. These are the only
  in-place edits to pre-existing criteria and all are annotated.

## 3. Sub-project coverage

PASS. Every required sub-project is covered as specified:

- **dataset-ownership-model**: ADR-001 amended via Req 22 -> `ff-dscatalog`
  authority; `ff-dataset-catalog` + `ff-vsam-services` marked
  DEPRECATED-FOR-MERGE (Req 22.2, 22.3); acyclic DAG with `ff-volume` owning
  Volume (Req 22.6); design delta added; fitness function update task (13.4).
- **virtual-file-system**: single `StorageProvider` physical seam (Req 13), one
  `VfsProvider`/single posix registrant seam (Req 14), hybrid storage retained
  (Req 13.4 keeps codecs in `ff-dscatalog`, no change to the hybrid model).
- **dataset-catalog**: single authority + reconciled traits (Req 33) +
  `DatasetAccess` (Req 34) + enum collapse (Req 33.4) + `storage_path`->Volume
  locator referencing Req 32 not duplicating (Req 34.3 + design "schema v4 ...
  owned by the CR-CH-057 delta ... CR-CH-059 only names DatasetAccess as the
  reader"). Terminology map lives in this one design.md.
- **dataset-allocator**: retarget `catalog_bridge` to `ff-dscatalog`, return
  `DatasetHandle` (Req 19); references Req 17-18 (the CR-NR-105/CR-CH-057 Volume
  flow) without contradiction (Req 19.5 preserves them).
- **idcams-emulator**: repoint private traits (Req 28.1); DEFINE/REPRO/DELETE map
  onto DatasetAccess (Req 28.3-28.5); references Req 27 (DEFINE VOLUME) and
  composes with it.
- **jes-emulator**: executor depends only on `ff-dsalloc` + `DatasetAccess`
  (Req 19.1-19.2); crate-name fixes (Req 19.3, Introduction/Req 1.7/11/12.5);
  JES stays DEFERRED (Req 19.5, scope note). No JES build obligation added.
- **volume-model**: Req 12 references Req 1-11 and dataset-catalog Req 32/34
  WITHOUT duplicating the entity model (explicit "reference, do NOT duplicate"
  note); schema v4 `storage_path`->DatasetVolume locator migration expressed
  (Req 12.2).

## 4. Framework conformance

PASS. The package builds ON the framework and surfaces the framework change
explicitly for owner confirmation (change-log "Framework-change note").

- Builds on `ff-vfs`/`ff-dscatalog`/`ff-volume` seams; `DatasetAccess` lives in
  `ff-dscatalog` over the single `ff-vfs::StorageProvider` seam resolving through
  `ff-volume`.
- Does not contradict virtual-catalog-manager Req 17-18 (the Volume UI is a
  `WorkspaceContext` via `render_workspace_context` with the pending_action
  effect pattern and a stable `egui::Id`; see vcm Req 17.2 and tasks 28-32).
- DAG asserted acyclic (ownership Req 22.6, volume-model Req 12.3,
  dataset-catalog design DAG).
- Catalogs hold metadata + locator only (ADR-002) preserved (ownership Req 22.7,
  volume-model Req 12, dataset-catalog design).
- The framework change (removing public crates, deleting the duplicate physical
  trait, adding the public `DatasetAccess` type, amending ADR-001) is flagged
  per framework-conformance.md as owner-directed-but-still-needing-explicit-gate-
  approval, and is additive-first so the build never breaks.

## 5. Task list shape

PASS.

- Ordered phases RC.A / RC.B / RC.C in project-master mirror report section E
  (RC.A cheap/safe-now consolidation; RC.B next-when-phase-2-opens; RC.C
  defer/owner-blocked). Per-spec task sections map to these (dataset-catalog
  design "Phase mapping" ties Task 38->RC.A step 2, 39->RC.A step 3, 40->RC.B
  step 6, 41->RC.B step 7, 42->RC.A tail step 9).
- All gate-added tasks use `[ ]`; no pre-marked `[x]` was introduced (the `[x]`
  lines present in the diffs are pre-existing context rows). Verified in
  dataset-catalog tasks 38-42, dataset-allocator 19-21, idcams 28-29, jes 35,
  vfs 17, vcm 28-32, volume-model 11, ownership 13.
- Every task is titled and cross-references criteria (`- Validates: Requirement
  X.Y`).
- The dscatalog-duplicate retirement procedure is encoded as tasks: add
  reconciled trait (38.1-38.3), repoint `ff-governance-tests` + mock_compilation
  (38.4), delete directory + `[workspace].members` line + grep-verify (42.1,
  42.2). This matches the report's steps 1-4.
- The "DO NOT build before consolidation" waste note is present (dataset-catalog
  design.md note + Req 35.5 + project-master RC.C.11).
- Matching Phase rows in project-master/tasks.md with updated Summary counts: a
  new "Phase (dataset-stack-rationalisation)" block (RC.A.1-RC.A.4 / RC.B.5-RC.B.8
  / RC.C.9-RC.C.11) with a Summary count row, and the existing Phase (volume-model)
  Summary line updated to reflect VM.1-VM.8 (second gate authored).

## 6. Governance plumbing

PASS.

- One NOT COVERED (red-circle) TCR row per NEW criterion, in the right crate
  sections. Verified the actual `docs/quality/TCR.md` uses the proper `U+1F534`
  red-circle emoji (allowed in TCR tables). Rows present for ownership 22.1-22.7
  (ff-governance-tests), vfs 13.1-13.5/14.1-14.4 (ff-vfs/ff-dscatalog),
  dataset-catalog 33.1-33.7/34.1-34.8/35.1-35.5 (ff-dscatalog/ff-vfs/
  ff-governance-tests), dataset-allocator 19.1-19.6 (ff-dsalloc), idcams
  28.1-28.7 (ff-idcams), jes 19.1-19.5 (ff-jes), volume-model 12.1-12.4
  (ff-volume), plus the CR-NR-105/CR-CH-057 second-gate rows (vcm 17/18,
  allocator 17/18, idcams 27).
- change-log entry present with a single consistent allocated CR id: `CR-CH-059`,
  unprefixed mainline (correct -- next free after CR-CH-058, no collision). Used
  consistently across all spec Source lines, tasks, TCR headers, and
  project-master.
- The vsam-wiring-worktree conflict is flagged EXPLICITLY as an owner decision in
  three places: change-log CR-CH-059 ("CRITICAL RECONCILIATION ... OWNER DECISION
  REQUIRED"), dataset-catalog design.md ("vsam-wiring worktree conflict (owner
  decision)"), and stream-prefixes.md (the `V` row set to "ON HOLD -- REDIRECT").

## 7. ASCII compliance

PASS. A programmatic scan of ALL added (`+`) lines in the `docs/` diff for the
prohibited set (em/en dash, curly quotes, ellipsis char, unicode arrows,
<=/>=/!= symbols, logic quantifiers, BOM) returned ZERO matches. Gate-added prose
uses `--`, `->`, straight quotes consistently. Box-drawing/DAG art appears only
inside fenced code blocks. The TCR red-circle emoji in `TCR.md` is allowed by the
documentation rule.

Note (NOT a finding, out of scope): pre-existing content in some touched files
contains en-dashes and math symbols (e.g. dataset-catalog/design.md proptest
pseudocode, the pre-existing Acceptance-Criteria-Coverage tables in several
tasks.md files, the jes-emulator pre-existing `U+2192` mojibake in a job-state
line). These are unchanged context, not gate-added, so they fall outside this
gate's ASCII obligation. Likewise bugs.md B083 belongs to the unrelated
`crch053-task17` stream, not this gate.

## 8. Docs-only

PASS for the gate package. All gate-authored changes are under `docs/`
(specs, status, quality, project-management) plus the ephemeral
`.agents/tasks/.../stream-prefixes` governance file under `docs/status/`. The
Rust source changes present in the working tree (`crates/ff-desktop/src/shell/*`)
and the `.agents/tasks/simplify-splits/*` edits belong to the separate in-flight
editor / simplify-core streams (CR-CH-058 / CR-CH-053 worktree work); they are
unrelated to the dataset-rationalisation subject and were not authored by this
gate. The gate-summary artifact is written by a later step and is excluded per
instruction. No Rust source was touched by this gate.

---

## Verified assumptions

- `DatasetAccess` method set and ownership match report section D exactly
  (verified against report.md section D and dataset-catalog Req 34 + design).
- `ff-dscatalog` is the live crate and `ff-dataset-catalog`/`ff-vsam-services`
  are orphan trait fixtures (verified against dscatalog-duplicate/report.md;
  retirement tasks match its RECOMMENDED removal procedure).
- Requirement numbering continuity verified by reading each file's prior top
  requirement in the diff context lines (Req 21/12/32/16/26/18/16/11 -> the new
  numbers).
- TCR.md uses the real red-circle emoji, not mojibake (verified by reading the
  file directly via grep, not the pwsh-redirected log which mis-encoded it).
- CR-CH-059 is the correct next unprefixed mainline CR-CH id (verified: single
  definition, highest prior was CR-CH-058).
- The `V` / vsam-wiring stream exists and its registry row reflects the flagged
  conflict (verified by reading stream-prefixes.md).
- Zero prohibited non-ASCII in added docs lines (verified by programmatic diff
  scan).

## Unverified / wrong assumptions

- None material to the verdict. The pwsh-redirected diff logs mis-rendered the
  TCR red-circle emoji and the bugs.md white-square emoji as mojibake
  (`U+2261 U+0192 ...`); I confirmed via direct file reads that the actual files
  hold the correct emoji, so this was a log-encoding artifact, not a defect.
- I did not independently re-verify every line of the pre-existing (unchanged)
  content of the touched specs; the gate obligation is scoped to gate-added
  content, which I reviewed in full.

---

## Findings

No HIGH, MEDIUM, or NIT findings. The gate package is a faithful, framework-
conforming, EARS-correct, ASCII-clean, docs-only encoding of the authoritative
report, with correct numbering, complete TCR/change-log/project-master plumbing,
and the vsam-wiring conflict explicitly escalated as an owner decision.
