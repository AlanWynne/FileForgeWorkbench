# Extension Integration Design

## Overview

This design defines the EXTENSION INTEGRATION CONTRACT: the concrete, implement-
able mapping from the already-built `ff-plugin` framework onto the live
`ff-desktop` framework seams, plus the `ff-desktop` extension host that connects
them. It is the extension-facing expression of `.kiro/steering/wiring-standard.md`
and the keystone prerequisite named in `docs/reentry-register.md`.

The `ff-plugin` crate is complete and unit-tested: it has the `FileForgePlugin`
trait, `PluginContext`, the `Capability` model, `PluginRegistry`, lifecycle,
versioning, and security boundaries. What does not exist is any connection to the
running workbench -- no registry is constructed in `ff-desktop`, no production
platform services implement the service traits, no discovery runs at startup, and
there is no mapping from a `Capability` onto the live seams (`resolve_target` /
`CommandTarget`, `WorkspaceContext` + `render_workspace_context`, the Kinds
registry, `WorkspaceDescriptor` persistence, `menus/*.toml`). This design
supplies the host and that mapping, and it carries four explicit owner decision
points (D1-D4) surfaced at the gate.

The design does NOT change any load-bearing framework seam. It requires exactly
one additive core generalization -- a single identifier-keyed extension-Context
dispatch arm (section "Context contribution") -- which is called out explicitly
as the one framework extension point to confirm, built as an addition to the
existing dispatch/nav/focus-latch paths, not a replacement.

## Technology stack (locked once approved)

- Language: Rust (existing workspace).
- Framework crate: `ff-plugin` (BUILT), to be renamed `ff-extension` per D1 on a
  staged plan.
- GUI: egui / eframe, only in `ff-desktop` adapters; the framework crate stays
  GUI-independent.
- Dispatch: the single front door `shell/dispatch.rs::dispatch_command_string` ->
  `resolve_target` (`command_config/mod.rs::builtin_workspace_target_for`) ->
  `shell/target_dispatch.rs::dispatch_command_target`.
- Focus latch: `render_workspace_context` / `apply_interior_focus`.
- Persistence: `WorkspaceDescriptor` + the session manager.
- Config: the configuration system (`extensions.disabled` session key -- the
  renamed, additive, backward-compatible form of `plugins.disabled`, D1 Wave B;
  absent-key default is an empty disabled set, Req 2.7).
- Events: the shared `Arc<Mutex<NotificationQueue>>`.
- Viewer bridge: `ff-viewers` + `plugin_bridge`.
- No new runtime dependencies are introduced by the contract itself.

---

## The extension host (`ff-desktop`)

The host is the integration layer that owns the lifecycle wiring. It is new code
in `ff-desktop`, grouped under a single module (proposed
`crates/ff-desktop/src/extension_host/`) and held on the shell inside one grouped
sub-struct (NOT a new flat `WorkbenchShell` field, per wiring-standard "State
Integration"). Proposed shape:

```
crates/ff-desktop/src/extension_host/
  mod.rs          -- ExtensionHost struct, construction, re-exports
  services.rs     -- production impls of the ff-plugin service traits
  apply.rs        -- apply_capability / revoke_capability onto the live seams
  discovery.rs    -- the D3 in-tree registry enumeration + startup load
```

`ExtensionHost` owns the `PluginRegistry` (`ff-plugin`), the production platform
services, and an index of what each extension currently contributes (so a
disable can revoke exactly those contributions). It exposes:

- `startup(&mut shell_services)` -- enumerate (D3), load, honour the persisted
  `extensions.disabled` set, activate the rest, apply each active extension's
  capabilities onto the live seams (Req 1.1, 2.1, 2.2). `extensions.disabled` is
  the renamed form of the existing `plugins.disabled` session key
  (plugin-manager-ui Req 2), renamed under D1 Wave B. It is an ADDITIVE,
  backward-compatible session-config key owned by the configuration system (NOT a
  framework-type change): when the key is ABSENT (first run, or a session saved
  before the key existed) the host treats the disabled set as EMPTY and activates
  all discovered extensions (Req 2.7).
- `enable(name)` / `disable(name)` -- drive `activate()` / `deactivate()` and
  apply/revoke contributions (Req 2.3, 2.4), persisting the disabled set.
- `shutdown()` -- registry reverse-order deactivate + shutdown within the
  5-second budget (Req 1.4).

### Production platform services

Each `ff-plugin` service trait gets exactly one production implementation in
`services.rs`, each delegating to the SAME seam the core uses (Req 1.2, 1.3):

| Service trait | Production impl delegates to |
|---------------|------------------------------|
| `CommandRegistration` | registers the extension's root(s) + classification into the runtime-extensible root registry consulted by `resolve_target` (see "Command contribution" / D5) |
| `PluginConfigAccess` | the configuration system, namespace-scoped to `[extensions.{name}]` |
| `PluginVfsAccess` | the `ff-vfs::BackendEnvironment` SAVE seam (see "VFS extension point") |
| `PluginEventBus` | the shared `NotificationQueue` (no new channel unless a real cross-thread producer exists) |
| `CapabilityRegistrar` | the `ff-plugin` `CapabilityRegistry`, bridged to the live seams by `apply.rs` |

An extension reaches these ONLY through its context object; no `ff-desktop`
internal API is handed to an extension (Req 1.2).

### Error handling (concrete, per operation)

- Registry construction fails at startup (Req 1.5): RECOVERABLE. The host logs
  ERROR, the shell continues as bare core with zero extensions. No panic.
- A single extension fails to load/initialize/activate: RECOVERABLE and ISOLATED.
  The registry's existing `catch_unwind` path (extension-architecture Req 5.3/5.4)
  sets the extension Failed with a reason string; other extensions and the core
  continue. The reason is surfaced in the Extension Manager (Req 8.4).
- A capability fails to apply onto a seam (e.g. a contributed Context id
  collides): RECOVERABLE. The host logs WARN with the extension name and the
  seam, applies the duplicate rule where relevant (first-wins, Req 3.3), and
  leaves the extension active with its other capabilities applied.
- Disable/enable failure (Req 2.5): RECOVERABLE. Caught, extension set Failed,
  workbench keeps running.
- Shutdown budget exceeded (Req 1.4): the registry forcibly drops remaining
  instances after 5 seconds (existing behaviour); logged at WARN.

All host errors use the `[extension:{name}] operation: description` message
format (carried from extension-architecture design constraint FFW Req 8).

---

## Capability-to-seam mapping (the contract)

Each `ff-plugin::Capability` variant maps to exactly one live seam. `apply.rs`
holds `apply_capability` (active) and `revoke_capability` (on disable), so a
disable removes precisely what an enable added (Req 2.3, 2.6).

| Capability | Live seam | Apply | Revoke |
|------------|-----------|-------|--------|
| `Commands` | `resolve_target` / `CommandTarget` + the runtime-extensible root registry (see D5) | register the ROOT verb + classification into the registry (+ optional alias rows) | deregister the root (aliases first) |
| (menu option, carried on a command capability's metadata) | `menus/*.toml` overlay merged with `DEFAULT_*_TOML` | add in-memory overlay option | remove overlay option |
| `Viewers` | `ff-viewers` `ViewerRegistry` via `plugin_bridge` | register viewer | auto-deregister (existing bridge) |
| `Providers` | `ff-vfs::BackendEnvironment` SAVE seam | register provider | deregister provider |
| (Context, carried on a command/Kind capability) | `WorkspaceContext` + `render_workspace_context` | register Context factory keyed by id | remove factory, close/transform open tabs |
| (Workspace Kind) | Kinds registry | register Kind | remove Kind |
| `LanguageSupport` | language-service seam (future consumer) | register | deregister |
| `ThemeContribution` | `ff-theme` token contribution | register tokens | remove tokens |

### Command contribution (Req 3)

Core commands resolve through `resolve_target` /
`command_config::builtin_workspace_target_for`, and the valid root set is
enumerated by `EnvironmentRegistry::root_commands` / `contains_root`. An extension
command must reach the user through the SAME single front door and the SAME root
registry. The MECHANISM is detailed in section D5: the host registers an
extension's contributed ROOT verb (with its `CommandTarget` classification and
owning environment) into a RUNTIME-EXTENSIBLE root registry, consulted alongside
the builtin classifier (after it, before the shrinking ladder) -- this is the
"runtime arm of the one classifier", NOT a parallel dispatcher or a second
command table. The verb then resolves to a `CommandTarget` identically from the
typed line, a menu option, a key binding, and a toolbar affordance.

Two viable approaches were considered:

1. A bespoke `if upper == "..."` intercept per extension verb in `handle_command`.
2. Making the ONE `resolve_target` classifier and the ONE root registry
   runtime-extensible (D5.2), so an extension root is classified and enumerated by
   the same code paths the built-ins use.

Approach 1 is explicitly an anti-pattern (framework-conformance mechanism 1;
wiring-standard anti-patterns). CHOSEN: approach 2 -- one classifier, one root
registry, made extensible, no second dispatcher. This requires the owner-confirmed
framework change flagged in D5.3. Collisions apply the existing first-wins
duplicate rule with a WARN log, built-ins first (Req 3.3).

### Menu contribution (Req 4)

Built-in menus are code-only `DEFAULT_*_TOML` and are never written to disk
(framework-conformance mechanism 4). An extension contributes menu options as an
IN-MEMORY overlay the host merges with the compiled baseline at render time; the
bare-core `DEFAULT_POM_TOML` is never modified (Req 4.3). A contributed option's
`command` resolves through `resolve_target` exactly like a code-defined option.
Menu-bar entries ride the existing horizontal renderer and honour
`show_in_menu_bar` (Req 4.4). Toolbar affordances invoke
`dispatch_bound_command` (Req 4.5).

### Context contribution (Req 5) -- THE ONE CORE GENERALIZATION

Every built-in Context today has a hard-coded `TabKind` variant and a dedicated
`match` arm in `shell/render_body.rs`. An extension cannot add a core `match` arm.
The contract therefore requires ONE additive generalization:

- Add a single `TabKind::Extension { id }` (or equivalent) variant and ONE
  `match` arm that dispatches ALL extension Contexts, keyed by the extension/
  Context identifier, through the existing `render_workspace_context` single
  focus-latch path.
- The host holds a map from id to an extension-provided Context factory; the one
  arm looks up the factory, does the owned-panel swap (`mem::take` ->
  `render_workspace_context` -> put back), and applies the stashed action.

This is additive: it does not change `resolve_target`, the per-tab nav stack, the
`InteriorFocus` type, or the focus-latch mechanism. It is called out as the ONE
framework extension point to confirm at the gate (Req 10.3). Every extension
Context returns an `InteriorFocus` with a stable first-interior `egui::Id` (or a
documented `none()`), and ships a full-shell first-Tab `egui_kittest` test
(Req 5.4; workspace-conformance). On disable, the factory is removed and any open
tab of that Context is closed or transformed to an "unavailable" notice through
the existing navigation path (Req 5.5).

### Workspace Kind contribution (Req 6)

An extension Kind registers in the existing Kinds registry with name, Tab_Header
tag, display title, menu bar, key list, and profile (Req 6.1), so Title_Line and
Tab_Header behave like built-in Kinds (Req 6.2, framework-conformance mechanism
7). Session persistence uses `WorkspaceDescriptor { CustomWorkspace { kind,
params } }` (Req 6.3, mechanism 6). On restore, if the contributing extension is
absent/disabled, the descriptor resolves to an "unavailable" notice reporting
`InteriorFocus::none()` rather than failing (Req 6.4). On disable, the Kind is
removed from the registry (Req 6.5).

### Viewer and VFS extension points (Req 7)

Viewers reuse the EXISTING `ff-viewers::plugin_bridge`, which already registers
and auto-deregisters plugin viewers on shutdown (consolidation report section 2);
no new mechanism. Provider file I/O goes exclusively through the
`ff-vfs::BackendEnvironment` SAVE seam re-expressed as a stable extension-facing
point. The `BackendEnvironment` store contract is a load-bearing core type; the
record-aware reshape under CR-CH-060 is already owner-confirmed, so the
extension-facing provider point is expressed on the CURRENT contract. Any FURTHER
reshape needed for providers is flagged as an owner-confirmed framework change and
not done without confirmation (Req 7.3; framework-conformance).

---

## The Extension Manager Context (Req 8)

The Plugin Manager Context is already a textbook `WorkspaceContext` wiring
(consolidation report section 3): `PluginManagerPanelState`, a stable filter
`egui::Id`, `InteriorFocus::single`, the owned-panel swap in `render_body.rs`, the
`PLUGINS` -> `plugin_manager` classifier path, Kind/descriptor/session round-trip,
and a passing first-Tab focus test. Its only gap is that its list is populated
only by tests -- there is no live registry behind it.

This design closes that gap by pointing the panel at the `ExtensionHost`'s live
registry (read each frame, immediate-mode, no cached copy) and routing its Enable/
Disable actions through `ExtensionHost::enable` / `disable` via the
`apply_<feature>_action` pending-action pattern (wiring-standard "State
Integration") -- the panel does not mutate the shell from inside render. Detail
view (Req 8.5) reads metadata from the registry entry. Session persistence is the
existing round-trip (Req 8.6).

Rename (D1): the panel, Kind, tag, and verb become the "extension" vocabulary
(`Extension Manager`, `[EXTENSIONS]`, `EXTENSIONS`). The staged rename plan (D1
below) governs when the underlying `plugin_manager` identifiers change. The
Extension Manager is reached by TYPING `EXTENSIONS` and is NOT compiled into the
bare-core POM (see the Extension Manager reachability decision in D1).

---

## Decision points for the owner gate

### D1 -- Rename "plugin" -> "extension" (RECOMMENDED: accept)

RECOMMENDATION: accept "extension" as the canonical name. Rationale: it matches
the VS Code / Kiro lineage this app descends from; it avoids collision with the
Rust/Cargo meaning of "plugin"/"package"; and `.kiro/steering/extension-
terminology.md` already records it as the standing owner direction. The UI surface
becomes the "Extension Manager" (the owner also called it the "Plugin
Administrator"/"Extension Administrator"; "Extension Manager" is the name used
here).

NOT a silent find-replace. This effort is docs-only; the code/spec rename lands in
WAVES, each wave behaviour-preserving and recorded in the change log, riding along
with real work per the terminology steering (no dedicated big-bang pass unless the
owner asks):

- Wave A (docs): author new material (this spec) in the "extension" vocabulary;
  rename spec folders `plugin-architecture` -> `extension-architecture` and
  `plugin-manager-ui` -> `extension-manager-ui`, folding their integration-contract
  content into / cross-referencing this `extension-integration` spec. (Folder
  renames are a bigger step: done when touched, recorded in the change log.)
- Wave B (UI strings + verbs): `PLUGINS` POM/verb -> `EXTENSIONS`; "Plugin
  Manager" title -> "Extension Manager"; `[PLUGINS]` tag -> `[EXTENSIONS]`. Keep
  `PLUGINS` as a hidden alias for one release if the owner wants continuity.
- Wave C (modules/types): `plugin_manager_panel` -> `extension_manager_panel`,
  `KindTag::PluginManager` -> `::ExtensionManager`, etc., via compiler-assisted
  rename.
- Wave D (crates): `ff-plugin` -> `ff-extension`, `ff-mdx-plugin` ->
  `ff-mdx-extension`. Largest blast radius (a public crate name + a persisted Kind
  identifier); scheduled last, landed on its own.

OWNER DECISION (final): D1 is ACCEPTED and is a COMMITTED standing decision
recorded in `.kiro/steering/extension-terminology.md` (gradual, in-place, no
per-rename confirmation, end state zero "plugin"). There is NO "decline keeps
plugin" fallback. This spec is authored fully in the "extension" vocabulary; the
spec-folder renames (`plugin-architecture` -> `extension-architecture`,
`plugin-manager-ui` -> `extension-manager-ui`) are approved to happen when those
folders are next touched. Proceed exactly as Waves A-D describe.

EXTENSION MANAGER REACHABILITY (OWNER-DECIDED): the Extension Manager is reachable
by TYPING the `EXTENSIONS` verb, classified by `resolve_target` as
`CustomWorkspace("plugin_manager" -> "extension_manager")` (the existing `PLUGINS`
-> `plugin_manager` path, renamed under Wave B). It is NOT added to the compiled
bare-core `DEFAULT_POM_TOML` -- the bare core POM stays exactly 0 Settings / 3
Help / X (CR-CH-073). A POM option or menu-bar entry for the Extension Manager, if
ever wanted, is contributed the same way any other menu option is: an in-memory
overlay (Req 4) or a user `menus/pom.toml` -- NOT compiled into the bare core. The
stale `plugin-manager-ui` "POM option 8 / =8" reachability claim is SUPERSEDED.

### D2 -- Catalogs and Files become an extension (RECOMMENDED: confirm)

RECOMMENDATION: confirm. Catalogs/File Catalogs and Files are re-homed OUT of the
bare-core POM and come back ONLY through a Catalogs/Files extension. HOW they
return is fully defined by this contract (Req 9): the extension contributes the
POM options (menu seam), the `CATALOGS` / `FILE CATALOGS` / `FILES` verbs (command
seam), and the Catalog Explorer and File Explorer Contexts and Kinds (Context +
Kind seams). When the extension is absent (bare core / Reset bare) they are gone;
when present and active they appear; when disabled they are revoked.

COORDINATION: a SEPARATE effort (workflow `wf_cd193669ad21dc14`) owns removing
Catalogs/Files from the default POM / bare baseline and has LANDED (CR-CH-073: the
bare-core POM is 0 Settings / 3 Help / X; `CATALOGS` / `FILES` remain typeable)
with OPTION B: only the default POM MENU OPTIONS for Catalogs and Files are
removed; the `CATALOGS` / `FILE CATALOGS` / `FILES` COMMANDS and their Catalog
Explorer / File Explorer CONTEXTS are KEPT INTACT and remain reachable if TYPED.
The backing Contexts are NOT torn down and are NOT replaced with "unavailable"
notices. This is deliberate: keeping the commands and Contexts reachable lets the
Catalogs/Files extension RE-HOME them (re-attaching the POM menu options and
surfacing the Contexts through the extension contract) WITHOUT a teardown/rebuild
of the Context code. This design defines only the RETURN path (re-attaching the
menu options and owning the Contexts/Kinds through the extension) and must not
duplicate the Option B menu-option removal (Req 9.4).

Consequence for Req 9.1: in the bare core after Option B, the Catalogs/Files POM
MENU OPTIONS are absent (the user cannot reach them from the POM), which is the
user-visible "gone from the bare core" state the owner asked for. The commands
remaining typeable is the intended Option B seam the extension builds on; the
extension's job is to re-attach the menu options (and own the Contexts/Kinds as
first-class extension contributions) when it is present and active, and to remove
those menu options again when it is disabled.

### D3 -- Extension Manager load model (RECOMMENDED: in-tree feature-gated registry)

The question: what does "pulling in" a packaged extension MEAN for a single Rust
desktop binary? Two models:

- In-tree, feature-gated crates: each extension is a workspace crate linked into
  `ffwb`, registered in a compile-time registry, and enabled/disabled at runtime
  through the Extension Manager (its `activate`/`deactivate` apply/revoke
  contributions). "Pulling in" = enabling an already-linked extension.
- Dynamic load: extensions are separate `cdylib` artifacts discovered on disk and
  loaded at runtime (the `scan_plugin_directory` path the framework already
  sketches).

TRADEOFF: dynamic load gives true out-of-process-of-build distribution but brings
Rust ABI instability (no stable `dyn` ABI across compiler versions), unsafe
`libloading`, and a much larger security/versioning surface -- heavy for a desktop
binary whose extension set is known at build time. In-tree feature-gating gives
type-safe, ABI-safe, testable integration and still satisfies the user-visible
goal (enable/disable, bare core by default) with none of the ABI risk.

RECOMMENDATION: in-tree, feature-gated registry now. The contract is authored so a
future dynamic-load model can be added behind the SAME `ExtensionHost` discovery
seam (section "discovery.rs") without re-opening the contract -- discovery becomes
pluggable, the rest of the host is unchanged. FLAGGED TRADEOFF: with this model,
adding a brand-new extension still requires a rebuild; only enable/disable is
runtime. If the owner needs runtime install of unbuilt extensions, that is the
dynamic-load model and a larger, separately-gated effort.

### D4 -- SCRM stays core (RECOMMENDED: confirm, out of scope here)

RECOMMENDATION: confirm SCRM (screen-snapshot-scrm; snapshot + capture) stays
CORE, not an extension. From the consolidation report (section 7): `ff-scrm` and
`ff-screen-model` are BUILT and in the retained core set; the `SNAPSHOT` and
`CAPTURE` verbs are wired via the command ladder; the `[REPLAY]` Context exists;
ROADMAP records SCRM done and merged. The owner wants SCRM surfaced via a menu
option and the menu bar -- that wiring is a SEPARATE core effort and is OUT of
scope here. This design only states the core-vs-extension line: SCRM is core; the
extension contract does not carry it.

### D5 -- Extension command contribution to FFCMD (OWNER-RAISED; the "how")

This section was REWORKED after an owner correction verified against the actual
code. The earlier framing ("ALIAS is only a convenience layer; a separate
extension command table is the mechanism") understated the alias model and
mis-located the dynamic-add. The corrected model below is grounded in the real
types: `crate::shell::user_aliases` (`AliasEnvironment`, `UserAliasRowToml`),
`crate::shell::environment_registry::EnvironmentRegistry`
(`contains_root` / `root_commands`), `command_config::builtin_workspace_target_for`
(`BUILTIN_WORKSPACE_VERB_ROOTS` / `BUILTIN_FFCMD_LADDER_VERB_ROOTS`),
`shell::alias_reader::builtin_ffcmd_roots`, and the Alias Editor
(`alias_editor_panel/{state,rows,render}.rs`, `shell/alias_editor.rs`).

#### D5.1 -- What the code actually shows (the three candidates, re-evaluated)

- ALIAS is a FIRST-CLASS candidate, not a mere convenience layer. An alias row
  (`UserAliasRowToml`) is a binding of SURFACE -> (`environment: AliasEnvironment`
  (`ffcmd` | `ffedit`), `canonical` ROOT verb, optional CR-NR-112
  `command_template`). That is precisely "invoke this ROOT verb in this Command
  Environment, optionally with parameter substitution" -- the exact shape an
  extension needs to express an entry verb with a friendly surface. The Alias
  Editor already lets a USER author such rows; aliases are persisted, locale-aware,
  and validated.
- BUT an alias can only target a root that ALREADY EXISTS. The alias save/load
  path validates each row's `canonical` against
  `EnvironmentRegistry::contains_root(row.environment.env_name(), canonical)`
  (`validate_user_alias_rows` in `shell/alias_editor.rs`;
  `revalidate_root_for_env` and the root dropdown source
  `EnvironmentRegistry::root_commands(env)` in `alias_editor_panel/rows.rs`).
  `root_commands` / `contains_root` today enumerate STATIC, COMPILED root sets
  (`builtin_ffcmd_roots()` = `BUILTIN_WORKSPACE_VERB_ROOTS` unioned with
  `BUILTIN_FFCMD_LADDER_VERB_ROOTS`; FFEDIT = `AliasTable::ffedit_english()
  .canonical_verbs_owned()`). An alias to a root NOT in those sets is REJECTED.
- A user-typed REGISTER verb is still NOT the mechanism (unchanged).

So the owner's read is correct: aliases + `resolve_target` are the OPERATING
layer, but they operate over a FIXED root registry. The load-bearing "dynamic add"
is therefore NOT a separate parallel command table bolted onto `resolve_target`;
it is making the root registry (and the classifier that maps a root to a
`CommandTarget`) RUNTIME-EXTENSIBLE so an extension can register its new root(s)
and classification at enable time. Once a root is registered, aliases,
`contains_root`/`root_commands`, the Alias Editor dropdown, and `resolve_target`
all work UNCHANGED over the extended registry.

#### D5.2 -- The chosen model: a runtime-extensible root/classification registry (option (b))

Three possibilities were on the table (owner's framing):

(a) The alias table IS the contribution mechanism (extension contributes alias
    rows); drop the separate command table.
(b) The two are complementary: a brand-new extension ROOT verb must first be
    REGISTERED into the root registry + classifier (because `contains_root`
    validates against a fixed set and the classifier maps only known roots); AFTER
    which aliases and `resolve_target` work unchanged.
(c) A hybrid.

CHOSEN: (b), confirmed against the code. Reasoning: an alias CANNOT create a
capability -- `validate_user_alias_rows` / `revalidate_root_for_env` reject a
canonical that is not already a valid root, and `builtin_workspace_target_for`
only classifies roots it knows. So (a) is impossible as stated: an extension
cannot "just add an alias row" for a verb whose root does not yet exist. The real
primitive is registering the ROOT (and its classification to a `CommandTarget`)
into the registry; aliases are then the optional friendly SURFACE over that root,
authored by the extension or later by the user. My earlier "separate extension
command table" COLLAPSES into exactly this: "the host registers the extension's
root(s) + classification into the registry at enable, and removes them at
disable." There is no parallel table and no second classifier -- the ONE
`resolve_target` classifier and the ONE root registry simply become extensible.

Concretely, `apply_capability` for a `Capability::Commands` at enable:
1. registers each contributed ROOT verb with its `CommandTarget` classification
   and owning environment name (FFCMD for an entry verb) into the (made
   runtime-extensible) registry, so `builtin_workspace_target_for` /
   `resolve_target` classify it and `contains_root`/`root_commands` include it;
2. optionally installs extension-shipped alias rows (surface -> that root in that
   environment, with an optional template) into the live alias store.
`revoke_capability` at disable removes both, in order (aliases first, then the
root), so no alias is left pointing at a removed root.

#### D5.3 -- FRAMEWORK CHANGE FLAG (owner-confirmation required)

Making the root registry and classifier runtime-extensible changes LOAD-BEARING
framework types, so per `framework-conformance.md` it is flagged here as an
owner-confirmed FRAMEWORK CHANGE, NOT an implementation side effect. What it
touches and the blast radius:

- `EnvironmentRegistry::root_commands(env_name)` and `contains_root(env_name,
  root)` are today ASSOCIATED functions over compiled constants (no instance
  state). To be extensible they must consult a runtime set of
  extension-registered roots in ADDITION to the compiled built-ins. Readers that
  must keep working over the extended set: the Alias Editor root dropdown
  (`render_root_combo`), row revalidation (`revalidate_root_for_env`), the filter
  (`render.rs`), and alias save/load validation (`validate_user_alias_rows`). All
  already call these two functions, so they benefit automatically IF the functions
  gain a registry instance to read; the signature change (associated fn ->
  method, or an added registry parameter) is the mechanical cost.
- The classifier `builtin_workspace_target_for` (and thus `resolve_target`) today
  maps a fixed verb set to `CommandTarget`, guarded by a drift test co-locating
  `BUILTIN_WORKSPACE_VERB_ROOTS` with the classifier. Extension roots must be
  classified too, via a runtime lookup consulted alongside the builtin classifier
  (after it, before the ladder) -- the same insertion point the earlier draft
  called the "extension command table", now understood as "the runtime arm of the
  one classifier", not a parallel dispatcher.
- Coexistence with compiled roots: built-ins remain compiled and
  authoritative/first; extension roots are additive and removed on disable; a
  collision applies the existing first-wins + WARN rule (Req 3.3). The drift guard
  stays valid because the built-in list is unchanged; a separate test covers the
  runtime set.
- `CommandTarget` itself is UNCHANGED (extensions reuse the existing variants).
  The change is to the registry/classifier EXTENSIBILITY, not to the target type.

If the owner does NOT confirm this framework change, the fallback is narrower and
weaker: extensions may only contribute roots that are ALREADY compiled built-ins
(e.g. re-homing `EDIT`/`FILES`/`CATALOGS`, which already exist as roots), plus
aliases over them -- which covers the Catalogs/Files re-home (D2) and the Editor
entry verb `EDIT` (already a `BUILTIN_FFCMD_LADDER_VERB_ROOTS` member) but NOT a
genuinely new verb. The recommendation is to CONFIRM the runtime-extensible
registry so new extensions can contribute genuinely new verbs.

#### D5.4 -- IN-CONTEXT (environment-owned) verbs and the CE registry

A verb that is the ENTRY to an extension Context (e.g. `EDIT` opens the Editor
Context) is an FFCMD root classified to a `CustomWorkspace`/`Function` target and
routed via `navigate_to` -- covered by D5.2/D5.3. Once the Context is focused, its
IN-BUFFER verbs (`FIND`, `CHANGE`, ...) are owned by the Context's Command
Environment (FFEDIT), selected by the EXISTING per-`TabKind` active-env derivation
(`EnvironmentRegistry::active_name`), not by FFCMD classification. These are
separate concerns: the entry verb lives in FFCMD; the in-buffer verbs live in the
Context's CE.

For the in-scope parked-domain re-entries (Editor, File navigator, Catalog, VFS),
their environments (FFEDIT, FFNAV, and vision-only FFAMS/FFVFS) are ALREADY in the
command-environments catalogue and register when their subsystem is built -- the
`active_name` derivation already maps editor/navigator `TabKind`s to FFEDIT/FFNAV.
So in-buffer verb ownership needs NO additional runtime-extensible CE-object
registry for the in-scope work; the one additive `TabKind::Extension { id }` arm
(Req 10.3) plus the existing derivation suffices. IF a future extension must
contribute a genuinely NEW claiming Command Environment OBJECT that the active-env
gate selects dynamically (beyond the catalogued FF* names), that is a FURTHER
runtime-extensible-CE-registry framework change, flagged for owner confirmation at
that time and NOT assumed here.

#### D5.5 -- Canonical example: EDIT -> Editor Context, end to end (corrected model)

1. The (future) Editor extension is enabled. Its activation calls the host's
   `apply_capability` for a `Capability::Commands` carrying the ROOT verb `EDIT`.
   The host REGISTERS `EDIT` into the runtime-extensible root registry with its
   FFCMD classification (a `CustomWorkspace`/`Function` target that opens the
   Editor Context). `EDIT` is in fact already a compiled
   `BUILTIN_FFCMD_LADDER_VERB_ROOTS` member, so for the Editor re-home this step
   is a no-op over the built-ins; the mechanism matters for a genuinely new verb.
   The extension MAY also install an alias row `ECOB -> (ffcmd, EDIT,
   "FFWB.COBOL.MASTER(&1)")` into the live alias store (the same shape the Alias
   Editor authors).
2. The user types `EDIT ffwb.cobol.master` (or `ecob helloW`, which the alias
   layer resolves to `EDIT ffwb.cobol.master(helloW)` BEFORE classification, per
   CR-NR-112 positional substitution).
3. `dispatch_command_string` runs the prelude (unchanged), then the active-env
   step (on a non-editor Context this is FFCMD, which falls through), then
   `resolve_target`: the builtin classifier plus the runtime-registered root set
   classify `EDIT` as the Editor Context target.
4. `dispatch_command_target` routes the `CustomWorkspace` target through
   `navigate_to`, transforming the active tab into the Editor Context (the single
   `TabKind::Extension { id }` arm renders it via `render_workspace_context`).
5. Once the Editor Context is focused, `active_name` selects the editor's Command
   Environment (FFEDIT), so subsequent in-buffer verbs (`FIND`, `CHANGE`, ...) are
   owned by FFEDIT, not FFCMD -- the EXISTING CE behaviour, no new mechanism.
6. On disable, `revoke_capability` removes the `ECOB` alias row then the `EDIT`
   runtime registration (only if `EDIT` was extension-registered rather than a
   compiled built-in); `EDIT` no longer classifies via the extension and the
   Editor Context is no longer navigable (Req 5.5).

#### D5.6 -- Load-time programmatic contribution; user and extension aliases coexist

The contribution is the host's `apply_capability` PROGRAMMATIC API invoked from an
extension's activation at LOAD/ENABLE time -- NOT a user-typed `REGISTER` verb. The
live command set is a deterministic function of the enabled-extension set plus the
persisted `extensions.disabled` list.

User-authored aliases and extension contributions COEXIST cleanly because they
live in different layers the code already separates: user aliases are the existing
USER override store (`user_aliases` / the Alias Editor), while extension-registered
roots and extension-shipped aliases are HOST-APPLIED at enable and removed at
disable. A user alias may target an extension-registered root (it validates once
the extension is active); an extension-shipped alias is additive sugar over the
extension's own root. The ROOT is always what is recorded and persisted
(command-semantics Req 12.5, 13.3); the surface is a pre-resolution display/input
form.

---

## Testability

- Unit-testable: an extension-registered root is classified by `resolve_target`
  and included by `EnvironmentRegistry::root_commands` / `contains_root` (and
  therefore offered by the Alias Editor dropdown and accepted by alias
  validation) while active, and absent when disabled; `apply_capability` /
  `revoke_capability` (a disabled extension's root no longer classifies, its menu
  overlay option is gone, its shipped alias no longer validates); the
  `extensions.disabled` persistence round-trip; the duplicate-root first-wins +
  WARN behaviour; the drift guard over the compiled built-in root list still
  passes with a separate test over the runtime set.
- Integration-testable (full-shell `egui_kittest`): the Extension Manager
  first-Tab focus (exists today, renamed); an extension Context's first-Tab focus
  landing on its reported first interior control (Req 5.4); a session round-trip
  restoring an extension Context, and the "unavailable" notice when the extension
  is absent (Req 6.4).
- The in-tree D3 model is the choice that KEEPS this testable: a dynamic-load
  model could not be driven headlessly without real artifacts, which is a further
  reason to prefer it.

---

## Migration plan summary (D1, staged)

| Wave | Scope | Blast radius | When |
|------|-------|--------------|------|
| A | Docs: author in "extension" vocab; rename the two spec folders, fold into this spec | docs only | this effort (folder renames when touched) |
| B | UI strings + verbs (`EXTENSIONS`, "Extension Manager", `[EXTENSIONS]`) | ff-desktop strings | rides next ff-desktop work |
| C | Modules/types (`extension_manager_panel`, `KindTag::ExtensionManager`) | ff-desktop internal | compiler-assisted, when touched |
| D | Crates (`ff-extension`, `ff-mdx-extension`) + persisted Kind id | public crate name + session id | last, on its own |

This effort performs Wave A authoring only (new `extension-integration` spec in
the extension vocabulary). The spec-folder renames and all of B-D are recorded
here as the plan and executed later per the terminology steering; if the owner
declines D1, none of B-D happen and the docs revert to "plugin".
