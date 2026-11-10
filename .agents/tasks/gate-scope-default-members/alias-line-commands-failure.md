# Alias Editor line-commands test failure — root-cause investigation

Test: `shell::tests_alias_editor::line_commands_insert_and_delete_the_working_copy`
(`crates/ff-desktop/src/shell/tests_alias_editor.rs:120-150`, failing assertion at
`:142` — `assert_eq!(shell.alias_editor_panel.rows.len(), 0)`).

Repo state at investigation: `main` @ `0459a20` (merged core-001 strip).

---

## Summary answer (verdict first)

**This is a TEST-ISOLATION gap, NOT a production bug and NOT caused by the core strip.**

The test redirects only the shell's `dir_overrides.user_aliases` override seam.
That seam is consulted at SAVE / REFRESH time, but it is NOT consulted when the
alias store is first built. The store that the Alias Editor reads on open is built
earlier, during `WorkbenchShell` construction, and that construction path overlays
the **process/machine-global** User_Locale_Override_Store base
(`user_alias_override_base()` → `FFWB_USER_ALIASES_DIR` env var, else
`dirs::data_dir()/FileForgeWorkbench/aliases/`). On this machine that global base
contains a stray, previously-written `en/FFCMD.toml` holding exactly one row
(`ECOB → EDIT`). That one row is the "left: 1".

The test passed in the finalization worktree only because that environment's
global user-data dir had no `en/FFCMD.toml`. Same code, different ambient on-disk
state — exactly the environment-dependent isolation gap the brief suspected.

**Proof:** re-running the identical isolated test with the global base redirected
to an empty directory (`FFWB_USER_ALIASES_DIR=<empty>`) makes it PASS; with the
ambient `%APPDATA%` file present it FAILS. (Logs below.)

---

## Evidence

### 1. What `make_shell()` and `point_user_aliases_at_temp()` isolate (and do not)

`make_shell()` — `crates/ff-desktop/src/shell/tests_common.rs:110-156`:
- Installs the English Identity_Base catalogue (thread-local) via
  `i18n_bootstrap::install_catalogue_for_tests()`.
- Injects a UNIQUE temp **user-config** path (`with_user_config_path`) and a unique
  temp **command-line-history** path. These isolate config + history ONLY.
- It does NOT set `dir_overrides.user_aliases` and does NOT set
  `FFWB_USER_ALIASES_DIR`. Nothing about the alias override base is isolated at
  construction.

`point_user_aliases_at_temp()` — `tests_alias_editor.rs:28-33`:
```rust
shell.dir_overrides.user_aliases = Some(dir.path().join("aliases"));
```
It sets ONLY the shell field `dir_overrides.user_aliases`, and it does so AFTER
`make_shell()` has already returned (i.e. after construction).

Where `dir_overrides.user_aliases` is actually consulted —
`crates/ff-desktop/src/shell/alias_editor.rs:21-27`:
```rust
pub(super) fn user_alias_override_base(&self) -> Option<PathBuf> {
    if let Some(path) = &self.dir_overrides.user_aliases { return Some(path.clone()); }
    user_aliases::user_alias_override_base()        // global fallback
}
```
This instance method is called ONLY from `save_user_aliases()` and
`refresh_alias_store_in_place()` (`alias_editor.rs:134, 201`). It is NOT on the
open/first-read path.

The global base function it falls back to —
`crates/ff-desktop/src/shell/user_aliases.rs:140-146`:
```rust
pub fn user_alias_override_base() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("FFWB_USER_ALIASES_DIR") { return Some(PathBuf::from(p)); }
    dirs::data_dir().map(|b| b.join("FileForgeWorkbench").join("aliases"))
}
```

**Isolation gap:** `point_user_aliases_at_temp` isolates the SAVE/REFRESH base but
leaves the CONSTRUCTION-TIME overlay pointed at the real machine-global
`dirs::data_dir()/FileForgeWorkbench/aliases/`.

### 2. Where the 1 row comes from (construction-time global overlay)

Store construction — `crates/ff-desktop/src/shell/construct.rs:155-180`:
```rust
let help_locale = config_handle.get_string(ui::LOCALE).unwrap_or_else(|_| "en".into());
...
let store = LocaleAliasStore::seed_and_overlay_for_locale(&root_refs, &help_locale);
```
`seed_and_overlay_for_locale` — `crates/ff-desktop/src/shell/alias_reader.rs:247-258`:
```rust
let mut store = Self::seed(ffcmd_roots);
if let Some(root) = i18n_bootstrap::resolve_i18n_root() {
    store.apply_overlay_from_dir(&root.join(locale).join("aliases"));   // shipped seed
}
if let Some(base) = user_aliases::user_alias_override_base() {           // GLOBAL user base
    store.apply_user_overlay_from_base(&base, locale);                  // <-- NOT isolated
}
```
This runs during `make_shell()` — before the test sets `dir_overrides.user_aliases`
— and uses the global `user_alias_override_base()` (there is no `self` here), so it
reads the real machine user-data dir.

The open path then reads that already-built store, with no refresh:
`dispatch_command_string("ALIASES")` → `open_alias_editor`
(`alias_editor.rs:52-74`) → `load_effective_alias_rows` (`alias_editor.rs:74-92`)
→ `store.effective_display_rows()`. There is NO `refresh_alias_store_in_place()`
call on open, so the test's later `dir_overrides.user_aliases` never takes effect
for the assertion.

`effective_display_rows` → `AliasTable::display_rows()`
(`crates/ff-desktop/src/shell/environment.rs:293-306`) returns ONLY `is_overlay`
rows; base/identity rows (FFEDIT English base aliases, FFCMD synthesised identity
roots) are excluded. So under `en` the editor is empty UNLESS a seed/override
contributes an overlay row.

The stray overlay row — the real on-disk file on this machine:
`C:\Users\A142367\AppData\Roaming\FileForgeWorkbench\aliases\en\FFCMD.toml` (89 bytes):
```toml
[[alias]]
surface = "ECOB"
canonical = "EDIT"
command_template = "FFWB.MASTER.COBOL(&1)"
```
That is the single row (`left: 1`). The dir also contains `af/FFCMD.toml` and a
stale CR-NR-113 flat `user.toml` (11 bytes), neither of which affects the `en`
count. These were written by an earlier manual/app run in the main checkout; the
finalization worktree's environment did not have them.

### 3. Active locale really is `en`

`make_shell()` initialises the config handle with a fresh temp user-config path and
no `ui.locale` key, so `get_string(ui::LOCALE)` returns the schema default `"en"`
(`construct.rs:155-157`, and `active_locale()` at `alias_editor.rs:36-40`). The
locale is not the issue; the ambient `en/FFCMD.toml` is. (The `af` file is present
but the active locale is `en`, so `af` is not consulted.)

### 4. Empirical proof — same test, two environments

Command (plain, ambient `%APPDATA%` file present) —
log `tools/logs/alias-test-plain.txt`:
```
test ...line_commands_insert_and_delete_the_working_copy ... FAILED
assertion `left == right` failed  left: 1  right: 0   (tests_alias_editor.rs:142)
```

Command (identical test, `FFWB_USER_ALIASES_DIR` → an EMPTY dir) —
log `tools/logs/alias-test-isolated.txt`:
```
test ...line_commands_insert_and_delete_the_working_copy ... ok
test result: ok. 1 passed; 0 failed; ...
```

Setting `FFWB_USER_ALIASES_DIR` to an empty directory makes the construction-time
global overlay find nothing, so the editor opens with zero rows and the test
passes. This isolates the cause to the un-isolated construction-time global
override base.

### 5. Does the failure predate the strip?

The construction/alias code is byte-identical across the strip range. Diff
`4e3db77..0459a20` restricted to the alias files
(log `tools/logs/strip-diff.txt`) shows changes ONLY in `construct.rs`, and ONLY
the core-strip removals (provider registry, files_panel, nav managers, scroll
chrome, FFEDIT environment registration). The alias-store build block
(`seed_and_overlay_for_locale(&root_refs, &help_locale)`) is UNCHANGED, and
`user_aliases.rs`, `alias_editor.rs`, `alias_reader.rs`, and
`tests_alias_editor.rs` have **no diff at all** in that range.

Commit history (`tools/logs/gitlog.txt`):
- `4e3db77` — Merge CR-CH-072 (pre-strip parent). The empty-en-editor test and the
  construction overlay path already exist (introduced by CR-CH-072 commits
  `2026c2f` / `b0485cc`).
- `414ba84` — the core strip.
- `0459a20` — strip test-cleanup (current `main`).

Because the construction path and the test body are identical at all three
commits, the per-commit outcome in THIS environment is determined solely by the
ambient `%APPDATA%\FileForgeWorkbench\aliases\en\FFCMD.toml` file, which is
commit-independent:

| Commit   | Expected result in THIS main-checkout environment |
|----------|---------------------------------------------------|
| 4e3db77  | FAIL (left 1, right 0) — same construction gap, same ambient file |
| 414ba84  | FAIL — unchanged construction gap |
| 0459a20  | FAIL — confirmed by direct run (`alias-test-plain.txt`) |

In any environment WITHOUT that ambient file (e.g. the finalization worktree) all
three PASS — confirmed for the current code by the `FFWB_USER_ALIASES_DIR=<empty>`
run.

I ran the live confirmation on `0459a20` only and proved the per-commit identity by
source diff rather than by rebuilding each commit, because (a) rebuilding ff-desktop
three times is slow and (b) the cause is ambient on-disk state the checkout cannot
change — a per-commit rebuild would show the same FAIL purely because the same
`%APPDATA%` file is present, adding no information beyond the diff proof. The FFEDIT
registration present at `4e3db77` but removed by the strip does not change the
count: `display_rows()` excludes base/identity rows, so the FFEDIT English base
contributes zero display rows regardless.

---

## Conclusions and recommendation

**Verdict: (a) test-isolation gap.** The test depends on ambient on-disk state
(`dirs::data_dir()/FileForgeWorkbench/aliases/en/FFCMD.toml`) that happens to be
absent in a fresh worktree but present in the main checkout. It is not a
production bug in the `en` effective-table computation (that logic is correct: an
`en` install with no seed and no override shows zero rows), and the core strip did
not cause or change it.

**Minimal fix direction (do not implement — pinpoint only):**

The real gap is that the CONSTRUCTION-time overlay base is not isolated, while the
SAVE/REFRESH base is. Two complementary options, smallest first:

1. **Isolate the global base in the test harness.** Make `make_shell()` (or the
   alias tests specifically) set `FFWB_USER_ALIASES_DIR` to a per-test temp dir
   BEFORE construction, mirroring the existing B048 config/history isolation. This
   closes the gap for construction, save, and refresh in one move. The env-var seam
   already exists for exactly this purpose (`user_aliases.rs:140-143`). Caveat:
   env vars are process-global, so under multithreaded `cargo test` it must be set
   process-wide once (an env guard / serialised setup), consistent with how the
   codebase already treats `FFWB_USER_ALIASES_DIR` as a test-isolation seam.

2. **Make the open path honour the shell override.** Have `open_alias_editor`
   build/read through the shell's `self.user_alias_override_base()` (the
   `dir_overrides`-aware method) rather than the construction-time store — e.g.
   call `refresh_alias_store_in_place()` at the start of `open_alias_editor`, or
   thread `dir_overrides.user_aliases` into the construction overlay. This makes
   `point_user_aliases_at_temp` effective as written, but is a slightly larger
   change and touches the open hot path.

Option 1 is the minimal, lowest-risk fix and is consistent with the existing
`FFWB_USER_ALIASES_DIR` test-isolation design. (Separately, the operator may want
to remove the stray `%APPDATA%\FileForgeWorkbench\aliases\` contents, but that only
masks the gap on one machine — the harness fix is the durable remedy.)

---

## Artifacts (this investigation; ephemeral, under tools/logs)

- `tools/logs/alias-test-plain.txt` — isolated run, FAIL (left 1 / right 0).
- `tools/logs/alias-test-isolated.txt` — same test with empty
  `FFWB_USER_ALIASES_DIR`, PASS.
- `tools/logs/strip-diff.txt` — `git diff 4e3db77 0459a20` over the alias files.
- `tools/logs/gitlog.txt` — commit history across the strip.
- `tools/powershell/run-alias-test-isolated.ps1` — the one-job isolation runner.
- No production or test source was modified. No git state was changed. No temp
  worktree was created (the source diff made a per-commit rebuild unnecessary).
