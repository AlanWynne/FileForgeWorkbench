# FFWB Core-Framework Simplification Reassessment (READ-ONLY)

Scope: deletion-over-addition. Identify dead/duplicate/superseded code to DELETE,
oversized files to SPLIT, orphan crates to classify, and duplicate action paths
to collapse. No code or docs were changed. All citations are from source read
during this investigation. ASCII only.

---

## SUMMARY ANSWER (deletion-first)

1. B080 Step 7 "dead ladder arms": The arms marked "SUPERSEDED / unreachable
   fallback" in the ladder are **NOT actually dead**. They are the LIVE path for
   several non-front-door callers of `handle_command` (menu-bar clicks, POM
   option-key recursion, chained fastpath segments, nav-stack reconstruction).
   `resolve_target` runs ONLY in the front door `dispatch_command_string`;
   `handle_command` deliberately does not run it. Therefore the only
   behaviour-preserving deletions in this area are TWO genuinely-dead
   `#[allow(dead_code)]` helpers plus one bare-THEME ladder branch -- NOT the
   CustomWorkspace/navigation arms. Confidence that the broad arms are reachable:
   certain. See Section 1.

2. Oversized files: all four split cleanly as pure refactors (no behaviour
   change). Concrete per-file splits in Section 2. Confidence: certain for the
   split boundaries; the only judgement call is `tab_state.rs` (constructor
   boilerplate), covered below.

3. Orphan crates: a large family of crates is outside the `ff-desktop` closure.
   Most are explicitly plumbing awaiting a PENDING-GATE wiring CR (editor
   decouple) or belong to the standalone `ff-mdx-*` app and the governance test
   harness -- keep-pending-wiring. A short list is genuinely unreferenced and is
   candidate-for-owner-decision. `ff-clipboard` is keep-pending-wiring (named by
   the parked editor-decouple CR). Section 3.

4. Duplicate action paths: `Ctrl+C` (copy) and `Ctrl+Z` (undo) are handled by
   raw `ui.input()` checks inside the editor paint/input code and mutate state /
   the OS clipboard directly, bypassing the single command dispatcher. These are
   genuine command-parity gaps (bug-class per the steering `1b` command-parity
   rule), but CLOSING them changes observable behaviour and is GATED work, not a
   deletion. Section 4.

---

## 1. B080 STEP 7 -- DEAD LADDER ARMS

### 1a. The reachability model (why most "superseded" arms are still live)

`resolve_target` is invoked in exactly ONE place: the front door
`dispatch_command_string` (crates\ff-desktop\src\shell\dispatch.rs, approx lines
50-85). `handle_command` (crates\ff-desktop\src\shell\commands.rs, approx lines
85-92) runs `run_command_prelude` then `run_command_ladder` and intentionally
does NOT call `resolve_target` -- the doc comment there (commands.rs approx lines
72-85, and dispatch.rs module doc approx lines 20-33) states this is deliberate
to avoid infinite recursion, because a resolved `Function` target is executed by
calling `handle_command(command_id)`.

Consequently every caller that invokes `handle_command` directly BYPASSES
`resolve_target` and reaches the ladder arms. Direct `handle_command` callers
found (grep of `handle_command(` in crates\ff-desktop):

- POM option-key recursion -- commands.rs `run_command_prelude`, approx line 158:
  `self.handle_command(&pom_command)`. A POM/menu Option_Command such as `FILES`,
  `CONFIG`, `KEYS`, `LOG`, `MACROS`, `PLUGINS` resolves its key to the command
  string and re-enters `handle_command`, hitting the ladder arm, NOT the front
  door.
- Menu-bar BUTTON clicks -- crates\ff-desktop\src\shell\render_chrome.rs approx
  lines 171-193: three sites call `self.handle_command(&option.command)` /
  `&child.command`. Clicking a menu-bar entry for an in-scope verb reaches the
  ladder arm directly.
- Chained fastpath segments -- crates\ff-desktop\src\shell\commands_fastpath.rs
  approx line 127: `self.handle_command(segment)` for each `=0.K` / `3.1` segment.
- Nav-stack reconstruction -- crates\ff-desktop\src\shell\nav_stack.rs approx
  line 232: `self.handle_command(&resolved)`.
- `AUTONUM`/`NUM` -> `NUMBER` redirects -- commands_ladder_c.rs approx lines
  133/142: `self.handle_command(&redirected)`.
- `ShellRequest::Command` -- crates\ff-desktop\src\shell\render.rs approx line
  147.
- `try_menu_name_dispatch` trailing-token re-dispatch -- commands_menu.rs approx
  line 225.

NOTE on the menu-OPTION-CLICK seam specifically: `activate_menu_option`
(commands_menu.rs approx lines 150-180) first calls
`resolve_and_dispatch_command` (which DOES run `resolve_target`) and only falls
through to `handle_command` on `FallThrough`. So the option-click seam DOES reach
`resolve_target` first. But the menu-BAR button seam (render_chrome.rs) calls
`handle_command` unconditionally and does not.

Conclusion: the arms the code comments label "SUPERSEDED (B080 Step 2) ...
unreachable fallback" are reachable through the menu-bar-click / POM-recursion /
chained-segment paths. **Deleting them would break those paths.** This is the
"looks dead but is still reachable via a non-resolve_target path" flag the brief
asked for -- and it applies to the ENTIRE CustomWorkspace/navigation arm set.

Affected arms that are NOT safe to delete (confidence: certain they are
reachable):

| Verb(s) | Arm location | Claimed earlier by (front door only) | Still reachable via |
|---------|--------------|--------------------------------------|---------------------|
| CONFIG [ns] | commands_ladder_b.rs approx L34-48 | builtin_workspace_target_for "config" | menu-bar click, POM option |
| FILES / =FILES | commands_ladder_b.rs approx L51-57 | "file_explorer" | menu-bar click, POM option |
| GSEARCH/SEARCH | commands_ladder_b.rs approx L59-64 | "search" | menu-bar click, POM option |
| COMMANDS | commands_ladder_b.rs approx L96-103 | "command_configurator" | menu-bar click, POM option |
| MENUS | commands_ladder_b.rs approx L112-118 | "menus" | menu-bar click (POM M / Settings M) |
| LOG | commands_ladder_b.rs approx L150-159 | "event_log" | menu-bar click, POM option |
| FILE CATALOGS / CATALOGS | commands_ladder_b.rs approx L161-166 | "files" | menu-bar click, POM option |
| PLUGINS | commands_ladder_b.rs approx L168-173 | "plugin_manager" | menu-bar click, POM option |
| MACROS | commands_ladder_b.rs approx L176-182 | "macro_library" | menu-bar click, POM option |
| KEYS [kind] | commands_ladder_a.rs approx L190-199 | "keys" | menu-bar click, POM option |
| KINDS | commands_ladder_a.rs approx L201-210 | "kinds" | menu-bar click, Settings option |
| THEME (bare) | commands.rs run_command_ladder approx L205-215 | "theme_editor" | menu-bar click, POM/Settings option |

The right way to make these arms genuinely dead is to route the menu-bar-click /
POM-recursion / chained-segment callers through the front door
(`dispatch_command_string`) instead of `handle_command`, so `resolve_target`
runs for them too. That is a (small) behaviour-neutral refactor that must land
BEFORE any arm deletion. Until then, B080 Step 7 arm deletion is blocked.
Confidence: needs-owner-decision (it is a refactor + sequencing decision, not a
pure delete).

### 1b. Genuinely dead code safe to DELETE now (confidence: certain)

These are dead regardless of the reachability question above:

- `function_target_with_arg` -- crates\ff-desktop\src\shell\dispatch.rs approx
  lines 325-345, annotated `#[allow(dead_code)] // no live caller yet; Step 3
  Function verbs were deferred.` Only caller is its own unit test in the same
  file. Deleting the fn AND its test
  (`single_verb_arg_split_populates_arg_param`, which exercises only the dead fn)
  is behaviour-preserving. Note the sibling test
  `verb_arg_split_preserves_argument_case` tests `split_verb_arg` (a live helper)
  and should be KEPT.
- `run_command_definition` -- crates\ff-desktop\src\shell\target_dispatch.rs
  approx lines 140-160, `#[allow(dead_code)]`, "its runtime caller is the
  binding-config surface ... a later Phase DB step". No non-test caller. This is
  intended future plumbing; deletion is behaviour-preserving TODAY but removes a
  staged API -- classify as needs-owner-decision (keep if the Command
  Configurator binding surface is imminent; delete under the deletion mandate if
  not).
- `NOT_IMPLEMENTED_MSG` const -- target_dispatch.rs approx lines 40-45,
  `#[allow(dead_code)]`, kept "ahead of its runtime consumers". Asserted by one
  test (`not_implemented_message_is_canonical`). Same classification as above:
  needs-owner-decision (staged, not yet consumed).

### 1c. Bare-THEME ladder branch (confidence: probable)

The bare `THEME` branch in `run_command_ladder` (commands.rs approx lines
205-215) is explicitly documented as "SUPERSEDED (B080 Step 2) ... unreachable
fallback (Step-2 follow-up deletes it)". BUT, like 1a, bare `THEME` is reachable
via a menu-bar click / POM option that calls `handle_command("THEME")`. The
`THEME <name>` apply branch directly below it is NOT superseded
(builtin_workspace_target_for returns None for a non-empty arg -- confirmed in
command_config\mod.rs approx lines 210-216 and the unit test
`builtin_workspace_target_classifies_nav_verb`). So the bare-THEME branch cannot
be deleted independently of the 1a refactor either. Keep together with the 1a
work.

### 1d. Net recommendation for Section 1

- DELETE now (certain, behaviour-preserving): `function_target_with_arg` + its
  single dedicated test.
- OWNER-DECISION: `run_command_definition`, `NOT_IMPLEMENTED_MSG` (staged APIs
  under a delete mandate).
- DO NOT delete the CustomWorkspace/navigation arms or bare-THEME until the
  menu-bar-click / POM-recursion / chained-segment callers are rerouted through
  the front door. The code comments claiming these arms are unreachable are
  OPTIMISTIC -- they describe the typed-line path only.

---

## 2. OVERSIZED FILES (pure REFACTOR, no behaviour change)

Line counts confirmed by reading each file; the 400-line rule (rust-standards.md)
excludes the `#[cfg(test)]` module.

### 2a. crates\ff-desktop\src\main.rs (~632 lines)

The bulk is `register_builtin_schema` (one giant `SchemaEntry` array, approx
lines 300-540) plus the OS reduce-motion FFI and the two-phase logging config.
`main()` and the CLI-arg helpers are small by comparison.

Proposed split (new sibling modules declared from `main.rs`):
- `main.rs` -- keep `main()`, `extract_profile_arg`, `resolve_cli_paths`, module
  declarations, and the `windows_subsystem` attribute. (coordinator)
- `startup_schema.rs` -- move `register_builtin_schema` and its `SchemaEntry`
  array (the largest block). This is self-contained (takes `&ConfigHandle` +
  `&Path`).
- `startup_env.rs` -- move `apply_os_reduce_motion`, `os_prefers_reduce_motion`
  (incl. the Windows FFI block), and `apply_logging_config`.
- Keep the existing `#[cfg(test)] mod tests` with each moved fn (or move the two
  schema-focused tests next to `startup_schema.rs`).

This follows the `_state` / helper-module split spirit; no `_render`/`_commands`
split applies since `main.rs` is boot wiring. Confidence: certain.

### 2b. crates\ff-desktop\src\config_panel\render.rs (~590 lines)

Already a sub-module of a `config_panel/` directory. Functions group into three
clean concerns (confirmed by the symbol scan):
- Keyboard/tree navigation: `key_widget_id`, `tree_has_keyboard`,
  `config_keyboard_effects` (approx lines 20-96).
- Rendering: `render` (entry, approx L98), `entry_matches`,
  `paint_cursor_highlight`, `render_entry`, `render_widget`, plus the small
  label helpers `namespace_of`, `ns_display_name`, `layer_label` (approx
  L98-305, 566-end).
- Value commit/validation: `commit_value`, `validate_against_constraints`
  (approx L482-564).

Proposed split (all within `config_panel/`):
- `render.rs` -- keep `render` + the paint helpers (`render_entry`,
  `render_widget`, `paint_cursor_highlight`, `entry_matches`, and the label
  helpers). (coordinator for the view)
- `keyboard.rs` -- `key_widget_id`, `tree_has_keyboard`,
  `config_keyboard_effects`.
- `commit.rs` -- `commit_value`, `validate_against_constraints`.

Confidence: certain. Boundaries are already function-clean; this is mechanical.

### 2c. crates\ff-desktop\src\tab_state.rs (~404 lines)

Barely over the limit. Content: the `TabKind` enum (large, heavily documented),
`UndoEntry`, `TabId`, the `TabState` struct, the `base_tab!` macro, and ~15
near-identical constructor fns (`pom`, `files_panel`, `config_panel`,
`file_explorer_panel`, `search_results_panel`, `plugin_manager`, `event_log`,
`scrm_viewer`, `macro_library`, `command_configurator`, `theme_editor`,
`menus_editor`, `menu_workspace_tab`, `untitled`, `for_file`), plus
`encoding_label`.

Proposed split:
- `tab_state.rs` -- keep `TabKind`, `UndoEntry`, `TabId`, the `TabState` struct
  definition, and `encoding_label` (the data model).
- `tab_state_ctors.rs` (or `tab_state/constructors.rs`) -- move the `base_tab!`
  macro and all the constructor fns in an `impl TabState` block.

This is the lowest-value split (file is only ~4 lines over, and the "concern"
boundary is model-vs-constructors rather than the canonical
state/render/commands). Still behaviour-preserving. Confidence: certain it works;
value is marginal -- consider deferring unless the file is being touched anyway.
Note: two constructors (`theme_editor`, `menus_editor`) carry
`#[allow(dead_code)]` with a CR-CH-022 "retained constructor, currently uncalled"
rationale -- they are navigable-kind plumbing, not deletion candidates without an
owner call.

### 2d. crates\ff-desktop\src\menu_workspace\loader.rs (~404 lines, incl. tests)

IMPORTANT: this file is dominated by its `#[cfg(test)] mod tests` block (the
tests span roughly from the "=== Tests ===" marker to end -- well over half the
file). The 400-line rule EXCLUDES the test module. Non-test code (the serde
shims, `load_menu_file`, `load_menu_file_with_limits`, `parse_menu_str`,
`apply_limits`, `validate_menu`, `validate_option`, `OptionLimits`,
`option_limits_from_config`, `read_u32_or_default`) is well under 400 lines.

Therefore loader.rs likely does NOT violate the rule once the test module is
excluded. The per-rule remedy, if the test module itself is the concern, is the
testing.md guidance: split the test module into a sibling file
(`loader_tests.rs`) when it alone exceeds ~200 lines. Confidence: probable
(verify with tools/python/check_line_limits.py which measures non-test lines; the
brief's ~404 count appears to include tests). Recommend: run the project's own
`check_line_limits.py` to confirm before doing any split here; if it is non-test
lines that exceed 400, split tests to `loader_tests.rs`.

---

## 3. ORPHAN CRATES

`ff-desktop` direct deps (from crates\ff-desktop\Cargo.toml): ff-core, ff-config,
ff-logging, ff-theme, ff-document-model, ff-viewport-scrolling,
ff-connector-local-fs, ff-vfs, ff-command, ff-command-semantics,
ff-edit-operations, ff-find-and-replace, ff-navigation-commands,
ff-exclude-show-filter, ff-display-line-mapping, ff-file-tree, ff-keys, ff-help,
ff-session, ff-zoom, ff-dscatalog, ff-toolchain-api, ff-gcc-toolchain,
ff-rust-toolchain, ff-fftest, ff-global-search, ff-plugin, ff-shell, ff-layout,
ff-screen-model, ff-scrm, ff-theme-editor, ff-toolchain-panel,
ff-catalog-registry, ff-catalog-dialog, ff-dataset-alloc-dialog, ff-context-menu,
ff-posix-provider, ff-nav-model, ff-explorer-view, ff-scroll-amount,
ff-exclude-manager, ff-editor-panel, ff-files-panel.

Transitive deps confirmed by reading manifests: ff-navigation-commands pulls
`ff-undo-redo` (so ff-undo-redo IS in-closure, NOT an orphan); ff-shell pulls
`ff-workflow` (in-closure); the toolchain/catalog/scrm/panel crates pull their
model crates. (This was derived by reading Cargo.toml files, not by running cargo,
to avoid the build lock -- per the brief.)

Members NOT reachable from the `ff-desktop` closure (orphans), classified:

### 3a. keep-pending-wiring -- editor aspect crates (parked CR, named in docs)
`ff-undo-redo` is in-closure; the rest of the editor "aspect" family is parked
by the PENDING-GATE editor-decouple CR (docs\status\change-log.md "do WHEN EDITOR
TESTING BEGINS", and .agents\tasks\wave6-editor-architecture-review.md which lists
exactly these and recommends KEEP):
- ff-clipboard  (named explicitly by the parked CR; keep-pending-wiring)
- ff-caret-selection
- ff-line-commands
- ff-syntax-highlighting
- ff-auto-indent
- ff-text-decorations
- ff-whitespace-guides
- ff-seqnum
- ff-wrap
- ff-select
- ff-tabmask
- ff-hex

Rationale (confidence: certain for the classification): wave6-editor-architecture
-review.md states these "exist with rich, GUI-independent public APIs" and that
the live editor is "NOT wired to them"; the explicit recommendation is to KEEP
them and wire the editor onto them under a gated CR, NOT to add new aspect crates.
`ff-clipboard` is specifically named as the target of the editor-decouple CR.
DO NOT delete these.

### 3b. keep-pending-wiring -- standalone ff-mdx-* app + viewer/export family
These form the separate `ffmdx` app (CR-NR-101/102) and its viewer/exporter
pipeline, deliberately outside the ffwb binary:
- ff-mdx-app, ff-mdx-installer, ff-mdx-plugin
- ff-md-viewer, ff-html-export, ff-pdf-export
- ff-viewers (consumed by ff-mdx-plugin)
Classification: keep-pending-wiring (they ship a different binary). Confidence:
probable (based on Cargo.toml membership + the specs list naming ffmdx-app and
the viewer/exporter family).

### 3c. keep-pending-wiring -- governance / dataset subsystem crates
- ff-governance-tests (a test harness crate; dev-depends on the two below)
- ff-dataset-catalog, ff-vsam-services (only referenced by ff-governance-tests'
  dev-deps)
- ff-dscatalog IS in the ff-desktop closure (direct dep) -- NOT an orphan; note
  the near-name collision with ff-dataset-catalog.
Classification: keep-pending-wiring / needs-owner-decision (test-only reach today).

### 3d. needs-owner-decision -- subsystem crates not yet wired to the shell
These are feature subsystems that exist but are not in the ffwb closure and are
not named by an active parked CR I could confirm:
- ff-database-tool, ff-compare-merge, ff-jes, ff-idcams, ff-dsalloc,
  ff-structure-catalog, ff-asa, ff-forge, ff-idle-processing,
  ff-large-file-performance, ff-background-io, ff-external-mod, ff-encoding,
  ff-language-service, ff-completion, ff-lua, ff-menu, ff-tabs,
  ff-connector-extensibility, ff-connector-local-fs (NOTE: ff-connector-local-fs
  IS a direct ff-desktop dep -- NOT orphan; listed here only to flag the earlier
  catalog line).
Classification: needs-owner-decision. Each maps to a spec sub-project under
docs\specs\, so most are "built subsystem awaiting shell wiring" rather than dead.
None should be deleted without the owner confirming the corresponding feature is
abandoned. Confidence: probable (membership-vs-closure is certain; the
abandoned-vs-pending judgement is the owner's).

### 3e. Orphan summary
No orphan crate is a confident deletion candidate. The deletion mandate is better
served in Section 1 (dead helpers) and Section 2 (file splits) than by removing
crates: every orphan traces to either the parked editor-decouple CR, the separate
ffmdx app, the governance test harness, or an un-wired-but-specced subsystem.
Recommend the owner review Section 3d list explicitly if crate-count reduction is
a goal; otherwise keep-pending-wiring.

---

## 4. DUPLICATE ACTION PATHS

The single dispatch seam is: typed line -> `run_command_line` ->
`dispatch_command_string`; key press -> `dispatch_key_command` ->
`dispatch_bound_command` -> (`resolve_and_dispatch_command` | front door); menu
option click -> `activate_menu_option` -> `resolve_and_dispatch_command`. These
all converge correctly. The Ctrl+S divergence noted in the brief is already fixed.

Remaining UI affordances that BYPASS the dispatcher (confidence: certain they
bypass; they are command-parity gaps per steering rule 1b):

- **Ctrl+C (copy selection)** -- crates\ff-desktop\src\editor_panel\paint.rs
  approx lines 267-289. A raw `ui.input(|i| i.key_pressed(egui::Key::C) &&
  i.modifiers.ctrl)` reads the selection, calls `arboard::Clipboard` directly,
  and returns a status string. It does NOT go through a COPY command. (Also noted
  in wave6-editor-architecture-review.md: "Ctrl+C copy ... arboard directly ...
  INLINE (NOT ff-clipboard)".)
- **Ctrl+Z (undo)** -- crates\ff-desktop\src\editor_panel\input.rs approx lines
  227-249. A raw `ui.input` Ctrl+Z check pops `tab.undo_stack` and applies the
  inverse op inline. No UNDO command; no `ff-undo-redo`.

Why these are not "just delete": they are the ONLY implementations of copy/undo
in the live editor. Collapsing them onto the command seam (a COPY/CUT/PASTE and
UNDO command, routed through the dispatcher, ideally backed by ff-clipboard /
ff-undo-redo) CHANGES observable behaviour and is exactly the parked
editor-decouple CR's job (change-log.md, PENDING GATE). Per steering (default to
action vs gate), this is GATED work, not a deletion. Flag to owner; do not
silently rewire.

Non-bypass note: the mouse-wheel Ctrl+Scroll ZOOM is explicitly consumed at shell
level before the editor sees it (input.rs approx lines 250-252 comment), so that
is not a duplicate path. Arrow/Enter/Home/End tree navigation in the Config panel
(config_panel\render.rs) is local widget interaction, not a user "command", so it
is out of scope for command parity.

---

## CONCLUSIONS AND RECOMMENDATIONS (priority order)

Deletion-first, highest confidence first:

1. DELETE (certain, behaviour-preserving): `function_target_with_arg` and its one
   dedicated test `single_verb_arg_split_populates_arg_param` in
   shell\dispatch.rs. Keep `verb_arg_split_preserves_argument_case`.

2. REFACTOR / SPLIT (certain, behaviour-preserving):
   - main.rs -> main.rs + startup_schema.rs + startup_env.rs.
   - config_panel\render.rs -> render.rs + keyboard.rs + commit.rs.
   - tab_state.rs -> tab_state.rs + tab_state_ctors.rs (marginal; optional).
   - menu_workspace\loader.rs -> confirm with check_line_limits.py first; if the
     non-test body is under 400, split only the test module to loader_tests.rs.

3. OWNER-DECISION (staged APIs under the delete mandate): `run_command_definition`
   and `NOT_IMPLEMENTED_MSG` in target_dispatch.rs -- delete only if the Command
   Configurator binding surface is not imminent.

4. OWNER-DECISION + prerequisite refactor (B080 Step 7): The CustomWorkspace/
   navigation ladder arms and the bare-THEME branch are NOT dead -- they serve the
   menu-bar-click / POM-recursion / chained-segment callers that invoke
   `handle_command` directly. Before any arm deletion, reroute those callers
   (render_chrome.rs menu-bar clicks, commands.rs POM recursion,
   commands_fastpath.rs segments, nav_stack.rs reconstruction) through the front
   door `dispatch_command_string` so `resolve_target` runs for them too. Only
   then do the arms become genuinely unreachable and safe to delete. The in-code
   "unreachable fallback" comments describe the typed-line path only and are
   misleading about full reachability.

5. GATED (not deletion): close the Ctrl+C / Ctrl+Z command-parity gaps by routing
   copy/undo through the dispatcher (backed by ff-clipboard / ff-undo-redo). This
   is the already-logged, PENDING-GATE editor-decouple CR; it changes observable
   behaviour and must run the requirements gate.

6. ORPHAN CRATES: no confident deletion. All trace to the parked editor-decouple
   CR (incl. ff-clipboard), the standalone ffmdx app, the governance test harness,
   or un-wired specced subsystems (Section 3d). Keep-pending-wiring unless the
   owner declares a specific subsystem abandoned.

Nothing in this report recommends building a new feature.
