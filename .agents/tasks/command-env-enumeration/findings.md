# Command Environment root-command enumeration -- feasibility findings (for the Alias Editor redesign)

Read-only architectural investigation against the main working tree at
`c:\workspace\apps\FileForgeWorkbench`, HEAD `aeff0d5`. No code was changed. All
claims are grounded in live source with `file:line` citations; the two spec
inputs (command-environments CR-CH-053, command-configurator Req 5 CR-NR-113)
and the prior Add-row bug findings were read for context.

---

## 1. Summary answer (the sizing verdict)

**Adding "list the valid root commands for a Command Environment" is CHEAP for
the two phase-1 real environments, because the per-environment root sets ALREADY
EXIST as enumerable data.** No registry has to be built from scattered dispatch
code; the roots are already collected in two co-located places that are used
TODAY:

- **FFEDIT** -- its canonical verb set is the `AliasTable::ffedit_english()`
  seed, and `AliasTable` already exposes `canonical_verbs_owned() -> Vec<String>`
  which returns exactly the distinct root verbs
  (`crates/ff-desktop/src/shell/environment.rs:~300`, the
  `canonical_verbs_owned` method; seed list at `environment.rs:~140-170`). This
  is classification **(a) trivially enumerable**.
- **FFCMD** -- its root set is already assembled by
  `builtin_ffcmd_roots() -> Vec<String>`
  (`crates/ff-desktop/src/shell/alias_reader.rs:152-179`), which unions the two
  built-in menus' option roots with `command_config::BUILTIN_WORKSPACE_VERB_ROOTS`
  (`crates/ff-desktop/src/command_config/mod.rs:167-185`). This is classification
  **(a) trivially enumerable with a caveat**: it is near-complete but derived
  from the two code-only built-in menus plus the published verb list, so it does
  NOT include roots reachable only through a user `menus/*.toml` override or any
  `resolve_target` path not represented in that list. For the Alias Editor
  dropdown it is the right, complete-enough canonical set (see section 4).

**The capability is ADDITIVE, not a reshape.** The shared contract
(`CommandEnvironment` trait, `crates/ff-desktop/src/shell/environment_contract.rs:26-33`)
has exactly one method today -- `claim(&mut shell, raw, upper) -> bool`. Adding a
read-only `root_commands()` / `contains_root()` capability is a new method on an
existing contract (or, better, a registry query -- see the recommendation),
touching the small, known set of environment implementors. Per
`framework-conformance.md` it still COUNTS as a change to a core type
(`CommandEnvironment` is named there as load-bearing), so it is an
**owner-confirmed framework change -- but a purely additive one**, with no change
to dispatch, navigation, persistence, or `CommandTarget`.

**Recommended shape:** put enumeration on the **Environment_Registry** (a
registry query keyed by environment NAME) rather than only on the per-env trait,
because (i) the two real root sets live in two DIFFERENT places (an `AliasTable`
for FFEDIT, a menu+verb union for FFCMD) that the registry can bridge, and (ii)
the Alias Editor consumes it by NAME (`AliasEnvironment::env_name()` already
yields `"FFCMD"` / `"FFEDIT"`), so a name-keyed query is the natural fit. The
dropdown wants **canonical roots only** (not localised surfaces); aliases are the
thing the user is creating.

---

## 2. The Command Environment contract and where each environment registers

### 2.1 The contract (CR-CH-053)

- **Trait:** `CommandEnvironment` in
  `crates/ff-desktop/src/shell/environment_contract.rs:26-33`. Single method:
  ```rust
  pub(crate) trait CommandEnvironment {
      fn claim(&mut self, shell: &mut super::WorkbenchShell, raw: &str, upper: &str) -> bool;
  }
  ```
  Re-exported through `shell/environment.rs:400` so existing
  `super::environment::CommandEnvironment` paths are unchanged. There is NO
  enumeration method today -- the contract is claim-only.
- **Dispatch outcome:** `EnvDispatchOutcome { Claimed{rc}, NotClaimed,
  NoSuchEnvironment }` (`environment_contract.rs:44-61`), returned by addressing.

Note a DISTINCT, deliberately separate contract: `ff_vfs::BackendEnvironment`
(`crates/ff-vfs/src/backend_environment.rs:5-19`) is the STORE/SAVE seam
(host-FS / mainframe), NOT the interactive `CommandEnvironment`. The header
comment there explicitly draws the line. The enumeration capability belongs on
the interactive side (the registry / `CommandEnvironment`), not on
`BackendEnvironment`.

### 2.2 The registry and where environments register

- **`EnvironmentRegistry`** (`crates/ff-desktop/src/shell/environment_registry.rs`)
  is the BUILT, shell-owned collection (CR-CH-053 Task 17). It holds an ordered
  `Vec<EnvironmentEntry>` of `(name, RegisteredEnv tag)`
  (`environment_registry.rs:~95-120`).
- **Registration point:** `with_builtins_and_mainframe(...)` /
  `builtin_entries()` (`environment_registry.rs:~155-210`) register four entries:
  - `FFCMD` -> `RegisteredEnv::FfCmdBase` (the base; `= resolve_target`, never
    claims at the active-env step)
  - `FFEDIT` -> `RegisteredEnv::FfEdit` (the real zero-sized
    `FfEditEnvironment` object, `environment_ffedit.rs`)
  - `HOSTFS` -> `RegisteredEnv::HostFsPlaceholder` (non-claiming placeholder)
  - `MAINFRAME` -> `RegisteredEnv::MainframePlaceholder` (addressing/backend only)
  Name constants: `FFCMD_NAME` / `FFEDIT_NAME` / `FFNAV_NAME` / `HOST_FS_NAME` /
  `MAINFRAME_NAME` at `environment_registry.rs:~48-78`.
- **FFNAV** is a derived NAME only (`active_name` maps file/catalog kinds to
  `FFNAV_NAME`, `environment_registry.rs:~250-262`) but is NOT a registered
  claiming environment -- it degrades to the FFCMD base
  (`active_env` / `absent_or_unknown_name_degrades_to_ffcmd_base` test).
- **Shell wiring:** the registry is built at construction
  (`crates/ff-desktop/src/shell/construct.rs:~270-282`, `environments:` field)
  and addressed via `dispatch_to_environment`
  (`crates/ff-desktop/src/shell/commands_environment.rs:33-80`).
- `RegisteredEnv` is an enum tag (`environment_registry.rs:~95-115`). The doc
  comment there notes Task 21's `ff-ce-*` crates extend it "a new member / a
  boxed-plugin arm" -- so a future environment is a new tag arm plus its name.

---

## 3. How each environment currently holds its ROOT commands

### 3.1 FFEDIT -- roots are already a data table (classification a)

FFEDIT's claim resolves the surface verb to a canonical verb through
`AliasTable::ffedit_english()` BEFORE matching
(`crates/ff-desktop/src/shell/environment_ffedit.rs:~75-90`, `claim` body:
`let aliases = AliasTable::ffedit_english(); ... aliases.canonical_verb(verb_token)`).
The canonical verb set is the seed list in
`AliasTable::ffedit_english()` (`shell/environment.rs:~140-170`): LOCATE, TOP,
BOTTOM, UP, DOWN, LEFT, RIGHT, SORT, EXCLUDE (surface X), SHOW (surface INCLUDE),
RESET, FIND, RFIND, CHANGE, RCHANGE, CAPS, NULLS, STATS, LOCK, PROFILE, HILITE,
SCROLL, SAVE.

Crucially, `AliasTable` ALREADY exposes the exact accessor enumeration needs:
```rust
pub(crate) fn canonical_verbs_owned(&self) -> Vec<String>  // environment.rs, distinct roots, first-seen order
```
(used today by `LocaleAliasStore` for a locale reload). So FFEDIT root
enumeration is "call an existing method on the seed table" -- **(a) trivially
enumerable**. The match arms in `claim` are keyed on the CANONICAL verb, so the
table and the handler cannot diverge in which verbs exist (though there is no
compile-time guarantee the arm set equals the table set -- see section 5 ripple).

### 3.2 FFCMD -- roots are already unioned by `builtin_ffcmd_roots()` (classification a, with a completeness caveat)

FFCMD is `resolve_target` + `builtin_workspace_target_for`. The verbs ARE in a
`match` (`command_config/mod.rs:187-275`, arms for FILES/CATALOGS/COMMANDS/LOG/
PLUGINS/MACROS/GSEARCH/SEARCH/KINDS/MENUS/ALIASES/CONFIG/KEYS/THEME/=FILES/
FILE CATALOGS), which on its own is imperative. BUT the project already solved
enumeration for the localisation work:

- `BUILTIN_WORKSPACE_VERB_ROOTS` (`command_config/mod.rs:167-185`) is a published
  `const &[&str]` co-located with the classifier, and a test
  (`builtin_workspace_verb_roots_list_matches_the_classifier`,
  `command_config/mod.rs:~506-513`) PINS the list against the match so they
  cannot drift.
- `builtin_ffcmd_roots()` (`alias_reader.rs:152-179`) unions those verb roots
  with the two code-only built-in menus' option roots
  (`recovery_pom_menu()` + `recovery_settings_menu()`), deduplicated
  case-insensitively.

So the FFCMD root set is ALREADY a single enumerable function. From the POM menu
(`menu_workspace/defaults.rs:32-60`) it gains SETTINGS, CATALOGS, FILES, HELP, X;
from the Settings menu (`defaults.rs:83-123`) CONFIG, ALIASES, THEME, MENUS,
KEYS, KINDS, RESET BARE; plus the verb-roots list. This is the **complete set
the Alias Editor dropdown needs** -- it is more complete than
`BUILTIN_WORKSPACE_VERB_ROOTS` alone (which omits the menu-only roots such as
SETTINGS/HELP/X/CONFIG). Classification **(a) trivially enumerable**.

**Caveat (must be stated in the gate):** `builtin_ffcmd_roots()` is derived from
the two COMPILED built-in menus only (they are code-only, never on disk) plus the
verb list. A root that is reachable ONLY via a user `menus/<name>.toml` override,
or via a `resolve_target` stage not represented in the verb list (user command
definitions, registered `CommandId`s, dynamic menu names), is NOT in this set.
For the Alias Editor this is acceptable and arguably correct: the dropdown should
offer the stable, canonical BUILT-IN roots of FFCMD, not an open-ended set that
shifts with user menu files. The gate should DOCUMENT that the dropdown source is
"the built-in canonical roots" and that validation uses the same set, so author
and validator agree.

### 3.3 Other environments

- **FFNAV / FFLINE** -- NAMED only; they register nothing claimable, own no verb
  table, and their handling stays where it is today
  (command-environments requirements.md phase-1 scope; `environment_registry.rs`
  FFNAV degrade test). For enumeration they are **(c)**: there is no central verb
  set to expose because the environment is not built. In phase 1 they are not
  selectable in the Alias Editor (only `AliasEnvironment::{Ffcmd, Ffedit}` exist,
  `shell/user_aliases.rs:31-58`), so this does not block the redesign.
- **HOSTFS / MAINFRAME** -- these are `BackendEnvironment` store seams, not
  interactive command environments; they own no user-typeable root verb set.
  Not relevant to the alias dropdown.

---

## 4. Is there ALREADY a per-environment enumeration to reuse?

**Yes, for exactly the two environments the Alias Editor needs** -- and no other
enumeration need be built:

| Environment | Existing enumeration to reuse | Location |
|-------------|-------------------------------|----------|
| FFEDIT | `AliasTable::ffedit_english().canonical_verbs_owned()` | `shell/environment.rs` (`canonical_verbs_owned`), seed at `environment.rs:~140-170` |
| FFCMD | `builtin_ffcmd_roots()` | `shell/alias_reader.rs:152-179` (unions `BUILTIN_WORKSPACE_VERB_ROOTS` + the two built-in menus) |

Adjacent enumerations that are NOT the right source (checked and rejected):

- **Command Palette** reads `CommandRegistry::list_all()`
  (`crates/ff-command/src/registry.rs:105-108`), which enumerates dotted
  `CommandId`s (e.g. `file.save`, `plugin.git.commit`) -- the Function/plugin
  command namespace, NOT the FFCMD workspace-verb roots or FFEDIT editor verbs.
  Wrong namespace for the alias dropdown.
- **`LocaleAliasStore`** (`alias_reader.rs:~180+`) holds one `AliasTable` per
  environment name (`tables: Vec<(String, AliasTable)>`) and is the LIVE,
  locale-overlaid, user-overlaid store. It is seeded from the two sources above
  (`seed(ffcmd_roots)` builds FFEDIT from `ffedit_english()` and FFCMD from
  `identity_for(ffcmd_roots)`). This is the single best place to ANCHOR a
  name-keyed enumeration because it already maps name -> table (see
  recommendation), but note its tables include OVERLAID alias surfaces, so
  enumeration must return CANONICAL verbs (`canonical_verbs_owned`), not surfaces.

So the cheap path exists: expose the two existing collectors behind one
name-keyed query. No scattered-dispatch registry has to be built.

---

## 5. Ripple onto CR-NR-112 / CR-NR-113 and the resolver (what a STRICTER model touches)

The current model is LOOSER than a validated `(environment, root)` pair in
several places; tightening it ripples as follows (all grounded):

1. **Alias row carries a free-text `canonical` and a 2-value `environment` tag,
   validated only against a shadow set.**
   - `AliasRowEdit { surface, canonical, environment, command_template }`
     (`alias_editor_panel/state.rs:8-26`) -- `canonical` is a free `String`.
   - `validate_user_alias_rows` (`shell/alias_editor.rs:185-...`) rejects: empty
     surface, empty canonical, same-surface-two-roots WITHIN one environment, and
     a surface equal to a `builtin_ffcmd_roots()` member (the no-shadow guard).
     It does NOT check that `canonical` is a real root of `environment`. **A
     stricter model adds exactly this check** -- `contains_root(environment,
     canonical)` -- here. This is the central ripple and it is additive (a new
     rejection branch), not a reshape of the row.
   - **Looseness to fix:** the shadow guard uses `builtin_ffcmd_roots()`
     UNCONDITIONALLY regardless of the row's `environment`
     (`alias_user_overlay.rs:59-72` `apply_user_rows`; `alias_editor.rs:185`
     validate). That is an FFCMD-root set applied to FFEDIT rows too. Under a
     per-environment model the shadow/validation set should be the TARGET
     environment's own root set. The gate should reconcile the no-shadow guard
     with the new per-environment `contains_root` so they use one source.

2. **Routing by environment already exists and is correct -- it just is not
   validated.** `apply_user_rows` (`alias_user_overlay.rs:54-95`) routes each row
   into `self.tables.find(|(n,_)| n == row.environment.env_name())` and skips an
   unknown environment name. `env_name()` yields `FFCMD`/`FFEDIT`
   (`user_aliases.rs:42-48`). So the (environment -> table) plumbing is in place;
   a stricter model adds a membership check on the ROOT before the row is
   accepted, using the same name key. Low ripple.

3. **CR-NR-112 parameterised-alias layer assumes nothing stricter.** The template
   lives on the alias entry (`AliasEntry.command_template`,
   `environment.rs:~55-70`) and expands `canonical + " " + template` at the
   pre-pass. It does not care whether `canonical` is a validated root -- it
   expands whatever root the row carries. Making the root validated does not
   change expansion; it only guarantees the root is real. **No ripple into the
   expansion/dispatch path** (consistent with command-configurator Req 5.8:
   "author the definition but introduce no new expansion/dispatch behaviour").

4. **Overlay collision discipline (`apply_locale_overlay_keyed`,
   `environment.rs`)** is per-environment and scratch-copy based; a
   within-environment different-canonical surface is a collision. Validating the
   root against the environment is ORTHOGONAL to this and does not change it. No
   ripple.

5. **FFEDIT verb-table vs claim-arm drift (a latent correctness gap the stricter
   model exposes).** FFEDIT's enumerable roots come from
   `ffedit_english()`; its executable roots are the `match` arms in
   `FfEditEnvironment::claim` (`environment_ffedit.rs`). These are maintained by
   hand in two places. If enumeration drives a dropdown, a root present in the
   table but missing a claim arm would be OFFERED but not EXECUTABLE. The gate
   should add a test pinning "every `canonical_verbs_owned()` entry has a claim
   arm" (mirroring the existing
   `builtin_workspace_verb_roots_list_matches_the_classifier` guard for FFCMD).
   This is a cheap test, not a refactor.

---

## 6. Framework-conformance determination

Per `framework-conformance.md`, `CommandEnvironment` is named in the
"what counts as a framework change" list ("Removing or reshaping a public
framework type (`CommandTarget`, `WorkspaceContext`, `InteriorFocus`,
`WorkspaceDescriptor`, `KindConfig`, `ff-vfs::BackendEnvironment`)"). The
`CommandEnvironment` trait and the `EnvironmentRegistry` are the CR-CH-053 core
model. Therefore:

- **This IS a framework change requiring explicit owner confirmation** -- it
  touches the CR-CH-053 Command Environment model.
- **But it is purely ADDITIVE, not a reshape:**
  - It adds a NEW read-only capability (`root_commands` / `contains_root`), it
    does not change `claim`, `EnvDispatchOutcome`, the active-env derivation, the
    dispatch front door, the Navigation_Stack, `CommandTarget`, or the
    persistence model.
  - No existing call site changes behaviour; the capability is read-only and
    queried by the Alias Editor (a consumer), not by the hot dispatch path.

- **Implementation sites that must provide the new capability** (the complete
  list, because the registry is closed-enum today):
  1. `EnvironmentRegistry` -- add the name-keyed query
     (`environment_registry.rs`). This is the single recommended home.
  2. Its data sources, already existing, that the query delegates to:
     - FFCMD -> `builtin_ffcmd_roots()` (`alias_reader.rs:152`)
     - FFEDIT -> `AliasTable::canonical_verbs_owned()` via
       `ffedit_english()` (`environment.rs`)
  3. If instead the capability is put on the `CommandEnvironment` trait: then
     `FfEditEnvironment` (`environment_ffedit.rs`) must implement it, and the
     FFCMD base (which is NOT a `CommandEnvironment` object -- it is
     `resolve_target`) would need a shim. This asymmetry (FFCMD has no trait
     object) is the main reason the registry-query shape is recommended over a
     trait method (section 7).

Because the FFCMD base is not a `CommandEnvironment` object, a trait-only method
does not cleanly cover FFCMD; the registry query does. State this explicitly in
the gate brief as the deciding design point.

---

## 7. Recommended API shape

### 7.1 Where it lives: on the Environment_Registry (name-keyed query)

```rust
// crates/ff-desktop/src/shell/environment_registry.rs
impl EnvironmentRegistry {
    /// The canonical ROOT commands owned by the named environment, uppercased,
    /// de-duplicated, in a stable order. Canonical roots ONLY -- NOT localised
    /// alias surfaces. Unknown / non-enumerable environment -> empty Vec.
    pub(crate) fn root_commands(&self, env_name: &str) -> Vec<String>;

    /// True iff `root` (case-insensitive) is a canonical root of the named
    /// environment. The validation primitive for the Alias Editor.
    pub(crate) fn contains_root(&self, env_name: &str, root: &str) -> bool {
        let r = root.trim();
        self.root_commands(env_name).iter().any(|c| c.eq_ignore_ascii_case(r))
    }
}
```

Implementation delegates to the EXISTING collectors by name:
- `FFCMD` -> `crate::shell::alias_reader::builtin_ffcmd_roots()`
- `FFEDIT` -> `AliasTable::ffedit_english().canonical_verbs_owned()`
- anything else (FFNAV/FFLINE/HOSTFS/MAINFRAME) -> `Vec::new()` (not enumerable /
  not an interactive verb owner in phase 1).

Rationale for the registry over a trait method:
- The two root sets live in two DIFFERENT shapes (a menu+verb union vs an
  `AliasTable`); the registry is the one place that already bridges names to
  per-environment data.
- FFCMD is not a `CommandEnvironment` trait object, so a trait method cannot
  cover it without a shim; the registry query covers all names uniformly.
- The consumer (Alias Editor) already works by NAME
  (`AliasEnvironment::env_name()`), so a name-keyed query is the natural seam and
  needs no `&mut shell`.

Optional refinement: source FFEDIT from the LIVE `LocaleAliasStore` table
(`canonical_verbs_owned()` on the stored table) rather than a fresh
`ffedit_english()`, so the enumeration reflects the active store. For canonical
roots the two are identical (overlay only adds surfaces, never roots), so the
cheaper `ffedit_english()` is fine; note the choice in the gate.

### 7.2 How the Alias Editor consumes it (env -> root dropdown cascade)

- `AliasRowEdit.environment` drives the FIRST dropdown
  (`AliasEnvironment::ALL`, already rendered at
  `alias_editor_panel/render.rs:174-181`). This stays.
- The ROOT field (currently a free `TextEdit` at
  `alias_editor_panel/render.rs:100-105`) becomes a ComboBox whose items are
  `registry.root_commands(row.environment.env_name())`. Changing the environment
  re-populates the root list next frame (pure function of the current
  environment value -- no extra state). The registry is reachable in render via
  `ShellServices` (the `WorkspaceContext::render` already takes
  `&mut ShellServices`, `alias_editor_panel/render.rs:31-41`); expose the
  registry (or a cloned `Vec<String>` per environment) on `ShellServices` so the
  pure render stays shell-decoupled.

### 7.3 How save/load validation uses `contains_root`

- `validate_user_alias_rows` (`shell/alias_editor.rs:185`) gains a per-row check:
  `if !registry.contains_root(row.environment.env_name(), &canonical) { return
  Err(...) }`. This is the central stricter-model gate and replaces/augments the
  current free-text acceptance.
- The no-shadow guard (`apply_user_rows`, `alias_user_overlay.rs:59`) should use
  the TARGET environment's `root_commands` as its reserved set rather than the
  unconditional `builtin_ffcmd_roots()` (section 5.1), so a FFEDIT row is checked
  against FFEDIT roots.

### 7.4 Canonical roots only (not localised surfaces)

The dropdown and validation use CANONICAL roots. Aliases/surfaces are what the
user is CREATING -- offering them as root choices would be circular. The
localisation column in the redesigned editor is a per-ROW attribute of the alias
the user authors, independent of the root list. `canonical_verbs_owned()` and
`builtin_ffcmd_roots()` both already return canonical/root forms, so this falls
out naturally.

---

## 8. Recommended sequencing and sizing for the two downstream gates

### Gate 1 -- Command Environment root-command enumeration API (FOUNDATION, framework change)

- **Nature:** additive, owner-confirmed framework change to the CR-CH-053 model
  (section 6). Requirements gate required (framework-conformance.md); surface it
  to the owner as "additive read-only enumeration on the Environment_Registry".
- **Scope / sizing: SMALL.**
  - Add `root_commands(name)` + `contains_root(name, root)` to
    `EnvironmentRegistry`, delegating to the two existing collectors
    (`builtin_ffcmd_roots`, `canonical_verbs_owned`). ~1 file, <400 lines, well
    under the file-size rule.
  - Add the FFEDIT table-vs-claim-arm pin test (section 5.5) and a registry
    enumeration test (`root_commands("FFCMD")` contains CONFIG/THEME/...,
    `root_commands("FFEDIT")` contains FIND/CHANGE/...; `contains_root`
    case-insensitive; unknown env -> empty).
  - Decide and document the FFCMD completeness posture (built-in canonical roots
    only; user-menu roots excluded -- section 3.2 caveat).
  - No dispatch/navigation/persistence change; compile + scoped `-p ff-desktop`
    check per the agent/owner test split.
- **Dependencies:** none. It only exposes data that already exists.

### Gate 2 -- Alias Editor redesign (CONSUMER of Gate 1)

Depends on Gate 1 (needs `root_commands` / `contains_root`). Sizing: MEDIUM
(single Context, but several coordinated changes). Scope:

1. **Validated `(environment, root)` anchor.** Wire `contains_root` into
   `validate_user_alias_rows` and reconcile the no-shadow guard to the target
   environment's root set (section 5.1, 7.3).
2. **env -> root dropdown cascade.** Replace the free-text root `TextEdit`
   (`alias_editor_panel/render.rs:100-105`) with a ComboBox populated from
   `root_commands(row.environment)`; expose the root list on `ShellServices`
   (section 7.2).
3. **ISPF-style I / D line commands.** Add a leading line-command column where
   typing `I` inserts a row and `D` deletes (mainframe idiom the owner asked
   for), alongside or replacing the current Add/Delete buttons. This is new per-row
   UI state on `AliasEditorState`; the existing `AddRow`/`DeleteRow` actions
   (`state.rs:28-42`) are the handlers to reuse.
4. **Filter by root.** Add a filter field (guaranteed-present control, good first
   interior focus stop) that filters visible rows by root -- mirrors the
   Plugin Manager / Config panel filter pattern.
5. **Column order** = Command-Environment / Root / Localisation / Alias /
   Parameter-template. This REORDERS the current Alias / Root / Env / Template
   header (`alias_editor_panel/render.rs:64-72`) and ADDS a Localisation column.
   `AliasRowEdit` (`state.rs:8-26`) currently omits `localisation_key`
   deliberately; adding a Localisation column means adding that field back to the
   row and to `UserAliasRowToml` (note: `user_aliases.rs` test rows already carry
   a `localisation_key` field -- confirm the TOML schema). This is the one piece
   that touches the persisted row schema, so it needs its own requirements-gate
   attention (additive optional field).
6. **Fold in the Add-button-visibility bug** from
   `.agents/tasks/alias-editor-workspace/add-row-bug-findings.md`: the
   footer-after-scroll-area layout in the EMPTY-store case
   (`render.rs:76-144`) is untested for render-visibility. The redesign should
   (a) make the footer / line-command affordance unmissable in the empty case,
   and (b) add the missing full-shell click-and-see-a-row egui_kittest test the
   prior findings specify. The I/D line-command model naturally removes the
   dependency on a footer button being on-screen.
7. **Focus contract + test.** Preserve the single `WorkspaceContext::render ->
   InteriorFocus` path (`render.rs:31-41`) and the first-Tab egui_kittest test
   (workspace-conformance.md / command-configurator Req 5.10); the new first
   interior control is likely the filter field.

**Sequencing rationale:** Gate 1 is a tiny, self-contained foundation that only
exposes existing data; Gate 2 cannot validate `(environment, root)` or populate
the root dropdown without it. Build Gate 1 first, confirm the framework change
with the owner (additive, read-only), then Gate 2 consumes it. The persisted
Localisation-column field (Gate 2 item 5) is the only schema touch and should be
called out in Gate 2's requirements gate as an additive optional TOML field.

---

## 9. Key file:line index

- `crates/ff-desktop/src/shell/environment_contract.rs:26-33` -- `CommandEnvironment`
  trait (claim-only; no enumeration).
- `crates/ff-desktop/src/shell/environment_contract.rs:44-61` -- `EnvDispatchOutcome`.
- `crates/ff-desktop/src/shell/environment_registry.rs:~48-78` -- env name
  constants; `~95-210` -- `RegisteredEnv`, `EnvironmentEntry`, `builtin_entries`,
  constructors; `~230-300` -- `resolve` / `active_name` / `active_env` /
  `is_ffedit_active` (the recommended home for `root_commands`/`contains_root`).
- `crates/ff-desktop/src/shell/environment_ffedit.rs:~75-90` -- `FfEditEnvironment::claim`
  resolves surface -> canonical via `ffedit_english()`; match arms are the
  executable FFEDIT roots.
- `crates/ff-desktop/src/shell/environment.rs:~140-170` -- `AliasTable::ffedit_english()`
  seed (the FFEDIT canonical verb set); `canonical_verbs_owned()` accessor
  (distinct roots) in the same file.
- `crates/ff-desktop/src/shell/alias_reader.rs:152-179` -- `builtin_ffcmd_roots()`
  (the FFCMD root set: built-in menus + verb list).
- `crates/ff-desktop/src/command_config/mod.rs:167-185` -- `BUILTIN_WORKSPACE_VERB_ROOTS`;
  `187-275` -- `builtin_workspace_target_for` match; `~506-513` -- drift guard test.
- `crates/ff-desktop/src/shell/user_aliases.rs:31-58` -- `AliasEnvironment`
  (`Ffcmd`/`Ffedit`, `env_name()`, `ALL`, `label()`).
- `crates/ff-desktop/src/alias_editor_panel/state.rs:8-42` -- `AliasRowEdit`,
  `AliasEditorAction` (`None/AddRow/DeleteRow/Save`).
- `crates/ff-desktop/src/alias_editor_panel/render.rs:64-72` -- current column
  header (Alias/Root/Env/Template); `100-105` -- free-text root field (becomes a
  ComboBox); `118-144` -- footer (the Add-visibility bug surface); `174-181` --
  env selector.
- `crates/ff-desktop/src/shell/alias_editor.rs:185+` -- `validate_user_alias_rows`
  (the stricter-model validation seam; no root-membership check today).
- `crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs:54-95` --
  `apply_user_rows` (env-routing + unconditional `builtin_ffcmd_roots` shadow
  guard -- the per-environment reconciliation point).
- `crates/ff-command/src/registry.rs:105-108` -- `CommandRegistry::list_all()`
  (Function/plugin `CommandId` namespace -- NOT the alias-dropdown source).
- `crates/ff-vfs/src/backend_environment.rs:5-19` -- `BackendEnvironment` (the
  SEPARATE store/SAVE contract; enumeration does NOT belong here).
- Context inputs: `.agents/tasks/alias-editor-workspace/add-row-bug-findings.md`;
  `docs/specs/command-configurator/requirements.md:150-229` (Req 5, CR-NR-113);
  `docs/specs/command-environments/requirements.md` (CR-CH-053 model + phase-1
  scope); `docs/specs/command-environments/EXPLAINER-command-environments.md`.

### Tooling note
Investigation used the dedicated read/grep/file-search tools throughout; no shell
commands were run, so no terminal "dead episode" artifacts arose. No code was
modified. Some line numbers for methods within large files are given as
approximate (`~`) where the file exceeded the single-read window; the symbol
names are exact and greppable.
