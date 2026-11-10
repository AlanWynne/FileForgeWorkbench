# True-Core Proposal (v2) -- MINIMAL SHELL CORE via Cargo default-members

STATUS: APPROVED by the owner (items A-F). This is the final design contract the
implementation steps consume. See "Approval record" at the end for the exact
owner decisions on the toggles and conditions.

Design-only proposal, REGENERATED for the owner's minimal-shell-core direction.
No source or manifest has been edited in THIS (design) step. ASCII only. All
paths absolute. Milestone: core-001.

Supersedes v1 (which kept the editor + ff-vfs; that is now WRONG). The owner's
new direction: strip core to the workbench SHELL and its intrinsic,
content-independent Contexts only. ALL editors out, the file navigator out,
attempt a FULL VFS park, catalog + volume parked as before.

Inputs consumed:
- closure.md (repo B ff-desktop/ffwb build closure: 54 in-closure, 38 parked).
- gate-current-model.md (repo A cargo-gate model, version 0.2.0, contract).
- Fresh grep verification of the minimal core's residual references (section 1).

Repos / base branches (owner-confirmed):
- Repo A (gate tool):  C:\workspace\tools\cargo-gate         base main, branch scope-default-members
- Repo B (workbench):  C:\workspace\apps\FileForgeWorkbench  base main, branch core-strip-001
- Worktrees under each repo's .worktrees/.

---

## 0. The guiding question and the headline finding

Guiding question for every crate: does it stand up WITHOUT any editor, file
navigation, VFS, catalog, or volume? If no, PARK it.

HEADLINE FINDING (grep-verified -- this is the big change from v1): with the
editor and navigator parked, the FULL VFS PARK IS ACHIEVABLE with NO framework
change, because:

1. The core command crates do NOT touch ff-vfs. Grep for `ff_vfs` in
   `crates/ff-shell`, `crates/ff-command`, `crates/ff-command-semantics` returns
   NOTHING. The command dispatch spine is ff-vfs-free.

2. Every `ff_vfs::` reference inside `ff-desktop` lives in code that is being
   PARKED: the editor SAVE seam (`tab_manager.rs::save_active_tab_via_backend`,
   `RecordImageSource`), the tab record-identity (`tab_state.rs::StoreIdentity`,
   `actions.rs`, `handlers.rs::recfm_kind_from_name`), the backend CE registry
   (`environment_registry.rs` host_fs/mainframe fields, `mainframe_backend.rs`),
   the provider seeding (`construct_provider.rs`), and the navigator render
   (`render_nav.rs`). All of these go when editors + navigator + catalog are
   unwired.

3. THE ONE RESIDUAL DRAG (must be handled, flagged for the owner): `ff-vfs` is
   pulled TRANSITIVELY into core by `ff-document-model` (its Cargo.toml has
   `ff-vfs = { path = "../ff-vfs" }`). And `ff-document-model` is pulled into
   core today by TWO places: the editor (parked) AND `ff-shell`
   (`ff-shell/Cargo.toml` lists `ff-document-model`). BUT grep shows `ff-shell`'s
   SOURCE does NOT use any `ff_document_model` type -- its capture/pipe code
   works purely on `&str` / `Vec<String>` (verified: no `ff_document_model`,
   `DocumentHandle`, or `new_document` reference in `crates/ff-shell/src`). The
   dep is declared (and named in the lib.rs architecture comment) but
   source-unused.

   => To FULLY park ff-vfs, the implementation must ALSO drop the UNUSED
   `ff-document-model` dependency from `ff-shell/Cargo.toml`. With editors parked
   AND that one dead dep removed, NOTHING in the minimal core references or drags
   ff-vfs, so ff-vfs parks fully with NO BackendEnvironment seam reshape and NO
   framework change. This is RECORDED here as the full-VFS-park finding.

   FLAG TO OWNER: dropping ff-shell's unused ff-document-model dep is a tiny,
   safe manifest cleanup, but it IS a change to ff-shell. If the owner would
   rather NOT touch ff-shell, the fallback is to keep a THIN ff-vfs stub in core
   (ff-document-model stays for ff-shell, dragging ff-vfs). Recommended: do the
   clean drop and park ff-vfs fully. Decision requested in section 7.

---

## 1. Per-crate ruling for the previously-CORE editor/editing primitives

The owner asked to RE-RULE every editor/editing primitive and verify by grep
whether any NON-editor core shell code still references it. Results:

| Crate | Referenced by non-editor core? (grep) | Ruling | Why |
|-------|----------------------------------------|--------|-----|
| ff-document-model | Only ff-shell's manifest (SOURCE-UNUSED) + the editor | PARK (after dropping ff-shell's dead dep) | Editor/document machinery; drags ff-vfs. Section 0 item 3. |
| ff-edit-operations | No (only tab_state/editor) | PARK | Edit profile/ops -- editor only. |
| ff-display-line-mapping | No (editor render only) | PARK | Display-list line mapping -- editor surface only. |
| ff-find-and-replace | YES -- ff-global-search depends on it; also find_manager.rs (editor) | KEEP (as a SEARCH dep) | global-search (core) needs it. See section 1.1. |
| ff-navigation-commands | Only nav_manager.rs / editor viewport nav | PARK | Caret/scroll navigation over the editor viewport. No non-editor use. |
| ff-exclude-show-filter | Only exclude_manager.rs + dispatch_ffedit.rs (FFEDIT editor env) | PARK | Exclude/show is an editor (FFEDIT) feature. |
| ff-exclude-manager | Only the editor exclude feature (exclude_manager.rs) | PARK | Editor exclude engine wrapper. |
| ff-viewport-scrolling | Only editor (tab_state cursor/viewport, editor_panel) | PARK | Editor viewport/caret model. |
| ff-scroll-amount | Only editor (scroll_amount.rs re-export, editor_panel) | PARK | Editor scroll translation. |
| ff-undo-redo | Only editor undo (not referenced by non-editor shell) | PARK | Editor transaction history. |
| ff-zoom | YES -- render_status.rs (status-bar zoom indicator), construct.rs, state.rs, update_startup.rs | KEEP (shell chrome) OR stub | See section 1.2 -- zoom is wired into the SHELL status bar + session restore, not only the editor. |

Two of the eleven are NOT purely editor machinery (find-and-replace, zoom) --
flagged and ruled below.

### 1.1 ff-find-and-replace + ff-global-search chain (explicit ruling)

- ff-global-search deps: `ff-find-and-replace` + `ff-file-ops` (+ ignore/regex).
  It has NO catalog/VFS/editor-panel dependency.
- ff-find-and-replace is ALSO used by the editor's `find_manager.rs`. But because
  ff-global-search (a CORE Context -- Search Results) depends on it, it STAYS in
  core as a SEARCH dependency even though it is also editor-related.
- RULING: KEEP ff-find-and-replace and ff-file-ops in core (they are
  global-search's dependency chain). The Search Results Context remains
  functional. find_manager.rs (the editor's find wiring) is PARKED with the
  editor, but the crate stays because search needs it.

### 1.2 ff-zoom ruling

ff-zoom is referenced by SHELL code, not only the editor:
- `shell/render_status.rs` -- the status-bar zoom indicator (ZoomIndicatorState).
- `shell/construct.rs`, `shell/state.rs` -- ZoomState/ZoomConfig shell fields.
- `shell/update_startup.rs` -- restore persisted global zoom offset.

RULING: KEEP ff-zoom in core as SHELL chrome (view-zoom applies to the whole
workbench, not just a document). It stands up without an editor (the zoom
indicator and global zoom offset are shell-level). Low cost, genuinely core.
If the owner prefers zoom to leave with the editor, it is a clean further-trim
toggle (TOGGLE Z1) -- cost: remove the status-bar zoom indicator + the
session zoom-restore; small but touches shell render/startup.

---

## 2. TRUE-CORE and PARKED lists (minimal shell core)

### TRUE CORE (the workbench shell + content-independent Contexts)

```
ff-core
ff-config
ff-logging
ff-theme
ff-theme-editor
ff-keys
ff-layout
ff-shell
ff-command
ff-command-semantics
ff-session
ff-help
ff-i18n
ff-plugin
ff-context-menu
ff-global-search
ff-find-and-replace     (global-search dep -- section 1.1)
ff-file-ops             (global-search dep -- section 1.1)
ff-toolchain-api
ff-gcc-toolchain
ff-rust-toolchain
ff-toolchain-panel
ff-screen-model
ff-scrm
ff-workflow
ff-zoom                 (shell chrome -- section 1.2)
ff-fftest
ff-desktop
```

Count: 28 core crates.

Contexts that remain (all content-independent): Home/POM, Settings/Config,
Theme Editor, Keys/Menus/Kinds editors, Alias editor, Help, Command
Configurator, Macro Library, Event Log, Plugin Manager, Search Results,
Toolchain panel, SCRM viewer. (The in-app "editors" here are CONFIG/menu/key/
theme editors -- shell configuration surfaces, NOT the text/file editor.)

NOTE on ff-help / ff-session / ff-keys / ff-layout / ff-i18n / ff-context-menu:
verified they are shell-infrastructure crates not gated behind the four domains
(ff-shell pulls ff-command/ff-config/ff-layout/ff-workflow/ff-logging only;
ff-scrm pulls ff-screen-model only; ff-workflow pulls ff-logging only). The
implementation step must still `cargo check -p ff-desktop` after unwiring to
catch any transitive surprise, but no residual coupling is visible in manifests.

### PARKED (moved out of default-members; stay in `members`)

Editor + editing machinery (11):
```
ff-document-model
ff-edit-operations
ff-display-line-mapping
ff-navigation-commands
ff-exclude-show-filter
ff-exclude-manager
ff-viewport-scrolling
ff-scroll-amount
ff-undo-redo
ff-editor-panel
(ff-find-and-replace STAYS -- search dep, NOT parked despite being editor-related)
```

File navigator (4):
```
ff-explorer-view
ff-nav-model
ff-file-tree
ff-files-panel
```

VFS (6 -- FULL park, section 0):
```
ff-vfs
ff-connector-local-fs
ff-posix-provider
ff-ce-host-fs
ff-ce-ntfs
ff-ce-posix
```

Catalog (5):
```
ff-dscatalog
ff-catalog-registry
ff-catalog-dialog
ff-dataset-alloc-dialog
ff-idcams
```

Volume (1):
```
ff-volume
```

Newly-parked from the 54-closure: 10 (editor machinery, excl. find-and-replace)
+ 4 (navigator) + 6 (VFS) + 5 (catalog) + 1 (volume) = 26. Core = 54 - 26 = 28.
Balanced against the 28-crate core list above.

The 38 crates already outside the closure (closure.md parked list) remain parked
and are unaffected.

### Further-trim toggles (independent, cost noted)

- TOGGLE Z1 -- park ff-zoom too: remove status-bar zoom indicator + session
  zoom-restore. Cost: small shell render/startup edits. Recommend KEEP zoom.
- TOGGLE S1 -- park SCRM (ff-scrm + ff-screen-model): if the owner wants the
  absolute minimum, SCRM is a self-contained Context. Cost: remove the ScrmViewer
  arm + nav descriptor + CAPTURE command. Recommend KEEP (self-contained, no drag).
- TOGGLE T1 -- move ff-fftest to dev-dependencies of ff-desktop: separate manifest
  change, not default-members scope. Recommend handle separately.
- TOGGLE G1 -- park global-search (and with it find-and-replace + file-ops): if
  the owner wants NO file content features at all in core. Cost: remove Search
  Results Context arm + nav + global search command. Recommend KEEP (a shell
  search over workspace roots is content-independent and useful).

---

## 3. Proposed root Cargo.toml default-members (minimal shell core)

`members` is UNCHANGED (nothing removed). Add `default-members` = the 28-crate
true-core list. Path prefix `crates/<name>` matches the existing `members`.

```toml
default-members = [
    "crates/ff-command",
    "crates/ff-command-semantics",
    "crates/ff-config",
    "crates/ff-context-menu",
    "crates/ff-core",
    "crates/ff-desktop",
    "crates/ff-fftest",
    "crates/ff-file-ops",
    "crates/ff-find-and-replace",
    "crates/ff-gcc-toolchain",
    "crates/ff-global-search",
    "crates/ff-help",
    "crates/ff-i18n",
    "crates/ff-keys",
    "crates/ff-layout",
    "crates/ff-logging",
    "crates/ff-plugin",
    "crates/ff-rust-toolchain",
    "crates/ff-screen-model",
    "crates/ff-scrm",
    "crates/ff-session",
    "crates/ff-shell",
    "crates/ff-theme",
    "crates/ff-theme-editor",
    "crates/ff-toolchain-api",
    "crates/ff-toolchain-panel",
    "crates/ff-workflow",
    "crates/ff-zoom",
]
```

(28 entries. If TOGGLE Z1/S1/G1 are taken, remove the corresponding lines.)

IMPORTANT: `default-members` only narrows the DEFAULT build/test SET. ff-desktop
must ALSO have the parked deps removed and the code unwired (section 4) or Cargo
still compiles them as ff-desktop dependencies. Plus the ff-shell dead-dep drop
(section 0 item 3). All ship together on branch core-strip-001.

---

## 4. Unwiring plan for ff-desktop (minimal-core park)

`ffwb` must still compile and run as a coherent shell after the unwiring.
Honour framework-conformance.md: parked Context render arms are cleanly REMOVED
or replaced by a documented stub that reports `InteriorFocus::none()` through the
single latch path -- no bespoke intercepts. One reversible commit per domain;
restore points recorded in the re-entry register (section 5). Order: Editor ->
Navigator -> Catalog -> VFS -> Volume (remove top consumers before the layers
they sit on, so each step's residual-reference grep shrinks).

### 4a. Editor (do first -- it is the top consumer of document-model/vfs save)

ff-desktop deps to drop: `ff-editor-panel`, `ff-document-model`,
`ff-edit-operations`, `ff-display-line-mapping`, `ff-navigation-commands`,
`ff-exclude-show-filter`, `ff-exclude-manager`, `ff-viewport-scrolling`,
`ff-scroll-amount`, `ff-undo-redo`. (KEEP `ff-find-and-replace` -- search dep.)

Code to remove/stub (grep-confirmed):
- `shell/render_body.rs`: the `TabKind::FileEditor | TabKind::Untitled` arm
  (calls `editor_panel::render`). Replace with a stub arm that shows "Editor is
  not available in the core build" and reports `InteriorFocus::none()` (the arm
  already reports none(), so the latch contract is preserved).
- `src/editor_panel/` (mod.rs/input.rs/paint.rs re-exports of ff_editor_panel):
  remove the module.
- `src/find_manager.rs` (editor FIND/CHANGE via ff-find-and-replace against the
  document snapshot): remove (the editor's find wiring). ff-find-and-replace
  itself stays for ff-global-search.
- `src/nav_manager.rs` (ff-navigation-commands viewport nav): remove.
- `src/exclude_manager.rs` (ff-exclude-manager / ff-exclude-show-filter): remove.
- `src/scroll_amount.rs` (ff-scroll-amount re-export): remove.
- `shell/dispatch_ffedit.rs`: the FFEDIT verb bodies (ffedit_exclude/_show/_reset
  etc., ff-exclude-show-filter) and `mainframe_backend_for_active_tab`
  (ff-vfs/ff-dscatalog): remove. FFEDIT is the editor Command Environment.
- `shell/environment_ffedit.rs` + the FFEDIT entry in `environment_registry.rs`
  `builtin_entries()` (`RegisteredEnv::FfEdit`): remove the FFEDIT environment.
  KEEP FFCMD (the base command environment -- ff-vfs-free, stays).
- `tab_state.rs` / `tab_state_ctors.rs`: the document/viewport/cursor fields
  (ff-document-model DocumentHandle, ff-viewport-scrolling CursorModel/
  ViewportModel, ff-edit-operations EditProfile) and `StoreIdentity`. With no
  editor Context, FileEditor/Untitled tabs cannot be opened; prune these fields
  or reduce TabState to the panel/menu Contexts only. (Largest single edit --
  TabKind::FileEditor/Untitled variants become stub-only.)
- `shell/render_status.rs`: any line-count/caret status that reads document
  state -- reduce to shell-only status.

UI capability lost: all text editing (open/edit/save a file into an editor
Context). `file.open` / Untitled creation should surface "Editor not available
in the core build".

### 4b. File navigator

ff-desktop deps to drop: `ff-explorer-view`, `ff-nav-model`, `ff-file-tree`,
`ff-files-panel`.

Code to remove/stub:
- `shell/render_body.rs`: `TabKind::FileExplorerPanel` arm (+ its split branch:
  `nav_explorer_seed`, `render_nav_explorer_body`, `apply_nav_explorer_effects`)
  and `TabKind::FilesPanel` arm (`render_body_files_panel` in render_body_arms.rs).
  Remove both; POM options 1 (Catalog Explorer) and 2 (File Explorer) show
  "unavailable in core" notices.
- `shell/render_nav.rs` (ff-vfs posix provider navigator render): remove.
- `shell/nav_reconstruct.rs`: `WorkspaceKind::Files` and
  `WorkspaceKind::FileExplorer` arms -- remove.
- `files_panel` module + `catalog_registry` field used by `collect_search_roots`:
  `collect_search_roots` (render_body.rs) must be reduced to active-workspace
  roots only (drop the `registry.list_by_type(Native)` branch) so Search Results
  still works without the catalog registry.

UI capability lost: File Explorer Context and Catalog Explorer Context (POM
options 1 & 2).

### 4c. Catalog

ff-desktop deps to drop: `ff-dscatalog`, `ff-catalog-registry`,
`ff-catalog-dialog`, `ff-dataset-alloc-dialog`, `ff-idcams`.

Code to remove/stub:
- `src/catalog_registry.rs`, `src/catalog_manager_dialog.rs` (re-exports): remove.
- `shell/construct.rs`: the `with_builtins_and_mainframe(build_mainframe_backend(..))`
  startup wiring -> build the registry with host-only built-ins and NO mainframe
  CE. (The mainframe-less path already exists as the `#[cfg(test)] with_builtins`
  constructor; promote an equivalent non-test constructor, OR -- since the whole
  backend save seam is being removed in 4d -- drop the backend fields entirely.)
- `shell/mainframe_backend.rs` (ff-dscatalog + ff-vfs): remove.
- `shell/render_body.rs` helpers `open_mainframe_dsn`, `mainframe_dataset_recfm`
  (ff-dscatalog): remove.
- `shell/handlers.rs::recfm_kind_from_name` (ff-vfs RecordFormatKind) and the
  mainframe recfm open path: remove.

UI capability lost: mainframe DSN open, catalog browsing, record-aware SAVE.

### 4d. VFS (full park -- the seam goes away)

ff-desktop deps to drop: `ff-vfs`, `ff-connector-local-fs`, `ff-posix-provider`,
`ff-ce-host-fs` (its ff-ce-ntfs/ff-ce-posix stop being pulled).

Code to remove/stub (all now dead once 4a-4c are done):
- `shell/construct_provider.rs::build_live_provider_registry` (ff-vfs
  ProviderRegistry + ff-connector-local-fs LocalFsProvider): remove; drop the
  `provider_registry` shell field and the `#[allow(dead_code)]`
  `actions.rs::provider_registry()` accessor.
- `shell/environment_registry.rs`: remove the `host_fs: Box<dyn
  ff_vfs::BackendEnvironment>` + `mainframe` fields and `backend_for` (the SAVE
  seam). With no editor, no tab has a store to save via a backend. FFCMD base
  stays; the registry becomes a pure name/claim lookup with no ff-vfs type.
- `tab_manager.rs::save_active_tab_via_backend`, `RecordImageSource`,
  `record_attrs_for` (ff-vfs StoreTarget/RecordSource/RecordAttrs): remove.
- `shell/actions.rs` recfm_lrecl branch (ff-vfs RecordFormatKind -> document
  RecordFormat): remove.
- REQUIRED CLEANUP (section 0 item 3): drop the UNUSED `ff-document-model` dep
  from `crates/ff-shell/Cargo.toml` so ff-shell stops transitively dragging
  ff-document-model -> ff-vfs. Verify ff-shell still compiles (its source uses
  only &str/Vec<String>).

After 4d, grep `ff_vfs` across the CORE crates must return NOTHING. The
implementation step MUST run that grep and record the result; if any residual
remains, STOP and flag (keep a thin ff-vfs stub) rather than reshape the seam.

### 4e. Volume

ff-desktop dep to drop: `ff-volume`. After 4c (catalog, the main ff-volume
consumer) is done, grep `ff_volume` in ff-desktop and remove residual references
(no dedicated Volume Context exists, so no render arm).

UI capability lost: none beyond catalog (Volume had no standalone Context).

### Net effect

`ffwb` becomes a workbench shell: POM/Home, Settings, Theme/Keys/Menus/Kinds/
Alias editors, Help, Command Configurator, Macro Library, Event Log, Plugin
Manager, Search Results (over workspace roots), Toolchain panel, SCRM. No text
editor, no file tree, no catalog/volume/VFS. POM options that opened parked
Contexts show an "unavailable in core" notice.

---

## 5. Re-entry register (milestone core-001)

Intent: parked domains return as PLUGINS against a matured ff-plugin API (NOT
re-added direct deps). Record, per domain, the restore points (the unwiring is
one reversible commit per domain) and the re-entry form.

| Domain | Parked crates | Removed wiring (restore points) | Re-enters as |
|--------|---------------|----------------------------------|--------------|
| Editor | ff-editor-panel, ff-document-model, ff-edit-operations, ff-display-line-mapping, ff-navigation-commands, ff-exclude-show-filter, ff-exclude-manager, ff-viewport-scrolling, ff-scroll-amount, ff-undo-redo | FileEditor/Untitled arm, editor_panel module, find_manager, nav_manager, exclude_manager, scroll_amount, FFEDIT env + dispatch_ffedit, TabState document/viewport fields | PLUGIN(s) -- see notes a/b |
| Navigator | ff-explorer-view, ff-nav-model, ff-file-tree, ff-files-panel | FileExplorer + FilesPanel arms, render_nav, Files/FileExplorer nav arms, collect_search_roots registry branch, POM options 1 & 2 | PLUGIN -- note c |
| VFS | ff-vfs, ff-connector-local-fs, ff-posix-provider, ff-ce-host-fs, ff-ce-ntfs, ff-ce-posix | build_live_provider_registry, provider_registry field/accessor, environment_registry backend fields + backend_for, tab_manager save-via-backend + RecordImageSource, actions recfm branch, ff-shell dead document-model dep | PLUGIN -- note d |
| Catalog | ff-dscatalog, ff-catalog-registry, ff-catalog-dialog, ff-dataset-alloc-dialog, ff-idcams | catalog_registry/catalog_manager_dialog re-exports, mainframe_backend, with_builtins_and_mainframe startup, mainframe DSN helpers, handlers recfm | PLUGIN -- note d |
| Volume | ff-volume | residual ff_volume refs after catalog unwiring | PLUGIN -- note d |

Notes:
- (a) The ISPF-style large-file editor (windowed view, primary + line commands)
  returns as a PLUGIN.
- (b) A basic/core editor engine (possibly an embedded third-party engine such
  as Zed/Lapce) is a LATER, SEPARATE decision -- not decided now.
- (c) The file navigator returns together with the editor/VFS work.
- (d) VFS/Catalog/Volume re-enter as plugins against the matured ff-plugin API.
- All re-entries depend on the ff-plugin API maturing to expose stable
  plugin-facing extension points (Context registration, command/dispatch seam,
  the former SAVE seam re-expressed for plugins). ff-plugin maturation is a
  PREREQUISITE and should be sequenced before the first plugin re-entry. This is
  a dependency/ordering note, not a task for milestone core-001.
- Milestone core-001 is this strip; the milestone counter increments as the core
  is built out.

---

## 6. SEMVER / contract summary for the gate crate (repo A) -- UNCHANGED from v1

Current version 0.2.0.
- DEFAULT BEHAVIOUR CHANGE (semver-notable): bare `cargo gate` moves from
  explicit `--workspace` to Cargo `default-members`. Prominent CHANGELOG note.
- ADDITIVE: new `--workspace` flag (alias `--all`) = full members set (today's
  default). Maps to the existing `Scope::Workspace`.
- NEW default scope variant `Scope::DefaultMembers` (emits EMPTY package
  selection so cargo falls back to default-members); `scope_args` (nextest.rs)
  emits no `-p`/`--workspace`; `clippy_args` (fmt_clippy.rs) updated in lockstep.
- PRESERVE (removing any is a break): `--crate`/`-p`, `--app-only` +
  `--closure-of` (derive_orphans), `--build`, `--release`, `--fast`, the `report`
  subcommand. Precedence: `--crate` > `--app-only` > `--workspace`/`--all` >
  DEFAULT (default-members).
- HISTORY CSV: columns UNCHANGED; add a new `scope` label value "default-members"
  so per-scope ETA medians stay coherent.
- EXIT CODES: unchanged (0 CLEAN / 1 ISSUES / 2 TIMEOUT|fatal).
- VERSION BUMP: 0.2.0 -> 0.3.0 (minor) with a prominent CHANGELOG behaviour-change
  note.

---

## 7. What this proposal asks the owner to approve

A. The MINIMAL TRUE-CORE list (section 2: 28 crates) and the PARKED list (26 of
   the 54 closure crates), plus any further-trim toggles (Z1 zoom, S1 scrm,
   G1 global-search, T1 fftest-to-dev).
B. The FULL-VFS-PARK FINDING (section 0): ff-vfs parks fully with NO framework
   change, CONDITIONAL on dropping ff-shell's source-UNUSED ff-document-model
   dep. Approve that small ff-shell cleanup (recommended) OR choose the fallback
   (keep a thin ff-vfs stub, ff-document-model stays for ff-shell).
C. The per-crate editor-primitive rulings (section 1), in particular KEEPING
   ff-find-and-replace (search dep) and ff-zoom (shell chrome) in core.
D. The literal default-members array (section 3).
E. The unwiring plan + order (section 4) and the re-entry register (section 5).
F. The gate flag/semver design (section 6).

Base branches are owner-confirmed (both main; repo B core-strip-001, repo A
scope-default-members).

---

## 8. Approval record (owner, items A-F)

APPROVED -- proceed to implementation. Decisions:

A. APPROVED the 28-crate true-core list and the 26-crate parked list (section 2)
   as written. Further-trim toggles DECLINED: KEEP ff-zoom (reject Z1), KEEP
   ff-scrm + ff-screen-model (reject S1), KEEP ff-global-search +
   ff-find-and-replace + ff-file-ops (reject G1). Toggle T1 (ff-fftest ->
   dev-deps) is NOT part of this change -- leave ff-fftest in core as a normal
   dep for now.

B. APPROVED the full-VFS-park finding, CLEAN approach: DROP ff-shell's
   source-UNUSED `ff-document-model` dependency from
   `crates/ff-shell/Cargo.toml` so ff-vfs parks FULLY with NO framework change
   and NO thin stub. MANDATORY CONDITION: after unwiring step 4d, grep `ff_vfs`
   across all 28 core crates and RECORD that it returns nothing. If ANY residual
   ff_vfs reference remains in core, STOP and flag to the owner (fall back to a
   thin ff-vfs stub) -- do NOT reshape the BackendEnvironment seam.

C. APPROVED the per-crate editor-primitive rulings (section 1), including KEEP
   ff-find-and-replace (search dep) and KEEP ff-zoom (shell chrome).

D. APPROVED the literal 28-entry default-members array (section 3). `members`
   stays UNCHANGED.

E. APPROVED the unwiring plan + order (section 4: Editor -> Navigator -> Catalog
   -> VFS -> Volume), one reversible commit per domain, parked Context arms
   cleanly removed/stubbed via the single focus-latch path
   (`InteriorFocus::none()`), no bespoke intercepts. APPROVED the re-entry
   register (section 5) -- to be written to `docs/reentry-register.md` in repo B
   (done by this design step; see that file).

F. APPROVED the gate flag/semver design (section 6): default = Cargo
   default-members; add `--workspace` (alias `--all`) for the full members set;
   preserve `--crate`/`--app-only`/`--closure-of`/derive_orphans; precedence
   `--crate` > `--app-only` > `--workspace`/`--all` > default; CSV columns
   unchanged plus the "default-members" scope label; exit codes unchanged; bump
   0.2.0 -> 0.3.0 minor with a prominent CHANGELOG behaviour-change note.

Base branches confirmed: repo A (cargo-gate) base main, branch
scope-default-members; repo B (FileForgeWorkbench) base main, branch
core-strip-001. Worktrees under each repo's .worktrees/.

Verification discipline for implementation steps (workspace execution-split
rule): COMPILE-VERIFY ONLY -- `cargo check` / `cargo clippy -- -D warnings` /
`cargo fmt --check`, SCOPED, logs read back, verify by EFFECT not the Exit Code
line. Do NOT run the whole workspace test suite in agent steps. The critical
end-to-end check for repo B: `cargo build` with no `-p`/`--workspace` builds
ONLY the 28-crate core and `ffwb` still builds and launches. Cross-verify must
demonstrate: default gate selects only the 28 core crates; `--all`/`--workspace`
selects the full members set; `--app-only` still works.
