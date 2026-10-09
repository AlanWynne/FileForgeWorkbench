# Implementation Plan -- CR-CH-053 Task 17: Build the Environment_Registry + make FFEDIT a real object

Behaviour-preserving refactor of the central dispatch path. Goal: replace the
CLOSED `EnvironmentKind` enum + hardcoded `environment_for_kind` match + the
single `== FfEdit` claim gate with a BUILT shell-owned registry of registered
command environments, and make FFEDIT a REAL registered `CommandEnvironment`
object -- with ZERO observable behaviour change. Implements command-environments
Requirement 13 (13.1-13.7), touching Req 4.2, 6.3, 1.4, 3.3, 2.1, 5.1.

ALL paths are absolute in the worktree `c:\workspace\VSC\FileForgeWorkbench\.worktrees\env-registry`
(branch `feature/environment-registry`). Git via
`git -C c:\workspace\VSC\FileForgeWorkbench\.worktrees\env-registry`.

---

## Design decisions (made here; grounded in the live code)

These resolve the open shape questions before implementation. Each is the
least-churn, behaviour-preserving choice.

### D1 -- Registry shape: name -> environment-slot, NOT `Vec<Box<dyn ...>>` that owns FFEDIT's logic
The live FFEDIT verb bodies (`ffedit_claim` in `dispatch.rs` and the `ffedit_*`
handlers in `dispatch_ffedit.rs`) all mutate `&mut self` (the `WorkbenchShell`
god-struct: `self.tabs`, `self.nav_manager`, `self.find_manager`,
`self.exclude_manager`, `self.scroll_amount`, `self.open_error`). A
`Box<dyn CommandEnvironment>` STORED on the shell cannot call back into `&mut self`
without a borrow conflict. The design.md ("From the closed set to a built
registry", "FFEDIT-as-object: ... the object is the trait wrapper the E0 design
already anticipated") EXPLICITLY permits FFEDIT to be a thin object that still
reaches shell state.

Decision: the registry is a shell-owned struct `EnvironmentRegistry` holding an
ordered collection of `EnvironmentEntry { name: &'static str, kind: RegisteredEnv }`
where `RegisteredEnv` is a small `enum { FfCmdBase, FfEdit, HostFsPlaceholder }`
tag that identifies WHICH registered environment a name resolves to. The registry
owns the SET (names, membership, lookup, default-to-FFCMD), satisfying Req 13.1/13.2;
it does NOT own the mutable claim bodies. The CLAIM is performed by the shell
calling the resolved environment through a `CommandEnvironment` trait whose method
TAKES `&mut WorkbenchShell` (see D2). This keeps the registry a pure data/lookup
structure (not a second dispatcher, Req 13.7) while the verb bodies stay where the
borrow checker needs them.

Rationale for an enum tag rather than true `dyn` erasure now: with only three
built-in members and FFEDIT's bodies requiring `&mut WorkbenchShell`, a trait
object that borrows the shell would force `self`-reentrancy gymnastics for zero
behaviour benefit. The enum tag is the minimal open structure that makes "which
environments exist" data-driven and lets Task 21's `ff-ce-*` crates register
additional members (they will add `RegisteredEnv` members / a boxed-plugin arm)
without touching the active-env derivation or the gate. This is additive and does
not reshape `CommandTarget` or the Navigation_Stack.

### D2 -- FFEDIT as a real `CommandEnvironment` object (trait method takes the shell)
Change the existing (currently dead-code) `CommandEnvironment` trait in
`environment.rs` from `fn claim(&mut self, raw, upper) -> bool` to
`fn claim(&mut self, shell: &mut WorkbenchShell, raw: &str, upper: &str) -> bool`.
Introduce a zero-sized `struct FfEditEnvironment` (unit struct; it holds no state
-- all state lives on the shell) and `impl CommandEnvironment for FfEditEnvironment`,
whose `claim` body is MOVED from today's `WorkbenchShell::ffedit_claim`. The moved
body calls `shell.ffedit_exclude(..)`, `shell.nav_manager.locate(..)`, etc. --
the SAME delegations, verbatim, so the observable result is byte-identical
(Req 13.5, 13.6, 4.2, 6.3). The `AliasTable::ffedit_english()` canonical-verb
resolution, the `RESET BARE` decline, and the SAVE/EXCLUDE/FIND/PROFILE/SCROLL
arms are preserved exactly.

`FfEditEnvironment` is the real registered object the registry resolves the name
`"FFEDIT"` to. The `WorkbenchShell::ffedit_claim` method is DELETED (its callers
go through the registry per D3); the `ffedit_*` verb-body helpers in
`dispatch_ffedit.rs` STAY as `pub(super)` methods on the shell (the object
delegates to them -- least churn, keeps `dispatch_ffedit.rs` unchanged).

### D3 -- Active-env derivation + claim gate READ FROM the registry
Replace both live hardcoded sites in `commands.rs`:
- The prelude gate (lines ~138-142): `active_environment(tag, is_home) == EnvironmentKind::FfEdit`
  becomes `self.environments.active_name(tag, is_home) == "FFEDIT"` (or an
  `is_ffedit_active(tag, is_home)` helper on the registry/shell that resolves the
  name and compares). The predicate is unchanged in meaning.
- The ladder claim gate (lines ~213-219): instead of
  `active_environment(..) == FfEdit && self.ffedit_claim(cmd, upper)`, resolve the
  active environment NAME from the registry, look up the registered env, and call
  its `claim(self, cmd, upper)`. When the name is absent/unknown the registry
  returns the FFCMD base name (Req 13.4), whose claim is a no-op `false`
  (FFCMD == resolve_target handles it later in the ladder), so the fall-through is
  identical to today.

The focused kind's environment NAME is derived THROUGH the kind, exactly as today:
the registry's `active_name(tag, is_home)` internally calls
`BuiltinKind::from_tab_kind(tag, is_home)` then maps the kind to its env name
(editor -> "FFEDIT"; Files/Catalogs -> "FFNAV"; else -> "FFCMD"). "FFNAV" is not a
registered claiming env in phase 1 (its verbs are handled where they are today),
so resolving "FFNAV" degrades to the FFCMD base at the gate -- identical to today
where `FfNav != FfEdit` made the gate false. This preserves Req 2.1's kind-supplied
derivation while removing the `environment_for_kind` match + `== FfEdit` literal
(Req 13.3).

### D4 -- 400-line rule: new file `shell/environment_registry.rs`
`environment.rs` is 447 lines total (~286 non-test). Adding the registry struct,
`RegisteredEnv`, `FfEditEnvironment`, and the trait change would push non-test
code past 400. Decision: put `EnvironmentRegistry`, `EnvironmentEntry`,
`RegisteredEnv`, `FfEditEnvironment`, and the name-derivation helper in a NEW
`c:\workspace\VSC\FileForgeWorkbench\.worktrees\env-registry\crates\ff-desktop\src\shell\environment_registry.rs`,
declared `mod environment_registry;` in `shell/mod.rs`. The `CommandEnvironment`
trait stays in `environment.rs` (it is the small shared contract). The closed
`EnvironmentKind` enum, `environment_for_kind`, and `active_environment` are
REMOVED from `environment.rs` (replaced by registry equivalents), which shrinks it.
Keep `AliasTable` where it is (`environment.rs`) -- `FfEditEnvironment::claim`
uses it via `super::environment::AliasTable`.

### D5 -- Where the registry is owned and built
Add a field `pub(super) environments: EnvironmentRegistry` to `WorkbenchShell`
(in `shell/state.rs`), inside a grouped sub-struct if one fits, else a single new
field (the wiring-standard prefers grouped sub-structs, but there is one registry
and it is a first-class dispatch structure like `cmd_registry`, so a single field
is acceptable and matches `kind_registry`/`cmd_registry` precedent). Build it with
`EnvironmentRegistry::with_builtins()` in `WorkbenchShell::new_with_history_store`
(`shell/construct.rs`), which registers FFCMD (base), FFEDIT (`FfEditEnvironment`),
and the host-FS placeholder in code at startup (Req 13.2). The registry's
`Default`/`with_builtins` is the single registration point.

---

## Ordered implementation steps (TDD: red before green on every behaviour step)

- [ ] 1. Add the `CommandEnvironment` trait signature change + the registry data structure (compile-only scaffold, no behaviour wired yet).
      Create `shell/environment_registry.rs` with: `enum RegisteredEnv { FfCmdBase, FfEdit, HostFsPlaceholder }`; `struct EnvironmentEntry { name: &'static str, env: RegisteredEnv }`; `struct EnvironmentRegistry { entries: Vec<EnvironmentEntry> }` with `with_builtins()` (registers FFCMD/FFEDIT/host-fs placeholder), `active_name(KindTag, is_home) -> &'static str` (via `BuiltinKind::from_tab_kind` then kind->name map, defaulting to "FFCMD" for absent/unknown -- Req 13.4), and `is_ffedit_active(KindTag, is_home) -> bool`; `struct FfEditEnvironment;`. In `environment.rs`, change the `CommandEnvironment` trait method to `fn claim(&mut self, shell: &mut WorkbenchShell, raw: &str, upper: &str) -> bool` (keep `#[allow(dead_code)]` until step 3 wires it). Add `mod environment_registry;` to `shell/mod.rs`. Do NOT yet remove `EnvironmentKind`/`environment_for_kind`/`active_environment` (keep the build green; remove in step 4).
      Files: `c:\workspace\VSC\FileForgeWorkbench\.worktrees\env-registry\crates\ff-desktop\src\shell\environment_registry.rs` (new), `...\src\shell\environment.rs`, `...\src\shell\mod.rs`.
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop"` compiles clean (new code unused is allowed via `#[allow(dead_code)]`).

- [ ] 2. Write the FAILING registry unit tests (red). Model them on the existing `environment.rs` tests being replaced.
      In `environment_registry.rs` `#[cfg(test)] mod tests`: `active_name_is_supplied_by_kind` (`// Validates: Requirement 13.3` -- Editor kind -> "FFEDIT"; Files/Catalogs -> "FFNAV"; Pom/Config -> "FFCMD"); `absent_or_unknown_name_degrades_to_ffcmd_base` (`// Validates: Requirement 13.4`); `registry_registers_the_builtin_environments` (`// Validates: Requirement 13.1, 13.2` -- `with_builtins()` contains FFCMD + FFEDIT + host-fs placeholder by name); `is_ffedit_active_matches_editor_contexts_only` (`// Validates: Requirement 13.3, 5.1` -- true for `KindTag::FileEditor`/`Untitled`, false for `MenuWorkspace`/`ConfigPanel`/`FileExplorerPanel`). These fail to compile/pass until `with_builtins`/`active_name`/`is_ffedit_active` return correct data.
      Files: `...\src\shell\environment_registry.rs`.
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo test -p ff-desktop environment_registry"` -- new tests FAIL (red) before the bodies are filled, then PASS once step 1's bodies are correct. (If step 1 bodies are already correct, assert the tests exercise the exact degrade-to-FFCMD path; confirm they would fail against a stub returning the wrong name.)

- [ ] 3. Make FFEDIT a real registered object: move `ffedit_claim` body into `FfEditEnvironment::claim`.
      Implement `impl CommandEnvironment for FfEditEnvironment` in `environment_registry.rs`, moving the ENTIRE match body of `WorkbenchShell::ffedit_claim` (from `dispatch.rs`) VERBATIM, rewritten to call through the passed `shell: &mut WorkbenchShell` (e.g. `shell.nav_manager.locate(..)`, `shell.ffedit_exclude(raw)`, `shell.open_error = ...`). Preserve: the `AliasTable::ffedit_english()` canonical resolution, the `RESET BARE` decline (`return false`), and every arm (LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT/EXCLUDE/SHOW/RESET/RFIND/RCHANGE/FIND/CHANGE/CAPS/NULLS/STATS/LOCK/PROFILE/HILITE/SCROLL/SAVE). Delete `WorkbenchShell::ffedit_claim` from `dispatch.rs`. Keep the `ffedit_*` helpers in `dispatch_ffedit.rs` as `pub(super)` (the object delegates to them). Remove the now-unused `#[allow(dead_code)]` on the trait.
      Files: `...\src\shell\environment_registry.rs`, `...\src\shell\dispatch.rs`.
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop"` compiles (the gate in step 4 will call it; a temporary direct call from the old gate site may be needed to keep step 3 self-contained -- prefer doing steps 3 and 4 as one commit if the borrow wiring requires it).

- [ ] 4. Wire the two live gates to READ FROM the registry; remove the closed enum/match/literal.
      Own the registry: add `pub(super) environments: EnvironmentRegistry` to `WorkbenchShell` in `shell/state.rs`; build it via `EnvironmentRegistry::with_builtins()` in `new_with_history_store` in `shell/construct.rs`. In `shell/commands.rs`: (a) prelude gate ~L138-142 -> `let editor_env_active = self.environments.is_ffedit_active(t.kind.tag(), t.is_home);`; (b) ladder claim gate ~L213-219 -> resolve active name, and when it is "FFEDIT" call `FfEditEnvironment.claim(self, cmd, upper)` (construct a `FfEditEnvironment` locally or fetch from the registry; it is zero-sized) returning early on `true`. Delete `EnvironmentKind`, `environment_for_kind`, and `active_environment` from `environment.rs` and update/replace the three `environment.rs` tests that referenced them (`environment_is_supplied_by_kind`, `active_environment_routes_through_kind`, `ffedit_claims_owned_verb_only_when_active_env_is_ffedit`) -- move their intent into the `environment_registry.rs` tests from step 2 (keep the `AliasTable` tests `alias_table_resolves_surface_to_canonical`, `ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs`, `equals_prefixed_command_bypasses_the_active_environment` in `environment.rs`). Confirm NO other live callers remain (grep showed only `commands.rs`).
      Files: `...\src\shell\state.rs`, `...\src\shell\construct.rs`, `...\src\shell\commands.rs`, `...\src\shell\environment.rs`.
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop"` compiles with no reference to the deleted symbols.

- [ ] 5. Write the FAILING full-shell parity tests (red before green), then confirm green after step 4 lands.
      Add full-shell `egui_kittest` `build_eframe` tests (model on the existing shell behaviour tests; use `make_shell()` which opens on an editor Context). Each carries `// Validates: Requirement 13.x`:
      - `ffedit_verb_via_registry_matches_prior_behaviour` (`// Validates: Requirement 13.5, 13.6, 4.2, 6.3`): on an editor Context, a representative FFEDIT verb (e.g. `EXCLUDE ALL`, `FIND` term, `CAPS ON`, bare `X` -> EXCLUDE) produces the identical observable result (exclusion state / `open_error`) through the registered object.
      - `ffcmd_verb_unaffected_by_registry` (`// Validates: Requirement 13.6, 13.7`): a representative FFCMD verb/exit (e.g. `=X`, `RETURN`, `THEME <name>`) resolves exactly as before.
      - `non_editor_context_does_not_route_through_ffedit` (`// Validates: Requirement 5.1, 13.3`): bare `X` on a non-editor (POM) Context still exits (not EXCLUDE).
      - `b062_change_case_preserved_via_registry` (`// Validates: Requirement 4.2, 6.3`): CHANGE argument case preservation + verb case-insensitivity through the object path.
      Prefer asserting against observable state/`open_error`, consistent with the existing E8/E9 shell tests. Reuse the suite that previously proved E8/E9 (`ffedit_claims_owned_verb_only_when_active_env_is_ffedit`, the bare-`X` editor/non-editor tests) -- they must stay green through the registry path.
      Files: the existing shell test module(s) that host the FFEDIT/E8/E9 full-shell tests (locate via grep for `make_shell` and bare-`X` editor tests; add alongside them -- split a test file if it would exceed ~200 lines per `testing.md`).
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo test -p ff-desktop"` -- the new tests FAIL before steps 3-4 are complete, then ALL pass after. Confirm the pre-existing E8/E9/exit/reset_bare tests remain green (no regression).

- [ ] 6. Lint + format the touched crate.
      Files: all of the above.
      Verify (run each as its own invocation): `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo clippy -p ff-desktop"` is clean (no new warnings; justify any `#[allow]` with a comment); `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt"` then `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt -- --check"` passes.

- [ ] 7. Confirm the 400-line rule holds for every touched `.rs` (excluding `#[cfg(test)]`).
      Check `environment_registry.rs`, `environment.rs`, `commands.rs`, `dispatch.rs`, `state.rs`, `construct.rs` stay under 400 non-test lines. If `environment_registry.rs` approaches the limit, split the `FfEditEnvironment` impl into `environment_ffedit.rs` (behaviour-preserving file split).
      Files: as needed.
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "(Get-Content '<path>').Count"` per file (one invocation each), subtracting the test module length; confirm non-test <= 400.

- [ ] 8. Update docs/status trackers for Task 17 (documentation-only; no gate).
      Mark `tasks.md` Task 17 (17.1/17.2/17.3) `[x]` with a DONE note (scoped test counts, clippy/fmt clean), and set the TCR rows for Req 13.1-13.7 to their covered status (PASS for the egui_kittest/unit-covered criteria) in `docs/quality/TCR.md`.
      Files: `...\docs\specs\command-environments\tasks.md`, `...\docs\quality\TCR.md`.
      Verify: re-read both files; confirm only `[ ]`/`[x]` markers used and every Req 13.x row updated.

---

## Scoped verification commands (ALL via the pwsh7 wrapper, ONE command per invocation)

Run in the worktree. NEVER `;`-chain; NEVER run `--workspace` or `cargo gate`
(those are the owner's manual full-gate step).

- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt -- --check"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo check -p ff-desktop"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo clippy -p ff-desktop"`
- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo test -p ff-desktop"`

If a `cargo` invocation needs the worktree as cwd, pass it to the tool's `cwd`
parameter (the worktree root), not via `cd`-chaining.

FORBIDDEN: `cargo build --workspace`, `cargo test --workspace`, `cargo gate`,
`cargo full-gate`, `cargo nextest run --workspace`. After the scoped checks are
clean, HAND OFF to the owner to run the full `cargo gate --build` manually.

---

## TDD test list (each mapped to a criterion, red before green)

Unit (in `environment_registry.rs`):
- `active_name_is_supplied_by_kind` -- Requirement 13.3
- `absent_or_unknown_name_degrades_to_ffcmd_base` -- Requirement 13.4
- `registry_registers_the_builtin_environments` -- Requirement 13.1, 13.2
- `is_ffedit_active_matches_editor_contexts_only` -- Requirement 13.3, 5.1

Full-shell `egui_kittest` (`build_eframe`):
- `ffedit_verb_via_registry_matches_prior_behaviour` -- Requirement 13.5, 13.6, 4.2, 6.3
- `ffcmd_verb_unaffected_by_registry` -- Requirement 13.6, 13.7
- `non_editor_context_does_not_route_through_ffedit` -- Requirement 5.1, 13.3
- `b062_change_case_preserved_via_registry` -- Requirement 4.2, 6.3

Retained (must stay green through the registry path): the existing E8/E9 bare-`X`
editor vs non-editor exit tests, `=X` exit, `RESET BARE` ownership, and the
`AliasTable` tests in `environment.rs`.

---

## Drift between spec and LIVE code (recorded)

1. The spec/tasks say "replace the `== FfEdit` claim gate" (singular). The LIVE
   code has TWO hardcoded `== EnvironmentKind::FfEdit` sites, BOTH in
   `shell/commands.rs`: the prelude EXIT-family skip gate (~L138-142) and the
   ladder claim gate (~L213-219). BOTH must be rerouted through the registry
   (step 4). This is the only drift of substance; the plan accounts for both.
2. `FFNAV` exists as an `EnvironmentKind` variant and a kind mapping
   (Files/Catalogs -> FFNAV) but is NOT a claiming environment -- its verbs are
   handled where they are today. The registry preserves this: `active_name`
   returns "FFNAV" for navigator kinds, which is not registered as a claiming env,
   so the gate degrades to the FFCMD base (identical to today's `FfNav != FfEdit`
   making the gate false). FFNAV is NOT registered as a claiming object in Task 17.
3. The `CommandEnvironment` trait is currently dead-code with signature
   `claim(&mut self, raw, upper)`. Making FFEDIT a real object requires the trait
   method to receive `&mut WorkbenchShell` (FFEDIT verbs mutate shell-entangled
   state). This is the design-sanctioned "thin object that still reaches shell
   state" (design.md "FFEDIT-as-object"), NOT a behaviour change.
4. `EnvironmentKind::name()` (FFEDIT/FFNAV/FFCMD strings) is dead-code reserved
   for macro ADDRESS (Task 10). The registry now owns environment NAMES; the
   `name()` method on the deleted enum is superseded by the registry's `&'static str`
   names. No live caller today, so removing it with the enum is safe.
5. `environment.rs` is 447 lines total (~286 non-test); adding the registry would
   breach the 400-line rule, hence the new `environment_registry.rs` file (D4).

---

## Why this is NOT decomposed into separable FEAT features

The three sub-tasks (registry structure, FFEDIT-as-object, read-from-registry
gates) are mutually dependent: the gates cannot read from the registry until the
registry and the FFEDIT object exist, and behaviour must stay IDENTICAL at the
commit boundary -- the build is only green once all three land together. This is
one coherent behaviour-preserving refactor of a single dispatch path, best
executed as the ordered steps above by the existing implement-and-review loop.
