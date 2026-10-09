# ADDRESS / Cross-Environment Routing Findings

READ-ONLY investigation of ONE question in the FileForgeWorkbench (FFWB)
command-environment design: does the CR-CH-053 model already support one Command
Environment ADDRESSing a command TO ANOTHER Command Environment (REXX
ADDRESS-style cross-environment command routing), and specifically how does
FFEDIT execute a store-affecting verb like SAVE today?

Workspace: `C:\workspace\VSC\FileForgeWorkbench`. No source changed. Plain ASCII
only (documentation.md). This report builds on the sibling
`editor-ce-extension-findings.md` (the environment set is CLOSED today) and
drills into the specific ADDRESS/SAVE routing question.

---

## Summary answer (read this first)

The owner's target model -- FFEDIT stays the universal editing environment, but
STORE-AFFECTING verbs (SAVE above all) are ADDRESSed to the Command Environment
of the file system behind the edited resource (Mainframe CE / POSIX CE / Native
CE), which owns the real I/O semantics -- is the RIGHT shape for CR-CH-053 and is
exactly what the EXPLAINER/vision say the model is FOR. But three concrete
findings determine the work:

1. **FFEDIT SAVE is hard-wired to a direct local-filesystem write.** `SAVE`
   resolves (FFEDIT alias table) to `ffedit_save()`
   (`crates/ff-desktop/src/shell/dispatch_ffedit.rs`), which calls
   `self.tabs.save_active_tab(&self.runtime)` (`tab_manager.rs:1261`). That
   method constructs a FRESH `LocalFsProvider::with_defaults()` and calls
   `provider.write(&path, &bytes)` -- a plain byte write to the physical path.
   There is NO branch on backing-store type, NO VFS-provider lookup, NO
   catalog/dataset-API call, NO record packing. SAVE is byte-identical for a
   mainframe DSN and a native file today.

2. **There is NO cross-environment ADDRESS routing primitive in the live
   dispatcher.** The single front door (`dispatch_command_string`) runs
   prelude -> `resolve_target` (FFCMD base) -> ladder, with ONE active-env claim
   step hardcoded to `== EnvironmentKind::FfEdit && self.ffedit_claim(...)`
   (`shell/commands.rs`). There is no `dispatch_to_environment(env, cmd)`, no
   `Environment_Registry` to look an environment up by name, and no "address a
   command to a NAMED environment" seam. The chained-command re-dispatch
   (`try_chained_fastpath`) re-enters `dispatch_command_string` but ALWAYS from
   the active context -- it canNOT target a named environment. The REXX ADDRESS
   bridge DOES exist (`crates/ff-lua/src/rexx.rs`: `RexxBridge`,
   `HostEnvironment`, `set_address`) but it is a SIMULATION (it records
   invocations, sets RC=0, never re-enters the shell dispatcher) and has ZERO
   references anywhere in `ff-desktop` -- it is not wired to the command line, to
   FFEDIT, or to a Lua `address` global. The spec is explicit: ADDRESS is
   "Macros only in phase 1" and the interactive address prefix is a "reserved,
   inert in phase 1: parse_address_prefix" seam that is NOT built.
   **Conclusion: cross-environment routing must be ADDED; nothing reusable
   exists in the live path.**

3. **A tab does NOT record its owning file system / CE.** `TabState`
   (`crates/ff-desktop/src/tab_state.rs`) has `path: Option<String>` and
   `kind: TabKind` but NO provider / CatalogType / file-system field. At open
   time the mainframe-ness IS known (`render_body_arms.rs` computes
   `is_mainframe` from the catalog's `CatalogType::Mainframe`), is used to
   resolve the DSN to a physical path via `open_mainframe_dsn`, and is then
   DISCARDED: both the mainframe and the native branch call
   `dispatch.execute_command("file.open", { path })` with ONLY the physical path.
   `open_file` builds a `TabState::for_file` carrying just the path string. So a
   `FileEditor` tab opened from a mainframe catalog is indistinguishable from one
   opened from native disk. **The tab->owning-CE binding is a NEW field (or must
   be re-derived), not something already captured.**

**Bottom line.** REXX-style `ADDRESS <env> <command>` routing is NOT built in the
runtime; it is designed and deferred. The owner's "FFEDIT addresses SAVE to the
owning CE" needs (a) a NEW small core cross-environment routing seam, (b) a NEW
tab->owning-environment binding, and (c) moving the RECFM/LRECL knowledge OUT of
the editor and INTO each FS CE's SAVE. All three fold cleanly into the ONE core
change already identified in the sibling findings (open the closed environment
set into an `Environment_Registry`) -- the registry that lets a Context pick its
active environment is the SAME registry that lets FFEDIT look up and address the
owning environment by name. This is building ON the CR-CH-053 model the EXPLAINER
already intends to generalise, not a new mechanism, and it is an owner-approved
CORE change per framework-conformance.md.

---

## Q1. How FFEDIT executes SAVE today -- the full call chain

Trace, with file/symbol citations:

1. **Front door.** Typed `SAVE` on an editor Context enters
   `dispatch_command_string(raw)`
   (`crates/ff-desktop/src/shell/dispatch.rs`). Prelude runs first; then the
   active-env claim gate in `shell/commands.rs` (~lines 138-219) checks
   `active_environment(active_kind, active_is_home) == EnvironmentKind::FfEdit`
   and, if so, calls `self.ffedit_claim(cmd, upper)`.

2. **FFEDIT claim.** `ffedit_claim` (`dispatch.rs`) resolves the surface verb via
   `AliasTable::ffedit_english()` -> canonical `"SAVE"`, matches the
   `"SAVE" => { self.ffedit_save(); true }` arm.

3. **FFEDIT SAVE body.** `ffedit_save()`
   (`crates/ff-desktop/src/shell/dispatch_ffedit.rs`):
   ```rust
   pub(super) fn ffedit_save(&mut self) {
       if !self.tabs.active_tab().is_modified {
           self.open_error = None;      // clean buffer -> no-op
           return;
       }
       match self.tabs.save_active_tab(&self.runtime) {
           Ok(()) => self.open_error = None,
           Err(msg) => self.open_error = Some(msg),
       }
   }
   ```
   Dirty-aware; delegates to the tab manager.

4. **The actual write.** `TabManager::save_active_tab`
   (`crates/ff-desktop/src/tab_manager.rs:1261`):
   ```rust
   let path = tab.path.as_deref().ok_or(...)?;         // untitled -> Err
   let bytes = runtime.block_on(async { doc.contiguous_view().to_vec() });
   runtime.block_on(async {
       let provider = LocalFsProvider::with_defaults()...;   // FRESH local FS provider
       provider.write(&path, &bytes).await ...               // plain byte write
   })?;
   tab.is_modified = false;
   doc.set_save_point();
   ```

**What runs:** a direct `ff-vfs` `LocalFsProvider::write` to the physical path.
NOT a catalog write, NOT `ff-dscatalog`, NOT a provider resolved from the tab's
origin, NOT a shell method that branches on store type. **There is NO branch on
the backing store type anywhere in this chain.** The `open_file` read path
(`tab_manager.rs:1219`) is symmetric: a fresh `LocalFsProvider::with_defaults()`
+ `provider.read(path)`. For mainframe datasets the record semantics live
elsewhere (`ff-dscatalog` dataset backends) and are simply not consulted on
editor save -- the dataset file is treated as an opaque byte blob.

---

## Q2. Is there a cross-environment ADDRESS primitive?

### (a) The dispatcher's active-env mechanism -- single, context-derived, not addressable
`dispatch_command_string` (`shell/dispatch.rs`) is: `=`-reinit -> prelude
(`run_command_prelude`) -> `resolve_target` (FFCMD) -> `run_command_ladder`. The
ONLY environment claim is in `shell/commands.rs`:
```
let editor_env_active = active_environment(t.kind.tag(), t.is_home)
    == EnvironmentKind::FfEdit;
...
if active_environment(active_kind, active_is_home) == EnvironmentKind::FfEdit
    && self.ffedit_claim(cmd, upper) { return; }
```
This is a HARDCODED equality to one variant plus one method. `EnvironmentKind`
(`shell/environment.rs`) is a CLOSED 3-variant enum (`FfEdit`/`FfNav`/`FfCmd`);
`environment_for_kind` is a hardcoded `match`. There is:
- NO `Environment_Registry` (the EXPLAINER's "shell-owned collection that ...
  looks one up by name for addressing" is listed as "New, small" -- not built).
- NO `dispatch_to_environment(env, cmd)` / `route_to` / `target_environment`
  method anywhere (grep of `crates/**/*.rs` for
  `dispatch_to|route_to|target_environment|redispatch|re_dispatch` returns only
  an idle-processing doc comment and the chained-fastpath re-dispatch comment).
- The `CommandEnvironment` trait exists but is `#[allow(dead_code)]` with NO
  implementor and NO registry holding `Vec<Box<dyn CommandEnvironment>>`.

So one environment cannot hand a command to another named environment through
the live dispatcher.

### (b) Chained-command sequential RE-DISPATCH -- re-enters, but cannot target an env
`try_chained_fastpath` (`crates/ff-desktop/src/shell/commands_fastpath.rs`)
splits a dotted/semicolon path (`=0.K`, `3.1`) and, for each segment, calls
`self.dispatch_command_string(segment)` -- a genuine re-entry into the single
front door. BUT every re-entry resolves against the CURRENT active context's
environment (the same `active_environment(...)` gate). It is sequential
re-dispatch of menu Option_Keys, NOT environment addressing: there is no syntax
or parameter to say "run this segment IN environment X". So it is not reusable
for "FFEDIT forwards SAVE to the Mainframe CE".

### (c) The Lua/REXX macro `address` binding -- a disconnected simulation
`crates/ff-lua/src/rexx.rs` implements the REXX surface:
`HostEnvironment {Tso, Ispexec, Isredit, Named(String)}`, `HostEnvironment::parse_env`,
`RexxBridge::set_address(env_name)`, `current_env()`, `RcVariable`. BUT:
- `RexxBridge::invoke` is a SIMULATION -- it checks a library index, pushes to an
  `invocation_log`, sets RC=0, and returns `Invoked`. It does NOT re-enter the
  FFWB command dispatcher or execute anything in the shell.
- `set_address` only mutates `current_env`; nothing consumes `current_env` to
  ROUTE a command to a shell-side environment.
- grep of `crates/ff-desktop/**/*.rs` for `RexxBridge|rexx::|set_address|
  HostEnvironment|ff_lua` returns NO matches -- the bridge is not wired into the
  shell at all.
- The Lua engine (`crates/ff-lua/src/engine.rs`) binds `editor.lines` and
  `editor.trace` stubs but does NOT create an `address` global (grep for
  `"address"` / `create_function` in `ff-lua/src` finds no address binding). The
  `EnvironmentKind::name()` method (`shell/environment.rs`) is explicitly
  `#[allow(dead_code)] // consumed by the macro ADDRESS wiring (task 10)` -- i.e.
  reserved for a future wiring that has not happened.

### (d) What the spec says the state is
- `environments-vision.md`: "an explicit address targets a specific environment
  regardless of context"; "Addressing follows REXX ADDRESS: active environment
  first, explicit address overrides" -- but marked "Macros only in phase 1".
- `EXPLAINER-command-environments.md`: `Environment_Registry` = "New, small";
  "Address / Alias_Map ... Macros only in phase 1"; the front-door pseudo-code
  shows `# [reserved, inert in phase 1: parse_address_prefix]`; the review calls
  the interactive address prefix "deferred but seam-reserved" and warns the
  reserved-but-inert seam "tends to rot".
- `design.md`: the registry's jobs include "(2) given a name (for addressing),
  return that environment" -- DESIGNED, phase-1-deferred, not implemented.

**Determination (Q2): REXX-style `ADDRESS <env> <command>` routing is NOT built
in the runtime. It is designed and explicitly deferred (macros-only, inert
seam). A cross-environment routing seam that FFEDIT could use to forward SAVE
must be ADDED.** The good news: the designed shape (Environment_Registry +
address-by-name) is precisely the seam FFEDIT needs, so adding it completes the
intended CR-CH-053 design rather than inventing a parallel mechanism.

---

## Q3. How does a tab know its owning file system / CE?

**It does not.** Evidence:

- `TabState` (`crates/ff-desktop/src/tab_state.rs`) fields: `id`, `kind: TabKind`,
  `title`, `path: Option<String>`, `document`, `viewport`, `cursor`,
  `is_modified`, `line_count`, `line_end_mode`, `undo_stack`, `prefix_inputs`,
  `is_floating`, `edit_profile`, `canvas_selection`, `workspace_name`, `is_home`,
  `nav_stack`. NONE records a VFS provider, a `CatalogType`, or any file-system /
  CE discriminator. `TabKind::FileEditor` / `KindTag::FileEditor` is the same tag
  regardless of origin.

- **Open-time: the origin IS known, then discarded.** In
  `shell/render_body_arms.rs`, the `FilesPanelAction::OpenFile` arm computes:
  ```rust
  let is_mainframe = ...selected_catalog...
      .map(|c| c.catalog_type == CatalogType::Mainframe).unwrap_or(false);
  if is_mainframe {
      match open_mainframe_dsn(&self.files_panel.registry, &catalog_name, &dsn) {
          Ok(path_str) => { p.insert("path", path_str); dispatch.execute_command("file.open", p); }
          ...
      }
  } else {
      p.insert("path", dsn); dispatch.execute_command("file.open", p);
  }
  ```
  Both branches converge on `file.open` with ONLY `path`. `open_mainframe_dsn`
  (`shell/render_body.rs`) resolves the DSN through the `CatalogRegistry` to a
  physical path `String` and returns it; the `CatalogType::Mainframe` fact and
  the owning catalog are NOT propagated onto the opened tab.

- `file.open` -> `FileOpenHandler` (`shell/handlers.rs`) -> `pending_open` ->
  `TabManager::open_file(path, runtime)` -> `TabState::for_file(id, path, ...)`.
  The constructor records the path string; no origin metadata.

- Other open seams (EDIT/BROWSE/VIEW ladder arms in `commands_ladder_a.rs` /
  `commands_ladder_c.rs`, the `OpenFile`/`Command` ShellRequests in
  `shell/render.rs`, `shell/actions.rs`) all likewise call
  `execute_command("file.open", { path })` with only a path.

**Determination (Q3): the tab->owning-environment binding is a NEW field** (e.g.
an `origin: FileSystemOrigin` / `owning_environment: EnvironmentName` on
`TabState`, set at open from the `CatalogType` the navigator already knows). It
is NOT reliably derivable from existing data: `path` is a plain host path for all
three kinds (mainframe DSNs resolve to real files under a catalog root), there is
no URL scheme, and `CatalogType` is not persisted on the tab. The cleanest
capture point is the `OpenFile` arm (and the EDIT/BROWSE/VIEW arms), where the
`CatalogType` is in scope; it must be threaded through `file.open` (a new param
on `CommandParams`) into `TabState`.

---

## Q4. The verb split -- which FFEDIT verbs are store-affecting

FFEDIT verbs, from the alias table `AliasTable::ffedit_english()`
(`shell/environment.rs`) and the `ffedit_claim` match arms (`shell/dispatch.rs`):

### PURE IN-BUFFER (no store I/O -- FFEDIT keeps these, no addressing)
- `LOCATE`, `TOP`, `BOTTOM`, `UP`, `DOWN`, `LEFT`, `RIGHT` -- viewport/cursor
  navigation via `nav_manager` (operate on the open buffer only).
- `SORT` -- reorders in-buffer lines (`nav_manager.sort`).
- `EXCLUDE`/`X`, `SHOW`/`INCLUDE`, `RESET` -- exclude/show display filter
  (`exclude_manager`), pure view state.
- `FIND`, `RFIND`, `CHANGE`, `RCHANGE` -- search/replace over the open buffer
  (`find_manager`). The REPLACE acts on the in-memory document, NOT the store.
- `CAPS`, `NULLS`, `STATS`, `LOCK`, `PROFILE`, `HILITE` -- ISPF edit-profile
  mutations on `tab.edit_profile` (`dispatch_ffedit.rs`), buffer/session state.
- `SCROLL` -- updates `self.scroll_amount`, pure UI state.

### STORE-AFFECTING (the owner's model would ADDRESS these to the owning CE)
- `SAVE` -- the ONLY store-writing verb currently in FFEDIT. Today -> direct
  `LocalFsProvider::write` (Q1). This is the primary verb to address to the
  owning CE's SAVE (mainframe CE packs records per RECFM/LRECL; native CE writes
  bytes).

### Ambiguous / not-yet-present (where the semantic lives today)
- `CANCEL`, `UNDO`, `REDO` -- NAMED in the vision's FFEDIT verb list but NOT in
  the live alias table. The `ffedit_english()` comment states UNDO/REDO are
  "deferred (Req 10.6: UNDO is keyboard-only today, REDO does not exist)". UNDO
  is an in-buffer operation (undo stack); CANCEL is discard-without-save
  (in-buffer/session). Neither writes the store, so neither needs addressing.
- `CREATE` / `REPLACE` member -- not present as FFEDIT verbs today; these are
  mainframe PDS-member write-backs. When implemented they ARE store-affecting and
  would address the owning (mainframe) CE. No code exists for them yet.
- **Save-time LRECL / RECFM validation** -- does NOT exist anywhere today
  (`ff-document-model` is purely UTF-8/line-ending oriented; grep for
  `recfm|lrecl|record_format` in `crates/ff-document-model/src` finds only
  UTF-8 "sequence" matches). Under the owner's model this validation belongs in
  the Mainframe CE's SAVE, not the editor.
- **CHANGE write-back** -- CHANGE edits the buffer only; it is store-affecting
  only via a subsequent SAVE. So CHANGE itself stays pure in-buffer; only the
  SAVE it precedes is addressed.

**So the verb split is clean:** essentially ALL current FFEDIT verbs are
in-buffer; SAVE is the single store-affecting verb to address today, with
CREATE/REPLACE-member and save-time validation as FUTURE store-affecting
operations that will live in the owning CE, not the editor.

---

## Q5. Recommendation

### (a) FFEDIT-addresses-owning-CE for SAVE -- reuse vs new seam
**A NEW cross-environment routing seam is required** -- there is nothing
reusable in the live path (Q2). The minimal shape, building ON CR-CH-053:

- Introduce the `Environment_Registry` the EXPLAINER/design already specify: a
  shell-owned collection that (1) derives the Active_Environment for a Context
  and (2) returns an environment BY NAME for addressing. This single structure
  replaces the hardcoded `environment_for_kind` match and the `== FfEdit` gate in
  `shell/commands.rs`.
- Add an addressing entry point on it, e.g.
  `dispatch_to_environment(name, raw)` (or `address(env, raw)`), that invokes the
  named environment's claim/handler. FFEDIT's `ffedit_save` then becomes:
  look up the tab's owning environment name (Q3 binding) and call
  `dispatch_to_environment(owning_env, "SAVE")` instead of calling
  `save_active_tab` directly. The Mainframe CE's SAVE performs the record-aware
  write-back (via `ff-dscatalog` / the dataset API); the Native/POSIX CE's SAVE
  performs the plain byte write (today's `save_active_tab` logic MOVES into the
  Native CE as its SAVE). FFCMD remains the fallback.
- This is REXX ADDRESS applied internally: FFEDIT = active env, SAVE forwarded to
  the resource's owning env. It reuses the designed `name()` / Alias_Map wiring
  (`EnvironmentKind::name`, currently dead-code-reserved for exactly this).

### (b) Tab->owning-CE binding -- new field
**A NEW `TabState` field** (Q3): the origin is known at open time and discarded,
not re-derivable from `path`. Add e.g. `owning_environment: EnvironmentName`
(or `origin: FileSystemKind`) defaulted to Native, set from the navigator's
`CatalogType` at open, threaded through `file.open` via a new `CommandParams`
entry. FFEDIT reads it to choose the SAVE target.

### (c) This REPLACES the earlier per-document FileSystemProfile hook
The sibling `editor-ce-extension-findings.md` proposed a per-document
`FileSystemProfile` on `TabState` carrying RECFM/LRECL/seq-number columns, read
by the editor's verbs. The owner's model is BETTER and simpler: move the
RECFM/LRECL knowledge OUT of the editor and INTO the owning CE's SAVE. The editor
then needs only (i) the lightweight owning-environment binding (b) and (ii) the
routing seam (a) -- NOT a RECFM/LRECL data struct on every tab. The record
semantics live once, in the Mainframe CE, instead of being duplicated into an
editor-side profile. This shrinks the editor-side core change to a single
name/handle field plus a routing call.

### (d) What remains for CHROME (seq-number ruler / column bounds)
Display chrome (a sequence-number ruler in cols 73-80, an LRECL right-margin
bound, caps-column hints) is NOT a command-line verb and cannot be addressed as a
SAVE is. Under this model the editor should ASK the owning CE/provider for
display metadata rather than carry a profile: add a small read-only query on the
environment/provider (e.g. `display_metadata() -> EditChrome { seq_cols,
lrecl_bound, ... }`) that the editor render surface consults for the active tab's
owning environment. Default (Native CE) returns "no ruler, no bound" so native
editing is byte-identical to today. This keeps chrome data-driven and sourced
from the one authority (the CE/provider), consistent with (c) -- the editor
carries a binding and asks, it does not store a profile.

### (e) Plugin vs core; building ON CR-CH-053
Per framework-conformance.md:

- **SMALL owner-approved CORE change (ONE change, not two):** open the closed
  environment set into the `Environment_Registry` AND give that registry the
  address-by-name entry point (`dispatch_to_environment`). These are the SAME
  structure -- the registry that derives the active environment is the registry
  that looks one up by name to address it -- so there is ONE core change, exactly
  as the brief asks. Add the ONE `TabState` owning-environment field and thread
  it through `file.open`. Make FFEDIT a real `CommandEnvironment` object and route
  its SAVE through the registry. All additive and behaviour-preserving when the
  default owning environment is Native (whose SAVE == today's `save_active_tab`),
  so FFWB keeps building and native behaviour is byte-identical until a plugin
  opts in.
- **PURE plugin extension (no further core change) once the registry exists:**
  the Mainframe CE and POSIX CE themselves (each a `CommandEnvironment`
  implementor registered into the registry, owning its SAVE/record semantics via
  `ff-dscatalog` / `ff-vfs`), their display-metadata responses, and the per-FS
  Workspace Kinds/menus as data. The per-FS record logic is plugin code, not core.
- This is squarely building ON the CR-CH-053 model: the EXPLAINER already frames
  the registry + address-by-name as the intended generalisation ("New, small ...
  Not a dispatcher"); the `EnvironmentKind::name()` dead-code and the inert
  `parse_address_prefix` seam are reserved for exactly this. We are completing a
  designed, phase-1-deferred capability, not adding a parallel dispatcher (which
  framework-conformance.md forbids). The interactive address prefix the review
  worried would "rot" gets a concrete first consumer: FFEDIT's internal SAVE
  forwarding.

### Folding into the single identified core change
The sibling findings identified "open the closed environment set" as the one core
change. This investigation adds that the SAME change must also expose
address-by-name routing and that a tab must carry its owning-environment name.
Because address-by-name is an intrinsic job of the registry (design.md job 2),
these are not two core changes -- they are one: **land the Environment_Registry
with BOTH the active-env derivation AND the dispatch_to_environment addressing
entry point, plus the one TabState owning-environment field.** FFEDIT's SAVE
forwarding and the per-FS CEs then sit on top as the first real consumers.

---

## Key file / symbol reference

| Concern | Path / symbol |
|---|---|
| FFEDIT SAVE body | `crates/ff-desktop/src/shell/dispatch_ffedit.rs` -> `ffedit_save` |
| Actual write (direct local FS) | `crates/ff-desktop/src/tab_manager.rs:1261` -> `save_active_tab` (`LocalFsProvider::with_defaults().write`) |
| Front door + FFEDIT claim | `crates/ff-desktop/src/shell/dispatch.rs` -> `dispatch_command_string`, `ffedit_claim` |
| Hardcoded `== FfEdit` gate | `crates/ff-desktop/src/shell/commands.rs` (~138-219) |
| Closed env enum + derivation | `crates/ff-desktop/src/shell/environment.rs` -> `EnvironmentKind`, `environment_for_kind`, `AliasTable`, `CommandEnvironment` (dead-code), `name()` (dead-code "macro ADDRESS wiring task 10") |
| Chained re-dispatch (not addressable) | `crates/ff-desktop/src/shell/commands_fastpath.rs` -> `try_chained_fastpath` |
| REXX ADDRESS simulation (unwired) | `crates/ff-lua/src/rexx.rs` -> `RexxBridge`, `HostEnvironment`, `set_address` (no `ff-desktop` references) |
| Tab state (no FS/CE field) | `crates/ff-desktop/src/tab_state.rs` -> `TabState`, `KindTag`, `TabKind` |
| Open-time origin computed+discarded | `crates/ff-desktop/src/shell/render_body_arms.rs` (`is_mainframe`), `shell/render_body.rs` (`open_mainframe_dsn`) |
| file.open path | `shell/handlers.rs` (`FileOpenHandler`) -> `tab_manager.rs` (`open_file` -> `TabState::for_file`) |
| Dispatch core (command exec) | `crates/ff-command/src/dispatch.rs` -> `CommandDispatch::execute_command` |
| Spec: registry/address designed+deferred | `docs/specs/command-environments/{EXPLAINER-command-environments,environments-vision,design}.md` |

---

## Conclusions

1. FFEDIT SAVE today is a direct, store-agnostic local-filesystem byte write
   (`save_active_tab` -> `LocalFsProvider::write`); there is NO branch on backing
   store and NO record semantics on save.
2. There is NO live cross-environment ADDRESS routing. The dispatcher's one
   active-env step is hardcoded to FFEDIT; the chained re-dispatch cannot target
   a named environment; the REXX `RexxBridge` is an unwired simulation; ADDRESS is
   designed but phase-1-deferred (macros-only, inert seam). The routing FFEDIT
   needs must be ADDED.
3. A tab does not record its owning file system / CE. The `CatalogType::Mainframe`
   origin is known at open and discarded; the binding is a new `TabState` field,
   not derivable from `path`.
4. The verb split is clean: all current FFEDIT verbs are in-buffer; SAVE is the
   sole store-affecting verb to address today (with CREATE/REPLACE-member and
   save-time LRECL/RECFM validation as future owning-CE operations, none of which
   exist yet).
5. Recommended: ONE owner-approved core change -- land the `Environment_Registry`
   with BOTH active-env derivation AND an `dispatch_to_environment` address-by-name
   entry point, add ONE `TabState` owning-environment field threaded through
   `file.open`, and make FFEDIT forward SAVE (and future store verbs) to the
   resource's owning CE. The RECFM/LRECL knowledge moves into each FS CE's SAVE,
   replacing the earlier per-document FileSystemProfile; chrome metadata is a
   read-only query the editor asks the owning CE. The per-FS CEs are then pure
   plugin extension. This completes the CR-CH-053 design rather than adding a
   parallel mechanism.
