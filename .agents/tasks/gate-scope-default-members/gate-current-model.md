# cargo-gate -- Current Scope/Selection Model and Public-Contract Surface

Read-only investigation (Phase 1). No files were edited. All paths absolute.
ASCII only.

Repo: C:\workspace\tools\cargo-gate
Current version: 0.2.0 (C:\workspace\tools\cargo-gate\Cargo.toml)
Base branch (checked out now): main
(confirmed via `git -C C:\workspace\tools\cargo-gate branch --show-current`,
output read back from a log == "main"; the terminal reported the spurious
"Exit Code: -1" capture artifact, so the result was verified by effect.)

Docs present under C:\workspace\tools\cargo-gate\docs: requirements.md,
design.md, tasks.md. NOTE: STATUS.md is referenced by design.md/tasks.md but
does NOT exist on disk (docs/STATUS.md not found).

---

## 1. How the gate currently selects crates and passes them to cargo

The gate does NOT invoke cargo on "default members". Today it ALWAYS passes an
EXPLICIT selection flag derived from the `Scope` enum. The one function that
turns a scope into package-selection args is
`scope_args` in C:\workspace\tools\cargo-gate\src\phases\nextest.rs:

```
match scope {
    Scope::Crate { name } => ["-p", name],
    Scope::Workspace      => ["--workspace"],
    Scope::AppOnly { .. } => ["--workspace", "--exclude", <orphan>, ...],
}
```

So for the DEFAULT run (no flags), `Scope::Workspace` is selected and cargo is
invoked with an EXPLICIT `--workspace`. It never relies on Cargo's
default-members fallback. This is the exact behavior we want to change: the new
default should pass NEITHER `--workspace` NOR an explicit `-p` list, letting
Cargo fall back to the workspace `default-members`.

`scope_args` is consumed by every cargo invocation the gate makes:
- `nextest_list_args`  -> `cargo nextest list --message-format json <scope_args>`
  (the up-front total count)
- `nextest_run_args`   -> `cargo nextest run --message-format libtest-json <scope_args>`
- `cargo_test_args`    -> `cargo test <scope_args>` (fallback runner)
- `cargo_build_args`   -> `cargo build <scope_args> [--release]` (the optional
  `--build` phase; `BuildTarget::Bin` overrides with `-p <name>`)

CLIPPY uses a SEPARATE arg builder, `clippy_args` in
C:\workspace\tools\cargo-gate\src\phases\fmt_clippy.rs, which does NOT share
`scope_args`:

```
Scope::Crate { name } => ["clippy", "-p", name, "--message-format=json"]
_ (Workspace | AppOnly) => ["clippy", "--workspace", "--message-format=json"]
```

Note the clippy builder collapses AppOnly to a plain `--workspace` (it does NOT
honor the orphan `--exclude` list -- clippy always runs the whole workspace
unless `--crate` is set).

FMT is scope-independent: `fmt_args` = `["fmt", "--check"]` (always whole repo;
`cargo fmt` has no per-package selection here).

Orchestration path (C:\workspace\tools\cargo-gate\src\gate.rs):
- `run()` resolves `orphans` once via `resolve_orphans` (only non-empty for
  `Scope::AppOnly`), then threads `&config.scope` + `&orphans` into clippy, the
  list pre-pass, the test run, and the optional app build.
- `resolve_orphans` -> `workspace::derive_orphans` (see section 3). On failure it
  prints "running full --workspace" and returns an empty orphan list.

Summary of the change surface for "default = default-members": the single
chokepoint is `scope_args` (nextest.rs) plus the parallel `clippy_args`
(fmt_clippy.rs). A new default scope variant that emits an EMPTY selection vec
would make nextest/cargo-test/cargo-build fall back to default-members; clippy's
builder must be updated in lockstep (it does not currently reach `scope_args`).

---

## 2. Exact current Scope and Mode enums and CLI flags

### Scope enum (C:\workspace\tools\cargo-gate\src\config.rs)
```
pub enum Scope {
    Workspace,                       // full `--workspace` -- the current DEFAULT
    AppOnly { closure_of: String },  // `--workspace --exclude <orphans>`
    Crate   { name: String },        // `-p <name>`
}
```
- `Scope::label()` -> history CSV `scope` column value: "workspace",
  "app-only", or "crate:<name>".
- `Scope::describe()` -> console banner text.

### Mode enum (config.rs)
```
pub enum Mode { Full, Fast }   // as_str() -> "FULL" / "FAST"
```
`--fast` sets `Mode::Fast` (exports PROPTEST_CASES=32); default is `Mode::Full`.

### BuildTarget enum (config.rs)
```
pub enum BuildTarget { Scope, Bin(String) }  // for the optional --build phase
```

### Resolution: Cli -> Config (config.rs `Config::from_cli`)
Scope precedence, highest first:
1. `--crate <name>`      -> `Scope::Crate`
2. `--app-only` (requires `--closure-of <crate>`) -> `Scope::AppOnly`
3. otherwise             -> `Scope::Workspace`  (the DEFAULT)

### Current CLI surface (clap derive, `Cli` struct in config.rs)
Gate flags (top-level, no subcommand == run the gate):
- `--fast`                     : fast inner-loop mode (PROPTEST_CASES=32)
- `--app-only`                 : requires `--closure-of`; sets AppOnly scope
- `--closure-of <CRATE>`       : the crate whose dep closure defines app-only
- `--crate <CRATE>`            : single-crate scope (`-p <name>`)
- `--log-dir <DIR>`            : artifact dir (default `.gate`)
- `--step-timeout <SECS>`      : per-step watchdog (default 1800)
- `--build [<BIN>]`            : optional post-test build phase (num_args 0..=1)
- `--release`                  : release profile (composes with --build)
- `--version` / `--help`       : clap built-ins

Subcommand:
- `report` (`cargo gate report`) with `--root <CRATE>` (repeatable),
  `--report-out <FILE>`, `--log-dir <DIR>`. Added in 0.2.0, strictly additive.

`strip_subcommand_token` drops the leading `gate` token so `cargo gate ...`
parses cleanly. No subcommand -> `Invocation::Gate(Config)`.

IMPORTANT: there is currently NO `--workspace` flag and NO `--all` flag on the
CLI. `Scope::Workspace` is reached implicitly as the default, never by an
explicit user flag.

---

## 3. What --app-only / derive_orphans does today

Purpose: run the gate over ONLY the dependency closure of one named binary,
excluding "orphan" workspace members (members not in that closure). This is the
existing, narrower scope concept and must be preserved (removing `--app-only`
would be a semver break).

- CLI: `--app-only --closure-of <crate>` -> `Scope::AppOnly { closure_of }`.
  clap `requires = "closure_of"` enforces the pairing.
- `gate::resolve_orphans` calls
  `workspace::derive_orphans(cargo, root, closure_of)`
  (C:\workspace\tools\cargo-gate\src\workspace.rs).
- `derive_orphans`:
  1. `member_names` via `cargo metadata --no-deps --format-version 1`.
  2. `dependency_closure` via
     `cargo tree -p <closure_of> --edges normal --prefix none` (workspace
     members in the normal-dep tree; the root itself always included).
  3. returns sorted (members - closure) == the orphan list.
  On ANY failure it returns an empty vec; the caller then prints
  "cargo-gate: --app-only could not derive the orphan list; running full
  --workspace." and runs the full workspace (never a wrong subset).
- `scope_args` turns AppOnly into `["--workspace", "--exclude", <orphan>, ...]`
  for nextest/cargo-test/cargo-build. (Clippy ignores the exclude list and runs
  plain `--workspace`, per section 1.)

Related but SEPARATE: `cargo gate report` reuses the same primitives
(`member_names`, `dependency_closure`, `workspace_facts`) to compute a
multi-root wired-in/orphan report. It does not change gate selection.

Relationship to the requested change: `--app-only` is a "workspace minus
orphans" selector; the requested change adds a DIFFERENT axis -- default to
Cargo `default-members`, with `--workspace`/`--all` restoring the full-members
set. These must be reconciled (e.g. decide how `--app-only` interacts with the
new default; today AppOnly implies the full `--workspace` base). Nothing here
may be silently removed.

---

## 4. Current version string

0.2.0 (Cargo.toml `version = "0.2.0"`).

CHANGELOG.md state: "## [Unreleased]" is empty; the last released section is
"## [0.1.0] - 2026-10-06". The 0.2.0 report feature is in Cargo.toml and in
docs but NOT yet reflected under a CHANGELOG 0.2.0 heading (Unreleased is bare).

---

## 5. Exact public-contract surface and which parts a default-change touches

The project treats three things as a published, semver-governed contract
(requirements.md NFR-2, Req 11; design.md section 8.9):

### (a) CLI surface (flags, subcommands, their meanings)
Current gate flags: `--fast`, `--app-only`, `--closure-of`, `--crate`,
`--log-dir`, `--step-timeout`, `--build`, `--release`; subcommand `report`
(+ `--root`, `--report-out`, `--log-dir`). Default (no flag) == full workspace.

TOUCHED by the requested change:
- CHANGE IN DEFAULT BEHAVIOR: default run moves from full `--workspace` to
  Cargo `default-members`. This is a semver-NOTABLE behavior change even though
  no flag is removed, because the meaning of the default invocation changes.
- ADDITIVE: a new scope flag `--workspace` with alias `--all` selecting the
  full members set (today's default behavior). Adding flags is additive, but
  the default-behavior change is the semver-notable part (minor at least; the
  change to what the bare command does is arguably a breaking behavior change
  for anyone relying on the bare command gating everything -- call this out).
- The new flag must map to `Scope::Workspace` (full `--workspace`), and a NEW
  default scope (default-members) must be introduced. The history `scope`
  label (see (c)) gains a new value for the new default.

NOT removed: `--crate`, `--app-only`/`--closure-of`, `--build`, `--release`,
`--fast`, `report` all stay (removing any is a break).

### (b) Exit-code meanings (C:\workspace\tools\cargo-gate\src\phase.rs
`Verdict::exit_code`, and main.rs)
- 0 = CLEAN
- 1 = ISSUES (any step had issues/test failures)
- 2 = TIMEOUT (a step hit the watchdog) AND also the fatal/usage exit
  (main.rs returns ExitCode 2 on clap parse error and on a fatal gate error;
  the `report` subcommand uses 0 success / 2 fatal-usage).

NOT touched by a default-scope change. Exit-code meanings stay identical.

### (c) gate.history.csv columns (C:\workspace\tools\cargo-gate\src\history.rs
`HEADER`)
```
run_id,timestamp,mode,scope,step,duration_secs,status,warnings,errors,
items_total,items_passed,items_failed
```
Plus a per-run "overall" row (step label "overall") written by
`append_overall_row`. `step` values: fmt, clippy, build, test, app-build,
overall. `scope` values today: workspace, app-only, crate:<name>.

TOUCHED only in CONTENT, not in COLUMNS: the new default scope needs a `scope`
label value (e.g. "default-members") so ETAs for the new default do not mix
with full-workspace history. The COLUMN SET does not change (additive label
value only, consistent with how "app-only"/"crate:x" are already just values).
Choosing a distinct label is important so per-scope ETA medians
(`eta_for_step`, `overall_eta_total`, filtered by the `scope` column) stay
coherent across the old and new defaults.

### Summary: what a default-change touches
- MUST change: default `Scope` resolution in `config.rs` (new default variant);
  `scope_args` in nextest.rs (emit empty selection for the new default);
  `clippy_args` in fmt_clippy.rs (lockstep: emit no `--workspace` for the new
  default); add `--workspace`/`--all` flag(s) mapping to the full-members scope;
  add a `Scope::label()`/`describe()` value for the new default.
- MUST preserve: all existing flags, exit codes, and the CSV column set.
- MUST reconcile: how `--app-only` (and `--crate`) coexist with the new
  default (precedence order), and the semver story (default-behavior change).
- Tests that encode current default: `config.rs`
  `default_scope_is_full_workspace` and `no_subcommand_resolves_to_a_gate_run`
  assert `Scope::Workspace` as the default; nextest.rs
  `scope_args_for_workspace_and_crate_and_apponly` asserts Workspace -> `--workspace`;
  fmt_clippy.rs `clippy_args_use_workspace_by_default`. These will need updating
  to reflect the new default and the new explicit flag.

---

## 6. Base branch

main (confirmed, see top of file). Do NOT assume this was main without the
check -- it was verified from the git output log.
