# Remediation Plan -- Product-Layered Build Order

This is the AUTHORITATIVE sequencing for executing the analysis remediation
(the PA-W0..PA-W5 phases in `docs/specs/project-master/tasks.md`). It is a VIEW
over those phases -- an ordering by PRODUCT LAYER, not a rewrite. The PA-W task
IDs remain the detail layer; this plan says WHAT ORDER to tackle them in and WHY.

Ordering rationale (owner-directed): complete the file + editing CORE first, then
MACROS (which unlock whole-product automated testing via FFTest), then syntax
highlighting, then the two PLUGIN layers (JES, Database) last. JES/JCL/Database
are deliberately late because they are plugin-layer, built on top of the core.

Within each bucket the internal order is:
  (a) foundation the bucket rests on -> (b) rewire/resolve orphans + duplication
  -> (c) complete the features -> (d) mechanical hygiene (ASCII, caps, TCR, docs).

Status legend: [ ] not started, [~] in progress, [x] done. (Buckets use plain
prose; the underlying PA-W tasks keep their own `[ ]`/`[x]` checkboxes.)

Cross-cutting items done UP-FRONT regardless of bucket (near-zero-risk, correct
the record so nothing downstream is misled):
- [x] PA-W5.2 data-safety atomic writes (file-operations + global-search + JES
      queue) -- atomicity DONE (commit dc45eca); VFS-routing/backup deferred.
- [ ] PA-W5.5 re-open database-tool false-positive-complete tracking (do before
      Bucket 5; it is a bookkeeping correction, safe to do now).
- [ ] PA-TRACK-007 / PA-TRACK-008 reconcile master + verification narrative to
      point at project-analysis as the source of truth (docs-only).

---

## Bucket 1 -- FILE + FILE-EDITING CORE (do FIRST; must be complete before all else)

Goal: a complete, safe, well-integrated file + text-editing core.

Foundation it rests on:
- platform-core, virtual-file-system, connector-local-fs (primary VFS provider),
  document-model, background-io, configuration-system, logging-subsystem.

File features:
- file-operations, external-modification, find-and-replace, global-search,
  file-tree-panel.

Editing features:
- edit-operations, undo-redo-transactions, caret-and-selection,
  clipboard-operations, navigation-commands, viewport-and-scrolling,
  display-line-mapping, line-commands, auto-indentation, sequence-numbers,
  whitespace-and-guides, encoding-and-characters, tabs-and-mask,
  line-wrap-toggle, text-decorations, exclude-show-filter, hex-display,
  custom-file-viewers, large-file-performance.

Editor chrome:
- multi-tab-editor, layout-and-docking, menu-and-statusbar, view-zoom,
  theme-and-appearance, notification-system, context-help, accessibility.

Orphans/duplication to RESOLVE in this bucket (owner-gated decisions):
- ff-file-tree orphan (PA-CONFLICT-011) -- shell reimplements file-explorer
  inline (files_panel 1195 / file_explorer 935) -> rewire onto ff-file-tree or
  delete + reconcile; ties the shell file-explorer split (PA-W4.3).
- ff-large-file-performance orphan (PA-CONFLICT-013) -- wire the render path onto
  it (60fps>1M-line promise) or reconcile.
- ff-external-mod orphan (PA-CONFLICT-014) -- resolve placement (keep standalone)
  + wire the detector into the shell/document lifecycle + notifications.
- ff-viewers vs ff-asa/ff-hex (PA-CONFLICT-019) + ASA-x3/EBCDIC-x2/hex-x2
  duplication (PA-CONFLICT-022): DECIDE the viewer architecture + consolidate.
  NOTE: ASA + FileForge flat-file rendering are mainframe-adjacent; the ASA
  consolidation decision may land partly in Bucket 4 -- but hex + the generic
  viewer framework are editor-core, so the DECISION is made here.
- PREVIEW command double-owner (ff-viewers + ff-asa) -> single owner.

Data-safety (DONE): PA-W5.2 atomic writes for file-operations/global-search.

Hygiene within bucket: relevant PA-STD cap splits + ASCII + PA-TCR enumeration +
PA-DOC crate-name-drift for the crates above (from PA-W0..W4 + PA-W5.7/8/10/11/12).

Underlying PA-W phases: PA-W0 (foundation), PA-W1 (editor core), PA-W4 (UI/panels/
layout), plus the file/editor slices of PA-W5.

---

## Bucket 2 -- MACRO FEATURES (next; CORE; unlocks whole-product automated testing)

Goal: working macros + the FFTest automated-testing harness, so the rest of the
product can be regression-tested automatically (owner's key rationale).

Sub-projects:
- lua-macro-engine (`ff-lua`) -- security-gate ENFORCED already (positive);
  complete the Macro Library panel (Req 12) + FFCMD-sequence chaining.
- command-framework, command-semantics, menu-workspace, function-keys-and-history
  -- the CR-NR-057 COMMAND-CHAINING bundle.
- command-palette, command-completion, command-configurator, shell-command
  (command/execution surface the macros drive).
- automated-dialog-testing (`ff-fftest`) -- egui-decoupled AutomationId model,
  62 TCR (strongest coverage after JES); the harness that delivers whole-product
  automated testing. batch-execution (headless runner over the same pipeline).

Key work to RESOLVE:
- CR-NR-057 Phase DH: unify command chaining across command-framework Req 10,
  command-semantics Req 11 (split_chain/execute_chain), menu-workspace Req 5,
  function-keys Req 17, and lua-macro FFCMD sequences -- ONE shared chain
  executor (PA-WATCH-017), do NOT build separate executors.
  (PA-INCOMPLETE-003/007/008/009/016.)
- lua-macro PA-INCOMPLETE-016 (Macro Library panel + FFCMD chaining).
- CR-NR-058 logging on the macro security-gate decisions (PA-LOG-046, MED-HIGH,
  audit-relevant) + batch/shell already log (positive exemplars).

Underlying PA-W phases: PA-W3 (shell/commands/menus/session) + the macro/command
slices of PA-W5 (PA-W5.6 lua panel + FFCMD, PA-W5.9 logging).

---

## Bucket 3 -- CODE-EDITING SYNTAX HIGHLIGHTING

Goal: syntax highlighting wired onto its intended foundation crates.

Sub-projects:
- syntax-highlighting, language-service.

Key work to RESOLVE (the core of this bucket):
- ff-syntax-highlighting IGNORES two foundation crates it was designed to
  consume, reimplementing them inline:
  - language detection -> ff-language-service (PA-CONFLICT-018).
  - idle-styling budget -> ff-idle-processing (PA-CONFLICT-012).
  Rewire syntax-highlighting onto BOTH (one coherent effort, same consumer) OR
  delete/absorb the orphans + reconcile the specs. Owner-gated.
- Standardise the language-definition TOML load path (currently raw fs in
  ff-language-service) when wired.

Underlying PA-W phases: PA-W1/PA-W2 highlighting slices + PA-W5.1 (language-service
consolidation).

---

## Bucket 4 -- JES FEATURES (PLUGIN)

Goal: the mainframe job-entry + dataset emulation, delivered as a plugin.

Sub-projects:
- jes-emulator (`ff-jes`, already a wired FileForgePlugin, 74 TCR -- exemplary),
  idcams-emulator (`ff-idcams`), jcl-resolver (stub), dataset-catalog,
  dataset-allocator, virtual-catalog-manager, dataset-ownership-model,
  record-selection-criteria, structure-catalog, asa-report-preview (ASA is
  mainframe-report rendering), fileforge-integration (flat-file EBCDIC/COMP-3).

Key work to RESOLVE:
- idcams orphan + MISSING dual integration (PA-CONFLICT-016): register with the
  command framework + wire the JES EXEC PGM=IDCAMS batch-step (ff-jes has 0
  idcams refs) so IDCAMS runs inside JES jobs.
- JES queue data-safety: atomicity DONE (PA-W5.2); VFS-routing of the queue store
  deferred (PA-CONFLICT-015 follow-up).
- SEVERE cap: idcams parser/mod.rs 1372 (worst in project) -> split (PA-STD-057).
- jcl-resolver fragmentation (PA-CONFLICT-017): JCL split across ff-dsalloc +
  ff-jes/ffjcl; decide ownership + fix the readiness-summary provenance
  (PA-TRACK-005).
- ASA / EBCDIC consolidation from Bucket 1's decision lands here for the
  mainframe-side implementations (ff-forge asa.rs/ebcdic.rs -> reuse ff-asa/
  ff-encoding), PA-CONFLICT-022.
- CR-NR-058 logging on the JES job engine (PA-LOG-041) + idcams (PA-LOG-042).

Underlying PA-W phases: PA-W2 (catalog/dataset) + the JES/IDCAMS/JCL slices of
PA-W5 (PA-W5.4, PA-W5.7).

---

## Bucket 5 -- DATABASE FEATURES (PLUGIN)

Goal: the integrated Database IDE, delivered as a plugin.

Sub-projects:
- database-tool (`ff-database-tool`, a registered plugin but currently a
  foundation SKELETON).

Key work to RESOLVE:
- PA-INCOMPLETE-014 (do the tracking re-open UP-FRONT): 157/157 tasks `[x]` but
  ~14/17 reqs unbuilt (no panels/pooling/async/ER/transfer/admin; deps only
  thiserror+serde). Re-open the unbuilt tasks, keep the foundation done, correct
  the "complete" framing. THEN an owner-prioritised implementation effort to
  actually build the IDE (add sqlx/rusqlite, tokio, egui, ff-vfs/ff-command/
  ff-layout; wire the 7 panels + pooling + async).
- DBeaver-CE licence review if any code/assets are ported (feature parity is fine).
- CR-NR-058 logging on connection/pool/query lifecycle when built (PA-LOG-043).

Underlying PA-W phases: PA-W5.5 (tracking re-open) + a new build-out phase (to be
specified via the requirements gate when the owner greenlights the implementation).

---

## Sequencing summary

1. Cross-cutting up-front: PA-W5.5 (database-tool re-open), PA-TRACK-007/008
   (master reconciliation). [PA-W5.2 data-safety already done.]
2. Bucket 1 -- file + editing core (foundation -> orphan rewiring -> features ->
   hygiene).
3. Bucket 2 -- macros + FFTest (unlocks automated testing) + CR-NR-057 Phase DH.
4. Bucket 3 -- syntax highlighting (rewire onto language-service + idle-processing).
5. Bucket 4 -- JES plugin (idcams integration, cap splits, JCL ownership, ASA/
   EBCDIC consolidation, job-engine logging).
6. Bucket 5 -- Database plugin (re-open tracking, then build the IDE).

Owner-gated decisions still required before executing the structural items:
orphan rewire-vs-delete (each crate), duplication single-owner choice
(ASA/EBCDIC/hex/PREVIEW/language-detection), and the Database IDE build greenlight.
Every code change follows the requirements gate + TDD + verify.ps1 discipline used
for PA-W5.2.
