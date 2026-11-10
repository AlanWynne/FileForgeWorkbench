# Gate 1 Implementation Summary -- Command Environment root-enumeration API (CR-CH-067)

Branch: `cmdenv-root-enum`
Worktree: `c:\workspace\apps\FileForgeWorkbench\.worktrees\cmdenv-root-enum`
Base: `aeff0d5` (main). Feature commit `cdc35b7`; final gate-doc commit `7894260`.

This is GATE 1: the additive foundation for the Alias Editor redesign (Gate 2).
Owner-confirmed as a framework change to CR-CH-053 ("Proceed...") plus the
data-integrity choice (option 2: "Reject with warning, continue to load").

---

## 1. Gate docs produced (requirements gate)

- **Change-log id**: CR-CH-067 -- recorded in `docs/status/change-log.md` under
  the "Modifications to existing behaviour" section. Covers the additive
  root-enumeration API and the reject-with-WARNING-at-load rule, the
  framework-conformance stance (additive, owner-confirmed, no reshape), and the
  FFCMD completeness posture.
- **Spec that received the EARS criteria**: `docs/specs/command-environments/`
  (the owner of the Command Environment model + the `EnvironmentRegistry`,
  CR-CH-053 Req 13).
  - `requirements.md` -- NEW **Requirement 20** with criteria **20.1 - 20.8**:
    - 20.1 name-keyed root-enumeration query ON `EnvironmentRegistry`.
    - 20.2 deterministic, de-duplicated order.
    - 20.3 canonical roots only (never localised alias surfaces).
    - 20.4 `contains_root` case-insensitive + trimmed.
    - 20.5 unknown env / unknown root -> empty / false, never panic.
    - 20.6 FFCMD returns the complete built-in canonical root set.
    - 20.7 reject-with-WARNING-at-load for an invalid user alias root (skip the
      row, WARN, continue loading).
    - 20.8 the same validity check surfaced for save-side use.
  - `design.md` -- NEW section: the API shape and the load-validation rule.
    (Non-blocking doc drift: design shows `&self` instance methods; the code uses
    associated functions -- see Known limitations.)
  - `tasks.md` -- NEW task group, Phase (cmdenv-root-enum) Tasks 27-29.
- **TCR rows**: `docs/quality/TCR.md` -- rows for Req 20.1-20.8 (24 insertions),
  carrying the evidence block for the owner's authoritative gate.
- **ROADMAP**: `docs/project-management/ROADMAP.md` -- CR-CH-067 entry describing
  the Gate 1 foundation and naming Gate 2 (Alias Editor redesign) as out of scope.

---

## 2. What was implemented (source)

### 2a. The root-enumeration API
Module: `crates/ff-desktop/src/shell/environment_registry.rs`
(associated functions on `EnvironmentRegistry`, non-test code ends before the
`#[cfg(test)]` module -- under the 400-line limit).

Final signatures (ASSOCIATED functions, name-keyed -- NOT `&self` instance
methods, so the load path which holds no registry handle can call them, and
because the FFCMD base is `resolve_target`, not a `CommandEnvironment` trait
object):

```rust
pub(crate) fn root_commands(env_name: &str) -> Vec<String>
pub(crate) fn contains_root(env_name: &str, root: &str) -> bool
```

- `root_commands` returns the environment's complete CANONICAL root set in
  deterministic sorted order:
  - FFCMD delegates to `crate::shell::alias_reader::builtin_ffcmd_roots()` (the
    two code-only built-in menus' option roots unioned with
    `BUILTIN_WORKSPACE_VERB_ROOTS` and `BUILTIN_FFCMD_LADDER_VERB_ROOTS`).
  - FFEDIT delegates to
    `AliasTable::ffedit_english().canonical_verbs_owned()` (canonicals only;
    surfaces such as `X` / `INCLUDE` are never returned).
  - Unknown env -> empty `Vec` (no panic).
  - `roots.sort()` applied for stable order; both source collectors already
    de-dup case-insensitively.
- `contains_root` trims `root` and matches via `eq_ignore_ascii_case` (the same
  comparison the existing verb resolution uses); false for unknown env / root,
  never panics.

Supporting change: `crates/ff-desktop/src/command_config/mod.rs` adds the
`BUILTIN_FFCMD_LADDER_VERB_ROOTS` const (EDIT/VIEW/BROWSE file-open ladder verbs)
plus a drift-guard test keeping that list disjoint from the classifier roots.
`crates/ff-desktop/src/shell/alias_reader.rs` -- `builtin_ffcmd_roots()` now
unions those ladder roots so an `ECOB -> EDIT` FFCMD alias is not wrongly
rejected.

### 2b. The validation hook (save-side)
`crates/ff-desktop/src/shell/alias_editor.rs` -- `validate_user_alias_rows`
(line ~184) gains a `contains_root(env_name, &canonical)` check so Gate 2's save
path can reject an invalid (environment, root) pair. Gate 1 adds only the hook.

### 2c. The load-time reject-with-warning rule
`crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs` --
`apply_user_rows` (branch at ~line 75-94). When
`EnvironmentRegistry::contains_root(env_name, &canonical)` is false, the row is
SKIPPED with a `log_warn!` naming the surface, root (canonical), and environment,
then `continue` -- loading never aborts and no other valid row is dropped. The
branch runs AFTER the malformed-row and no-shadow guards and BEFORE the row is
applied, mirroring the existing skip-with-WARN discipline. Those guards stay
intact.

---

## 3. Files changed

Source (commit `cdc35b7`):
- `crates/ff-desktop/src/command_config/mod.rs`
- `crates/ff-desktop/src/shell/alias_editor.rs`
- `crates/ff-desktop/src/shell/alias_reader.rs`
- `crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs`
- `crates/ff-desktop/src/shell/environment_registry.rs`
- `crates/ff-desktop/src/shell/tests_alias_editor.rs` (load-skip + re-baselined routing test)

Docs (commit `cdc35b7`):
- `docs/quality/TCR.md`
- `docs/specs/command-environments/design.md`
- `docs/specs/command-environments/requirements.md`
- `docs/specs/command-environments/tasks.md`

Docs (final flip, commit `7894260`):
- `docs/project-management/ROADMAP.md`
- `docs/status/change-log.md`

No new `.rs` files. No Alias Editor UI change (Gate 2 scope, correctly deferred).

---

## 4. Commands run and log results

All commands were run via the clean pwsh7 wrapper, redirected to a log, then read
back (per terminal-discipline). The reported `Exit Code: -1` is the known capture
artifact; results were confirmed from the logs.

| Command | Log | Result |
|---------|-----|--------|
| `cargo check` (workspace) | `tools/logs/cmdenv-check.log` | Finished dev profile, no errors |
| `cargo clippy -p ff-desktop` | `tools/logs/cmdenv-clippy.log` | Finished clean, no warnings |
| `cargo fmt -p ff-desktop --check` | `tools/logs/cmdenv-fmt3.log` | Empty output = clean (no diff) |
| scoped `root_commands` tests | `tools/logs/cmdenv-test-rootcmds.log`, `-rootcmds2.log`, `-rootcmds3.log` | 5 passed; 0 failed (environment_registry root_commands/contains_root suite) |
| git log / status / commit | `tools/logs/cmdenv-finalize-*.log` | Feature commit `cdc35b7`, doc commit `7894260`, worktree clean |

Note on the agent/owner verification split (testing.md): the agent verified
COMPILE-CLEAN + clippy-clean + fmt-clean + the scoped new tests passing. The FULL
`cargo gate` / nextest suite is the OWNER's authoritative green run and was not
run by the agent. The TCR evidence block records the owner's prior authoritative
gate (fmt OK / clippy CLEAN / build OK / 6911 passed, the only failure being the
unrelated pre-existing flaky `ff-logging` wall-clock test which passed on an
isolated re-run).

Dead-terminal limitation: none of consequence. One stray invocation
(`cmdenv-test-registry.log`) failed with "unexpected argument 'contains_root'"
(a malformed test filter) and was superseded by the correctly-scoped
`-rootcmds*` runs, which all passed.

---

## 5. Review verdict

`gate1-review.json`: **APPROVED**, `findings: []` (no blocking findings). The
loop did NOT hit max-iterations; there were no unresolved findings. Review doc:
`gate1-review.md`. Two non-blocking notes for Gate 2: (1) design-doc `&self`
signature drift (doc-only), (2) the FFCMD no-shadow guard in `apply_user_rows`
still uses `builtin_ffcmd_roots()` unconditionally regardless of the row's
environment -- an acknowledged pre-existing looseness whose per-environment
reconciliation is deferred to Gate 2.

---

## 6. Known limitations

- `design.md` shows the API as `&self` instance methods; the implementation uses
  name-keyed associated functions. Req 20.1 (keyed by name ON the registry type)
  is satisfied; the doc snippet is stale on the `&self` detail only. Worth a
  one-line doc correction in Gate 2.
- `root_commands("FFCMD")` returns the BUILT-IN canonical roots only. A root
  reachable only via a user `menus/*.toml` override or a `resolve_target` stage
  not represented in the built-in lists is intentionally EXCLUDED. For the Gate 2
  dropdown this is the correct, stable canonical set, and load/save validation
  uses the SAME set so author and validator agree.
- The FFCMD no-shadow guard in `apply_user_rows` is not yet per-environment
  (deferred to Gate 2, as above).
- Only FFCMD and FFEDIT enumerate roots in phase 1; every other environment
  (FFNAV / HOSTFS / MAINFRAME / FFLINE / unknown) returns an empty set by design.

---

## 7. GATE 2 HANDOFF NOTE (Alias Editor redesign)

Final API to call (both associated functions on `EnvironmentRegistry`, module
`crates/ff-desktop/src/shell/environment_registry.rs`):

```rust
EnvironmentRegistry::root_commands(env_name: &str) -> Vec<String>
EnvironmentRegistry::contains_root(env_name: &str, root: &str) -> bool
```

Where Gate 2 should call them:

1. **Env -> Root dropdown cascade** (the Alias Editor row UI, `alias_editor_panel/*`):
   when the user selects a Command Environment for a row, populate the Root
   dropdown with `EnvironmentRegistry::root_commands(selected_env_name)`. The
   returned `Vec<String>` is already sorted, de-duplicated, and canonical-only --
   use it directly as the dropdown options. An empty return means that environment
   has no enumerable roots (disable / empty the Root dropdown).

2. **Alias-root validation (save-side)**: validate each edited row with
   `EnvironmentRegistry::contains_root(row.environment, row.root)` before persist.
   The hook is already wired into `validate_user_alias_rows`
   (`crates/ff-desktop/src/shell/alias_editor.rs`, ~line 184) -- Gate 2 supplies
   the save-side rejection UX (surface the error to the user) on top of it.

3. The LOAD path already enforces the same rule
   (`apply_user_rows` in `crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs`):
   an invalid (environment, root) row is skipped with a WARN and loading
   continues, so Gate 2 does not need to re-implement load-time protection.

Deferred into Gate 2 scope: the env->root->localisation->alias->parameter row
layout, ISPF-style I/D line commands, the root filter, and reconciling the FFCMD
no-shadow guard to be per-environment.
