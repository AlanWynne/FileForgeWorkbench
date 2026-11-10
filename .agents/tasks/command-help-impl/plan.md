# Implementation Plan -- CR-NR-119 Root-Command Help Model

Worktree (read + edit source HERE): `c:/workspace/apps/FileForgeWorkbench/.worktrees/command-help-impl`
Git: `git -C c:/workspace/apps/FileForgeWorkbench/.worktrees/command-help-impl ...`

Approved sources (already read; do not re-audit): context-help requirements.md Req 21/22/23,
context-help design.md section "CR-NR-119 -- Root-Command Help Model (design)" (from line 1718),
context-help tasks.md tasks 24-29, command-semantics Req 15, command-configurator Req 9,
command-framework Req 3.8/3.9, `.kiro/steering/command-help-guideline.md`.

Build/test per workspace gate-policy: SCOPED to the touched crate(s), logs read back.
Agent verification is COMPILE-ONLY plus quick scoped tests:
`cargo check -p <crate>`, `cargo clippy -p <crate> -- -D warnings`, `cargo fmt -p <crate> -- --check`,
and `cargo test -p <crate> <focused_name>` where quick. The owner runs the full `cargo gate`.
Keep every new/changed source file under 400 non-test lines (rust-standards); split tests near 200.
All Markdown/help prose ASCII-only (documentation-charset). TDD: failing test first (testing.md),
each test carries `// Validates: Requirement X.Y`.

Framework verdict (design): NO framework change. The `--help` convention is a prelude-level
short-circuit on the EXISTING single front door `dispatch_command_string`, reusing the EXISTING
`open_help` seam -- no new dispatcher, no new navigation stack (framework-conformance mechanisms 1/5).

---

- [ ] 1. Add the pure one-line-description extractor to ff-help (context-help Req 21).
      Create `extract_one_line_description(body: &str) -> Option<String>` in a NEW file
      `crates/ff-help/src/one_liner.rs`: scan the body for a `## Description` heading
      (case-insensitive, any `#`-level heading whose text is "Description"), collect lines until the
      next heading or EOF, take the FIRST blank-line-delimited non-empty paragraph, strip inline
      Markdown (`` `code` `` -> code, `**bold**` -> bold, `[text](key)` -> text), collapse internal
      whitespace to single spaces, trim. Return `None` when there is no Description section / empty
      first paragraph / result length > 200 (Req 21.4). Pure: depends only on `body`. Register
      `pub mod one_liner;` and `pub use one_liner::extract_one_line_description;` in
      `crates/ff-help/src/lib.rs`. Write failing unit tests FIRST (in `one_liner.rs`, or a split
      `one_liner_tests.rs` if the file nears 200 test lines): first-Description-paragraph happy path;
      inline-markup stripping; multi-paragraph returns FIRST paragraph only; missing/empty Description
      -> None; over-200-chars -> None; a body modelled on `help/commands/session.help.md` cmd:SAVE.
      Files: `crates/ff-help/src/one_liner.rs`, `crates/ff-help/src/lib.rs`
      Verify: `cargo test -p ff-help one_liner` -- new unit tests pass; `cargo clippy -p ff-help -- -D warnings`.

- [ ] 2. Add the short-description lookup to the registry (context-help Req 22.2).
      Add method `pub fn short_description(&self, key: &TopicKey) -> Option<String>` on
      `HelpTopicRegistry` in `crates/ff-help/src/registry.rs`, delegating:
      `self.get(key).and_then(|t| crate::one_liner::extract_one_line_description(t.body()))`.
      Write failing tests FIRST in the registry test module: present topic with a Description returns
      the one-liner; absent topic returns `None`; a topic whose body lacks a Description returns `None`.
      Files: `crates/ff-help/src/registry.rs`
      Verify: `cargo test -p ff-help short_description` -- pass; `cargo fmt -p ff-help -- --check`.

- [ ] 3. Add the enforced-root-aware help open to ff-desktop (context-help Req 22.1, 22.8; design "HELP <cmd> routing").
      In `crates/ff-desktop/src/shell/help.rs` add `pub(super) fn open_root_help(&mut self, verb: &str)`:
      uppercase and trim `verb`; if it is a member of `crate::shell::alias_reader::builtin_ffcmd_roots()`
      (case-insensitive), resolve `ff_help::TopicKey::command(<UPPER>)` DIRECTLY and call
      `self.display_help_topic(&key)` (bypassing `resolve_help_argument`'s `feature:` keyword table so
      `KEYS` -> `cmd:KEYS` and `CONFIG` -> `cmd:CONFIG`, Req 22.8); otherwise fall back to
      `self.open_help(verb)` (existing behaviour, unchanged no-help affordance on a miss). Route the
      `HELP <ROOT>` ladder path for an enforced root through this same method so there is ONE resolution:
      in `crates/ff-desktop/src/shell/commands_ladder_a.rs` where `verb_arg(cmd, "HELP")` currently calls
      `self.open_help(arg)`, keep bare `HELP` -> `open_help("")`, but for `HELP <arg>` call
      `self.open_root_help(arg)` (open_root_help falls back to open_help for non-roots, preserving
      LINECOMMANDS/MACRO/OFF/etc. because those are resolved inside open_help's `resolve_help_argument`
      only for non-root args; verify OFF/INDEX/LINECOMMANDS/MACRO/API are NOT in builtin_ffcmd_roots()).
      Write failing tests FIRST (ff-desktop, a tests_* module near the HELP tests): `HELP KEYS` opens
      `cmd:KEYS` not `feature:function_keys`; `HELP CONFIG` opens `cmd:CONFIG` not `feature:configuration`;
      `HELP LINECOMMANDS` still opens `line:index`; `HELP OFF` still closes. (Assert via the help context
      panel's shown topic / registry miss, mirroring existing tests_nav HELP tests.)
      Files: `crates/ff-desktop/src/shell/help.rs`, `crates/ff-desktop/src/shell/commands_ladder_a.rs`
      Verify: `cargo test -p ff-desktop open_root_help` (or the chosen test name) -- pass; `cargo check -p ff-desktop`.

- [ ] 4. Add the `--help` / `-h` / `/?` prelude short-circuit on the single front door (command-semantics Req 15).
      Add helper `first_token_is_help_flag(arg: &str) -> bool` (match the FIRST whitespace token of
      `arg` against `--help`, `-h` case-insensitively and `/?` exact) -- place it in
      `crates/ff-desktop/src/shell/helpers.rs` next to `split_verb_arg`. In the front door
      `crates/ff-desktop/src/shell/dispatch.rs::dispatch_command_string`, AFTER the `rewrite_verb_to_root`
      rewrite and BEFORE `run_command_prelude`/`resolve_target`, split the (already-root-rewritten) `raw`
      via `split_verb_arg`; if `first_token_is_help_flag(arg)` then call `self.open_root_help(verb)` and
      `return` (short-circuit: command never executes; not recorded in history/undo like HELP). Single-token
      verbs only -- two-word roots (`FILE CATALOGS`, `RESET BARE`) are a documented deferred limitation
      (Req 15.9): their first arg token is not a help flag so the branch naturally does not fire.
      Write failing ff-desktop tests FIRST: `EXIT --help` opens `cmd:EXIT` and does NOT set the app-close
      flag (no execution); `-h` and `/?` behave identically; `SS --help` opens `cmd:SNAPSHOT` (rewrite ran
      first); `FIND --help foo` where `--help` is the first arg token short-circuits but `FIND foo --help`
      does NOT; a missing topic degrades to the no-help affordance; `FILE CATALOGS --help` /
      `RESET BARE --help` do NOT short-circuit while `HELP FILE CATALOGS` / `HELP RESET BARE` resolve their
      `cmd:` topics.
      Files: `crates/ff-desktop/src/shell/helpers.rs`, `crates/ff-desktop/src/shell/dispatch.rs`, a ff-desktop tests_* module
      Verify: `cargo test -p ff-desktop help_flag` (chosen names) -- pass; `cargo clippy -p ff-desktop -- -D warnings`.

- [ ] 5. Author cmd:<ROOT> help topics for the missing FFCMD roots (context-help Req 21/23 content).
      Enumerate the authority at implementation time: the roots are
      `crate::shell::alias_reader::builtin_ffcmd_roots()` (= `BUILTIN_FFCMD_LADDER_VERB_ROOTS` +
      `BUILTIN_WORKSPACE_VERB_ROOTS` + the code-only built-in menu option roots); do NOT trust a prose
      list -- read the constants in `crates/ff-desktop/src/command_config/mod.rs` and the recovery menus in
      `crates/ff-desktop/src/menu_workspace/defaults.rs`. Already have topics (verify, do not duplicate):
      SAVE, CANCEL, END, UNDO, REDO, HELP, KEYS (in `help/commands/menu-options.help.md`), POM, SETTINGS,
      FILES, navigation verbs. Author a `cmd:<ROOT>` topic (minimum a non-empty `## Description` whose first
      paragraph is a <=200-char one-liner; add Syntax/See Also per Req 7.2 where natural) for every MISSING
      enforced root, grouped into a few new files under `help/commands/` (e.g. `session-lifecycle.help.md`
      for EXIT/START/MENU/CLOSE/RETURN; `scrm.help.md` for SNAPSHOT/CAPTURE START/STOP/REPLAY/RETRIEVE;
      `workspace-verbs.help.md` for CONFIG/THEME/CATALOGS/COMMANDS/LOG/PLUGINS/MACROS/GSEARCH/SEARCH/
      KINDS/MENUS/ALIASES/WORKSPACE/DOCK/DETACH/SPLIT/UNSPLIT/FOCUS/SWAP/PFSHOW/COMMAND/SUBMIT/TIME/
      STATUS/REFRESH/EDIT/VIEW/BROWSE/COMPARE/CREATE/REPLACE). Author `cmd:CONFIG` (new; MAY See-Also
      `feature:configuration`). Two-word roots key as `cmd:FILE CATALOGS` and `cmd:RESET BARE`
      (TopicKey preserves the space; confirmed). Use the `<!-- TOPIC: cmd:<ROOT> -->` / `<!-- TITLE: ... -->`
      delimiters exactly as `help/commands/session.help.md`. ASCII-only.
      Files: new `help/commands/*.help.md` files (plus `help/commands/menu-options.help.md` only if KEYS
      needs a Description added for the extractor)
      Verify: `cargo test -p ff-desktop <enforcement_test>` from task 6 goes from RED to GREEN once all topics exist.

- [ ] 6. Add the ff-desktop enforcement test + command-framework description-rule alignment (context-help Req 23; command-framework Req 3.8/3.9).
      Write the enforcement test in ff-desktop (the crate owning the FFCMD root set and the registry
      wiring): enumerate `EnvironmentRegistry::root_commands("FFCMD")` (= `builtin_ffcmd_roots()`), load the
      shipped `help/` tree the way `shell/help.rs` does (reuse `load_help_topics_for_locale("en")` /
      `ContentLoader` + a registry), and assert for EACH root that a `cmd:<ROOT>` topic is present AND
      `registry.short_description(&TopicKey::command(<ROOT>))` is `Some(non-empty, <=200)`. FAIL if any
      enforced root lacks a conforming topic. Add a comment: FFEDIT roots are OUT OF SCOPE now (Req 23.4,
      on the roadmap); extension commands are NEVER asserted (Req 23.5) and degrade gracefully; NO
      registration-time validation of extension help is added. Confirm no code path treats command-framework
      Req 3 as REQUIRING a separate description literal for a home-grown FFCMD root (the derived one-liner
      satisfies it) -- adjust a test comment if a registration path asserts a description.
      Files: a new/updated ff-desktop test module (e.g. `crates/ff-desktop/src/shell/tests_help_enforcement.rs`)
      Verify: `cargo test -p ff-desktop ffcmd_roots_have_conforming_help` -- GREEN after task 5; `cargo check -p ff-desktop`.

- [ ] 7. Wire display-time short-description inheritance into the alias editor and listings (command-configurator Req 9; context-help Req 22.3-22.7).
      Inheritance is DISPLAY-ONLY and MUST NOT be persisted (Req 9.2 / 22.7): the stored
      `AliasRowEdit.description` stays the user's own value (empty for the seed), so SAVE / Reset-bare are
      unchanged. Add a helper on the shell (e.g. in `crates/ff-desktop/src/shell/alias_editor.rs`)
      `effective_alias_description(env, canonical, stored: &str) -> String`: if `stored` non-empty return it;
      else return `self.help.registry.short_description(&TopicKey::command(&canonical.to_uppercase()))`
      unwrapped to empty (Req 9.4 graceful None). Apply it ONLY at display seams, NOT in
      `load_effective_alias_rows` (which feeds the persisted working copy): (a) the egui render
      `crates/ff-desktop/src/alias_editor_panel/rows.rs::render_data_row` -- when `row.description` is empty,
      show the inherited one-liner as the field's `hint_text` (keeps `row.description` empty so SAVE persists
      empty); (b) the capture/display `crates/ff-desktop/src/shell/screen_providers.rs` alias table
      (`alias_table()`) -- fill the Description cell with the effective value. Chained alias (`SS -> SNAPSHOT`)
      inherits SNAPSHOT's one-liner because `canonical` is the resolved root. Wire the same
      `short_description` lookup into the command configurator / palette listing descriptions
      (`command_configurator_screen_model` / its displayed defs) for home-grown FFCMD roots (Req 9.3).
      Write failing ff-desktop tests FIRST: an empty-description row for `X -> EXIT` DISPLAYS the `cmd:EXIT`
      one-liner; a NON-empty user description wins; a root with no topic shows empty; the PERSISTED seed row
      keeps its empty description (assert the save path / working row stored value is unchanged);
      `SS -> SNAPSHOT` inherits SNAPSHOT's one-liner.
      Files: `crates/ff-desktop/src/shell/alias_editor.rs`, `crates/ff-desktop/src/alias_editor_panel/rows.rs`,
      `crates/ff-desktop/src/shell/screen_providers.rs`, a ff-desktop tests_* module
      Verify: `cargo test -p ff-desktop alias_description` (chosen names) -- pass; `cargo clippy -p ff-desktop -- -D warnings`; `cargo fmt -- --check`.

---

## Digest

### crates/ff-help/src/one_liner.rs (NEW)
- Add `pub fn extract_one_line_description(body: &str) -> Option<String>` -- pure; finds `## Description`
  (case-insensitive, any heading level), first non-empty paragraph, strips inline markup, collapses
  whitespace, trims; `None` if missing/empty or > 200 chars (Req 21.3/21.4).
- Tests here (or split `one_liner_tests.rs` if > ~200 lines). Model fixture bodies on `session.help.md`.
- Verified: file does NOT exist today (`one_liner.rs` absent). `HelpTopic::body()` exists (used in
  registry tests as `topic.body()`).

### crates/ff-help/src/lib.rs
- Block to change: the "Public Modules" list and the "Public API Re-exports" block (the `pub mod` list
  ending with `pub mod config;` and the `pub use ...` list ending with
  `pub use topic_key::{TopicCategory, TopicKey};`). Add `pub mod one_liner;` and
  `pub use one_liner::extract_one_line_description;`.
- Covered by: ff-help one_liner + short_description tests.

### crates/ff-help/src/registry.rs
- Block to change: `impl HelpTopicRegistry` (currently `new` ... `is_empty`). Add
  `pub fn short_description(&self, key: &TopicKey) -> Option<String>` delegating to `get` + the extractor.
- Test module at file end (`#[cfg(test)] mod tests`) -- add `short_description_*` tests.
- Verified today: `get(&self, key) -> Option<HelpTopic>` and `contains` exist; tests use `TopicKey::command(..)`,
  `register_file_topic`, `topic.body()`.

### crates/ff-desktop/src/shell/help.rs
- Block to change: `impl WorkbenchShell` -- `open_help(&mut self, arg: &str)` and the private
  `display_help_topic(&mut self, key: &TopicKey)` already exist and are the single display seam. Add
  `pub(super) fn open_root_help(&mut self, verb: &str)` that resolves `cmd:<UPPER>` directly for a
  `builtin_ffcmd_roots()` member (Req 22.8) else delegates to `open_help`.
- Verified: `self.help.registry` is `Arc<HelpTopicRegistry>`; `display_help_topic` already handles
  dynamic index/function_keys topics and the miss -> index-with-message path; `resolve_help_argument`
  (ff-help) special-cases KEYS/CONFIG to `feature:*` BEFORE the `cmd:` fall-through (the divergence 22.8 fixes).
- Test: existing HELP tests in `shell/tests_nav.rs` / `tests_command.rs` (`shell.handle_command("HELP")`);
  add new ones asserting the KEYS/CONFIG precedence and the root-vs-feature resolution.

### crates/ff-desktop/src/shell/commands_ladder_a.rs
- Block to change: the `HELP` arm (~line 189): `if upper == "HELP" { self.open_help(""); ... }` and
  `if let Some(arg) = verb_arg(cmd, "HELP") { self.open_help(arg); ... }`. Repoint the `HELP <arg>` arm to
  `self.open_root_help(arg)` (bare HELP stays `open_help("")`).
- Covered by: the new HELP-precedence tests in task 3.

### crates/ff-desktop/src/shell/helpers.rs
- Block to change: add `pub(super) fn first_token_is_help_flag(arg: &str) -> bool` next to the existing
  `pub(super) fn split_verb_arg(cmd: &str) -> (&str, &str)` (line ~289, splits on first whitespace only --
  this is exactly why two-word roots are deferred, Req 15.9).
- Covered by: the `--help` short-circuit tests in task 4.

### crates/ff-desktop/src/shell/dispatch.rs
- Block to change: `impl WorkbenchShell { pub(super) fn dispatch_command_string(&mut self, raw: &str) }`.
  After `let raw = self.rewrite_verb_to_root(raw); let raw = raw.as_str();` and BEFORE
  `if self.run_command_prelude(raw, &upper) { return; }`, insert the help-flag short-circuit:
  `split_verb_arg(raw)` -> if `first_token_is_help_flag(arg)` then `self.open_root_help(verb); return;`.
- Verified: `dispatch_command_string` is the ONE front door; the `=` strip and `rewrite_verb_to_root`
  already run before the prelude; adding the short-circuit here keeps a single intercept (framework mechanism 1).
- Test: the existing `dispatch.rs` test module and a new ff-desktop tests_* module for the short-circuit.

### help/commands/*.help.md (NEW files + possibly menu-options.help.md)
- Author `cmd:<ROOT>` topics for every MISSING `builtin_ffcmd_roots()` member (enumerate from the code
  at implementation time; see task 5 grouping). Delimiter format = `session.help.md`. ASCII-only.
- Verified existing topics: `help/commands/session.help.md` (SAVE/CANCEL/END/UNDO/REDO),
  `help/commands/navigation.help.md`, `help/commands/menu-options.help.md` (KEYS), plus change/display/find.
- Covered by: the enforcement test (task 6).

### crates/ff-desktop/src/shell/tests_help_enforcement.rs (NEW)
- Enforcement test: `root_commands("FFCMD")` each has a conforming `cmd:<ROOT>` topic with a valid one-liner.
  Loads the shipped `help/` tree (reuse the `shell/help.rs` loader path). FFEDIT excluded; extensions never asserted.
- Verified: `EnvironmentRegistry::root_commands("FFCMD")` (environment_registry.rs:234) returns
  `builtin_ffcmd_roots()`; `builtin_ffcmd_roots()` lives in `shell/alias_reader.rs:155`.

### crates/ff-desktop/src/shell/alias_editor.rs
- Block to change: `load_effective_alias_rows(&self) -> Vec<AliasRowEdit>` (keeps stored description; do NOT
  inherit here). Add `effective_alias_description(&self, env, canonical, stored) -> String` using
  `self.help.registry.short_description(cmd:<ROOT>)` with empty fallback (Req 9.4).
- Verified: `open_alias_editor` -> `load_effective_alias_rows` -> `alias_editor_panel.load_rows(rows)`;
  the working copy is serialised on SAVE (`alias_editor_persist` -> `save_user_aliases`), hence inheritance
  must stay out of the stored field (Req 9.2).

### crates/ff-desktop/src/alias_editor_panel/rows.rs
- Block to change: `render_data_row(...)` Description cell (binds `egui::TextEdit::singleline(&mut row.description)`
  with `.hint_text("note")`). When `row.description` is empty, set `hint_text` to the inherited one-liner so the
  stored value stays empty.
- Covered by: task 7 tests.

### crates/ff-desktop/src/shell/screen_providers.rs
- Block to change: `alias_editor_screen_model` / `alias_table()` (the capture Description column) and
  `command_configurator_screen_model` (listing descriptions). Fill the effective (inherited) description for display.
- Covered by: task 7 tests.

### Build/test commands (discovered)
- Per-crate scoped (gate-policy): `cargo check -p <crate>`, `cargo clippy -p <crate> -- -D warnings`,
  `cargo fmt -p <crate> -- --check`, `cargo test -p <crate> <focused_name>`.
- Crates touched: `ff-help` (tasks 1-2), `ff-desktop` (tasks 3-7).
- Full `cargo gate` is the OWNER's manual run; agents compile-verify + quick scoped tests only.
- Verified green today (user message 10): full gate CLEAN 4990/4990 on the sibling worktree baseline.
