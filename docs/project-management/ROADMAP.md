# FileForgeWorkbench -- Product Roadmap (Control Tower)

This is the ONE document to read to answer "what is the product, what is core,
what is a plugin, where are we, and what is being worked on right now." It sits
ABOVE the detailed tracking layers (project-analysis buckets, the PA-W phases,
and the per-sub-project task lists) and says which of them is active.

- Detailed remediation ordering: `project-analysis/remediation-plan.md`
- Cross-cutting phase tracking: `project-master/tasks.md` (PA-W0..PA-W5 phases)
- Per-feature specs: `docs/specs/<sub-project>/` (requirements / design / tasks)
- Core sign-off checklist: `core-acceptance-test-plan.md` (this folder)
- Change log (new/changed requirements): `docs/status/change-log.md`
- Bug register: `docs/status/bugs.md`

---

## 1. Product Vision

FileForgeWorkbench (FFWB) is an ISPF-style desktop workbench: a tabbed,
detachable-workspace shell driven as far as possible by a command line, in which
every action has a command equivalent so activities can be scripted as macros.
The core is a general file + text-editing environment; specialised capabilities
(mainframe/POSIX emulation, file formatting, database, job monitoring) are
delivered as PLUGINS layered on that core.

Design principles that must survive every phase:
- ISPF familiarity: Primary Option Menu (POM) style menus, a persistent
  `Command ===>` line, primary + line commands in the editor.
- Command-first: every activity has a command; commands compose into macros.
- Core vs plugin: keep the core small, stable, and thoroughly tested; add
  everything else as plugins, each built and signed off before the next.

---

## 2. CORE vs PLUGIN (the top-level split)

**CORE** = the workbench that must work and be signed off BEFORE any plugin work
resumes. See Section 3.

**PLUGINS** = everything else, built one at a time, each ending in a thorough
test sign-off before the next begins. See Section 5.

Rationale (owner-directed): the project tried to build too much at once. We
stabilise and test the core first, then add plugins in a disciplined order.

---

## 3. CORE requirements (must work + be tested first)

The six core capabilities. Status reflects the current build; "get it working +
tested" is the active goal (Section 6). Each item lists the sub-project specs it
draws on.

| # | Core capability | Status | Primary specs |
|---|-----------------|--------|---------------|
| 1 | **ISPF-like shell framework** -- open tabbed, DETACHABLE workspaces. | Built; detach + tab lifecycle need test pass | `platform-core`, `layout-and-docking`, `multi-tab-editor`, `menu-and-statusbar`, `startup-and-session`, `workspace-model` |
| 2 | **Menu + configurable workspaces** -- two workspace kinds: MENU workspaces (all modelled on the POM: option / command / description) and configurable workspaces. Plus a **Create-Menu dialog**: name a menu (e.g. "Settings"), enter a list of option/command/description rows, save as a new menu. | Menu-workspace pattern built (POM, Settings); the create/edit-menu DIALOG is the key gap | `menu-workspace`, `command-semantics` |
| 3 | **File directory navigator** -- select files for View / Edit / delete / create + standard file ops. | Native-file navigation DONE (modern ff-file-tree explorer, CR-NR-060; edit ops incl. catalog subtrees, B042); dataset/volume catalog navigation DEPENDS on the Dataset Stack Rationalisation (see Section 6a) | `file-tree-panel`, `file-operations`, `virtual-file-system`, `connector-local-fs` |
| 4 | **ISPF editor** -- primary commands + line commands, two modes: Browse/View (no mutation; display filters like INCLUDE/EXCLUDE/FIND still work) and Edit (mutation allowed). | Largely built, NOT fully working -- primary/line command execution + mode enforcement need fixing and a test pass. End-to-end editor testing DEPENDS on the Dataset Stack Rationalisation (Section 6a): the navigator must be able to open files from dataset/volume catalogs and SAVE them record-aware before the editor can be tested against real datasets. | `edit-operations`, `line-commands`, `multi-tab-editor`, `document-model`, `undo-redo-transactions`, `caret-and-selection`, `viewport-and-scrolling` |
| 5 | **Command-line execution of everything** -- as far as reasonable, every activity has a command, so any activity can run inside a macro as a list of commands. | Command framework + palette built; coverage/parity + chaining need completion (CR-NR-057) | `command-framework`, `command-semantics`, `command-palette`, `command-completion`, `shell-command`, `function-keys-and-history` |
| 6 | **Macros as commands** -- save a macro to a file, and (in Settings) configure a saved macro as an individual command. | Lua engine + Macro Library partially built; save-macro-as-command wiring is the gap | `lua-macro-engine`, `command-configurator`, `batch-execution` |

CORE is considered DONE only when `core-acceptance-test-plan.md` is fully signed
off.

---

## 4. Where the existing tracking maps onto CORE

The prior "Bucket 1 (file + editing core)" work is essentially CORE. Nothing is
discarded; it is re-framed:

- Bucket 1 (file/editing core) -> CORE items 1, 3, 4 (+ chrome).
- The macro/command work (former Bucket 2 lead, CR-NR-057 command chaining) ->
  CORE items 5, 6.
- Buckets 3-5 (syntax highlighting, JES, Database) -> PLUGIN phases (Section 5).

So the remediation-plan buckets remain the DETAIL layer; this roadmap says the
core slices come first and must be tested before the plugin buckets proceed.

---

## 5. PLUGIN build order (after CORE sign-off)

Each plugin is built, then thoroughly tested and SIGNED OFF before the next
begins. Order (owner-directed):

1. **Utilities** -- container for in-app tools. Candidates (to be confirmed):
   IDCAMS dialog, Lua scripting, Rexx, language processors (GCC, Rust compiler).
   Open question: exact contents of Utilities.
2. **Mainframe file / dataset emulation** -- virtual dataset catalog + storage.
   Informed by the new Storage & Catalog Data Model (Section 7a).
3. **POSIX file emulation** -- POSIX-style virtual filesystem catalogs.
4. **File Formatter** -- File-AID-inspired record-structure View/Edit
   (Section 7b).
5. **Database** -- integrated Database IDE, delivered as a plugin (Section 7c).
6. **Job Monitor** -- JES / SDSF emulator, delivered as a plugin.

All new plugin capability is framed as a FileForgePlugin on the core product
(`plugin-architecture`, `plugin-manager-ui`, `connector-extensibility`).

---

## 6. Current status + the SINGLE active phase

**Active phase: CORE STABILISATION + TEST.**
Get the six core capabilities working and sign off `core-acceptance-test-plan.md`
before resuming plugin work. The concrete, re-sequenced forward plan (established
by the 2026-10-11 status review) lives in Section 6a: localisation string
extraction first, then the Volume UI + catalog/navigator rework cluster, with the
editor windowed-record finish (F3-F5) running in parallel, converging on the
item-4 ISPF-editor test pass. Item 2 (Create-Menu dialog) and items 5/6 (command
parity + macros-as-commands) follow once the editor-against-datasets path is
clear.

Recently completed (context):
- File navigator modernised onto `ff-file-tree` (CR-NR-060 Slice A); it is now
  the sole File Explorer with full edit ops.
- Bug fixes from testing: B040 (preview crash), B041 (catalog repository init),
  B042 (catalog-subtree edit ops).
- Cross-cutting doc corrections: PA-W5.5 (database-tool tracking re-open),
  PA-TRACK-007/008 (narrative reconciliation), and this docs reorganisation.

NOT active right now (deferred until after CORE sign-off): the plugin buckets
and the new-requirement gates in Section 7.

---

## 6a. CORE forward path -- re-sequenced after the 2026-10-11 status review

This section supersedes the earlier "dataset prerequisite only" framing. A
read-only, git-verified status review (`.agents/tasks/core-resequencing-review/
report.md`, 2026-10-11) established the real state of every stream. The headline:
much of what looked like forward work is ALREADY DONE and merged into `main`.

### What is DONE and merged (struck from the forward path)

- **Core-framework decomposition** (DECOMP Waves 3-7): the ff-desktop god-crate
  was split into `ff-files-panel`, `ff-editor-panel`, `ff-catalog-dialog`,
  `ff-nav-model`, `ff-posix-provider`, `ff-explorer-view`, `ff-context-menu`, and
  more. Merged. (Residual: CR-NR-078 WF.6 Context-migration tidy -- low priority.)
- **SCRM** (CR-NR-098 Screen Snapshot + Collection/Replay): crates
  `ff-screen-model` + `ff-scrm` built and wired; SCRM.0-SCRM.6 done. Merged.
  (Residual: SCRM.7 TCR bookkeeping only.)
- **Localisation MECHANISM** (CR-NR-103 Phase 1): the `ff-i18n` crate, the
  `ui.locale` config key, and the alias-overlay loader. Merged, owner-gate clean.
- **Dataset rationalisation ENGINE** (CR-CH-059 RC.A + RC.B, CR-CH-060, CR-CH-061):
  single catalog authority, the `DatasetAccess` contract, the `ff-volume` crate,
  record-aware MAINFRAME SAVE, retirement of the duplicate trait crates. DONE and
  full-gate CLEAN from `main` on 2026-10-11 (9866 run / 9866 passed / 0 failed).
- **Windowed-record editor F1 + F2** (CR-CH-058 WRF.2/WRF.3): piece-table spine,
  immutable original index, append buffer, byte-identical native SAVE, windowed
  byte residency. Merged.

All non-main branches are git-confirmed 0 commits ahead of `main` -- there is NO
unmerged work. `main` is the single source of truth.

### The remaining CORE forward path (the active plan)

Ordered with dependencies. The editor finish (F3-F5) and the dataset/volume UI
cluster are INDEPENDENT and run in PARALLEL; localisation extraction is pulled to
the FRONT by owner decision (2026-10-11) because it is a large, cross-cutting,
mechanical change -- doing it first means every panel built afterward is written
localisation-aware and avoids re-touching new UI and merge collisions.

0. **Housekeeping** (non-blocking): prune the stale merged branches + the
   `.worktrees/wrf-foundation` worktree; close SCRM.7 and WF.6 tidy later.
1. **Localisation Phase 2 -- full string extraction + Settings locale picker**
   (CR-NR-103 L10N.4): extract all user-facing strings across `ff-desktop` and the
   panel crates to `ff-i18n` keys (English base), add the locale picker to the
   Settings "Reset Bare" menu. Large footprint -- run in its OWN worktree, FIRST,
   while the UI panels are still quiet. Phase 3 (other-language DATA, L10N.5) stays
   the later capstone -- NOT in scope now.
   - **CR-NR-109 (locale-aware alias resolution + menu-kind localised options)**
     PENDING GATE (docs authored 2026-10-23; gate approval not yet given). FRAMEWORK
     CHANGE (owner-confirmed direction): grows the `AliasTable` entry with a
     localisation key + auto-synthesised editable identity rows, builds the on-disk
     alias-catalogue reader (unblocks L10N Task 10.2), adds the unified
     command-line/menu resolver, and renders menu-kind option VERBS via the active
     locale (ALL menu kinds). Sequence AFTER this step's English-key extraction is
     substantially done (there must be English keys to alias against); it is the
     bridge from "descriptions localise" to "verbs display localised too". See
     `docs/specs/localization/` Req 11 and change-log CR-NR-109.
   - **CR-NR-111 (menu Kinds localise FULLY -- mnemonic + command alias +
     description)** GATE AUTHORED (requirements + design + tasks written
     2026-10-xx; owner approval pending at the gate CONFIRM step). The DELTA on top
     of CR-NR-109: localises the remaining raw part (the MNEMONIC) via a new
     catalogue key mirroring the description seam, settles the saved-command shape
     as the ROOT (Locale_Independent_Stored_Value, Option A -- no model change / no
     migration), and makes FIRST-PRIZE auto-conversion, SECOND-PRIZE per-language
     saved menus, and per-part fallback explicit (the fix for "not consistently
     Afrikaans"). NOT a framework change (the `rewrite_verb_to_root` ->
     `resolve_target` pre-pass already delivers routing invariance on both seams);
     NO new dispatcher / nav stack / persistence format / Kinds-registry field.
     Sequence AFTER CR-NR-109 (it builds on the locale-aware alias layer + resolver).
     Governing plan: `docs/specs/localization/tasks.md` Phase 8 (Tasks 16-22). See
     `docs/specs/localization/` Req 12 and change-log CR-NR-111.
   - **CR-NR-112 (parameterised command aliases -- verb -> ROOT command WITH a
     parameter template; `&1..&N` / `&*` positional substitution)** GATE AUTHORING
     (requirements + design + tasks written; owner approval pending). Extends the
     SAME `AliasTable` as CR-NR-109/111 with an additive `command_template` field:
     `ecob helloW` expands to `EDIT FFWB.COBOL.MASTER(helloW)` in the EXISTING
     `rewrite_verb_to_root` pre-pass (before `resolve_target`), then routes through
     the one front door unchanged. Pure text substitution (empty-sub for missing
     `&N`, surplus args appended when no `&*`), SINGLE-PASS in v1 (depth-limit +
     visited-set guards documented as the forward-compatible chaining path),
     templateless entries byte-identical. A general user-facing shorthand, NOT just
     localisation. Menu-option argument supply OUT OF SCOPE (no field merge on
     click). NOT a framework change. Coexists with CR-NR-111 (template on the alias
     entry, not the menu option; no double-expansion). Can run in PARALLEL with the
     CR-NR-111 gate; both touch the alias layer, so coordinate the `AliasEntry` /
     `AliasRowToml` edits. Governing plan: `docs/specs/command-framework/tasks.md`
     + `docs/specs/command-semantics/tasks.md` Phase (parameterised-aliases). See
     command-framework Req 19, command-semantics Req 14, and change-log CR-NR-112.
   - **CR-NR-113 (Alias Editor Workspace Context -- create/edit/SAVE aliases)**
     GATE AUTHORING (requirements + design + tasks written; owner approval
     pending). The AUTHORING + PERSISTENCE surface for CR-NR-112: a new Custom
     Workspace Context (configurator family, `WorkspaceKind::CommandConfigurator`
     with `editor = "aliases"`, modelled on the Menus Editor) that lists, adds,
     edits, deletes, and SAVES command aliases into a NEW user-scoped,
     locale-independent alias store (`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`),
     SEPARATE from the per-locale `i18n/<locale>/aliases/` catalogues. New pieces:
     an alias serialiser (none exists -- the alias layer is read-only today) and a
     user-store load path on `LocaleAliasStore` (a THIRD overlay, per-row
     `environment` routing to ffcmd/ffedit) with a built-in-root shadowing guard
     (a NEW CR-NR-113 pre-pass invariant grounded on `builtin_ffcmd_roots()`, NOT
     inherited from command-framework Req 8.10) + a live in-place refresh via the
     EXTENDED reload seam (one re-seed re-applies BOTH the per-locale overlay AND
     the user-store overlay). Full sibling-parity wiring like the Menus Editor (a
     new runtime `TabKind`/`KindTag`, BuiltinKind mapping, non-persist arms, ctor,
     render_body arm, panel field -- "no new WorkspaceKind" does NOT mean "no new
     TabKind"). v1 edits the USER store ONLY (no CR-NR-111 overlap). NOT a
     framework change; the one new mechanism is the user store + its resolver load
     path (three owner-confirm points parked for CONFIRM: store path/shape incl.
     the environment field; precedence + no-shadow rule; v1 scope). HARD
     build-order dependency on CR-NR-112 (it authors CR-NR-112's 4-field record):
     the template parts (Req 5.1/5.3/5.8) are gated until CR-NR-112 lands.
     Governing plan: `docs/specs/command-configurator/tasks.md` Task 7. See
     command-configurator Req 5 and change-log CR-NR-113.
   - **CR-CH-066 (Alias Editor added to the Settings menu; Config mnemonic A -> C)**
     GATE AUTHORED / OWNER-CONFIRMED (content-only; owner picked the final order).
     Adds a NEW `A` -> Alias Editor option to the code-only built-in Settings menu
     (`DEFAULT_SETTINGS_TOML` Recovery_Baseline) in SECOND position and moves
     Config's mnemonic from `A` to `C`. Final order: `C` Config, `A` Alias Editor,
     `T` Theme, `M` Menus, `K` Keys, `W` Kinds, `R` Reset bare. The `A` option
     REUSES the CR-NR-113 `ALIASES` open path verbatim (single front door:
     `resolve_target` -> `builtin_workspace_target_for` -> `open_alias_editor` ->
     `navigate_to`); no new verb/dispatcher/Context. CONTENT-only change to a
     never-written-to-disk built-in (CR-CH-021); the `A` -> `C` remap is a
     DELIBERATE change (no existing users), not a regression. NOT a framework
     change. DEPENDS on CR-NR-113 for the `ALIASES` verb -- sequence AFTER (or with)
     it. Governing plan: `docs/specs/menu-workspace/tasks.md` Phase
     (settings-alias-option) Task 50. See menu-workspace Req 12.3 (amended) + Req 22
     and change-log CR-CH-066.
   - **CR-CH-067 (EnvironmentRegistry root-command enumeration + reject-with-WARNING
     -at-load for invalid user alias roots -- Gate 1 foundation for the Alias Editor
     redesign)** GATE AUTHORED / OWNER-CONFIRMED (additive framework change to
     CR-CH-053; owner confirmed "Proceed..." + chose option 2 "Reject with warning,
     continue to load"). Adds a READ-ONLY name-keyed query on the built
     `EnvironmentRegistry` (Req 13): `root_commands(env)` returns an environment's
     complete CANONICAL root set (FFCMD via `builtin_ffcmd_roots`, FFEDIT via
     `canonical_verbs_owned` over `ffedit_english`), deterministic order, canonical
     roots only (no alias surfaces); `contains_root(env, root)` is case-insensitive
     and graceful for an unknown env/root. PLUS a data-integrity rule: a user alias
     row (CR-NR-113 `user.toml`) whose root is not valid in its declared
     environment is SKIPPED at load with a WARN and loading CONTINUES (mirrors the
     existing no-shadow/collision skip-with-WARN in `apply_user_rows`; startup never
     blocked, no valid row dropped); the same check is surfaced in
     `validate_user_alias_rows` for Gate 2's save-side rejection. ADDITIVE / read-only
     + one load-time skip branch: no second dispatcher, no second navigation stack,
     no new persistence format, no reshape of a public framework type. This is the
     FOUNDATION for Gate 2 (the Alias Editor redesign -- env->root dropdown cascade,
     ISPF I/D line commands, filter, validated (environment, root) anchor), which is
     NOT in scope. Governing plan: `docs/specs/command-environments/tasks.md` Phase
     (cmdenv-root-enum) Tasks 27-29. See command-environments Req 20, change-log
     CR-CH-067, command-configurator Req 5 (CR-NR-113, the future consumer), and
     `.agents/tasks/command-env-enumeration/findings.md`.
   - **CR-CH-069 (correct editor command-environment placement: AUTONUM/NUM ->
     FFEDIT; af SOEK->FIND -> FFEDIT catalogue; EDIT doc-comment)** GATE AUTHORED /
     PENDING OWNER APPROVAL. The ONE gated change the command-environment placement
     audit recommends (`.agents/tasks/command-placement-audit/command-environment-audit.md`,
     sections 5 and 8). THREE corrections: (A) MOVE AUTONUM and NUM from the FFCMD
     ladder to FFEDIT -- they are in-buffer line-renumbering verbs (ownership
     principle Req 2a.4 / Req 10), claimed only when an editor Context is active
     (Req 5.1); a PURE CLASSIFICATION MOVE since their target verb NUMBER is NOT
     wired on main (the NUMBER/UNNUM sequence-number ENGINE is the deferred
     `ff-sequence-numbers` effort, NOT built here); the only observable change is
     WHERE they are claimed. (B) RELOCATE the mis-placed af SOEK->FIND alias from
     `i18n/af/aliases/FFCMD.toml` to `i18n/af/aliases/FFEDIT.toml` (FIND is an
     FFEDIT root; command-semantics Req 12.1). (C) FIX the stale EDIT doc-comment in
     `user_aliases.rs` (EDIT is an FFCMD file-open root; ECOB->EDIT is an FFCMD
     alias). Interacts with CR-CH-067: the corrected (environment, root) sets feed
     `root_commands`/`contains_root`, so an FFCMD-anchored FIND/AUTONUM/NUM alias is
     now correctly rejected (intended, not a regression). Lands BEFORE the Afrikaans
     alias-data authoring and the `=FILES` root-token fix (audit item D, a separate
     change request) so the root sets are corrected once. NOT a framework change
     (classification/DATA/doc only; no second dispatcher, no second navigation
     stack, no new persistence format). Governing plan:
     `docs/specs/command-environments/tasks.md` Phase (command-placement-audit)
     Task 30. See command-environments Req 21, change-log CR-CH-069,
     command-semantics Req 12, CR-CH-067 (the validation safety net), and the
     deferred `docs/specs/sequence-numbers/` NUMBER engine.
2. **Volume UI** (CR-CH-057/CR-NR-105 VM.6-VM.8 / RC.C.10): Volume management
   WorkspaceContext (VTOC/report), Define_Volume_Dialog, DEFINE VOLUME / vary /
   ALTER actions via the single command path, volume picker in catalog creation,
   SPACE-against-volume allocation, DEFINE VOLUME + VOLUMES() in IDCAMS. Unblocked
   (`ff-volume` + `DatasetAccess` exist). Written localisation-aware (after step 1).
3. **Volume in the "Reset Bare" POM** (menu-workspace / CR-NR-090): a POM option
   routing to the Volume Context via the single command path. After step 2.
4. **Catalog UI rework** (CR-CH-057 catalog-as-pure-locator): the catalog UI
   reflects catalog-as-locator + volume indirection. After step 2.
5. **ff-vfs Volume/Catalog UI-path wiring** (CR-CH-059 RC.A.3/RC.A.4 seam already
   DONE): wire the navigator/catalog UI THROUGH the existing single
   `ff-vfs::StorageProvider` seam for volumes. Integration, not re-architecture.
   After steps 2+4.
6. **File Navigator dataset/volume duality** (CR-NR-060 Slice B): dataset/volume
   navigation + PDS members (native-file nav / Slice A is already DONE). After
   steps 2+4+5.
7. **Editor windowed-record F3-F5** (CR-CH-058 WRF.4-WRF.6): background index +
   instant-open UX (F3), windowed FIND/CHANGE + destructive-scale guard (F4),
   scalability guard (F5). Depends only on F1/F2 (DONE) -- runs in PARALLEL with
   steps 2-6.
8. **CORE #4 editor test pass against real datasets** -- the convergence /
   sign-off milestone. Needs step 6 (navigator opens dataset/volume files) AND
   step 7 (editor finished), per `core-acceptance-test-plan.md`.

**This path must complete and be gate-clean before the editor (CORE #4) is signed
off.** The JES/JCL executor (RC.C.9) remains DEFERRED to PLUGIN phase 6, after the
Volume UI, and is NOT part of this core path.

---

## 7. New / changed requirements (recorded, gated later)

These are learnings captured as source documents. They are LOGGED in
`docs/status/change-log.md` as PENDING GATE and will each go through the
requirements gate when their phase arrives -- NOT folded into CORE.

### 7a. Storage & Catalog Data Model (modifies existing catalog/dataset/VFS reqs)
- Source: `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md`
  (+ `FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`).
- Effect: the current virtual mainframe/POSIX catalog is inadequate; this
  reworks/extends `virtual-file-system`, `dataset-catalog`, `virtual-catalog-manager`,
  `dataset-allocator`, `dataset-ownership-model`. Feeds PLUGIN phases 2 and 3.
- NOTE: the consolidation half of this (CR-CH-059 Dataset Stack Rationalisation,
  plus CR-CH-057/CR-NR-105 Volume model and CR-CH-060) is NOT deferred -- it is a
  CORE PREREQUISITE for editor testing. See Section 6a.

### 7b. File Formatter plugin (new sub-project)
- Source: `docs/specs/file-formatter/design-input.md` (+
  `docs/source-documents/FileForgeWorkbench_FileAID_EARS_Requirements.md`).
- Effect: a File-AID-inspired record-structure View/Edit plugin (PLUGIN phase 4),
  bundling relevant existing specs: `record-selection-criteria`,
  `structure-catalog`, `fileforge-integration`. Change-log: CR-NR-061.

### 7c. Database as a plugin (reconfiguration)
- Source: `docs/source-documents/FileForgeWorkbench Database Plugin Specification.md`.
- Effect: reconfigure `database-tool` as a proper FileForgePlugin (PLUGIN
  phase 5). Note: database-tool tracking was re-opened (PA-W5.5) -- it is a
  foundation skeleton, not built.

### 7d. JES as a plugin (reconfiguration)
- Effect: formalise `jes-emulator` (+ `idcams-emulator`) as the Job Monitor
  plugin (PLUGIN phase 6).

---

## 8. How to use this document

- Start here. Section 6 tells you the ONE active phase.
- For the detailed ordering of remediation work, read
  `project-analysis/remediation-plan.md`.
- For per-feature detail, read the sub-project spec under `docs/specs/`.
- When a new/changed requirement is approved, it runs the requirements gate
  (`.kiro/steering/workflow.md`) and its phase is added to
  `project-master/tasks.md`; update Section 6/7 here to point at it.
