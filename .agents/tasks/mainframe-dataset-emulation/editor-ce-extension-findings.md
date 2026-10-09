# Editor + Command-Environment Extension Findings

READ-ONLY investigation of the FileForgeWorkbench (FFWB) EDITOR and
COMMAND-ENVIRONMENT extension surface, validating the owner's target model (ONE
file-system-aware editor + per-FS Command Environments, delivered as a PLUGIN
that does NOT change the core framework). Workspace:
`C:\workspace\VSC\FileForgeWorkbench`. This report changes NO source; it is
diagnosis and recommendation only. It builds on the sibling `findings.md`
(VFS/catalog/VSAM already ~70-80% built) and `architecture-revision-findings.md`
(the duplication/drift that would block a pure-plugin path).

Plain ASCII only (documentation.md).

---

## Summary answer (read this first)

The owner's model -- core UNCHANGED, extended via documented seams, with ONE
file-system-aware editor and per-FS Command Environments -- is almost entirely
realisable on seams that ALREADY EXIST, because the hardest part (the Command
Environment framework) was already built under CR-CH-053. Specifically:

- **The Command Environment model is REAL and in the running app.** FFWB has
  `EnvironmentKind {FfEdit, FfNav, FfCmd}`, a kind->environment derivation
  (`environment_for_kind` / `active_environment`), a per-environment `AliasTable`
  (surface verb -> canonical verb, locale-overlayable), a `CommandEnvironment`
  trait, and a live FFEDIT claim step (`ffedit_claim`) wired into the single front
  door AHEAD of FFCMD. This is exactly the "POSIX CE / Mainframe CE, FFCMD as
  fallback" shape the owner wants (`crates/ff-desktop/src/shell/environment.rs`,
  `dispatch.rs`, `dispatch_ffedit.rs`, `commands.rs`).

- **BUT the environment set is CLOSED today.** `EnvironmentKind` is a fixed
  3-variant enum, `environment_for_kind` is a hardcoded `match` on `BuiltinKind`,
  and the claim gate in `commands.rs` is hardcoded to `== EnvironmentKind::FfEdit`
  with a single `ffedit_claim` method on the god-struct. There is NO registry a
  plugin can push a new environment into, and FFEDIT is NOT a free-standing object
  -- it is methods on `WorkbenchShell`. So registering a brand-new "Mainframe CE"
  as a first-class claiming environment TODAY requires editing those core files.
  That is the pivotal finding (Q2).

- **The editor is ONE editor, byte/text oriented, with NO per-FS awareness yet.**
  The Editor Context renders the active `TabState` directly (it is deliberately
  NOT a `WorkspaceContext`). `TabState` carries an `EditProfile` (ISPF
  CAPS/NULLS/STATS/LOCK/HILITE) but NO record-format / LRECL / sequence-number /
  file-system attribute, and `ff-document-model` is purely UTF-8/line-ending
  oriented (no RECFM concept). So the "adaptive editor" needs a NEW
  behaviour-profile hook; it does not exist yet (Q1, Q3).

- **The File Navigator is already ONE FS-aware Context.** The live navigator keys
  off `CatalogType {Mainframe, Posix, Native}` and already branches per kind
  (it even shows "Editing Mainframe datasets is available in a later update" and
  "member navigation deferred to Slice B"). The seam for a new FS kind is this
  enum + its match arms (Q4).

- **The plugin system is in-process, compiled-in, trait-based.** `ff-plugin` has a
  `Providers` capability for VFS/data-source providers, but there is NO capability
  for a Command Environment and NO capability for a WorkspaceContext/editor
  behaviour, and the live app does not even register an `ff-vfs` `ProviderRegistry`
  at startup. "Plugin" here means "a compiled-in crate wired through `ff-desktop`",
  not a dynamically loaded module (Q5).

**Bottom line (Q7):** VFS providers and the FS-aware navigator are pure extension
on existing seams. The Command Environments and the adaptive editor need a SMALL,
owner-approved core extension -- open the closed environment set into a registry
and add an editor behaviour-profile hook -- after which the per-FS CEs and editor
adaptations become pure plugin data/registration. The recommended first core
change is to generalise the ALREADY-INTENDED CR-CH-053 model (make the
environment pluggable; make FFEDIT consult a per-FS behaviour profile) rather than
invent a new mechanism -- which is squarely the "build ON the framework, with
owner-confirmed extension" path of framework-conformance.md. The duplication in
`architecture-revision-findings.md` (two catalog crates, two StorageProvider
traits, provider stack not registered live) is a DEPENDENCY for the VFS/navigator
slice and should be a core-team cleanup before the mainframe VFS provider is wired
live.

---

## Q1. The Editor Context -- where it lives and how it is dispatched

### Dispatch and ownership
The Editor Context is the `TabKind::FileEditor | TabKind::Untitled` arm of
`render_active_tab_body` in
`crates/ff-desktop/src/shell/render_body.rs`. Unlike the Config / Plugin Manager /
Event Log / Search / Help / SCRM Contexts (which are owned-panel-swapped through
`render_workspace_context` as `WorkspaceContext` implementors), the editor is
EXPLICITLY NOT a `WorkspaceContext`. The code comment states it "renders the
ACTIVE `TabState` (not a shell-owned panel) and needs shell-entangled inputs
(cmd_engine, exclude_manager, runtime, the mutable tab) that do not fit the
`ShellServices`-only trait". It reports `InteriorFocus::none()` through the single
latch path (workspace-conformance exception 2).

### Core components and their crates
- **Document / text model:** `ff-document-model` (`DocumentHandle`, `GapBuffer`,
  `LineEndMode`, UTF-8/CRLF navigation). Purely text/byte oriented -- grep for
  `recfm|lrecl|record_format|RecordFormat` in `crates/ff-document-model/src`
  returns ONLY UTF-8 "sequence" and line-ending matches; there is NO record-format
  concept.
- **Per-tab state:** `crate::tab_state::TabState` (`crates/ff-desktop/src/tab_state.rs`).
  Holds `kind: TabKind`, `document: DocumentHandle`, `viewport`, `cursor`,
  `undo_stack`, `prefix_inputs`, `edit_profile: EditProfile`, `canvas_selection`,
  `nav_stack`. The constructors live in the sibling `tab_state_ctors.rs` (the file
  is kept under the 400-line rule).
- **Render surface + input:** `crate::editor_panel` adapter
  (`editor_panel/{mod,input,paint}.rs`) delegating pure geometry to the
  `ff-editor-panel` crate (CR-NR-098 Wave 6). The `TabState`/`CommandEngine`/
  `ExcludeManager`/runtime wiring stays shell-side.
- **FFEDIT command handling:** `shell/dispatch.rs` (router + `ffedit_claim`) and
  `shell/dispatch_ffedit.rs` (verb bodies). The semantics engine
  (`ff-command-semantics`, reached via `self.cmd_engine`) remains the fallback
  executor for prefix-area/scope commands.
- **ISPF edit profile:** `ff-edit-operations::EditProfile`
  (`crates/ff-edit-operations/src/profile.rs`): `caps/nulls/stats/lock/hilite`
  only. Serde round-trips per-file (profile_persistence.rs).

### Is the editor a WorkspaceContext? How dispatched?
NO. It is a bespoke `match` arm that calls `editor_panel::render(ui, tab, ...)`
directly on the active `TabState`. This matters for the owner's model: the
adaptive editor cannot be delivered as "just another `WorkspaceContext` plugin" --
it is the one Context whose body is the shared TabState surface, so adapting it is
about parameterising THAT surface, not swapping in a plugin-owned panel.

---

## Q2. The FFEDIT Command Environment -- the pivotal question

### How a Command Environment is defined and derived TODAY
The model is CR-CH-053 ("Command Environments", REXX/ISPF ADDRESS-inspired),
implemented in `crates/ff-desktop/src/shell/environment.rs`:

- `EnvironmentKind` -- a CLOSED enum: `FfEdit`, `FfNav`, `FfCmd`. `name()` returns
  "FFEDIT"/"FFNAV"/"FFCMD" (reserved for macro ADDRESS wiring).
- `environment_for_kind(BuiltinKind) -> EnvironmentKind` -- a hardcoded `match`:
  `Editor -> FfEdit`, `Files | Catalogs -> FfNav`, `_ -> FfCmd`. The doc comment
  frames this as "each kind supplies its environment, the way it supplies its
  `default_title()`" (dependency inversion through the kind, not a handler-owned
  `TabKind` match).
- `active_environment(KindTag, is_home)` -- routes through
  `BuiltinKind::from_tab_kind` to `environment_for_kind`.
- `AliasTable` -- per-environment surface-form -> canonical-verb map
  (`ffedit_english()` seeds the English verbs; `apply_locale_overlay` layers
  CR-NR-103 locale rows). Collision-checked.
- `CommandEnvironment` trait -- `claim(&mut self, raw, upper) -> bool`. Defined but
  marked `#[allow(dead_code)]`; FFEDIT does NOT implement it as an object yet.

### What environments exist
- **FFCMD** -- the base; it IS `ff_command::resolve_target` / `ShellTargetResolver`.
  Not a claiming environment; reached via the front door's existing
  `resolve_target` call.
- **FFEDIT** -- BUILT. Owns the editor command-line verbs (LOCATE/TOP/BOTTOM/UP/
  DOWN/LEFT/RIGHT/SORT/EXCLUDE/SHOW/RESET/FIND/RFIND/CHANGE/RCHANGE/CAPS/NULLS/
  STATS/LOCK/PROFILE/HILITE/SCROLL/SAVE). Implemented as the `ffedit_claim` method
  ON `WorkbenchShell` plus the `ffedit_*` verb bodies in `dispatch_ffedit.rs`
  (not a free-standing object -- it mutates shell-entangled `self.tabs` + managers).
- **FFNAV** -- NAMED only. The navigator kinds map to it, but it CLAIMS NOTHING;
  navigator verbs stay handled where they are today.
- **FFLINE** -- NAMED/modelled only (prefix-area line commands). The vision
  catalogue (`environments-vision.md`) names many more (FFBROWSE/FFAMS/FFJES/
  FFSQL/...), all vision-only.

### How the dispatcher consults the active environment before FFCMD
The single front door `dispatch_command_string`
(`crates/ff-desktop/src/shell/dispatch.rs`) is: `=`-prefix reinit -> prelude
(`run_command_prelude`) -> `resolve_target` (FFCMD base) -> ladder
(`run_command_ladder`). The FFEDIT claim is inserted in the ladder path in
`shell/commands.rs`:

```
// commands.rs (~lines 138-219)
let editor_env_active = active_environment(t.kind.tag(), t.is_home)
    == EnvironmentKind::FfEdit;
...
if active_environment(active_kind, active_is_home) == EnvironmentKind::FfEdit
    && self.ffedit_claim(cmd, upper) { return; }
```

So active-env claiming is gated by a HARDCODED equality to `FfEdit`, and the only
claim method is `ffedit_claim`. The exit-family prelude is also conditionalised on
`editor_env_active` so FFEDIT's `X` (EXCLUDE) shadows the FFCMD `X` exit alias
(active-wins).

### THE SEAM a plugin would use to register a NEW environment -- is it open?
**It is NOT open today.** Registering a first-class claiming environment (POSIX CE,
Mainframe CE) currently requires editing CORE files:

1. Add a variant to the CLOSED `EnvironmentKind` enum (`environment.rs`).
2. Add an arm to the hardcoded `environment_for_kind` match.
3. Add a claim method and a hardcoded gate in `commands.rs` (today
   `== FfEdit && self.ffedit_claim(...)`), because there is NO
   `Environment_Registry` the EXPLAINER sketches as "shell-owned collection" --
   it is modelled in prose but NOT built. The `CommandEnvironment` trait exists
   but has no registry, no `Vec<Box<dyn CommandEnvironment>>`, and FFEDIT does not
   implement it.

This matches the EXPLAINER's own analysis (section 6.2.1): "FFEDIT starts as
methods on the god-struct ... the abstraction is real for FFCMD and future envs,
but partly aspirational for the one env being built now", and (6.3.1) "Make FFEDIT
a real object sooner". The registry-based, pluggable form is the DOCUMENTED future
direction, not yet implemented.

**Conclusion (Q2):** The Command Environment MODEL is exactly right for the
owner's per-FS CE goal, but the environment SET is closed. Opening it (an
`Environment_Registry` + making FFEDIT a real `CommandEnvironment` object + a
per-kind/per-FS environment lookup that is DATA, not a hardcoded match) is a SMALL
core change that COMPLETES the already-intended CR-CH-053 design. It is a core
change (it reshapes `environment_for_kind` and the `commands.rs` gate), so it needs
owner confirmation per framework-conformance.md -- but it is building ON the
framework, not fighting it.

---

## Q3. Editor behaviour adaptation hook

### What exists today
`TabState.edit_profile: EditProfile` is the only per-tab behaviour parameter, and
it is ISPF-generic (CAPS/NULLS/STATS/LOCK/HILITE) with NO file-system / record
dimension. There is:
- NO record-format (RECFM), LRECL, BLKSIZE, or sequence-number attribute on
  `TabState` or `EditProfile`.
- NO file-system discriminator on `TabState` (it has `path: Option<String>` and
  `kind: TabKind`, but `TabKind` for an editor is just `FileEditor`/`Untitled` --
  it does not say whether the backing content is native / POSIX / mainframe).
- NO branch in `ffedit_claim` or the `ffedit_*` verb bodies on any file/document
  attribute -- the verbs behave identically regardless of backing store.
- `KindProfile` (on `KindConfig`, Workspace Kinds) carries `edit_profile`,
  `tab_size`, `line_end_mode`, `command_line_position` -- again no record/FS
  dimension, and it is a per-KIND default, not a per-document FS binding.

So the "edit mode" notion that exists is the ISPF profile; there is NO per-FS
behaviour profile and NO place on the document that records "this is a fixed-80
mainframe PS dataset, show sequence numbers in cols 73-80 and bound lines at 80".

### Can per-FS behaviour be injected WITHOUT a core change?
No -- not meaningfully. Both candidate options require a core addition:

- **Option (a): a behaviour-profile hook consumed by the existing FFEDIT.** Add a
  per-document `FileSystemProfile` (record format, LRECL, seq-number columns,
  caps-lock bounds, chrome flags) to `TabState` (or a new field), set it when a
  tab is opened from a given VFS kind, and have the existing `ffedit_*` verb
  bodies + the editor render surface READ it. This is the cleaner fit: ONE editor,
  ONE FFEDIT, parameterised by data. BUT the field, its population at open-time,
  and the verb/render reads are all in CORE crates (`tab_state.rs`, `editor_panel`,
  `dispatch_ffedit.rs`), so adding the hook IS a (small) core change. Once the hook
  exists, a PLUGIN can supply the profile DATA for its FS kind without further core
  edits.

- **Option (b): a per-FS editor Command Environment.** Give each FS its own
  claiming environment (Mainframe CE claims FIND/CHANGE with record-aware
  semantics). This leans on the Q2 registry work and splits editor-verb behaviour
  by environment. It is heavier (duplicates verb bodies per FS) and muddies the
  "ONE editor" goal; the chrome adaptations (seq-number ruler, LRECL bound) are NOT
  command-line verbs and still need the Option (a) profile hook regardless.

### Minimal addition and whether it is a core change
The minimal addition is Option (a): a per-document `FileSystemProfile` /
`EditBehaviourProfile` on `TabState`, defaulted to a "native text" profile (so
existing behaviour is byte-identical) and set from the opening VFS provider's
capabilities. This IS a core change (new public field on a core type + reads in
core editor code), so it needs owner confirmation -- but it is additive and
behaviour-preserving when the default profile equals today's behaviour. The
per-FS profile VALUES and the decision "DSN opened from a Mainframe catalog gets
RECFM=FB,LRECL=80" are plugin-side data once the hook exists.

---

## Q4. The File Navigator Context

### One Context, FS-aware by CatalogType
The navigator is ONE Context (the Catalog Explorer / Files Panel,
`TabKind::FilesPanel`, rendered by `render_body_files_panel` in
`shell/render_body_arms.rs` via `files_panel::render`). It is NOT per-kind; it
renders all kinds and BRANCHES on `CatalogType`.

`CatalogType {Mainframe, Posix, Native}` + `VirtualCatalog` live in
`ff-catalog-registry` (re-exported through `crate::catalog_registry`). The branch
points found:
- `shell/render_body_arms.rs` -- `FilesPanelAction::OpenFile`: computes
  `is_mainframe = catalog_type == CatalogType::Mainframe` and routes to
  `open_mainframe_dsn(...)` -> `file.open`, else opens the path directly.
- `shell/render_nav_expand.rs` -- expand/edit arms match on `CatalogType`:
  `Mainframe` lists datasets / shows a Slice-B member-navigation placeholder and
  REFUSES editing ("Editing Mainframe datasets is available in a later update");
  `Posix | Native` list a real host directory.
- `shell/render_body.rs` lists Native catalog roots; `commands_ladder_c.rs`,
  `workspace_io.rs`, `update.rs` register Native catalogs.

### The seam for a new FS kind
Making the navigator FS-aware for a NEW kind means extending the `CatalogType`
enum (in `ff-catalog-registry`) and adding arms to the `match CatalogType`
sites listed above. That is a core/shared-crate change, not a plugin registration
-- there is no data-driven "navigator provider" seam. HOWEVER, for the owner's
model the existing three kinds (Native/POSIX/Mainframe) ALREADY cover the plan;
the mainframe arm already exists and is deferred, so the navigator work is
FILLING IN the mainframe arm (dataset/member browse + edit-open), not adding a new
kind. That is bug-class completion of existing arms, largely without new seams.

---

## Q5. The plugin seam for all three (VfsProvider / CE / editor behaviour)

From `ff-plugin` (`capability.rs`, `traits.rs`, registry) and the prior findings:

- **(a) VfsProvider:** There IS a `Capability::Providers(ProvidersCapability{
  provider_type})` variant (documented example `"vfs"`), and `ff-vfs`'s
  `ProviderRegistry` has `register` / `register_storage` methods. So a provider is
  structurally pluggable. BUT the live app registers NO `ProviderRegistry` at
  startup (grep of `ff-desktop/src` for `ProviderRegistry` / `register_storage`
  finds only the `CatalogRegistry`/`VirtualCatalog` model and direct
  `PosixProvider` use in nav ops). So "pluggable" is true at the `ff-vfs` layer but
  the wiring into the running shell is NOT done -- a provider added via the plugin
  capability has nowhere to land live today.

- **(b) Command Environment:** NOT pluggable. There is NO capability variant for a
  Command Environment, and no registry (Q2). This requires the core change in Q2.

- **(c) WorkspaceContext / editor behaviour:** NOT pluggable. There is no
  `Capability` for a `WorkspaceContext`; the central-panel `match tab.kind` arms
  and the `WorkspaceContext` dispatch are all compiled into `ff-desktop`. The
  editor behaviour hook (Q3) is a core field read by core code. So neither a new
  Context nor editor adaptation is a plugin registration today.

- **Reality check:** `ff-plugin` is in-process, trait-based, same-address-space
  (plugin-architecture Req 7.1); `catch_unwind` isolates panics; there is NO stable
  C-ABI dynamic loader. In practice a "plugin" is a compiled-in crate wired through
  `ff-desktop`'s `Cargo.toml` + a registration call. The owner's "deploy as a
  plugin" is therefore satisfied as "a set of workspace crates compiled into ffwb
  and registered at the documented seams" -- NOT a hot-loaded binary.

---

## Q6. Workspace Kinds (CR-NR-090)

The Workspace Kinds registry (`crates/ff-desktop/src/workspace_kind/mod.rs`,
`registry.rs`) IS a genuine extension seam:

- `BuiltinKind` (15 compiled kinds), `KindConfig` (name / `modelled_on` /
  title / `menu_bar` / `key_list` / `profile: KindProfile`), `KindConfigToml`
  (stable TOML schema), and a `KindRegistry` with compiled defaults + user
  overrides from `workspace-kinds/<name>.toml`.
- `BaseKind::External(name)` is ALREADY accepted by the persisted schema for a
  "future Lua/REXX-provided base", though UNRESOLVED in v1 (falls back to a
  built-in with a notice).

What a Kind can carry WITHOUT core changes: a title, a menu bar, a key list, and a
`KindProfile` (edit profile, tab size, line-end mode, command-line position). A
plugin/user can therefore add a "Mainframe Editor" Kind (title `[MF-EDITOR]`, its
own menu bar + key list, an edit profile) as DATA.

What it CANNOT do without core changes:
- A user/external Kind cannot bind to NEW core behaviour: `modelled_on` resolves
  only to a `BuiltinKind` in v1 (`External` is unresolved), so a Kind cannot
  introduce a new Context render arm, a new command environment, or a new editor
  behaviour by itself.
- `KindProfile` has no record-format / FS dimension (same gap as Q3).

So Workspace Kinds give the owner the per-FS CHROME (title/menu/keys/profile) as
pure data, but NOT the per-FS editor BEHAVIOUR or a new CE -- those still need the
Q2/Q3 core hooks. The `External` base is the natural future home for "a plugin
registers a new Kind base", but its resolver is not built.

---

## Q7. Recommendation -- realise the model as extension, flag the small core hooks

### Mapping the owner's model to seams

| Owner model element | Seam | Pure plugin? |
|---|---|---|
| VFS providers per FS (Native/POSIX/Mainframe) | `ff-vfs` `VfsProvider` + `ProviderRegistry` (+ `StorageProvider` below); already built, catalog provider exists | PLUGIN at the ff-vfs layer, BUT the live shell does not register a `ProviderRegistry` yet -- a core WIRING step is needed (not a framework change, but shell wiring). |
| POSIX CE / Mainframe CE (FFCMD fallback) | CR-CH-053 environment model -- needs an `Environment_Registry` + FFEDIT-as-object + data-driven per-kind env lookup | NEEDS a SMALL owner-approved CORE change (open the closed environment set). Then CEs are plugin data/registration. |
| ONE adaptive editor | A per-document `FileSystemProfile`/behaviour hook on `TabState`, read by the existing FFEDIT verbs + editor render surface | NEEDS a SMALL owner-approved CORE change (the hook). Then per-FS profile VALUES are plugin data. Option (a) recommended over per-FS CE (b). |
| ONE FS-aware navigator | Existing `CatalogType {Mainframe,Posix,Native}` match arms; the mainframe arm already exists and is deferred | Mostly completing EXISTING arms (bug-class), not a new seam. A genuinely new FS kind would extend the shared enum (core/shared-crate change). |
| Per-FS chrome (title/menu/keys/profile) | Workspace Kinds registry (`KindConfig`/`KindProfile`, user `workspace-kinds/*.toml`) | PLUGIN/data, no core change. |

### Pure plugin extension (no core change)
- A new `VfsProvider` crate per FS, consuming `ff-vfs` (and `ff-dscatalog` for the
  mainframe one) -- CONSUME, do not modify.
- New Workspace Kinds as `workspace-kinds/*.toml` data (titles, menu bars, key
  lists, profiles) + new `menus/*.toml`.
- New command verbs registered as `Function` Command_Ids or new `menus/*.toml`
  (the ordinary command seam), where they fit a `CommandTarget`.
- Per-FS editor-behaviour PROFILE VALUES, once the Q3 hook exists.

### SMALL core extensions the owner must explicitly approve (framework-conformance.md)
1. **Open the Command Environment set (Q2).** Build the `Environment_Registry`
   the EXPLAINER already sketches: a shell-owned collection of
   `Box<dyn CommandEnvironment>`, a DATA-driven kind->environment lookup (replacing
   the hardcoded `environment_for_kind` match and the `== FfEdit` gate in
   `commands.rs`), and make FFEDIT a real `CommandEnvironment` object. This
   COMPLETES CR-CH-053's intended design rather than inventing anything. Workaround
   to avoid it: none clean -- a plugin cannot claim command-line verbs without a
   registry; the only alternative is a bespoke intercept, which framework-conformance
   explicitly forbids.
2. **Add the editor behaviour-profile hook (Q3, Option a).** A per-document
   `FileSystemProfile` on `TabState`, defaulted to today's native-text behaviour
   (byte-identical), populated at open from the VFS kind, read by the existing
   FFEDIT verbs and the editor render surface for chrome (seq-number ruler, LRECL
   bound). Workaround: none that keeps "ONE editor" -- the only alternative is a
   separate editor per FS, which the owner explicitly rejected.
3. **Register an `ff-vfs` `ProviderRegistry` in the live shell (Q5).** Shell
   wiring (not a framework-type change), needed so plugin-provided providers land
   live. Should be done together with the architecture-revision consolidation.

Each of these is additive and behaviour-preserving when the defaults equal today's
behaviour, so FFWB keeps building and the core behaviour is unchanged until a
plugin opts in.

### Proposed plugin crate layout (consume, do not modify)
- `ff-mf-vfs` (or reuse `ff-dscatalog`'s `CatalogVfsProvider`) -- mainframe
  `VfsProvider`, consuming `ff-vfs` + `ff-dscatalog`.
- `ff-posix-vfs` -- POSIX `VfsProvider` (reconcile with the EXISTING duplicate
  `ff-posix-provider` / `ff-vfs::PosixNativeProvider` per
  architecture-revision-findings; do NOT add a third).
- `ff-ce-mainframe` / `ff-ce-posix` -- the per-FS Command Environments, each
  implementing the (newly-opened) `CommandEnvironment` trait, consuming
  `ff-command` + the editor managers. Register via the new `Environment_Registry`.
- `ff-edit-fsprofile` -- the per-FS `FileSystemProfile` data + the mapping from a
  VFS kind to a profile. Consumes `ff-edit-operations` + `ff-dscatalog` dataset
  attributes (RECFM/LRECL).
- Workspace Kinds + menus as DATA files (no crate).

All register through `ff-desktop`'s existing wiring (Cargo.toml path dep +
registration calls at the documented seams), per the in-process plugin reality.

### Phased delivery (FFWB builds throughout, core untouched until each hook is approved)
0. **(Dependency) Core-team consolidation** from architecture-revision-findings:
   resolve the two catalog crates, the two `StorageProvider` traits, and register
   the `ff-vfs` provider stack live. This UNBLOCKS the VFS/navigator slice; without
   it a mainframe `VfsProvider` has nowhere to land live.
1. **Skeleton + ONE CE vertical slice.** Approve + build the Q2 `Environment_Registry`
   (make FFEDIT an object, open the set); add a trivial POSIX CE that claims ONE
   verb to PROVE the per-FS namespace end-to-end (the EXPLAINER's own 6.3.3
   recommendation). Full-shell first-Tab and env-claim tests.
2. **FS-aware navigator.** Fill in the EXISTING deferred `CatalogType::Mainframe`
   navigator arms (dataset/member browse, edit-open) -- bug-class completion.
3. **Adaptive editor.** Approve + build the Q3 `FileSystemProfile` hook on
   `TabState` (default = native text), populate it on open per FS, and adapt FFEDIT
   verbs + chrome (seq-number ruler, LRECL bound) by reading it. Mainframe CE from
   phase 1 gains record-aware verb behaviour.
4. **VSAM.** Wire the concrete `VsamService` (per sibling findings) behind the
   mainframe VFS provider; the record backends already exist in `ff-dscatalog`.

### Blockers from architecture-revision-findings that gate a PURE-plugin path
- **Two catalog crates (`ff-dscatalog` vs `ff-dataset-catalog`) with divergent
  `Dsorg`/`Recfm`.** The editor FS-profile hook (Q3) needs ONE authoritative RECFM/
  LRECL source; the drift must be resolved first (core-team).
- **Two `StorageProvider` traits.** The mainframe VFS provider's physical seam is
  ambiguous until unified onto `ff-vfs::StorageProvider`.
- **Provider stack not registered live.** Until `ff-desktop` registers an
  `ff-vfs` `ProviderRegistry` at startup, a plugin-provided `VfsProvider` cannot be
  reached by the running navigator/editor. This is the single most important
  wiring prerequisite for the whole model.

These are core-team cleanups and should be scheduled as a DEPENDENCY (phase 0)
before the mainframe VFS provider is wired live.

---

## Key file reference

| Concern | Path |
|---|---|
| Command Environment model | `crates/ff-desktop/src/shell/environment.rs` |
| Single front door (router) | `crates/ff-desktop/src/shell/dispatch.rs` |
| FFEDIT verb bodies | `crates/ff-desktop/src/shell/dispatch_ffedit.rs` |
| FFEDIT claim gate (`== FfEdit`) | `crates/ff-desktop/src/shell/commands.rs` (~138-219) |
| CE explainer / vision | `docs/specs/command-environments/EXPLAINER-command-environments.md`, `environments-vision.md` |
| Editor Context dispatch arm | `crates/ff-desktop/src/shell/render_body.rs` (`FileEditor | Untitled`) |
| Per-tab state | `crates/ff-desktop/src/tab_state.rs` (+ `tab_state_ctors.rs`) |
| Editor render/input adapter | `crates/ff-desktop/src/editor_panel/{mod,input,paint}.rs` + `ff-editor-panel` |
| ISPF edit profile (no FS dim) | `crates/ff-edit-operations/src/profile.rs` (`EditProfile`) |
| Document model (no RECFM) | `crates/ff-document-model/src/{document,line_end,encoding_nav}.rs` |
| Navigator FS-aware arms | `crates/ff-desktop/src/shell/render_body_arms.rs`, `render_nav_expand.rs` |
| Catalog kind enum | `ff-catalog-registry` (`CatalogType {Mainframe,Posix,Native}`, `VirtualCatalog`) |
| Workspace Kinds registry | `crates/ff-desktop/src/workspace_kind/mod.rs` (+ `registry.rs`) |
| Plugin capabilities | `crates/ff-plugin/src/capability.rs` (`Providers`, no CE/Context variant) |
| Framework rules | `.kiro/steering/{framework-conformance,wiring-standard,workspace-conformance}.md` |

---

## Conclusions

1. The owner's model is well-aligned with the ALREADY-BUILT CR-CH-053 Command
   Environment framework: per-context command namespaces with FFCMD as fallback is
   not a new idea here, it is a live, tested mechanism.
2. The pivotal gap is that the environment SET is closed (fixed enum + hardcoded
   match + a single `ffedit_claim` gate, no registry, FFEDIT not an object). The
   per-FS CEs the owner wants need the SMALL, already-intended core change of
   opening that set -- building ON the framework, owner-confirmed per
   framework-conformance.md.
3. The editor is ONE byte/text editor with no FS/record awareness; the adaptive
   editor needs a per-document behaviour-profile hook (Option a), which is a small
   additive core change, after which per-FS values are plugin data. A separate
   editor per FS is correctly rejected.
4. The navigator is already ONE FS-aware Context; the mainframe path exists and is
   deferred -- that work is completing existing arms, not new seams.
5. The plugin system is in-process/compiled-in; VFS providers are pluggable at the
   `ff-vfs` layer but NOT wired live, and there is no plugin capability for a CE or
   a Context. "Plugin" = compiled-in crates registered at documented seams.
6. The duplication/drift in architecture-revision-findings (two catalog crates,
   two StorageProvider traits, no live provider registry) is a hard DEPENDENCY
   (phase 0) for the VFS/navigator slice and should be a core-team cleanup first.

This report implements nothing. Any of the proposed core hooks must run the
requirements gate (workflow.md) and be owner-confirmed as framework changes before
code.
