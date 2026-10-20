# FFWB Core Status and Re-Sequencing Review

Read-only investigation. No code, branch, worktree, or commit changes were made.
Workspace root: c:\workspace\VSC\FileForgeWorkbench. All findings are git- or
doc-verified; where something could not be verified it is called out explicitly.

## Summary answer (read this first)

The single most important finding is that **the owner's proposed forward path is
mostly already DONE**. Three of the items the owner listed as forward work --
Core-framework decomposition, SCRM, and localization Phase 1 (mechanism) -- are
ALREADY BUILT and ALREADY MERGED INTO main. They drop off the forward path.

- **Every branch in the repo is fully merged into main.** `main` @ 526d9a6 is the
  single source of truth. `decomp-and-scrm-wave` (local e6a88b0 / origin 58cac12),
  `feature/windowed-record-foundation` (6471b3a), and all six `feature/*` branches
  (db9/db10/db-menu/environment-registry/vsam-wiring) report 0 commits ahead of
  main and appear under `git branch --merged main`. There is NO unmerged work
  anywhere. All of these branches are STALE and safe to delete.
- **CR-NR-098 SCRM is BUILT, not pending.** The brief's open question ("is SCRM
  unbuilt?") is resolved: SCRM.0-SCRM.6 are all `[x]` done, crates `ff-screen-model`
  and `ff-scrm` exist and are wired into ff-desktop. Only SCRM.7 (TCR-flip
  bookkeeping) is marked in-progress. SCRM drops off the forward path.
- **The ff-desktop decomposition (DECOMP Waves 3-7) is BUILT and merged.** The
  shell was split into ff-files-panel, ff-editor-panel, ff-catalog-dialog,
  ff-nav-model, ff-posix-provider, ff-explorer-view, ff-context-menu, and more.
  Decomp drops off the forward path (WF.6 residual Context migration remains).
- **CR-NR-103 localization Phase 1 (mechanism + English base) is BUILT and merged**
  (`ff-i18n` crate exists and is wired). Phases 2 (string extraction) and 3
  (other-language data) are NOT built. So "Localisation UI" is PARTLY DONE: the
  mechanism exists; the UI-string extraction and the Settings-menu locale picker
  do not.
- **CR-CH-058 windowed-record F1 AND F2 are BOTH merged into main** (tip 6471b3a
  is an ancestor of main; `record_format.rs`, `original_index.rs`,
  `document_records.rs` are on main). The worktree is NOT ahead -- it is stale.
  F3/F4/F5 (WRF.4-WRF.6) remain unbuilt. So the editor foundation rework is
  PARTLY DONE.
- **Dataset rationalisation RC.A + RC.B are code-complete and the full gate ran
  CLEAN** from main on 2026-10-11 (9866/9866). CR-CH-060 and CR-CH-061 are DONE.
  The remaining dataset work is the **Volume UI (RC.C.10 / VM.6-VM.8)**, which is
  NOT built, and the JES/JCL executor (RC.C.9), deferred after it.

Net: the genuinely-not-built forward work is **Volume UI, the Catalog/File-Navigator
rework for volumes, localization extraction + locale picker, and the windowed-record
F3-F5 editor finish** -- in that broad order. Details and a recommended sequence
are in Section 5.

---

## 1. Branch / stream reality (git-verified)

Method: `git branch -a`, `git rev-list --left-right --count main...<branch>`,
`git merge-base`, `git branch --merged/--no-merged main`, `git branch --contains`.
Evidence captured under tools/logs/branch-audit*.txt and decomp-wrf-content.txt.

| Branch | Tip SHA | Behind / ahead of main | Merge-base | Verdict |
|--------|---------|------------------------|------------|---------|
| `main` | 526d9a6 | -- | -- | HEAD; in sync with origin/main |
| `decomp-and-scrm-wave` (local) | e6a88b0 | 32 / 0 | e6a88b0 | MERGED (fully behind; stale) |
| `origin/decomp-and-scrm-wave` | 58cac12 | 40 / 0 | 58cac12 | MERGED (fully behind; stale) |
| `feature/windowed-record-foundation` | 6471b3a | 6 / 0 | 6471b3a | MERGED (ancestor of main; stale) |
| `feature/environment-registry` | cdb0389 | 21 / 0 | -- | MERGED (stale) |
| `feature/vsam-service-wiring` | bcad883 | 31 / 0 | -- | MERGED (stale; was the `V` throwaway stream) |
| `feature/db-menu-cmd-external-adapter` | 61745f8 | 360 / 0 | -- | MERGED (stale) |
| `feature/db10-db4-command-target-binding` | 8b6f606 | 364 / 0 | -- | MERGED (stale) |
| `feature/db9-command-configurator-ui` | a1dfe91 | 363 / 0 | -- | MERGED (stale) |

Key points:
- `git branch --no-merged main` is EMPTY (both local and remote). Nothing is ahead
  of main. (tools/logs/branch-audit3.txt.)
- The "ahead" count is 0 for every branch; the "behind" count is just how far main
  has moved since each branch's tip.
- `decomp-and-scrm-wave`'s content IS on main: DECOMP Waves 3-7 commits
  (e00cef4, 1043ff7, dd6cfac, 985c36d, da34f05, 6e8b5fe, 1cf535d, 6488d8c, bace0fd,
  a5bae43, 9dd4adb) and the CR-NR-103 commits (0552ff6, 4456543, 32b8cb9, 55ba31d)
  all appear in `git log main` (tools/logs/decomp-wrf-content.txt).
- `feature/windowed-record-foundation` tip 6471b3a is an ancestor of main
  (`git merge-base --is-ancestor 6471b3a main` exit 0; `git branch --contains
  6471b3a` lists main). The worktree holds nothing main does not already have.

CONCLUSION: main is the sole source of truth. The owner's observation (one
remaining worktree `.worktrees/wrf-foundation`) is correct, but that worktree is
NOT ahead of main; it is a fully-merged, stale branch. All nine non-main branches
can be pruned. This is housekeeping, not a merge risk.

---

## 2. CR-CH-058 -- windowed-record editor (piece-table spine)

Backing: change-log.md CR-CH-058 (PENDING GATE line is now stale given the F1/F2
commits on main); project-master Phase (windowed-record-foundation) WRF.1-WRF.6
(tasks.md lines ~3380-3409). No dedicated `docs/specs/windowed-record-foundation/`
folder exists (file_search found none) -- CR-CH-058 is specced across
`document-model` (Req 11-12 + revised Req 4), `viewport-and-scrolling` (Req 15),
`large-file-performance` (Req 10), `display-line-mapping` (Req 11),
`edit-operations` (Req 18), `undo-redo-transactions` (Req 20), `find-and-replace`
(Req 21). This matches WRF.1's enumerated requirement edits.

What is DONE (on main, git-verified in crates/ff-document-model/src):
- **WRF.1 requirements gate** -- authored.
- **WRF.2 (F1)** -- record abstraction + immutable OriginalIndex + piece list +
  append buffer; byte-identical native SAVE = re-baseline; undo inverse-splice
  journal. Evidence on main: `record_format.rs`, `original_index.rs`,
  `document_records.rs`, `text_buffer.rs` + their `_tests.rs`. Source commits
  ed09410 (F1 part 1), f2a978d (F1 part 2a), 907265e (F1 part 2b) are ancestors
  of main.
- **WRF.3 (F2)** -- windowed byte residency (Window_Band + Total_Records
  authority). Source commit 6471b3a is an ancestor of main. (NOTE: a `git grep`
  for the literal token "Window_Band" on main returned nothing, so the F2 slice
  likely uses a different identifier in code; the commit itself is on main, so F2
  is present. This is the one spot I could not pin to an exact symbol name -- the
  commit is verified, the exact in-code identifier is not.)

What REMAINS (NOT built):
- **WRF.4 (F3)** -- background index build + instant-open UX + estimated-then-exact
  scrollbar + "indexing..." indicator.
- **WRF.5 (F4)** -- windowed FIND/CHANGE via a CharacterIndexer trait +
  destructive-scale CHANGE ALL guard (Confirmable_Command) + streaming-transform-
  at-save.
- **WRF.6 (F5)** -- scalability guard (max_resident_records budget + reserved
  sparse/mmap mode).

Where it physically lives: ALL F1/F2 code is on main. The `.worktrees/wrf-foundation`
branch carries nothing extra (Section 1).

What it blocks for CORE #4 editor testing: the owner has declared CR-CH-058 a CORE
requirement that must be completed and tested. F1/F2 give the piece-table spine and
windowed residency (which fixes the scrollbar-collapse bug the owner observed).
F3-F5 are needed for the large-file open UX, windowed FIND/CHANGE, and the
scalability guard. The editor can be exercised on normal files today; full
large-file editor testing wants F3-F5.

---

## 3. decomp-and-scrm-wave -- what it actually is, and its merge state

The branch name bundles THREE distinct bodies of work. All three are MERGED into
main (Section 1). Resolving the owner's uncertainty definitively: **this branch is
NOT still open -- it is fully merged and stale.**

1. **ff-desktop DECOMPOSITION (DECOMP Waves 3-7)** -- the shell god-crate split into
   focused crates. Merged. Evidence: commits e00cef4 "Commit shell decomposition +
   dispatch-unify + file-splits baseline (gate CLEAN 9517)", the DECOMP Wave 5-7
   commits, and the resulting crates all present and wired into ff-desktop:
   `ff-files-panel`, `ff-editor-panel`, `ff-catalog-dialog`, `ff-dataset-alloc-dialog`,
   `ff-exclude-manager`, `ff-explorer-view`, `ff-context-menu`, `ff-nav-model`,
   `ff-posix-provider`, `ff-scroll-amount` (tools/logs/crates.txt;
   crates/ff-desktop/Cargo.toml). Residual: project-master Phase
   (workspace-framework) WF.6 "LATER" -- migrate the remaining Contexts (Plugin
   Manager, Event Log, Macro Library, Search Results, Command Configurator) and the
   FilesPanel/FileEditor special cases, then delete the `match kind` arms. That is
   framework tidy-up, not a blocker.

2. **CR-NR-098 SCRM (Screen Snapshot + Screen Collection/Replay Manager)** -- BUILT
   and merged, delivered as the first extracted vertical of the decomposition.
   Evidence: project-master Phase (scrm-eval) SCRM.0-SCRM.6 all `[x]` done
   (tasks.md ~3066-3072); crates `ff-screen-model` (Screen_Model + ScreenProvider +
   5 text renderers) and `ff-scrm` (collection/replay/export, zip persistence,
   dependency-free selectable-text PDF + lopdf-protected PDF) exist and are wired
   into ff-desktop. SNAPSHOT and CAPTURE commands, the ScrmViewer WorkspaceContext,
   and its mandatory first-Tab test all landed. Spec `docs/specs/screen-snapshot-scrm/`
   exists (file_search confirmed). ONLY SCRM.7 (per-criterion TCR flips +
   before/after line-count measurement) is `[ ]`/in-progress -- bookkeeping, not a
   feature gap. CORRECTION TO THE BRIEF: the brief flagged SCRM as possibly unbuilt;
   it is in fact essentially complete.

3. **CR-NR-103 localization Phase 1 (i18n mechanism)** -- BUILT and merged. Evidence:
   `ff-i18n` crate exists (Fluent + unic-langid catalogue, `t()`/`t_args()` seam,
   English Identity_Base, fallback-with-WARN); the `ui.locale` config key landed in
   ff-config; a per-environment/per-locale alias loader overlays the AliasTable.
   Commits 0552ff6/4456543/32b8cb9/55ba31d on main; change-log CR-NR-103 status
   "PHASE 1 COMPLETE, owner-gate CLEAN (9552)". Phase 2 (string extraction, L10N.4)
   and Phase 3 (other-language data, L10N.5) are NOT built.

Is decomp-and-scrm-wave a CORE prerequisite? The DECOMP + SCRM content is already
in CORE (merged), so it is not forward-blocking. The localization MECHANISM is in;
the localization UI (extraction + locale picker) is forward work but is an additive
presentation layer, not a hard editor-testing prerequisite.

---

## 4. Dataset rationalisation (CR-CH-059 and dependents)

Backing: change-log.md CR-CH-059/060/061; project-master Phase
(dataset-stack-rationalisation) RC.A/RC.B/RC.C and Phase (volume-model) VM.1-VM.8;
ROADMAP Section 6a; `.agents/tasks/dataset-vision-fit/report.md` (confirmed present).

- **RC.A (consolidation)** -- DONE. Single catalog authority (`ff-dscatalog`), one
  `Dsorg`/`Recfm`, one `ff-vfs::StorageProvider` seam, one posix registrant. RC.A.1-
  RC.A.4 `[x]`. (project-master ~3446-3449.)
- **RC.B (access contract + volume + record-aware SAVE)** -- DONE (code-complete,
  now gate-confirmed). RC.B.5 landed the `ff-volume` crate + schema v4 dual-read
  migration; RC.B.6 defined `DatasetAccess` + ff-dsalloc returns a handle; RC.B.7
  wired VSAM under `DatasetAccess` and retired `ff-vsam-services`; RC.B.8 repointed
  `ff-idcams` and retired `ff-dataset-catalog`. RC.B.5-RC.B.8 `[x]`
  (~3453-3456).
- **The full gate confirms RC.A + RC.B.** The owner's pasted `cargo gate --build`
  from main on 2026-10-11 was CLEAN: 9866 run / 9866 passed / 0 failed. HEAD commit
  526d9a6 "docs(status): mark CR-CH-059 RC.A/RC.B, CR-CH-060, CR-CH-061 DONE after
  clean full gate". So RC.A/RC.B are DONE, not merely code-complete.
- **CR-CH-060 (record-aware BackendEnvironment store contract, Shape 2)** -- DONE on
  main (commits 0a7da61 BRC.2 / 1812d4d BRC.3 / cb4720e BRC.4). Editor SAVE walk
  selects byte-vs-record by the owning CE's RecordFormat; native/host SAVE stays
  byte-identical. (change-log CR-CH-060, status DONE; gate-clean.)
- **CR-CH-061 (same-session MAINFRAME SAVE store-visibility)** -- DONE on main
  (commit 145ea81, gate-clean). This closes the RC.B.8 "live-catalog-handle"
  follow-up: catalog-backed `DatasetAccess` added in `ff-dscatalog`, shell threads
  the active mainframe catalog repository path so a same-session SAVE of a
  Files-Panel-created dataset now resolves against the same on-disk store. No
  follow-up remains per the change-log entry.
- **`ff-volume` crate** -- EXISTS (crates/ff-volume) and is wired into
  crates/ff-desktop/Cargo.toml (`ff-volume = { path = "../ff-volume" }`). The engine
  (VM.2-VM.5) is `[x]` done.
- **Volume UI (RC.C.10 / VM.6-VM.8)** -- NOT built. VM.6 (Volume management
  WorkspaceContext + Define_Volume_Dialog + volume picker), VM.7 (SPACE-against-
  volume allocation + VOL=SER/UNIT uncataloged in ff-dsalloc), VM.8 (DEFINE VOLUME
  command + VOLUMES() binding in ff-idcams) are all `[ ]` (~3340-3342). The gate is
  AUTHORED (virtual-catalog-manager Req 17-18, dataset-allocator Req 17-18,
  idcams-emulator Req 27 under CR-CH-057); code is pending. This is the next real
  dataset deliverable.
- **JES/JCL executor (RC.C.9)** -- NOT built, deliberately DEFERRED after RC.C.10
  per the owner (project-master ~3463, ~3469; ROADMAP 6a). Contract-only; it is
  ROADMAP PLUGIN phase 6, not core.

---

## 5. The proposed re-sequenced CORE path -- evaluation and recommendation

The owner proposed a 10-step order (a-j). Evaluated item by item against verified
status. The headline: items a, b, and most of c are ALREADY DONE and should be
struck from the forward plan; the real remaining work is e-j plus the editor
F3-F5 finish.

| Owner item | Status | Backing CR / spec | Depends on | Assessment |
|------------|--------|-------------------|------------|------------|
| a. Decomp of Core Framework | **ALREADY DONE** (WF.6 residual tidy only) | DECOMP Waves 3-7; CR-NR-078 WF.6 | -- | Merged into main. Drop from forward path; optionally schedule WF.6 Context-migration cleanup later. |
| b. SCRM work | **ALREADY DONE** (SCRM.7 TCR bookkeeping only) | CR-NR-098; `ff-screen-model`, `ff-scrm` | decomp (done) | Merged. Drop from forward path; close SCRM.7 bookkeeping. |
| c. Localisation UI | **PARTLY DONE** -- mechanism in (`ff-i18n`), UI extraction NOT | CR-NR-103 (L10N.4) | i18n mechanism (done) | Only Phase 2 extraction remains; this is "localisation-aware UI", not a new mechanism. |
| d. Localisation added to "Reset Bare" Settings menu | **NOT BUILT** | CR-NR-103; configuration-system `ui.locale`; menu-workspace | c (extraction) + `ui.locale` key (done) | A locale picker option in the Settings menu + RESET BARE scoping. Small, follows c. |
| e. Volume UI | **NOT BUILT** | CR-CH-057 / CR-NR-105 VM.6; virtual-catalog-manager Req 17-18 | `ff-volume` (done), `DatasetAccess` (done) | The next real deliverable. Unblocked -- engine exists. |
| f. Add Volume to "Reset Bare" POM | **NOT BUILT** | CR-NR-090 Kinds / menu-workspace; POM menu TOML | e (Volume UI exists to navigate to) | A POM option routing to the Volume Context via the single command path. Follows e. |
| g. Rework of Catalog UI | **NOT BUILT** (catalog engine reworked; UI not) | CR-CH-057 catalog-as-locator; virtual-catalog-manager | e (volumes are the physical layer the catalog now locates into) | Catalog UI must reflect catalog-as-pure-locator + volume indirection. |
| h. Rework ff-vfs re Volume + Catalog | **LARGELY DONE at the seam; UI-driven paths NOT** | CR-CH-059 RC.A.3/RC.A.4 (one StorageProvider seam); CR-CH-057 | e, g | The single `ff-vfs::StorageProvider` seam + `DatasetAccess` already landed (RC.A/RC.B). What remains is wiring the navigator/catalog UI THROUGH that seam for volumes -- an integration task, not a re-architecture. |
| i. Rework File Navigator for Volume + Catalog | **NOT BUILT** (native nav done; dataset/volume duality not) | CR-NR-060 Slice B (mainframe qualifier/dataset duality + PDS members, ADR-002 D2); file-tree-panel | e, g, h | Native-file navigation is DONE (CR-NR-060 Slice A). Slice B (dataset/volume navigation) is the gap and depends on e/g/h. |
| j. Rework Editor windowed-record-foundation | **PARTLY DONE** (F1/F2 merged; F3-F5 not) | CR-CH-058 WRF.4-WRF.6 | document-model F1/F2 (done) | Finish F3 (open UX), F4 (windowed FIND/CHANGE + destructive-scale guard), F5 (scalability guard). Then the CORE #4 editor test pass. |

Ordering soundness and dependency conflicts:
- The owner's order is broadly sound, but a/b are already complete and c is half
  complete, so the forward plan is shorter than it looks.
- Dependency chain for the dataset/navigator cluster is: `ff-volume` + `DatasetAccess`
  (DONE) -> Volume UI (e) -> Catalog UI rework (g) -> ff-vfs UI-path wiring (h) ->
  File Navigator dataset/volume duality (i). The owner's e -> g -> h -> i order
  respects this. (Note h is mostly a wiring task since the ff-vfs seam already
  exists; it is not a from-scratch rework.)
- f (Volume in POM) sensibly follows e (the Context it points at must exist first).
- j (editor F3-F5) is INDEPENDENT of the dataset/volume cluster -- it only depends on
  document-model F1/F2 (done). It can proceed in parallel with e-i or after. Placing
  it last (as the owner did) is reasonable because the final CORE #4 editor sign-off
  wants BOTH the editor finished AND the navigator able to open dataset/volume files
  record-aware (ROADMAP 6a). But the F3-F5 code itself is not blocked by e-i.
- Localisation (c/d) is an additive presentation layer and is NOT a hard prerequisite
  for editor testing. It can be sequenced wherever convenient; doing it early (as the
  owner proposes) is fine and keeps hardcoded strings from re-accumulating (the
  CR-NR-103 forward-build directive), but it could equally be deferred without
  blocking the dataset/editor critical path.

Reconciliation with ROADMAP Section 6a (just added): 6a states the Dataset Stack
Rationalisation (CR-CH-059 + CR-CH-057/CR-NR-105 Volume + CR-CH-060) is a CORE
PREREQUISITE for editor testing, with RC.C.10 (Volume UI) before RC.C.9 (JES). This
review AGREES and refines it: the rationalisation ENGINE is done and gate-clean; the
remaining CORE-prerequisite work is the Volume UI + the catalog/navigator UI rework
(items e-i), which is exactly what must land before the CORE #4 editor pass can run
against real datasets.

---

## RECOMMENDED FORWARD SEQUENCE

Annotated with status, backing, and hard dependencies. Items marked ALREADY DONE are
struck from active work and listed only to record that they are complete.

0. (housekeeping, do first, non-blocking) Prune the nine stale merged branches
   (decomp-and-scrm-wave local+origin, feature/windowed-record-foundation, the four
   db*/menu branches, environment-registry, vsam-wiring) and remove the
   `.worktrees/wrf-foundation` worktree. NOT BUILT work: none -- this is cleanup.
   Dependency: none (all are 0-ahead of main, Section 1). Also close SCRM.7 and the
   WF.6 Context-migration tidy as low-priority bookkeeping.

1. **Volume UI (VM.6-VM.8 / RC.C.10)** -- NOT BUILT. Backing: CR-CH-057 / CR-NR-105;
   virtual-catalog-manager Req 17-18, dataset-allocator Req 17-18, idcams-emulator
   Req 27. Depends on: `ff-volume` + `DatasetAccess` (DONE). This is the next real
   deliverable and the first CORE-prerequisite gap.

2. **Add Volume to the "Reset Bare" POM (owner item f)** -- NOT BUILT. Backing:
   menu-workspace / POM menu + CR-NR-090 Kinds. Depends on: step 1 (the Volume
   Context must exist). Small; a POM option through the single command path.

3. **Catalog UI rework (owner item g)** -- NOT BUILT. Backing: CR-CH-057
   catalog-as-pure-locator; virtual-catalog-manager. Depends on: step 1 (volumes are
   the physical layer the catalog locates into).

4. **ff-vfs Volume/Catalog UI-path wiring (owner item h)** -- SEAM ALREADY DONE; UI
   wiring NOT. Backing: CR-CH-059 RC.A.3/RC.A.4 (single StorageProvider seam, done),
   CR-CH-057. Depends on: steps 1, 3. This is integration, not re-architecture.

5. **File Navigator dataset/volume duality (owner item i / CR-NR-060 Slice B)** --
   NOT BUILT. Backing: file-tree-panel; CR-NR-060 Slice B (mainframe qualifier/
   dataset duality + PDS members, ADR-002 D2). Depends on: steps 1, 3, 4. Native-file
   navigation (Slice A) is already done.

6. **Localisation UI extraction + Settings locale picker (owner items c, d)** --
   PARTLY DONE (mechanism in). Backing: CR-NR-103 L10N.4/L10N.5. Depends on: `ff-i18n`
   mechanism (DONE). Additive layer; schedulable in parallel -- not on the dataset/
   editor critical path. Can be moved earlier if the owner prefers to attack
   hardcoded strings sooner.

7. **Editor windowed-record F3-F5 finish (owner item j / WRF.4-WRF.6)** -- PARTLY
   DONE (F1/F2 merged). Backing: CR-CH-058; document-model/viewport/large-file/
   find-and-replace specs. Depends on: document-model F1/F2 (DONE). Independent of
   steps 1-5; can run in parallel.

8. **CORE #4 editor test pass against real datasets** -- the convergence point.
   Depends on: steps 5 (navigator can open dataset/volume files) AND 7 (editor
   finished) per ROADMAP Section 6a. This is the CORE sign-off milestone, not new
   feature work.

ALREADY DONE (removed from the forward path; recorded for completeness):
- a. Decomp of Core Framework -- merged (DECOMP Waves 3-7; WF.6 tidy pending).
- b. SCRM -- merged (CR-NR-098; SCRM.0-SCRM.6; `ff-screen-model` + `ff-scrm`).
- Localisation MECHANISM (Phase 1) -- merged (`ff-i18n`, `ui.locale`); only the UI
  extraction remains (folded into step 6).
- Dataset rationalisation ENGINE (RC.A + RC.B, CR-CH-060, CR-CH-061) -- DONE and
  full-gate CLEAN from main (9866/9866, 2026-10-11).

## Evidence index (files read / commands run)
- git: `branch -a`, `log --oneline`, `worktree list`, `rev-list --left-right --count
  main...<branch>`, `merge-base`, `merge-base --is-ancestor`, `branch --merged/
  --no-merged main`, `branch --contains`, `grep -l` on main (logs under tools/logs/).
- docs/project-management/ROADMAP.md (Sections 1-8, esp. 6a).
- docs/project-management/project-master/tasks.md (Phases: scrm-eval, localization,
  volume-model, windowed-record-foundation, dataset-stack-rationalisation RC.A/B/C,
  workspace-framework WF.6).
- docs/status/change-log.md (CR-CH-058/059/060/061, CR-NR-098/103/105).
- docs/status/bugs.md (only open bugs: B083 flaky-test infra, Low; no open editor/
  navigator/catalog/volume functional bugs found).
- docs/status/stream-prefixes.md (V stream ON HOLD / throwaway).
- crates/ listing + crates/ff-desktop/Cargo.toml (ff-volume, ff-scrm,
  ff-screen-model, ff-i18n, ff-nav-model all exist and are wired).
- file_search: screen-snapshot-scrm, localization specs present; no dedicated
  windowed-record-foundation spec folder (CR-CH-058 lives in document-model et al.).

## Items that could NOT be fully verified
- The exact in-code identifier for the F2 "Window_Band" slice: the source commit
  6471b3a is confirmed on main, but a literal grep for "Window_Band" on main returned
  no match, so the slice is present under a different symbol name than the task text
  uses. F2 presence is verified via the commit; the symbol name is not pinned.
- The precise set of ff-desktop UI paths still bypassing the ff-vfs volume seam
  (step 4 scope) was not exhaustively enumerated -- that requires a code-level audit
  beyond this status review.
