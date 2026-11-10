# Root-command help model: one help file, two conventions, enforced for FFCMD

CR-NR-119 gives every home-grown FFCMD root command a single Markdown help file that is the source of truth for both its full help and its one-line description. A pure extractor in `ff-help` derives the one-liner from the first paragraph of a topic's `## Description` section, so no description string is duplicated between code and docs. Two dispatch reach-paths bring the ISPF `HELP <cmd>` form and the POSIX `<cmd> --help` form to the same `cmd:<ROOT>` topic through the one `open_help`/`display` seam, with no new dispatcher. The missing `cmd:<ROOT>` topics are authored for every FFCMD root, an enforcement test fails the suite if a root ever lacks one, and the alias editor and command palette inherit the derived one-liner at display time without persisting it.

Watch for: `shell/help.rs` is now ~499 non-test lines, over the 400-line rust-standards limit the change grew rather than split (confirmed, self-reported as a follow-up). Everything else conforms to the spec and the framework rules.

**Verdict**: APPROVED

## High-level view

The extractor (`ff-help/src/one_liner.rs`) is a pure function of the topic body: it finds the `## Description` heading, takes the first blank-line-delimited paragraph, strips inline Markdown, collapses whitespace, and returns `None` on a missing section, empty paragraph, or a result over 200 chars. `HelpTopicRegistry::short_description` is the single lookup every consumer calls. This matches Req 21.3-21.5 and, because it reads whatever topic the registry holds for the active locale, 21.6 falls out for free.

Dispatch adds exactly two reach-paths and no new front door. `open_root_help(verb)` resolves `cmd:<ROOT>` directly for any `builtin_ffcmd_roots()` member, which is what repoints `HELP KEYS` to `cmd:KEYS` and `HELP CONFIG` to `cmd:CONFIG` ahead of the legacy `feature:` keyword special-cases (Req 22.8); a non-root argument delegates unchanged to `open_help`. The `--help` / `-h` / `/?` short-circuit sits on the one front door after `rewrite_verb_to_root` and before the prelude, so `EXIT --help` opens help without exiting and `SS --help` resolves `cmd:SNAPSHOT` because the alias rewrite ran first (Req 15.1-15.4). The enforced set used by `open_root_help` and the set the enforcement test enumerates are the same source (`root_commands("FFCMD")` is built from `builtin_ffcmd_roots()`), so the precedence rule and the authoring gate cannot drift.

Every FFCMD root has a conforming topic. The enforcement test loads the actual shipped `help/` tree through the same `ContentLoader` the shell uses and asserts a `cmd:<ROOT>` topic with a valid one-liner for each root `root_commands("FFCMD")` enumerates; FFEDIT is explicitly out of scope and extensions are never asserted (Req 23.4/23.5). The authored topics plus the pre-existing ones (KEYS, FILES, SAVE, CANCEL, END, POM, SETTINGS) cover the union, and the new files are ASCII-only.

Inheritance is display-time only. The alias editor shows the canonical root's one-liner as the Description cell hint when a row's own description is empty, and the command palette falls back to the same lookup when a command's metadata description is empty. The stored row value stays empty, so the `X -> EXIT` seed write and Reset-bare are untouched and do not depend on help-registry readiness (Req 9.2/22.7). A missing topic degrades to an empty string, never a panic.

The one blemish is file size: `shell/help.rs` crossed the 400-line limit (462 before, ~499 after). The coder flagged a behaviour-preserving split as separate refactor work.

<details>
<summary>Issues (1)</summary>

1. **help.rs over the 400-line limit** — `shell/help.rs` is ~499 non-test lines (was 462); rust-standards requires splitting before adding more. Confirmed, behaviour-correct, self-reported as a follow-up refactor. Non-blocking.

</details>

<details>
<summary>Details</summary>

### One-liner extraction is a pure, bounded function

`extract_one_line_description` (confirmed) scans for a heading whose text equals `Description` case-insensitively, collects lines until the next heading or a blank line, joins the first paragraph, strips links/code/emphasis to plain text, collapses whitespace, and rejects an empty result or one over 200 chars (`MAX_ONE_LINE_CHARS`). This is the Option (c) design from Req 21 confirmed in `design.md`: it reuses the existing `## Description` section and adds no new parse syntax, so Req 21.7 (standard Markdown, no new markup) holds. `short_description` on the registry delegates to it and returns `None` for an absent key or a topic with no usable Description, which is the single API Req 22.2 mandates. The link stripper handles `[label](target) -> label` and leaves unbalanced brackets literal; the unit tests cover the happy path, inline markup, multi-paragraph (first only), missing/empty section, the 200-char cap, a wrapped multi-line first paragraph modelled on the real `cmd:SAVE` shape, and case-insensitive heading match, each with a `// Validates:` annotation.

### The `--help` short-circuit short-circuits execution, not just display

The short-circuit is placed in `dispatch_command_string` after the `=` strip and `rewrite_verb_to_root`, before `run_command_prelude` and `resolve_target`:

```
raw -> strip '=' -> rewrite_verb_to_root -> [--help short-circuit] -> prelude -> resolve_target -> ladder
```

Because it returns before the prelude, `EXIT --help` opens `cmd:EXIT` without running the EXIT handler (test asserts `should_close` stays false), satisfying Req 15.2's "side effects SHALL NOT run". Running after the rewrite is what makes `SS --help` resolve `cmd:SNAPSHOT` (Req 15.3), proven by seeding the `SS -> SNAPSHOT` overlay then dispatching. `first_token_is_help_flag` inspects only the first whitespace-delimited arg token, so `FIND --help foo` fires and `FIND foo --help` does not (Req 15.4/15.7), both tested. A missing topic opens the Help Context on the index-with-message miss path rather than erroring (Req 15.5), tested with an empty registry.

### Two-word roots are a declared, tested limitation

`split_verb_arg` splits on the first whitespace, so for `FILE CATALOGS --help` the first arg token is `CATALOGS`, not a help flag, and the short-circuit naturally does not fire. Req 15.9 declares this a deferred known limitation, and the ISPF reach-path still covers it: `two_word_root_help_flag_is_deferred_but_help_cmd_resolves` confirms `HELP FILE CATALOGS` resolves `cmd:FILE CATALOGS` (the space-bearing `TopicKey` is intact). Both two-word roots retain a reachable single source of truth, so nothing is left unhelpable.

### Root precedence and the enforcement gate share one source

`open_root_help` checks membership in `builtin_ffcmd_roots()`; the enforcement test enumerates `EnvironmentRegistry::root_commands("FFCMD")`, which is built from `builtin_ffcmd_roots()`. A new FFCMD root therefore lands in both the direct-resolution precedence set and the authoring gate at once, so `HELP <newroot>` and the "every root has a topic" assertion cannot disagree. The test loads the shipped tree with `ContentLoader` and asserts both a loaded `cmd:<ROOT>` topic and a non-`None` `short_description` per root, so authoring a root without a help file breaks the suite (Req 23.2/23.3). `ffedit_roots_are_not_enforced_by_this_check` keeps FFEDIT out of scope (Req 23.4) and documents the distinct sets.

### Inheritance stays on the display side of the save boundary

`refresh_alias_inherited_descriptions` builds a `UPPERCASE canonical -> one-liner` map over both environments' root commands plus the working rows, once per open/reload, and stores it transiently on `AliasEditorState`. The render reads it as the Description cell `hint_text` only when the row's own description is empty, so a user-entered value always wins (Req 9.1/22.4) and the inherited value is a placeholder, never the field value. The map omits roots with no topic, so a row targeting an un-topic'd command shows the generic `note` hint and the stored description stays empty (Req 9.4). The seed row's persisted Description is unchanged, which is what keeps the seed write and Reset-bare independent of help-registry readiness (Req 9.2/22.7) — `empty_description_row_inherits_root_one_liner_display_only` asserts both the inherited map carries the `cmd:EXIT` one-liner and the stored row stays empty. The palette applies the same lookup as a fallback only when `meta.description` is empty, never writing back into the registry (Req 9.3).

</details>

<details>
<summary>File map</summary>

- `crates/ff-help/src/one_liner.rs` (new) — pure one-liner extractor + unit tests.
- `crates/ff-help/src/registry.rs` — `short_description` lookup delegating to the extractor + tests.
- `crates/ff-help/src/lib.rs` — module + re-export.
- `crates/ff-desktop/src/shell/help.rs` — `open_root_help` direct `cmd:<ROOT>` resolution (file now over 400 lines).
- `crates/ff-desktop/src/shell/dispatch.rs` — `--help`/`-h`/`/?` front-door short-circuit.
- `crates/ff-desktop/src/shell/helpers.rs` — `first_token_is_help_flag`.
- `crates/ff-desktop/src/shell/commands_ladder_a.rs` — `HELP <arg>` arm repointed to `open_root_help`.
- `crates/ff-desktop/src/shell/alias_description.rs` (new) — display-time inherited-description helpers + tests.
- `crates/ff-desktop/src/shell/alias_editor.rs` — `refresh_alias_inherited_descriptions` on open/reload.
- `crates/ff-desktop/src/alias_editor_panel/{state,host,rows}.rs` — inherited-descriptions map + Description-cell hint.
- `crates/ff-desktop/src/shell/update_dialogs.rs` — palette entries inherit the root one-liner when metadata is empty.
- `crates/ff-desktop/src/shell/tests_help_flag.rs` (new) — 11 dispatch reach-path tests.
- `crates/ff-desktop/src/shell/tests_root_help_enforcement.rs` (new) — FFCMD root help enforcement + FFEDIT-exclusion tests.
- `crates/ff-desktop/src/shell/tests_alias_editor.rs`, `shell/mod.rs` — new tests + module registration.
- `help/commands/{session-lifecycle,scrm,file-open-verbs,workspace-verbs,status-verbs}.help.md` (new) — authored `cmd:<ROOT>` topics for every FFCMD root (ASCII-only).

Full diff: `git -C .worktrees/command-help-impl diff b81783d...HEAD`

</details>
