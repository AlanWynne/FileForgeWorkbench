# Core Re-entry Register -- Milestone core-001

ASCII only. Authoritative record of what the core-001 strip PARKS and how each
parked domain comes back. Owner-approved from the v2 minimal-shell-core proposal
(`.agents/tasks/gate-scope-default-members/true-core-proposal.md`), as adjusted
by the owner's OPTION A decision (see "Option A adjustment" below).

## Purpose

Milestone core-001 strips the FileForgeWorkbench default build/test scope down
to a minimal workbench SHELL and its content-independent Contexts, so the gate
round-trip is fast while the core is rationalised. Nothing is deleted -- every
parked crate stays in the workspace `members`; only `default-members` is
narrowed, and the wiring that pulled parked crates into the `ffwb` binary is
removed behind ONE reversible commit per domain. This register is the map for
pulling each domain back in later.

Parked crates WILL bit-rot as the core API changes under them: the plugin API,
the command/dispatch seam, the `WorkspaceContext`/`InteriorFocus` focus contract,
and the former VFS SAVE seam are all expected to evolve while these domains sit
out. Each parked domain is therefore expected to need REWORK on re-entry, not a
mechanical revert -- the "restore points" below are a starting map, not a
guarantee the old wiring still fits.

## Milestone

- core-001 -- this strip. The milestone counter increments as the core is built
  back out (core-002, core-003, ...), one domain/plugin at a time.

## Option A adjustment (owner-approved) -- TabState infrastructure is RETAINED

The original proposal planned a FULL VFS park and the removal of the editor's
document/viewport/edit crates. Implementation found that the shell's per-tab
`TabState` struct -- used by EVERY retained Context (POM, Config, Search Results,
Plugin Manager, Event Log, Theme/Keys/Menus/Kinds/Alias editors, Help, SCRM) --
depends DIRECTLY on `ff-document-model` (its `document: DocumentHandle` field),
`ff-viewport-scrolling` (`viewport`/`cursor`), and `ff-edit-operations`
(`edit_profile`); and `ff-document-model` transitively pulls `ff-vfs`. These
cannot be dropped without a substantial `TabState` refactor.

The owner therefore chose OPTION A:

- `ff-document-model`, `ff-viewport-scrolling`, `ff-edit-operations`, and
  `ff-vfs` are RETAINED as core TabState infrastructure. They stay in
  `ff-desktop`'s dependencies AND in `default-members`. They are NOT parked.
- The editor, file navigator, dataset catalog, VFS PROVIDERS, and volume are
  still fully UNWIRED as user-facing capabilities and their crates parked.
- The ff-shell dead-dependency drop and the mandatory post-unwiring
  "grep `ff_vfs` returns nothing in core" condition from the original proposal
  are OUT of scope under Option A (ff-vfs legitimately remains). `crates/ff-shell`
  was not touched.

FOLLOW-UP (deferred, ~core-002, to be done WITH the editor rework): a TabState
refactor to remove `document`/`viewport`/`cursor`/`edit_profile` from the core
Context tabs would allow `ff-document-model` + `ff-viewport-scrolling` +
`ff-edit-operations` + `ff-vfs` to be fully parked and VFS to leave the tree
entirely. This is a behaviour-touching architecture change needing its own
design + gate; it is NOT part of milestone core-001.

## The minimal core (what stays) -- 32 crates (Option A)

ff-command, ff-command-semantics, ff-config, ff-context-menu, ff-core,
ff-desktop, ff-document-model, ff-edit-operations, ff-fftest, ff-file-ops,
ff-find-and-replace, ff-gcc-toolchain, ff-global-search, ff-help, ff-i18n,
ff-keys, ff-layout, ff-logging, ff-plugin, ff-rust-toolchain, ff-screen-model,
ff-scrm, ff-session, ff-shell, ff-theme, ff-theme-editor, ff-toolchain-api,
ff-toolchain-panel, ff-viewport-scrolling, ff-vfs, ff-workflow, ff-zoom.

(= the approved 28-crate true-core list PLUS the four Option A TabState
infrastructure crates: ff-document-model, ff-viewport-scrolling,
ff-edit-operations, ff-vfs.)

Contexts that remain: Home/POM, Settings/Config, Theme Editor, Keys/Menus/Kinds/
Alias editors, Help, Command Configurator, Macro Library, Event Log, Plugin
Manager, Search Results (over workspace roots), Toolchain panel, SCRM viewer.

UI capabilities TEMPORARILY REMOVED by the unwiring:
- Text/file editing (the Editor Context). `file.open`, Untitled creation, and
  BROWSE/VIEW surface "editor not available in the core build".
- File Explorer Context (VFS tree browser, POM option 2) -> "unavailable" notice.
- Catalog Explorer Context (Virtual Catalog Manager, POM option 1) -> notice.
- Mainframe DSN open, catalog browsing, dataset allocation, and the record-aware
  / host-FS SAVE backend seam.
- Volume had no standalone Context, so no user-visible surface beyond catalog.

## Parked domains and restore points

Each domain is unwired as one reversible commit on the feature branch. Re-entry
is preferably as a PLUGIN against the matured ff-plugin API (NOT a re-added
direct dep). The restore points below are the exact wiring removed.

### Domain: Editor (text/file editor + editing machinery)

Parked crates:
- ff-editor-panel, ff-display-line-mapping, ff-navigation-commands,
  ff-exclude-show-filter, ff-exclude-manager, ff-scroll-amount, ff-undo-redo.

(NOTE: ff-document-model, ff-viewport-scrolling, and ff-edit-operations are NOT
parked -- under Option A they are retained as core TabState infrastructure.
ff-find-and-replace is NOT parked -- it stays as a ff-global-search dependency,
even though the editor also used it.)

Removed wiring (restore points):
- `shell/render_body.rs`: the `TabKind::FileEditor | TabKind::Untitled` arm
  (replaced by an "editor not available in the core build" notice reporting
  `InteriorFocus::none()` through the single latch path).
- `src/editor_panel/` module (mod.rs/input.rs/paint.rs re-exports of
  ff_editor_panel) -- deleted.
- `src/find_manager.rs` (editor FIND/CHANGE wiring) -- deleted.
- `src/nav_manager.rs` (ff-navigation-commands viewport navigation) -- deleted.
- `src/exclude_manager.rs` (ff-exclude-manager / ff-exclude-show-filter) -- deleted.
- `src/scroll_amount.rs` (ff-scroll-amount re-export) -- deleted.
- `shell/dispatch_ffedit.rs` (FFEDIT verb bodies) and `shell/environment_ffedit.rs`
  (the FfEditEnvironment object) -- deleted; the `RegisteredEnv::FfEdit` entry
  and the host-FS / mainframe SAVE backend seam were removed from
  `environment_registry.rs`. The FFCMD base is retained.
- The shell state fields `find_manager`, `nav_manager`, `exclude_manager`,
  `scroll_amount`, `scroll_field_text` and the SCROLL ===> command-line field.
- `shell/render_command_line.rs`: the SCROLL ===> field block.
- The editor render arm, the Ctrl+S SAVE now resolves to no claiming environment.

Re-enters as:
- (a) The ISPF-style large-file editor (windowed view, primary + line commands)
  returns as a PLUGIN.
- (b) A basic/core editor engine (possibly an embedded third-party engine such
  as Zed/Lapce) is a LATER, SEPARATE decision -- not decided at core-001.

### Domain: File navigator

Parked crates: ff-explorer-view, ff-nav-model, ff-file-tree, ff-files-panel.

Removed wiring (restore points):
- `shell/render_body.rs`: the `TabKind::FileExplorerPanel` arm (+ its split
  branch) and the `TabKind::FilesPanel` arm (`render_body_files_panel` in
  render_body_arms.rs) -- now render "unavailable in core" notices.
- `shell/render_nav.rs`, `shell/render_nav_expand.rs`, `shell/render_nav_ops.rs`
  (navigator render) -- deleted.
- `src/explorer_view.rs`, `src/nav_model.rs`, `src/posix_provider.rs` -- deleted.
- `render.rs`: the ctx-level full-window File Explorer branch and the Files Panel
  internal command-field Tab redirect.
- `update_input.rs`: the File Explorer tree-transfer / Escape-exit branch.
- The shell state fields `files_panel`, `file_explorer_panel_width`, `nav_model`,
  `nav_ui` (the NavUiState sub-struct in state_groups.rs).
- `collect_search_roots` (render_body.rs) reduced to active-workspace roots only
  (the catalog-registry Native branch removed) so Search Results works without
  the catalog registry.
- POM options 1 (Catalog Explorer) and 2 (File Explorer) -> "unavailable" notices.

Re-enters as: a PLUGIN, together with the editor/VFS work (a navigator with no
editor to open files into has no standalone core value).

### Domain: VFS providers (the VFS crate itself is RETAINED under Option A)

Parked crates: ff-connector-local-fs, ff-posix-provider, ff-ce-host-fs,
ff-ce-ntfs, ff-ce-posix.

(NOTE: `ff-vfs` itself is NOT parked under Option A -- it is retained
transitively via ff-document-model, which TabState depends on. Only the VFS
PROVIDERS and host-FS backend Command Environments are parked.)

Removed wiring (restore points):
- `shell/construct_provider.rs`::build_live_provider_registry (ff-vfs
  ProviderRegistry + ff-connector-local-fs LocalFsProvider) -- deleted; the
  `provider_registry` shell field and the `actions.rs::provider_registry()`
  accessor removed.
- `shell/environment_registry.rs`: the `host_fs` + `mainframe`
  `Box<dyn ff_vfs::BackendEnvironment>` fields, `backend_for` (the SAVE seam),
  and the `with_builtins_and_mainframe` constructor removed; the registry now
  builds only the FFCMD base via `with_builtins`.
- `tab_manager.rs`::save_active_tab_via_backend, RecordImageSource,
  record_attrs_for, clear_dirty_after_save (ff-vfs record-store contract) --
  deleted. `open_file` now returns an "editor not available in core" error (it
  used ff-connector-local-fs LocalFsProvider).
- `shell/actions.rs`::apply_pending_open_identity (ff-vfs RecordFormatKind) --
  deleted; `handlers.rs`::recfm_kind_from_name and the file.open recfm/lrecl/dsn/
  catalog params removed; `state.rs` PendingOpenReq reduced to path + owning_env.

Re-enters as: the VFS PROVIDERS and backend CEs re-enter as PLUGINS against the
matured ff-plugin API. The former SAVE seam (ff-vfs BackendEnvironment) must be
re-expressed as a plugin-facing extension point before they re-enter.

### Domain: Catalog (dataset catalog + IDCAMS)

Parked crates: ff-dscatalog, ff-catalog-registry, ff-catalog-dialog,
ff-dataset-alloc-dialog, ff-idcams.

Removed wiring (restore points):
- `src/catalog_registry.rs`, `src/catalog_manager_dialog.rs`,
  `src/dataset_alloc_dialog.rs` (re-exports) -- deleted.
- `shell/mainframe_backend.rs` (ff-dscatalog + ff-vfs) -- deleted.
- `shell/construct.rs`: the `with_builtins_and_mainframe(build_mainframe_backend(..))`
  startup wiring -> the registry now builds only the FFCMD base.
- `shell/render_body.rs` helpers open_mainframe_dsn, mainframe_dataset_recfm --
  deleted.
- `shell/handlers.rs`::recfm_kind_from_name and the mainframe recfm open path --
  deleted.
- `shell/update_dialogs_catalog.rs` (Catalog Manager dialogs) -- deleted; the
  `render_catalog_dialogs` call removed from update_dialogs.rs.
- `update.rs`::ensure_default_home_catalog + the CatalogRegistry startup/restore
  seeding; `session_manager.rs`::save_catalog_registry / load_catalog_registry /
  catalogs_path; `update_startup.rs` catalog restore; `reset_bare.rs` catalog
  reset; `workspace_io.rs` / `commands_ladder_c.rs` root-as-Native-catalog
  registration -- all removed.

Re-enters as: a PLUGIN against the matured ff-plugin API.

### Domain: Volume

Parked crates: ff-volume.

Removed wiring (restore points):
- Residual `ff_volume` references in ff-desktop after the catalog unwiring (no
  dedicated Volume Context exists, so there is no render arm to remove).

Re-enters as: a PLUGIN against the matured ff-plugin API (with the catalog/VFS
work).

## Cross-cutting prerequisite

All plugin re-entries depend on the core ff-plugin API MATURING to expose stable
plugin-facing extension points: Context registration, the command/dispatch seam,
and the former VFS SAVE seam re-expressed for plugins. ff-plugin maturation is a
PREREQUISITE and should be sequenced BEFORE the first plugin re-entry. It is not
a task for milestone core-001.

## Framework-conformance note

The unwiring honours framework-conformance.md: every parked Context render arm
is cleanly replaced by a documented stub that reports `InteriorFocus::none()`
through the single focus-latch path -- no bespoke intercepts, no second
dispatcher, no reshaped core type. The FFCMD base command environment is
retained; FFEDIT (the editor claiming environment) degrades to the base at the
gate, exactly as FFNAV already did.
