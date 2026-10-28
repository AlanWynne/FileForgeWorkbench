# Extension System Consolidation Report

Read-only inventory. This document pulls together EVERYTHING plugin/extension
related that exists in FileForgeWorkbench today so the owner can evaluate and
re-design the extension system from a single place. It makes NO code changes and
proposes NO new wiring. ASCII only.

Terminology note: the code, specs, and UI currently say "plugin" everywhere. A
standing owner decision renames the concept to "extension" (Extension Manager,
EXTENSIONS, ff-extension, extension-architecture, etc.), rolled out gradually as
each item is touched. This report records the CURRENT names as they exist in the
tree (so paths are accurate) and flags the extension-equivalent where useful. No
rename is performed here.

---

## 1. Overview and the "strip to core" context

The application has just been stripped to a bare core (milestone core-001). The
authoritative record of that strip is `docs/reentry-register.md`, complemented by
`docs/status/RESUME-ffdesktop-simplification.md` (the architectural simplification
pass) and `docs/project-management/ROADMAP.md` (the forward plan).

Key facts from the strip (all from `reentry-register.md`):

- core-001 narrowed `default-members` to a minimal workbench SHELL plus its
  content-independent Contexts. Nothing was deleted: every parked crate stays a
  workspace `member`; only the WIRING that pulled a crate into the `ffwb` binary
  was removed, one reversible commit per domain.
- The minimal core is 32 crates (Option A). `ff-plugin` IS in the retained core
  set. So is `ff-scrm` and `ff-screen-model` (SCRM stays core -- see section 7).
- The domains deliberately UNWIRED and parked are: Editor, File navigator,
  VFS providers, Catalog (dataset catalog + IDCAMS), and Volume. Each is
  "preferably re-entered as a PLUGIN against the matured ff-plugin API (NOT a
  re-added direct dep)."
- A named CROSS-CUTTING PREREQUISITE is recorded: "All plugin re-entries depend
  on the core ff-plugin API MATURING to expose stable plugin-facing extension
  points: Context registration, the command/dispatch seam, and the former VFS
  SAVE seam re-expressed for plugins. ff-plugin maturation is a PREREQUISITE and
  should be sequenced BEFORE the first plugin re-entry."

That prerequisite is the whole reason for this consolidation: the extension
system (ff-plugin + its wiring contract) must be evaluated and re-designed before
any parked domain can come back as an extension. See the gap summary (section 9)
for the keystone contract the next step must author.

Framework-conformance constraint (from `.kiro/steering/framework-conformance.md`
and `wiring-standard.md`): the extension system must build ON the existing
framework seams (single `resolve_target` / `CommandTarget` dispatch, per-tab
navigation, `WorkspaceContext` + `render_workspace_context` focus latch,
`WorkspaceDescriptor` persistence, code-only `DEFAULT_*_TOML` menus, the Kinds
registry). Changing any of those load-bearing seams is a framework change that
needs express owner confirmation.

---

## 2. Crate inventory (BUILT / STUBBED / SPEC-ONLY)

All crates live under `crates/`. Status key: BUILT = substantial implementation
with tests; STUBBED = skeleton/placeholder; SPEC-ONLY = specified but no crate.

### ff-plugin -- BUILT (not wired into the live binary beyond one type)

Path: `crates/ff-plugin/`. The extensibility framework crate. Modules:
`lib.rs`, `traits.rs`, `context.rs`, `capability.rs`, `capability_registry.rs`,
`metadata.rs`, `registry/{mod,lifecycle,load}.rs`, `loader.rs`, `dependency.rs`,
`lifecycle.rs`, `version.rs`, `security.rs`, `event.rs`, `error.rs`, plus
`tests/integration.rs` and `tests/property_tests.rs`.

What is BUILT:
- `FileForgePlugin` trait (object-safe, `Send + Sync`): lifecycle
  `initialize/activate/deactivate/shutdown`, `metadata()`, `plugin_capabilities()`,
  default `supports_hot_reload() -> false`. (`traits.rs`)
- Service traits DEFINED here, to be IMPLEMENTED by downstream crates:
  `CommandRegistration`, `PluginConfigAccess`, `PluginVfsAccess`,
  `PluginEventBus`, `CapabilityRegistrar`, plus `PluginCommand`. (`traits.rs`)
- `Capability` enum (non_exhaustive): `Commands`, `Viewers`, `Providers`,
  `LanguageSupport`, `ThemeContribution`, each with a per-capability `version`;
  plus `CapabilityType`, `CapabilityDescriptor`, `CapabilityFilter`.
  (`capability.rs`)
- `PluginRegistry` (thread-safe via `RwLock`): discover (scan a
  `plugin_directory` for `plugin.toml`), register, load/unload, state query,
  list, shutdown-all, panic isolation via `catch_unwind`. (`registry/`)
- `PluginContext` + `PlatformServices` (the sandboxed service gateway),
  `CapabilityRegistry`, `DependencyGraph`, `PluginState`, `PluginMetadata` +
  `parse_manifest`, `Version`/`VersionReq`/`PLUGIN_API_VERSION` (1.0.0),
  `loader::scan_plugin_directory`.
- The `plugin-architecture` tasks.md is fully checked (every task `[x]`).

IMPORTANT WIRING GAP: `ff-plugin` is declared as a dependency in
`crates/ff-desktop/Cargo.toml`, but ff-desktop uses it ONLY for the
`PluginState` type inside the Plugin Manager panel. There is NO live
`PluginRegistry` constructed anywhere in ff-desktop, no `PlatformServices`
implementation, no discovery call, and no plugin loaded at startup. The service
traits (`CommandRegistration`, `PluginVfsAccess`, etc.) have NO production
implementations -- only mock impls inside ff-plugin's own tests. So the framework
is built and unit-tested in isolation but is NOT yet connected to the running
workbench.

### ff-mdx-plugin -- BUILT (worked example), NOT wired into ff-desktop

Path: `crates/ff-mdx-plugin/` (`Cargo.toml`, `src/lib.rs`, `src/viewer.rs`).
A worked example plugin: `MdxPlugin` implements `FileForgePlugin`, advertises a
`Capability::Viewers` for `text/markdown`, and contributes `MdxFileViewer` (an
`ff_viewers::FileViewer`) that renders Markdown via `ff-md-viewer`. It is the
reference for "how a plugin is written." It is NOT listed in ff-desktop's
Cargo.toml, so it is not compiled into or loaded by the live binary.
Extension-equivalent name: `ff-mdx-extension`.

### ff-viewers -- BUILT, NOT wired into ff-desktop

Path: `crates/ff-viewers/`. The extensible file-viewer framework: `FileViewer`
trait (`trait_def.rs`), `ViewerRegistry` (`registry.rs`), built-in viewers
(`built_in/asa_report.rs`, `csv_table.rs`, `hex.rs`, `image.rs`), `PREVIEW`
command (`command.rs`), `ViewerPanel` (`panel.rs`), content selection
(`selection.rs`), refresh (`refresh.rs`), read-only guard (`readonly.rs`), and --
directly relevant -- `plugin_bridge.rs`, which registers/deregisters
plugin-provided viewers into the registry and auto-deregisters them on plugin
shutdown. It is the concrete consumer of a plugin `Capability::Viewers`.
`ff-viewers` is NOT in ff-desktop's Cargo.toml dep list (confirm during design):
it is built and tested but not wired into the current bare core.

### ff-md-viewer -- BUILT (support crate)

Path: `crates/ff-md-viewer/` (`lib.rs`, `renderer.rs`, `scanner.rs`,
`watcher.rs`, `error.rs`). The Markdown-to-HTML engine used by
`ff-mdx-plugin::MdxFileViewer`. Not a plugin itself; a model crate a plugin
depends on.

### ff-scrm -- BUILT and WIRED (core, NOT an extension)

Path: `crates/ff-scrm/` (see section 7). Declared in ff-desktop's Cargo.toml and
wired. Intended to stay CORE.

### Plugin-adjacent crates that are parked domains (re-enter AS extensions)

These are BUILT crates for the domains the strip parked; they re-enter as
extensions later (section 6 catalog). Named in `reentry-register.md`:
- Editor domain: `ff-editor-panel`, `ff-display-line-mapping`,
  `ff-navigation-commands`, `ff-exclude-show-filter`, `ff-exclude-manager`,
  `ff-scroll-amount`, `ff-undo-redo`.
- File navigator: `ff-explorer-view`, `ff-nav-model`, `ff-file-tree`,
  `ff-files-panel`.
- VFS providers: `ff-connector-local-fs`, `ff-posix-provider`, `ff-ce-host-fs`,
  `ff-ce-ntfs`, `ff-ce-posix` (plus `ff-connector-extensibility`).
- Catalog: `ff-dscatalog`, `ff-catalog-registry`, `ff-catalog-dialog`,
  `ff-dataset-alloc-dialog`, `ff-idcams`.
- Volume: `ff-volume`.
- Database / Job Monitor / File Formatter candidates: `ff-database-tool`,
  `ff-jes`, `ff-idcams`, `ff-structure-catalog`, `ff-forge`, toolchains
  (`ff-gcc-toolchain`, `ff-rust-toolchain`, `ff-toolchain-api`,
  `ff-toolchain-panel`), `ff-lua`.

(Full crate list: 90+ crates under `crates/`; the above are the ones that are
plugin/extension-relevant.)

---

## 3. ff-desktop plugin_manager module + Plugin Manager Context wiring

The Plugin Manager Context is BUILT and fully wired as a `WorkspaceContext`,
following the canonical wiring standard. Details, with file paths:

- Panel module: `crates/ff-desktop/src/plugin_manager_panel.rs`.
  - `PluginManagerPanelState { plugins: Vec<(String, PluginState)>, filter,
    selected_index, first_interior_id }`.
  - `sort_plugins` (alphabetical), `filter_plugins` (case-insensitive), state
    badge colours/labels, `filter_field_id()` (stable `egui::Id`
    "plugin_manager_filter"), and the `render()` function.
  - It reads `ff_plugin::PluginState` for the type, but its `plugins` vector is
    only ever populated by tests -- there is no live registry feeding it (see the
    wiring gap in section 2).
  - Declared in `main.rs` as `mod plugin_manager_panel;`.

- `WorkspaceContext` impl (CR-NR-078 WF.6): at the bottom of
  `plugin_manager_panel.rs`, `impl WorkspaceContext for PluginManagerPanelState`
  renders and returns `InteriorFocus::single(filter_field_id())`. The Filter
  field is the first (and last) interior control.

- TabKind / Kind registry:
  - `KindTag::PluginManager` and payload `TabKind::PluginManager` (tab_state).
  - `BuiltinKind::Plugins` in `workspace_kind/mod.rs`: kind name "plugins",
    Tab_Header tag "[PLUGINS]", display title "Plugin Manager". Mapped from
    `KindTag::PluginManager`.
  - `WorkspaceKind::PluginManager` for descriptors/session.

- Navigation + render dispatch:
  - `shell/nav_stack.rs`: `KindTag::PluginManager ->
    custom(WorkspaceKind::PluginManager, DescriptorParams::new())`.
  - `shell/nav_reconstruct.rs`: `WorkspaceKind::PluginManager ->
    set_active_tab_context(TabKind::PluginManager, "[PLUGINS]")`.
  - `shell/render_body.rs`: the `TabKind::PluginManager` arm does the
    owned-panel swap: `mem::take` the panel ->
    `render_workspace_context(ctx, ui, &mut panel)` -> put it back. This is the
    single focus-latch path (no hand-written latch).
  - `shell/state.rs` + `shell/construct.rs`: holds `plugin_manager_panel:
    PluginManagerPanelState` on the shell.
  - `shell/commands_session.rs`: `WorkspaceKind::PluginManager ->
    self.tabs.open_plugin_manager_tab(&self.runtime)`.
  - Session round-trips via `PersistedTabKind::PluginManager` and
    `session_manager.rs`.

- InteriorFocus contract: `InteriorFocus::single(filter_field_id())`; a
  full-shell egui_kittest first-Tab test exists
  (`full_shell_plugin_manager_first_tab_focuses_interior`, tests_focus.rs),
  asserting the first Tab lands on the Filter field (no phantom stop,
  CR-CH-023 / B059).

Conclusion: the Plugin Manager Context is a textbook `WorkspaceContext` wiring.
Its only gap is that it has no live `PluginRegistry` behind it -- the enable/
disable actions (plugin-manager-ui Req 2) and the detail view (Req 3) have no
backing registry to act on in the running app.

---

## 4. The PLUGINS POM option and its dispatch path

### Command classification (BUILT)

`crates/ff-desktop/src/command_config/mod.rs` is the verb-table classifier
(`builtin_workspace_target_for`). The verb `PLUGINS` is in the in-scope list and
resolves to `custom("plugin_manager")`, i.e. a
`CommandTarget::CustomWorkspace { kind: "plugin_manager" }`.

Dispatch path (single front door, B080):
1. Typed/menu/key seam -> `shell/dispatch.rs::dispatch_command_string`
   (prelude -> `resolve_target`).
2. `resolve_target` / `builtin_workspace_target_for` classifies `PLUGINS` as
   `CustomWorkspace("plugin_manager")`.
3. `shell/target_dispatch.rs` CustomWorkspace arm: the `"plugin_manager"` case
   calls `self.nav_to_kind(WorkspaceKind::PluginManager)`.
4. `nav_to_kind` transforms the active tab in place (per-tab Navigation_Stack).

Confirmed by `tests_command.rs::plugins_command_routes_to_plugin_manager`
(typing `PLUGINS` sets the active tab kind to `KindTag::PluginManager`).

### POM menu entry (GAP / discrepancy)

The Plugin Manager UI spec says the Context is reached via "POM option 8
(Plugins)" and `=8`. BUT the current code-only POM baseline
(`crates/ff-desktop/src/menu_workspace/defaults.rs::DEFAULT_POM_TOML`) has NO
PLUGINS option. The baseline POM options today are:
- `0` Settings, `1` Catalogs, `2` Files, `3` Help, `X` collapse/close.

So in the current bare core, the Plugin Manager is reachable only by TYPING
`PLUGINS` (or via the classifier), NOT from a POM menu option. The spec's
"option 8" and the live POM menu disagree. The re-design should decide the POM /
menu entry for the renamed EXTENSIONS Context (and whether it belongs in the bare
core POM at all, mirroring the Catalogs/Files removal in section 8).

---

## 5. The two specs, summarized

### docs/specs/plugin-architecture/ (requirements 16.9 KB, design 47.9 KB, tasks 28.9 KB)

The foundational `ff-plugin` spec. Seven requirements:
1. Plugin Trait Definition -- `FileForgePlugin` lifecycle + metadata +
   capabilities; object-safe; structured `PluginError` (no panics to host).
2. Plugin Context -- `PluginContext` as the SOLE interface to platform services
   (logging, command registration, config, VFS, events); thread-safe;
   namespace-scoped config (`[plugins.{name}]`).
3. Plugin Registration and Loading -- discover -> load -> validate -> initialize
   -> activate; dependency graph + topological load order; circular-dependency
   rejection; duplicate-capability first-wins; optional hot-reload; skip on
   unmet dependency.
4. Capability Discovery -- typed `Capability` enum; dynamic Capability_Registry;
   type-safe + attribute queries; change events.
5. Plugin Lifecycle Management -- state machine Discovered -> Loaded ->
   Initialized -> Active -> Deactivating -> Shutdown; panic isolation via
   `catch_unwind`; 5-second shutdown budget; guaranteed cleanup.
6. Versioning and Compatibility -- `PLUGIN_API_VERSION` (semver 1.0.0);
   `required_api_version`; major-mismatch reject, forward-compatible minor;
   per-capability versions.
7. Plugin Security and Sandboxing -- in-process, API-level boundaries via
   `PluginContext`; VFS-only file access (no `std::fs`); capability-gated network
   access; no cross-plugin state; namespace-scoped config; forgery-proof
   capability ownership.

tasks.md: every task `[x]` (the crate is built to spec).

### docs/specs/plugin-manager-ui/ (requirements 4.8 KB, design 2.2 KB, tasks 4.1 KB)

The UI layer on top of the (already-built) registry. Four requirements:
1. Plugin Manager Context -- opened via POM option 8 / `=8` / `PLUGINS`;
   scrollable list of registered plugins; per-plugin name/version/description/
   state/capabilities; failure reason shown; alphabetical sort; filter field.
2. Enable and Disable Plugins -- Disable calls `deactivate()`, Enable calls
   `activate()`; failure sets Failed state without crashing; enabled/disabled
   state persisted in session config (`plugins.disabled: Vec<String>`).
3. Plugin Details View -- detail area: full description, author, licence,
   homepage, capabilities, registered commands + shortcuts, config keys with a
   link to the Settings Context filtered to the plugin namespace.
4. Session Persistence -- the PluginManager tab persists and restores, reloading
   the live registry state.

design.md decisions: new `TabKind::PluginManager`; `PluginManagerPanelState`;
enable/disable via the registry's `activate/deactivate`; NO plugin marketplace
(remote install out of scope); two-pane layout (list + detail).

Gap in the specs vs reality: Req 1 (list), 2 (enable/disable), 3 (detail), 4
(reload registry state) all assume a LIVE registry reachable from the shell. The
built panel renders a list and filter, but Req 2/3 and the "reload registry
state" half of Req 1/4 are not realizable until ff-desktop constructs and owns a
`PluginRegistry`. The spec also assumes POM option 8, which the live POM does not
have (section 4).

---

## 6. The catalog of intended extension KINDS

There is no single file literally titled "extension kinds catalog"; the catalog
is expressed across the ROADMAP, the remediation plan, and the re-entry register.
Folded together below. This is the list of the kinds of plugins/extensions the
owner intends to incorporate.

### A. ROADMAP Section 5 -- PLUGIN build order (owner-directed, after CORE sign-off)

From `docs/project-management/ROADMAP.md` (quoted / paraphrased; content
rephrased for compliance):

1. Utilities -- a container for in-app tools. Candidate contents (to be
   confirmed): IDCAMS dialog, Lua scripting, Rexx, language processors (GCC,
   Rust compiler). Open question: the exact contents of Utilities.
2. Mainframe file / dataset emulation -- virtual dataset catalog plus storage;
   informed by the new Storage and Catalog Data Model.
3. POSIX file emulation -- POSIX-style virtual filesystem catalogs.
4. File Formatter -- a File-AID-inspired record-structure View/Edit capability.
5. Database -- an integrated Database IDE, delivered as a plugin.
6. Job Monitor -- a JES / SDSF emulator, delivered as a plugin.

"All new plugin capability is framed as a FileForgePlugin on the core product
(plugin-architecture, plugin-manager-ui, connector-extensibility)."

### B. ROADMAP Section 7 -- new-requirement gates feeding the plugin phases

- 7b. File Formatter -- File-AID-inspired record-structure View/Edit plugin
  (PLUGIN phase 4); bundles `record-selection-criteria`, `structure-catalog`,
  `fileforge-integration`; CR-NR-061. Foundation skeleton, not built.
- 7c. Database as a plugin -- reconfigure `database-tool` as a proper
  FileForgePlugin (PLUGIN phase 5). Source: "FileForgeWorkbench Database Plugin
  Specification".
- 7d. JES as a plugin -- formalise `jes-emulator` (+ `idcams-emulator`) as the
  Job Monitor plugin (PLUGIN phase 6).

### C. reentry-register.md -- parked domains, each re-entering AS an extension

Each parked domain "re-enters as a PLUGIN against the matured ff-plugin API":
- Editor -- the ISPF-style large-file editor (windowed view, primary + line
  commands) returns as a plugin. (A basic/core editor engine, possibly an
  embedded third-party engine, is a later separate decision.)
- File navigator -- re-enters as a plugin, together with the editor/VFS work.
- VFS providers + backend Command Environments -- re-enter as plugins; the former
  VFS SAVE seam (`ff-vfs::BackendEnvironment`) must be re-expressed as a
  plugin-facing extension point first.
- Catalog (dataset catalog + IDCAMS) -- re-enters as a plugin.
- Volume -- re-enters as a plugin (with the catalog/VFS work).

### D. Capability-type catalog (the typed extension points ff-plugin already models)

From `ff-plugin::Capability`: the extension KINDS the framework can already carry
are `Commands`, `Viewers`, `Providers`, `LanguageSupport`, `ThemeContribution`.
`ff-viewers` is the built consumer of the `Viewers` capability; `ff-mdx-plugin`
is the worked example that advertises it. The other four capability types have no
production consumer wired yet.

### E. Other "developed as a plugin" notes found in the tree

- SCRM: an owner note (change-log CR-NR-098) says SCRM is "of plugin size,
  perhaps should be a plugin, developed as a plugin but not necessarily
  marketed/framed as a plugin." Current decision (ROADMAP/this task): SCRM stays
  CORE (section 7).
- Compiler toolchain: the requirements-review (OV-07) notes the compiler
  toolchain "is implemented as a plugin" and should state it implements a
  `ToolchainPlugin` sub-trait of `FileForgePlugin`.
- Automated dialog testing Req 9 (Plugin Testing): plugin developers test plugin
  dialogs with FFTest.

Summary of intended extension kinds (consolidated):
Utilities (IDCAMS dialog, Lua, Rexx, GCC/Rust language processors) | Mainframe
dataset emulation | POSIX file emulation | File Formatter | Database IDE | Job
Monitor (JES/SDSF) | Editor (ISPF-style) | File navigator | VFS providers +
backend CEs | Dataset catalog + IDCAMS | Volume | Viewers (built: ff-viewers +
ff-mdx-plugin) | Language support | Themes | Commands | Providers.

---

## 7. SCRM current state (CORE, out of scope for extension wiring)

SCRM (Screen Collection and Replay Manager, CR-NR-098) is intended to stay CORE,
not become an extension. Recorded here only so the core-vs-extension line is
explicit; do NOT design its wiring in the extension re-design.

Current state:
- `crates/ff-scrm/` -- BUILT and in the core set. No egui dependency; owns the
  collection/capture model, persistence, replay state machine, capture/masking
  rules, and exporters (text / markdown / html / pdf / protected pdf, evidence
  packages). Modules: `model.rs`, `persist.rs`, `replay.rs`, `rules.rs`,
  `export.rs`, `evidence.rs`, `pdf.rs`, `pdf_protected.rs`, `error.rs`.
  Validates screen-snapshot-scrm Req 7-10, 12-15, 17, 18.
- `crates/ff-screen-model/` -- the logical screen model + `SnapshotFormat`
  (PlainText / Ansi / Markdown / Html / Yaml), also core.
- Shell wiring (BUILT, core): `crates/ff-desktop/src/scrm_session.rs`
  (`ScrmSession` with capture + auto-capture); the `SNAPSHOT [TEXT|ANSI|MARKDOWN|
  HTML|YAML|AI]` and `CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY` verbs
  in `shell/commands_ladder_b.rs` -> `handle_snapshot` / `handle_capture`;
  `TabKind::ScrmViewer` / `KindTag::ScrmViewer` / `WorkspaceKind::ScrmViewer`
  (the "[REPLAY]" Context); helpers in `shell/helpers.rs`.
- ff-desktop Cargo.toml declares `ff-scrm` and `ff-screen-model`.
- ROADMAP records SCRM as DONE and merged (SCRM.0-SCRM.6; residual SCRM.7 is TCR
  bookkeeping).

Relevant to the extension re-design: SNAPSHOT/CAPTURE are currently reached via
the command ladder, not yet via a menu-bar / POM menu option. The owner's
original request ("make them available via menu option and the menu bar") is a
separate core wiring task, not part of the extension system; noted here for
completeness.

---

## 8. Catalogs and Files POM options -- current wiring + parallel removal effort

### Current POM wiring (BUILT)

From `DEFAULT_POM_TOML` (`menu_workspace/defaults.rs`):
- Option `1` = command `Catalogs`, description "Virtual File Catalogs --
  Mainframe, POSIX, Native".
- Option `2` = command `Files`, description "File Explorer -- Browse catalogs and
  files in a tree view".

Dispatch (the classifier in `command_config/mod.rs` and the CustomWorkspace
dispatch arm):
- `CATALOGS` / `FILE CATALOGS` -> `CustomWorkspace("...")` -> the Catalog
  Explorer Context (Virtual Catalog Manager), `KindTag::FilesPanel` /
  `BuiltinKind::Catalogs` ("[CATALOGS]", "Catalog Explorer").
- `FILES` / `=FILES` -> the File Explorer Context, `KindTag::FileExplorerPanel` /
  `BuiltinKind::Files` ("[FILES]", "File Explorer").

### The parallel removal effort (in flight NOW)

A separate effort is removing Catalogs and Files from the bare core POM (the
owner's messages 3-4: "Catalogs and Files ... should not be part of the core and
the reset bare application ... they belong in a plugin to be pulled in later").
`reentry-register.md` already records the strip's effect on these Contexts:
- "POM options 1 (Catalog Explorer) and 2 (File Explorer) -> 'unavailable'
  notices." The File navigator and Catalog domains are parked; their render arms
  were replaced with "unavailable in core" stubs reporting
  `InteriorFocus::none()` through the single latch path.
- Both domains "re-enter as a PLUGIN against the matured ff-plugin API."

So the current tree is in a transitional state: the POM baseline STILL lists
Catalogs (1) and Files (2), while the backing Contexts are being unwired and the
options are slated to leave the bare-core / reset-bare POM. The extension
re-design defines how Catalogs and Files RETURN as extensions (with their menu/
POM entries reinstated by the extension, not baked into the core baseline). This
report does not change either side.

---

## 9. Gap / overlap summary and the keystone contract for the next step

### What exists and is solid
- `ff-plugin` is a complete, unit-tested framework crate (trait, context,
  capability model, registry, lifecycle, versioning, security boundaries).
- A worked example (`ff-mdx-plugin`) and a real capability consumer
  (`ff-viewers` + `plugin_bridge`) exist and are tested.
- The Plugin Manager Context is a correct `WorkspaceContext` wiring with a
  passing first-Tab focus test, and `PLUGINS` dispatches correctly through the
  single front door.

### The central gap (the keystone the next step must close)
`ff-plugin` is NOT connected to the running workbench. There is:
- No live `PluginRegistry` constructed in ff-desktop.
- No production `PlatformServices` (no real `CommandRegistration`,
  `PluginConfigAccess`, `PluginVfsAccess`, `PluginEventBus`, `CapabilityRegistrar`
  implementations -- only test mocks).
- No startup discovery / load, and no plugin loaded.
- No plugin-facing EXTENSION POINTS for the three seams `reentry-register.md`
  names as the prerequisite: (a) Context registration (a plugin contributing a
  `WorkspaceContext` / TabKind / Kind without editing the core `match` arms),
  (b) the command/dispatch seam (a plugin registering a verb/`CommandTarget`
  through `resolve_target`), and (c) the former VFS SAVE seam
  (`ff-vfs::BackendEnvironment`) re-expressed for plugins.

The `Capability` model covers Viewers/Commands/Providers/LanguageSupport/Themes,
but the framework has no mapping from those capabilities onto the live framework
seams (WorkspaceContext + render_workspace_context, resolve_target/CommandTarget,
WorkspaceDescriptor persistence, the Kinds registry, menus TOML). That mapping is
exactly what `wiring-standard.md` prescribes for a CORE feature; the extension
system needs the plugin-facing equivalent.

### Overlaps / discrepancies to resolve in the re-design
- POM entry mismatch: plugin-manager-ui assumes "POM option 8"; the live
  baseline POM has no PLUGINS option (section 4).
- plugin-manager-ui Req 2/3/4 presuppose a live registry the shell does not own.
- Terminology: the entire surface is to be renamed plugin -> extension
  (ff-plugin -> ff-extension, Plugin Manager -> Extension Manager, PLUGINS ->
  EXTENSIONS, the two spec folders -> extension-architecture / extension-manager-
  ui), per the standing owner rename decision. The re-design authors new material
  under the extension name.
- Catalogs/Files are mid-removal (section 8); the re-design must define their
  return as extensions, including how an extension reinstates a POM/menu option.

### The keystone contract the next (design) step must author
A single "extension integration contract" that maps each `Capability` /
extension kind onto the existing framework seams, i.e. the plugin-facing
expression of `wiring-standard.md`:
- how an extension contributes a Context (WorkspaceContext + TabKind + Kind +
  render dispatch) without editing core `match` arms;
- how it registers commands/verbs through `resolve_target` / `CommandTarget`
  and contributes menu options (menus TOML, not a bespoke intercept);
- how it participates in `WorkspaceDescriptor` session persistence and the Kinds
  registry;
- how the former VFS SAVE seam (`BackendEnvironment`) and the viewer bridge
  become stable plugin-facing extension points;
- the live `PluginRegistry` + `PlatformServices` construction, discovery, and
  lifecycle wiring in ff-desktop;
- and how all of the above is reconciled with framework-conformance (no new
  dispatcher, no second nav stack, one focus latch, code-only built-in menus) --
  flagging anything that genuinely needs an owner-confirmed framework change.

That contract is the prerequisite `reentry-register.md` names as gating the first
parked-domain re-entry, and it is what should be designed and named from this
consolidated picture.

---

## Appendix: primary sources read

- `docs/status/RESUME-ffdesktop-simplification.md`
- `docs/reentry-register.md`
- `docs/project-management/ROADMAP.md` (Sections 5, 6, 7)
- `docs/specs/plugin-architecture/{requirements,design,tasks}.md`
- `docs/specs/plugin-manager-ui/{requirements,design,tasks}.md`
- `.kiro/steering/{framework-conformance,wiring-standard}.md`
- `crates/ff-plugin/src/{lib,traits,capability,loader}.rs`,
  `crates/ff-plugin/src/registry/mod.rs`
- `crates/ff-mdx-plugin/src/lib.rs`
- `crates/ff-viewers/src/{lib,plugin_bridge}.rs`
- `crates/ff-scrm/src/lib.rs`
- `crates/ff-desktop/src/plugin_manager_panel.rs`,
  `crates/ff-desktop/src/menu_workspace/defaults.rs`,
  `crates/ff-desktop/src/command_config/mod.rs`,
  and the shell wiring files (nav_stack, nav_reconstruct, render_body,
  target_dispatch, state, construct, commands_session, commands_ladder_b)
- `crates/ff-desktop/Cargo.toml`

Content from steering, specs, and status docs was rephrased/summarized for this
report; see the originals for authoritative wording.
