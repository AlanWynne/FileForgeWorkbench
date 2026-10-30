# Extension Integration Requirements

## Introduction

This sub-project defines the EXTENSION INTEGRATION CONTRACT for
FileForgeWorkbench: the single, authoritative statement of how an extension (the
renamed "plugin") integrates into the live `ffwb` workbench through the EXISTING
framework seams. It is the keystone contract named as a cross-cutting
prerequisite in `docs/reentry-register.md` -- every parked domain (Editor, File
navigator, VFS providers, Catalog, Volume) re-enters as an extension against the
contract defined here, so this contract must be settled before the first
parked-domain re-entry.

This spec CONSOLIDATES and supersedes the integration-contract concerns
previously spread across two specs:

- `docs/specs/plugin-architecture/` -- the `ff-plugin` framework crate (trait,
  context, registry, capability model, lifecycle, versioning, security). That
  crate is BUILT and fully unit-tested in isolation.
- `docs/specs/plugin-manager-ui/` -- the Plugin Manager Context UI. The Context
  is BUILT and correctly wired as a `WorkspaceContext`, but has no live registry
  behind it.

In particular, the `plugin-manager-ui` "POM option 8 / =8" reachability claim is
SUPERSEDED by Req 8.1 (and design D1): the Extension Manager is reachable by
TYPING the `EXTENSIONS` verb and is NOT compiled into the bare-core
`DEFAULT_POM_TOML`. The two source specs are no longer authoritative on how the
Extension Manager is reached from the POM.

The gap both specs leave open, and the reason this spec exists, is that
`ff-plugin` is NOT connected to the running workbench: there is no live registry
constructed in `ff-desktop`, no production platform services, no discovery or
load at startup, and -- critically -- no mapping from the typed `Capability`
model onto the live framework seams (`resolve_target` / `CommandTarget`,
`WorkspaceContext` + `render_workspace_context`, the Kinds registry,
`WorkspaceDescriptor` persistence, `menus/*.toml`). This spec defines that
mapping. It is the extension-facing expression of
`.kiro/steering/wiring-standard.md`.

The design document (`design.md`) carries four explicit owner decision points
(D1 rename, D2 Catalogs/Files as an extension, D3 Extension Manager load model,
D4 SCRM stays core). This requirements document is authored under the RECOMMENDED
positions for those decisions and notes each assumption inline; the owner gate
confirms or overrides them.

## Assumptions (to be confirmed at the owner gate)

- A1 (D1): The concept is renamed "plugin" -> "extension" per the standing owner
  decision and `.kiro/steering/extension-terminology.md`. This spec is authored
  in the "extension" vocabulary. The underlying BUILT crate is still named
  `ff-plugin` in the tree today; the migration is staged (design D1), so the
  requirements refer to the concept as "extension" and the current crate as
  "`ff-plugin` (to be renamed `ff-extension`)" where a concrete path matters.
- A2 (D2): Catalogs and Files have LEFT the bare-core POM via a SEPARATE effort
  that has LANDED (CR-CH-073: the bare-core POM is 0 Settings / 3 Help / X;
  `CATALOGS` / `FILES` remain typeable); this spec defines only how they RETURN as
  an extension.
- A3 (D3): The recommended load model for this desktop Rust binary is a REGISTRY
  of in-tree, feature-gated extension crates linked into the binary and
  enabled/disabled at runtime through the Extension Manager -- NOT an external
  dynamically-loaded (`cdylib`) binary artifact. The contract is authored to
  keep a future dynamic-load model possible without re-opening it.
- A4 (D4): SCRM stays CORE and is out of scope here.

## Glossary

| Term | Definition |
|------|-----------|
| Extension | A unit of optional functionality implementing the `FileForgePlugin` trait (to be renamed `FileForgeExtension`), registered with the Extension_Registry. Formerly "plugin". |
| Extension_Registry | The runtime store of all discovered/loaded extensions and their states, owned by `ff-plugin` (to be `ff-extension`) and CONSTRUCTED and owned by `ff-desktop`. Formerly "plugin registry". |
| Extension Manager | The Workspace Context that lists, enables, disables, and shows detail for extensions. Formerly "Plugin Manager"; the owner also refers to it as the "Extension Administrator" / "Plugin Administrator". |
| Capability | A typed contribution an extension makes: `Commands`, `Viewers`, `Providers`, `LanguageSupport`, `ThemeContribution` (the existing `ff-plugin::Capability` variants). |
| Contribution seam | A framework seam through which a capability becomes live: command dispatch (`resolve_target` / `CommandTarget`), menu (`menus/*.toml`), Workspace Context (`WorkspaceContext` + `render_workspace_context`), Workspace Kind (Kinds registry), viewer bridge (`ff-viewers`), VFS SAVE seam (`BackendEnvironment`). |
| Platform services | The production implementations, owned by `ff-desktop`, of the `ff-plugin` service traits (`CommandRegistration`, `PluginConfigAccess`, `PluginVfsAccess`, `PluginEventBus`, `CapabilityRegistrar`) that an extension reaches ONLY through its context object. |
| Extension host | `ff-desktop`'s integration layer that constructs the registry, implements platform services, runs discovery/load at startup, and applies each capability onto the correct contribution seam. |

---

## Requirement 1: Extension Host and Live Registry

**User Story:** As a platform developer, I want `ff-desktop` to construct and own
a live Extension_Registry wired to production platform services, so that the
built `ff-plugin` framework is actually connected to the running workbench
instead of only unit-tested in isolation.

**Source:** Consolidation report section 9 (the central gap); reentry-register
cross-cutting prerequisite.

### Acceptance Criteria

1. WHEN the workbench starts, THE extension host SHALL construct exactly one
   Extension_Registry instance owned by the shell (held inside a grouped
   sub-struct on `WorkbenchShell`, not a new flat field), and SHALL make it
   reachable by the Extension Manager Context and the contribution seams.
2. THE extension host SHALL provide one production implementation of each
   `ff-plugin` service trait (`CommandRegistration`, `PluginConfigAccess`,
   `PluginVfsAccess`, `PluginEventBus`, `CapabilityRegistrar`) and SHALL inject
   them into each extension's context -- an extension SHALL reach platform
   services ONLY through its context object, never through an `ff-desktop`
   internal API.
3. WHEN an extension calls a platform service through its context, THE
   production implementation SHALL route the call through the SAME framework
   seam the equivalent core feature uses (command dispatch through
   `resolve_target` / `CommandTarget`, config through the configuration system,
   file access through the VFS SAVE seam, events through the shared
   `NotificationQueue`) and SHALL NOT introduce a parallel dispatcher, a second
   navigation stack, or a bespoke command intercept.
4. WHEN the workbench shuts down, THE extension host SHALL deactivate and shut
   down all active extensions through the registry's existing lifecycle path
   (reverse dependency order, the 5-second budget defined in extension-
   architecture Req 5.5) before the process exits.
5. IF the Extension_Registry fails to construct at startup, THEN THE workbench
   SHALL continue to start as a bare core with zero extensions, SHALL log an
   ERROR-level record naming the failure, and SHALL NOT crash.

---

## Requirement 2: Extension Discovery, Load, Enable, and Disable

**User Story:** As a workbench user, I want extensions to be discovered and
loaded at startup and to enable or disable them at runtime, so that I control
which optional features are active without rebuilding or restarting.

**Source:** extension-architecture Req 3; plugin-manager-ui Req 2; D3.

### Acceptance Criteria

1. WHEN the workbench starts, THE extension host SHALL enumerate the available
   extensions (per the D3 load model: the in-tree feature-gated extension
   registry) and transition each to the Discovered state, then load each through
   the existing discover -> load -> validate -> initialize -> activate sequence
   (extension-architecture Req 3.2).
2. THE extension host SHALL respect the persisted disabled set
   (`extensions.disabled: Vec<String>` in the session configuration): an
   extension named in that set SHALL be loaded but left Inactive at startup, not
   activated.
3. WHEN the user disables an active extension in the Extension Manager, THE host
   SHALL call the extension's `deactivate()` lifecycle method, remove every
   capability it contributed from the live seams (its commands, menu options,
   Contexts, Kinds, viewers), set its state to Inactive, and add its name to the
   persisted disabled set.
4. WHEN the user enables an inactive extension, THE host SHALL call `activate()`,
   re-apply every capability it contributes onto the live seams, set its state to
   Active, and remove its name from the persisted disabled set.
5. IF enabling or disabling an extension fails (lifecycle method returns an error
   or panics), THEN THE host SHALL catch the failure (per extension-architecture
   Req 5.3/5.4), set the extension to the Failed state with the failure reason,
   leave the rest of the workbench running, and surface the reason in the
   Extension Manager -- THE workbench SHALL NOT crash.
6. WHEN an extension is disabled, THE capability it contributed SHALL be absent
   from every seam it touched (the command no longer resolves, the menu option is
   gone, the Context and Kind are no longer navigable) until it is re-enabled.
7. WHEN the session configuration has no `extensions.disabled` key (first run, or
   a pre-existing session saved before the key existed), THE host SHALL treat the
   disabled set as empty and activate all discovered extensions. The
   `extensions.disabled` key is the renamed form of the existing `plugins.disabled`
   key (plugin-manager-ui Req 2), renamed under the D1 plugin -> extension
   migration (Wave B); it is an ADDITIVE, backward-compatible session-config key
   owned by the configuration system, NOT a framework-type change.

---

## Requirement 3: Command Contribution Seam

**User Story:** As an extension developer, I want to contribute commands that
reach the user through the same single dispatch front door as core commands, so
that typed, menu, key, and toolbar seams all resolve my command identically.

**Source:** wiring-standard "Command Registration"; framework-conformance
mechanism 1; extension-architecture Req 2/4.

### Acceptance Criteria

1. WHEN an active extension contributes a `Capability::Commands`, THE host SHALL
   register each command as a ROOT in the runtime-extensible root registry
   (Requirement 11) so that it resolves through the SINGLE front door
   (`shell/dispatch.rs::dispatch_command_string` -> `resolve_target`) as a
   `CommandTarget` (`Function`, `CustomWorkspace`, `Menu`, or `Macro`), reached
   identically from the typed command line, a menu option, a key binding, and a
   toolbar affordance. The MECHANISM is the runtime-extensible root registry of
   Requirement 11, not a parallel command table or a second classifier.
2. THE host SHALL NOT satisfy an extension command with a bespoke
   `if upper == "..."` intercept, a second dispatcher, or any path outside
   `resolve_target` / `dispatch_command_target` (framework-conformance
   mechanism 1).
3. WHEN two active extensions (or an extension and the core) contribute the same
   command verb, THE host SHALL apply the existing duplicate-capability rule
   (extension-architecture Req 3.5: first-registered wins as default, alternates
   remain queryable) and SHALL log a WARN-level record naming the collision.
4. WHEN an extension that contributed a command is deactivated, THE command verb
   SHALL no longer resolve to that extension through `resolve_target`.
5. THE command verb an extension contributes SHALL be namespaced or validated so
   it cannot shadow a reserved core verb without the first-wins rule and the WARN
   log of AC 3 applying.

---

## Requirement 4: Menu and Menu-Bar Contribution Seam

**User Story:** As an extension developer, I want to contribute menu options
(including POM and menu-bar entries) that behave exactly like code-defined
options, so that my extension's features are reachable from the menus a user
already knows.

**Source:** wiring-standard "Menu Integration" / "Toolbar Integration";
framework-conformance mechanisms 2 and 4; D2.

### Acceptance Criteria

1. WHEN an active extension contributes a menu option, THE host SHALL surface it
   through the existing menu model (a `menus/*.toml` overlay contributed by the
   extension, or an in-memory menu overlay the host merges with the compiled
   `DEFAULT_*_TOML`), with the option's `command` value resolving through
   `resolve_target` exactly like a code-defined option.
2. THE host SHALL NOT write any built-in or default menu file to disk
   (framework-conformance mechanism 4) -- an extension's contributed menu
   content SHALL be composed in memory or supplied as the extension's own
   package data, never materialized into the user `menus/` directory.
3. WHEN an extension contributes a POM option, THE option SHALL appear in the POM
   only while the extension is active, and SHALL disappear from the POM when the
   extension is disabled or absent -- the compiled bare-core `DEFAULT_POM_TOML`
   SHALL NOT be modified to carry extension options.
4. WHEN an extension contributes a menu-bar entry, THE entry SHALL be rendered by
   the existing horizontal menu-bar renderer and SHALL honour `show_in_menu_bar`
   the same way a code-defined option does.
5. WHEN an extension contributes a toolbar or other UI affordance, THE affordance
   SHALL invoke the command through the existing `dispatch_bound_command` seam
   and SHALL NOT call the extension's handler directly (wiring-standard "Toolbar
   Integration").

---

## Requirement 5: Workspace Context Contribution Seam

**User Story:** As an extension developer, I want to contribute a Workspace
Context that participates in the unified tab-order and focus model, so that my
Context behaves like every built-in Context without the core editing a `match`
arm for it.

**Source:** wiring-standard "View Integration"; workspace-conformance
(CR-CH-023); framework-conformance mechanism 5; master screen design directive
(`docs/specs/master-screen-directive/`, Req 6 -- the extension side of the
cross-reference).

**Cross-reference:** The full every-Context contract an extension Context must
satisfy (focus + save-convention-if-editable + ScreenProvider) is defined ONCE in
`docs/specs/master-screen-directive/requirements.md`. AC 6 below binds an
extension Context to it. This spec does NOT duplicate those criteria.

### Acceptance Criteria

1. WHEN an active extension contributes a `Context` capability, THE host SHALL
   render it through the SINGLE focus-latch path
   (`render_workspace_context`, or `apply_interior_focus` for a documented
   variance) -- the host SHALL NOT hand-write a per-Context focus ring.
2. THE extension's contributed Context SHALL return an `InteriorFocus` from its
   render (a stable first-interior `egui::Id` via
   `InteriorFocus::single(id)` / `{ first, last }`, or `InteriorFocus::none()`
   for a documented no-interior Context).
3. THE host SHALL dispatch an extension Context without adding a bespoke
   hard-coded `TabKind` match arm per extension: it SHALL route through a single
   extension-Context arm keyed by the extension/Context identifier, so a new
   extension Context requires no new core `match` arm (framework-conformance
   mechanism 5; this is the extension-facing generalization the core currently
   lacks).
4. WHEN the first Tab is pressed from the command field of an extension Context,
   THE focus SHALL land EXACTLY on the Context's reported first interior control
   (no phantom stop), provable by a full-shell `egui_kittest` first-Tab test.
5. WHEN an extension contributing a Context is deactivated, THE Context SHALL no
   longer be navigable and any open tab of that Context SHALL be closed or
   transformed to an "unavailable" notice through the existing navigation path
   (not left dangling).
6. AN extension-contributed Context SHALL satisfy the MASTER SCREEN DESIGN
   DIRECTIVE (`docs/specs/master-screen-directive/requirements.md`) in full, not
   only the focus latch of AC 1-4: it SHALL (a) report interior focus through the
   single latch path (AC 1-4 / master directive Req 2); (b) WHERE the Context is
   EDITABLE, comply with the ISPF save convention
   (`docs/project-management/project-master/requirements.md` Requirement 11;
   master directive Req 3) by wiring the CR-CH-071 editable-Context seam rather
   than an in-panel autosave; and (c) provide a per-Context `ScreenProvider` so
   SNAPSHOT / CAPTURE work on it (screen-snapshot-scrm Req 2; master directive
   Req 4), OR declare the explicit documented opt-out for a Context that has no
   logical screen. THIS is a CROSS-REFERENCE to the master directive; the
   criteria are NOT duplicated here. (framework-conformance: all three contracts
   are delivered on existing seams.)

---

## Requirement 6: Workspace Kind Contribution Seam

**User Story:** As an extension developer, I want to contribute a Workspace Kind
so that my Context has a title, menu bar, key list, and profile consistent with
built-in Kinds and persists across sessions.

**Source:** wiring-standard "Feature Registration"; workspace-kinds (CR-NR-090);
framework-conformance mechanisms 3, 6, 7.

### Acceptance Criteria

1. WHEN an active extension contributes a Workspace Kind, THE host SHALL register
   it in the existing Kinds registry with its Kind name, Tab_Header tag, display
   title, menu bar, key list, and profile, the same way a built-in Kind is
   registered.
2. WHEN an extension Context is open, THE Title_Line SHALL use the Kind's
   descriptive display title and the Tab_Header SHALL use the Kind's `[XXX]` tag
   (framework-conformance mechanism 7), consistent with built-in Contexts.
3. WHEN the workbench persists a session with an extension Context open, THE
   session SHALL store a `WorkspaceDescriptor` (`CustomWorkspace { kind, params }`)
   identifying the extension Kind, and SHALL restore it by re-opening that
   descriptor -- provided the contributing extension is present and active at
   restore time (framework-conformance mechanism 6).
4. WHEN the workbench restores a session whose descriptor names an extension Kind
   whose contributing extension is ABSENT or disabled, THE host SHALL replace
   that Workspace with an "unavailable" notice (reporting
   `InteriorFocus::none()` through the single latch path) rather than failing the
   restore or crashing.
5. WHEN an extension contributing a Kind is deactivated, THE Kind SHALL be
   removed from the Kinds registry so it is no longer selectable or navigable.

---

## Requirement 7: Viewer and VFS Extension Points

**User Story:** As an extension developer, I want stable extension points for
file viewers and for the file SAVE path, so that viewer and provider extensions
(including the parked VFS/Catalog domains) can re-enter without changing a core
seam.

**Source:** consolidation report sections 2 (ff-viewers, plugin_bridge) and 9;
reentry-register (the VFS SAVE seam prerequisite); framework-conformance
(`BackendEnvironment` note under CR-CH-060).

### Acceptance Criteria

1. WHEN an active extension contributes a `Capability::Viewers`, THE host SHALL
   register the viewer into the `ff-viewers` `ViewerRegistry` through the
   existing `plugin_bridge`, and SHALL auto-deregister it when the extension
   shuts down.
2. WHEN an active extension contributes a `Capability::Providers` that performs
   file I/O, THE extension SHALL reach the file store ONLY through the
   `ff-vfs::BackendEnvironment` SAVE seam re-expressed as a stable
   extension-facing extension point -- the extension SHALL NOT use `std::fs`
   directly (extension-architecture Req 7.2).
3. IF a change to the `BackendEnvironment` store contract is required to express
   the provider extension point, THEN THE design SHALL flag it as an
   owner-confirmed framework change (framework-conformance; CR-CH-060) and SHALL
   NOT reshape the contract without that confirmation.
4. WHEN a viewer or provider extension is deactivated, THE host SHALL remove its
   viewer and provider registrations so the capability is no longer available.

---

## Requirement 8: Extension Manager Context

**User Story:** As a workbench user, I want an Extension Manager Context to list,
enable, disable, and inspect extensions, so that I can bring extensions in and
control them from one place.

**Source:** plugin-manager-ui Req 1-4 (renamed and reconciled with the live
registry); D3.

### Acceptance Criteria

1. WHEN the user issues the Extension Manager command (the renamed `EXTENSIONS`
   verb; see design D1 for the reachability decision), THE workbench SHALL open
   the Extension Manager Context, resolving the verb through `resolve_target` as a
   `CustomWorkspace` target (the existing `PLUGINS` -> `plugin_manager` path,
   renamed). THE Extension Manager SHALL NOT be compiled into the bare-core
   `DEFAULT_POM_TOML`; a POM option or menu-bar entry for it, if ever wanted, is
   contributed the same way any other menu option is (an in-memory overlay,
   Requirement 4, or a user `menus/pom.toml`).
2. THE Extension Manager Context SHALL display a scrollable, alphabetically
   sorted list of all extensions in the live registry, each showing name,
   version, description, state (Active / Inactive / Failed), and contributed
   capabilities, with a filter field as the first interior control (stable
   `egui::Id`, `InteriorFocus::single`).
3. WHEN an extension is Active, THE Context SHALL offer a Disable action; WHEN it
   is Inactive, THE Context SHALL offer an Enable action; each action SHALL drive
   the enable/disable behaviour of Requirement 2.
4. WHEN an extension is Failed, THE Context SHALL display the failure reason
   alongside the entry.
5. WHEN the user selects an extension, THE Context SHALL show a detail area with
   full description, author, licence, homepage, capabilities, contributed
   commands with any bound shortcut, and configuration keys with a link to the
   Settings Context filtered to the extension namespace.
6. WHEN the workbench exits with an Extension Manager tab open, THE session SHALL
   persist and restore it, reloading live registry state on restore.

---

## Requirement 9: Catalogs and Files Return as an Extension

**User Story:** As a workbench user, I want Catalogs and Files to come back only
when their extension is pulled in, so that the bare core stays minimal and the
feature set grows by choice.

**Source:** owner messages 3-4; consolidation report section 8; D2.

### Acceptance Criteria

1. WHEN no Catalogs/Files extension is present (bare core or after Reset bare),
   THE POM SHALL NOT list a Catalogs option or a Files option (the user-visible
   "gone from the bare core" state). PER THE APPROVED OPTION B of the separate
   removal effort (design D2), the `CATALOGS` / `FILE CATALOGS` / `FILES` commands
   and their Contexts are KEPT reachable-if-typed in the bare core and are NOT
   torn down; this spec SHALL NOT change that. The extension's contribution
   (AC 2) is to RE-ATTACH the POM menu options and OWN the Contexts/Kinds as
   first-class extension contributions, not to re-create torn-down Contexts.
2. WHEN the Catalogs/Files extension is present and active, THE extension SHALL
   re-introduce the Catalogs and Files POM options (Requirement 4), the
   `CATALOGS` / `FILE CATALOGS` / `FILES` command verbs (Requirement 3), and the
   Catalog Explorer and File Explorer Contexts and Kinds (Requirements 5 and 6)
   through the extension contract -- with NO change to the compiled bare-core
   `DEFAULT_POM_TOML`.
3. WHEN the Catalogs/Files extension is disabled, THE Catalogs and Files POM menu
   options SHALL be removed (Requirement 2.6), returning the workbench to the
   bare-core state of AC 1 (menu options gone; per Option B the base commands
   remain typeable and the Contexts are not torn down).
4. THE removal of Catalogs and Files from the bare-core POM is OUT of scope for
   this spec (a separate in-flight effort owns it); this requirement defines only
   how they RETURN as an extension and SHALL be coordinated with, not duplicate,
   that removal.

---

## Requirement 10: Framework Conformance of the Contract

**User Story:** As a maintainer, I want the extension contract to build strictly
on the existing framework seams, so that extensions cannot re-open the bug
classes the framework was built to close.

**Source:** framework-conformance.md (all mechanisms); wiring-standard.md.

### Acceptance Criteria

1. THE extension contract SHALL express every capability through an EXISTING
   framework seam (single `resolve_target` dispatch, per-tab navigation via
   `navigate_to`, the single `render_workspace_context` focus latch, the Kinds
   registry, `WorkspaceDescriptor` persistence, code-only `DEFAULT_*_TOML`
   menus) and SHALL NOT introduce a parallel mechanism.
2. IF any capability genuinely cannot be expressed without altering a core
   framework mechanism (a new dispatch path, a second nav stack, a per-Context
   focus mechanism, a new persistence format, or reshaping a public framework
   type such as `CommandTarget`, `WorkspaceContext`, `InteriorFocus`,
   `WorkspaceDescriptor`, `KindConfig`, or `BackendEnvironment`), THEN THE design
   SHALL flag it explicitly as an owner-confirmed FRAMEWORK CHANGE and SHALL NOT
   proceed on it without that confirmation.
3. THIS contract requires TWO owner-confirmed additive framework extension points,
   each called out in the design and built as an ADDITION to (not a replacement
   of) the existing dispatch, nav, and focus-latch paths:
   a. a single extension-Context dispatch arm keyed by identifier (Requirement
      5.3) so a new extension Context needs no new hard-coded `match` arm; and
   b. a runtime-extensible root registry + classifier (Requirement 11.5) so an
      extension can register a new command root that `resolve_target`,
      `root_commands` / `contains_root`, and the alias layer all honour while it
      is active.
   Both are flagged as FRAMEWORK CHANGES per AC 2 and are owner-confirmed before
   implementation.

---

## Requirement 11: Command Contribution to FFCMD -- Runtime-Extensible Root Registry

**User Story:** As an extension developer, I want a single, unambiguous way to add
a new command verb (root) to the workbench, so that aliases, the classifier, and
the Command Environment model operate over my verb without a parallel dispatcher.

**Source:** owner D5 (how extensions dynamically add commands to FFCMD, corrected
against the code); verified against `EnvironmentRegistry` (`contains_root` /
`root_commands`), `command_config::builtin_workspace_target_for`,
`shell::user_aliases` (`AliasEnvironment` / `UserAliasRowToml`), the Alias Editor;
command-semantics Req 12-14; command-environments (CR-CH-053); design D5.

### Acceptance Criteria

1. THE primitive by which an extension creates a NEW command verb SHALL be the
   host's load-time `apply_capability` REGISTERING the verb as a ROOT -- with its
   `CommandTarget` classification and owning environment name -- into a
   RUNTIME-EXTENSIBLE root registry, so that `resolve_target` classifies it and
   `EnvironmentRegistry::root_commands` / `contains_root` include it while the
   extension is active; THE contribution SHALL NOT be a user-typed `REGISTER` verb.
2. AN alias SHALL be recognized as a first-class binding of SURFACE ->
   (environment, ROOT, optional parameter template), NOT merely a convenience
   layer; BUT an alias SHALL NOT itself create a capability -- it SHALL only
   target a ROOT that is already valid in its environment (enforced by the
   existing `contains_root` validation in the Alias Editor and the alias load
   path). An extension-contributed root SHALL be registered via AC 1 before any
   alias can validate against it.
3. WHEN an extension ships alias rows pointing at its own contributed roots, THE
   host SHALL install them into the live alias store at enable and remove them at
   disable (aliases removed before the root), and the ROOT (not the surface) SHALL
   be what is recorded and persisted (command-semantics Req 12.5, 13.3).
   User-authored aliases (the existing user override store) and extension-shipped
   aliases SHALL coexist.
4. WHEN an extension contributes a root that is the ENTRY to an extension Context,
   THE root SHALL resolve to a `CustomWorkspace`/`Function` target through FFCMD
   and route via `navigate_to`; the Context's in-buffer verbs SHALL be owned by
   that Context's Command Environment selected by the EXISTING per-`TabKind`
   active-environment derivation (`EnvironmentRegistry::active_name`) -- NOT by
   FFCMD root registration.
5. MAKING the root registry and classifier runtime-extensible (AC 1) IS a change
   to load-bearing framework types (`EnvironmentRegistry::root_commands` /
   `contains_root` and the `builtin_workspace_target_for` / `resolve_target`
   classifier) and SHALL be flagged as an owner-confirmed FRAMEWORK CHANGE
   (design D5.3) -- NOT undertaken as an implementation side effect. Compiled
   built-in roots SHALL remain authoritative and first; extension roots SHALL be
   additive, removed on disable, and subject to the first-wins + WARN duplicate
   rule (Requirement 3.3).
6. THE in-scope parked-domain re-entries (Editor, File navigator, Catalog, VFS)
   SHALL need NO runtime-extensible claiming-Command-Environment-OBJECT registry;
   their environments (FFEDIT, FFNAV, catalogued FF* names) are already modeled
   and the per-`TabKind` derivation already selects them. IF a future extension
   requires a genuinely new claiming Command Environment object selected
   dynamically, THEN that SHALL be a FURTHER owner-confirmed framework change and
   SHALL NOT be assumed by this contract.
7. THE live command set SHALL be a deterministic function of the enabled-extension
   set and the persisted `extensions.disabled` list -- an extension-contributed
   root SHALL resolve if and only if its extension is active.
8. IF the owner DECLINES the runtime-extensible registry (AC 5), THEN an extension
   SHALL be limited to contributing roots that are ALREADY compiled built-ins
   (e.g. re-homing `EDIT` / `FILES` / `CATALOGS`) plus aliases over them, and
   SHALL NOT contribute a genuinely new verb -- a narrower fallback the design
   records explicitly. Req 11.8 is a RECORDED FALLBACK (contingency) retained only
   should the owner reverse the D5.3 confirmation; under the current
   owner-confirmed position (Req 11.5, D5.3 is owner-confirmed) it DOES NOT APPLY.

---

## Non-Functional Requirements

1. ASCII-only in all spec and contributed documentation
   (`.kiro/steering/documentation-charset.md`).
2. The extension framework stays GUI-independent at the `ff-plugin` /
   `ff-extension` crate level (no egui imports); egui adapters live in
   `ff-desktop` (consolidation report; extension-architecture design constraint).
3. An extension failure SHALL be isolated: a panic or error in any lifecycle or
   contribution path SHALL be caught and SHALL NOT crash the host
   (extension-architecture Req 5.3/5.4).
4. Each new or changed source file stays under 400 non-test lines
   (`.kiro/steering/rust-standards.md`).

---

## Out of Scope

- SCRM wiring to a menu option / menu bar (SCRM stays core; separate effort, D4).
- The removal of Catalogs/Files from the bare-core POM (separate in-flight
  effort; Requirement 9.4).
- A remote extension marketplace / remote install (explicitly excluded, carried
  over from plugin-manager-ui design decision 4).
- The full TCR report for core functionality (owner deferred it until after this
  design/build; consolidation report / owner message 4).
- The actual re-entry implementation of each parked domain (Editor, File
  navigator, VFS, Catalog, Volume) -- this spec defines the contract they use,
  not their individual re-entries.
- Any change to the `ff-plugin` crate's already-built and already-passing
  internal behaviour (trait, lifecycle, versioning, security) beyond the rename
  staging in design D1.
