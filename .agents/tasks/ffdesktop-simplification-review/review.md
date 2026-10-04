# FileForgeWorkbench (ffwb) -- Architectural Simplification Review

Read-only review of the CURRENTLY ACTIVE application stack reachable from the
`ffwb` binary (`crates/ff-desktop`, entry `crates/ff-desktop/src/main.rs`). No
source was modified. Scope is restricted to `ff-desktop` and the crates it wires
in via its `Cargo.toml` dependency closure; orphan crates (roughly half the
workspace) are out of scope and are not assessed here.

Summary answer first, then evidence with file/symbol citations, then
conclusions and recommendations.

---

## 1. Executive Summary

The live stack is competently layered and the heavy lifting (document model,
viewport, VFS, config, theme, keys, command semantics) is already extracted into
focused crates that `ff-desktop` consumes. The decomposition goal is largely
met at the CRATE boundary. The remaining complexity is concentrated inside the
`ff-desktop` shell itself, where three things have accumulated: a very large
linear command-intercept chain, a per-frame mega-struct (`WorkbenchShell`) with
roughly 90 fields, and a focus/dispatch framework (`WorkspaceContext`,
`ShellRequest`/`ShellServices`, `resolve_target`) that is only partly wired to
the behaviour it was meant to unify.

Scores (1-10, higher is better):

| Dimension        | Score | One-line reasoning |
|------------------|-------|--------------------|
| Architectural    | 7     | Clean crate layering and a real single-latch focus path; undercut by two parallel command pipelines and a god-struct shell. |
| Maintainability   | 5     | Core files far exceed the project's own 400-line rule (`shell/tests.rs` ~11k lines, `render.rs`/`commands.rs`/`tab_manager.rs` all 2.4k-2.7k), so changes touch very large files. |
| Simplicity        | 5     | `handle_command` is a ~900-line `if upper == "..."` ladder of ~60 branches; several abstractions (ShellRequest, notification channel, panel_layout) exist ahead of any consumer. |
| Extensibility     | 7     | Adding a Workspace Context is genuinely a repeatable pattern (owned-panel swap + `WorkspaceContext` + `apply_*_action`); adding a COMMAND still means editing the giant ladder. |

The single most valuable simplification is to make `resolve_target` the real
front door for the typed command line too (today it is bypassed), and to turn
the `handle_command` ladder into a command table. The single most valuable
cleanup is to split `shell/tests.rs` and shrink `WorkbenchShell`.

Overall: a solid foundation that is close to its stated goal. The work left is
consolidation and deletion inside the shell, not redesign.

---

## 2. Current Architecture Overview

### Runtime structure (startup)
`main.rs` boots in a fixed order (`crates/ff-desktop/src/main.rs`): profile arg
extraction -> logging phase 1 -> `ff_config::init` + `register_builtin_schema`
-> logging phase 2 -> Tokio runtime -> `ff_core::WorkbenchApp` -> startup theme
palette -> CLI/headless/batch branches -> `eframe::run_native` with a
`WorkbenchShell`. This is clear and well-commented; no issue.

### Active module relationships
`WorkbenchShell` (`shell/mod.rs`) is the single `eframe::App`. It owns:
- `TabManager` (`tab_manager.rs`) -> `Vec<TabState>` (`tab_state.rs`), each tab
  carrying document/viewport/cursor plus kind-specific state;
- the command stack: `CommandDispatch` + `CommandRegistry` (ff-command),
  `CommandEngine` (ff-command-semantics), `CommandLineHistory`/`HistoryStore`
  (ff-keys), and the shell-local `FindManager`/`NavManager`/`ExcludeManager`;
- the panel/Context states (config, theme/menus/keys/kinds editors, plugin
  manager, event log, macro library, command configurator, search results,
  files panel, help, scrm viewer);
- key maps + label bar (ff-keys), zoom (ff-zoom), session (`SessionManager` +
  ff-session), notifications (channel + queue), and a long tail of per-feature
  flags (detach/split, dialogs, focus latches).

### Dependency flow
`ff-desktop` -> leaf crates. The GUI shell is the only crate that knows about
egui/eframe plus the full feature set; the extracted panels (ff-theme-editor,
ff-editor-panel, ff-files-panel, ...) depend on egui + a narrow model crate and
NOT on the shell. The `WorkspaceContext` trait deliberately lives in
`ff-desktop` (orphan-rule reason, documented in `theme_editor_panel.rs`), so the
thin `impl WorkspaceContext` adapters sit in `ff-desktop` while the pure render
lives in the extracted crate. This is a good layering decision.

### Command flow (the important part -- see Findings F1)
There are effectively THREE entry seams but ONE real sink:
- Typed command line: `run_command_line` -> `handle_command`
  (`shell/commands.rs:679`). This is a ~900-line ladder of built-in intercepts;
  only after all of them does it try `try_menu_name_dispatch` and then
  `cmd_engine.execute_command_line`. It does NOT call `resolve_target`.
- Menu-option click: `execute_menu_option` -> `resolve_and_dispatch_command`
  (`shell/target_dispatch.rs`) -> `ff_command::resolve_target` ->
  `dispatch_command_target`. For `Function` targets this calls `handle_command`
  anyway; on `FallThrough` it also calls `handle_command`.
- Function key / label bar: `dispatch_key_command` -> `dispatch_bound_command`
  -> `resolve_and_dispatch_command` -> same path, FallThrough to
  `handle_command`.
So `handle_command` is the terminal sink for all three, and `resolve_target` is
a thin front-door on only two seams. Critically, the shell's
`ShellTargetResolver::builtin_workspace_target` returns `None`
(`command_config/mod.rs`), so EVERY built-in verb (FILES, CONFIG, LOG, SPLIT,
CAPS, ...) is classified by `resolve_target` as "not mine" and falls through to
the ladder. The "single command dispatch" mechanism is real for CLASSIFYING
user/menu/macro targets but is not the single path for built-in behaviour.

### State flow
Panels follow a "pure render returns an action; shell applies it" pattern. The
migrated Contexts do an owned-panel swap (`std::mem::take`), render through
`render_workspace_context`, then the shell drains either a `ShellRequest` list
or (more commonly) a stashed `pending_action` applied by an `apply_*_action`
method (`shell/configurator.rs`, `shell/menus_editor.rs`,
`commands.rs::apply_theme_editor_action`, etc.). The focus contract
(`InteriorFocus`) is applied in exactly one place
(`render.rs::apply_interior_focus`). This part of the design is clean and worth
preserving as the template (section 8).

---

## 3. Findings

### F1 -- Critical -- Two parallel command pipelines; `resolve_target` is bypassed by the typed path
- Location: `shell/commands.rs::handle_command` (line 679),
  `shell/target_dispatch.rs`, `command_config/mod.rs::ShellTargetResolver::builtin_workspace_target`.
- Description: The documented "single command dispatch" (framework mechanism 1)
  routes menu clicks and keys through `resolve_target`, but the typed command
  line goes straight into the `handle_command` ladder, and `resolve_target`'s
  `builtin_workspace_target` is a stub returning `None`. So built-in verbs have
  exactly one implementation (the ladder), but it is reached by two different
  routes with different pre-processing (the key path merges the field argument
  and wraps the Command_Line_Outcome; the typed path does its own thing). The
  claim that "the typed command line, menu clicks, and keyboard shortcuts all
  route through this single path" is only half-true today.
- Impact: The invariant the framework exists to guarantee (one path, no
  divergence) is not actually enforced for built-ins; a future verb added to the
  ladder but not reflected in resolver classification can behave differently via
  a binding than when typed. The ladder is also the single biggest source of
  shell complexity.
- Recommended change: Make `handle_command` dispatch through a command table
  (see F2) and have `builtin_workspace_target` classify those same verbs, so
  `resolve_target` becomes the genuine front door for all three seams. This is
  consolidation, not redesign: the behaviour already lives in one place; only
  the routing is doubled.

### F2 -- High -- `handle_command` is a ~900-line linear intercept ladder
- Location: `shell/commands.rs` lines ~679-1830 (EXIT, EDIT, START, MENU, CLOSE,
  DOCK, HELP, KEYS, KINDS, PFSHOW, END, RETURN, CONFIG, FILES, SEARCH, COMMAND,
  COMMANDS, MENUS, SNAPSHOT, CAPTURE, RESET BARE, LOG, CATALOGS, PLUGINS, MACROS,
  RETRIEVE, LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT, EXCLUDE/SHOW/RESET,
  FIND/RFIND/CHANGE/RCHANGE, CAPS/NULLS/STATS/LOCK/PROFILE/HILITE, SCROLL, NAME,
  DETACH, SPLIT/UNSPLIT/FOCUS, SWAP, AUTONUM/NUM, SUBMIT/TIME/STATUS/CREATE/
  REPLACE/BROWSE/VIEW/COMPARE, WORKSPACE ...).
- Description: Roughly 60 branches of `if upper == "..."` / `verb_arg(...)`, each
  returning early. The project's own framework-conformance rule explicitly names
  this pattern as the thing to avoid ("Do NOT: add a bespoke `if upper == "..."`
  intercept in `handle_command`").
- Impact: Every new command edits this one function; branch ordering is
  load-bearing and subtle (e.g. COMMAND must precede COMMANDS; SPLIT DETACH
  before bare SPLIT); the function is far past the 40-line and 400-line limits;
  it is hard to test a single verb in isolation.
- Recommended change: Replace the ladder with a dispatch table keyed by verb
  (a `HashMap<&str, fn/handler>` or a `match` on a parsed verb token), with
  argument-bearing verbs grouped. Several families (navigation, exclude/show,
  find/change, edit-profile, workspace) are already delegated to a manager and
  can become one table entry each. This also removes the ordering hazards.

### F3 -- High -- `shell/tests.rs` is a single ~11,000-line file
- Location: `crates/ff-desktop/src/shell/tests.rs` (10,999 lines).
- Description: One monolithic test module. The project's testing/standards rules
  call for splitting a test file once it alone exceeds ~200 lines.
- Impact: Slow to open and edit, high merge-conflict surface, and it is the
  single largest file in the binary crate by a wide margin.
- Recommended change: Split by feature area into `tests/` integration files or
  `shell/tests/<area>.rs` submodules (command dispatch, nav stack, focus/Tab,
  menu workspace, detach/split, session). Pure refactor, no behaviour change.

### F4 -- High -- `WorkbenchShell` is a god-struct (~90 fields)
- Location: `shell/mod.rs` (the `pub struct WorkbenchShell` definition spans
  roughly lines 230-640).
- Description: One struct owns command state, every panel's state, key maps,
  zoom, session, notifications, detach/split bookkeeping, five test-only
  `*_dir_override` fields, numerous one-shot `pending_*`/`*_requested` flags, and
  per-dialog booleans. Kind-specific state also leaks onto `TabState` (e.g.
  `menu_workspace: Option<MenuWorkspaceState>` on every tab regardless of kind).
- Impact: Weak cohesion; any method can touch any field; borrow-checker friction
  forces the `mem::take` swap dance; the struct is hard to reason about as a
  whole; test-only fields ship in the production struct.
- Recommended change: Group related fields into sub-structs (e.g.
  `CommandLineState`, `FocusState`, `DetachSplitState`, `EditorsState`,
  `DirOverrides`). This is mechanical and low-risk and immediately shrinks the
  cognitive load of `mod.rs`. Longer term, move kind-specific tab state behind
  the `TabKind` so a `TabState` does not carry fields for kinds it is not.

### F5 -- Medium -- `WorkspaceContext` service surface is largely speculative
- Location: `shell/workspace_context.rs`.
- Description: `ShellRequest`, the `ShellServices.request_*` helpers,
  `InteriorFocus::none`/`new`, and most `ShellServices` fields are
  `#[allow(dead_code)]`. In practice the migrated Contexts stash a rich
  `pending_action` and the shell applies it via `apply_*_action`; they do not use
  the `ShellRequest` queue (the Theme Editor's `render` even takes `_services`).
  So two parallel "Context -> shell effect" mechanisms exist: the unused
  `ShellRequest` vocabulary and the actually-used `pending_action` stash.
- Impact: Readers must learn a request vocabulary that nothing exercises; the
  trait advertises a capability (`ShellServices`) its real users bypass.
- Recommended change: Keep `WorkspaceContext` + `InteriorFocus` +
  `render_workspace_context` (these earn their keep -- see F8). Either (a) delete
  `ShellRequest`/`request_*` and the unused `ShellServices` fields and
  standardise on the `pending_action` return, or (b) migrate the editors to
  `ShellRequest` and delete `pending_action`. Pick one. Given the bias toward
  deletion and that `pending_action` is what actually runs, (a) is the smaller,
  safer move.

### F6 -- Medium -- Notification system carries an unused channel alongside the queue
- Location: `shell/mod.rs` (`notification_rx`, `notification_tx` marked
  `#[allow(dead_code)]`, `notification_sender()` also `#[allow(dead_code)]`),
  `notification/mod.rs` (`#![allow(dead_code)]` at module scope).
- Description: The shell holds both an mpsc channel (rx/tx) AND a shared
  `Arc<Mutex<NotificationQueue>>`, draining the channel into the queue each
  frame. The only sender factory (`notification_sender`) is dead code, so no
  background producer currently uses the channel; in-process notifications go
  straight to the queue.
- Impact: Two mechanisms for one job; a mutex-wrapped queue plus a channel plus a
  per-frame drain, none of which is exercised by a real cross-thread producer
  today.
- Recommended change: Until a real background producer exists, keep only the
  shared queue and drop the channel (rx/tx/drain/`notification_sender`). Re-add a
  channel the day a background task needs to post from another thread. Net: fewer
  fields, one less per-frame step.

### F7 -- Medium -- Speculative/ahead-of-consumer modules and retained duplicates
- Location: `panel_layout.rs` (entire file `#[allow(dead_code)]`:
  `DataEntryPanelLayout`/`ListPanelLayout` builders with no caller);
  `shell/command_line_outcome.rs` `OutcomeData` (de)serialisation exercised only
  by round-trip tests; `tab_manager.rs` retained `open_theme_editor_tab`,
  `open_menus_editor_tab`, `open_menu_workspace_here`, `transform_active_pom_tab`
  (all `#[allow(dead_code)]` after the in-place `navigate_to` model replaced
  them); `workspace_kind/registry.rs::resolve_base` (consumer "lands in Slice
  B.2").
- Description: Code kept against future phases or as superseded primitives.
- Impact: Each is small, but together they add surface the reader must triage as
  "is this live?". `panel_layout.rs` in particular is a whole unused module.
- Recommended change: Delete `panel_layout.rs` and the superseded `tab_manager`
  openers now (git recovers them if a later phase needs them -- the standards
  explicitly prefer deletion over dead-code retention). Leave the test-exercised
  items (`OutcomeData`) until their consumer lands, but track them.

### F8 -- Low -- The focus-latch framework IS justified; do not simplify it away
- Location: `shell/render.rs::apply_interior_focus` / `render_workspace_context`
  / `honour_interior_focus_latch`, `shell/workspace_context.rs::InteriorFocus`.
- Description: This review deliberately checked whether the `InteriorFocus`
  single-latch path is over-built. It is not: the central-panel match resets the
  focus anchors to `None` every frame, and the one latch path forces every arm to
  report them. The two no-interior arms (Editor, Files Panel) call
  `InteriorFocus::none()` EXPLICITLY. This directly prevents the recurring
  phantom-Tab-stop bug class (B056-B059) and is cheap.
- Impact: Positive. Removing it would re-open a proven bug class.
- Recommended change: None. Keep as-is; make it the model for section 8.

### F9 -- Low -- `#[windows_subsystem="windows"]` + raw FFI; acceptable but note
- Location: `main.rs` (`SystemParametersInfoW` extern block for reduce-motion).
- Description: One small `unsafe` FFI call, correctly `SAFETY`-free because it is
  a documented Win32 call with a stack out-param; macOS/Linux return false.
- Impact: Minimal. Just flagging as the only FFI in the live path.
- Recommended change: None required; optionally move behind a tiny
  `os_prefs` helper if more OS queries appear.

---

## 4. Simplification Opportunities

### S1 -- Collapse the typed-command ladder into one dispatch table (addresses F1, F2)
- Current Design: `handle_command` is ~60 sequential `if`/`verb_arg` branches;
  `resolve_target` is bypassed for the typed path and `builtin_workspace_target`
  is a stub.
- Proposed Design: Parse the first verb token once, look it up in a verb table
  mapping to a handler (closure or method), passing the remaining argument.
  Argument-less and argument-bearing verbs become table rows; the manager-backed
  families (nav, exclude, find, profile, workspace) are one row each delegating
  to the existing manager. Then implement `builtin_workspace_target` by consulting
  the SAME table so `resolve_target` can classify built-ins, unifying all three
  seams on one front door.
- Benefits: reduced complexity (no ordering hazards, each verb testable in
  isolation); reduced code volume in `commands.rs`; clearer ownership (one table,
  one place to register a verb); improved testability (unit-test a verb's handler
  directly); and the framework's single-dispatch claim becomes true.
- Migration Effort: Large (the ladder is big and some branches have subtle
  pre-processing), but mechanical and incremental -- verbs can move into the table
  a family at a time while the ladder shrinks.

### S2 -- Split `shell/tests.rs` and the oversized source files (addresses F3)
- Current Design: `tests.rs` ~11k lines; `render.rs` 2720; `commands.rs` 2638;
  `tab_manager.rs` 2403; `mod.rs` 1764; `update.rs` 1565 -- all far over the
  400-line rule.
- Proposed Design: Split tests by feature area; split `render.rs` into
  `render.rs` (frame orchestration) + per-region render helpers; split
  `commands.rs` once S1 extracts verb handlers; group `WorkbenchShell` fields
  (S3) to shrink `mod.rs`.
- Benefits: reduced maintenance, far smaller edit/merge surface, conformance with
  the project's own file-size rule, no behaviour change.
- Migration Effort: Medium (pure refactor, but touches the largest files).

### S3 -- Group `WorkbenchShell` fields into cohesive sub-structs (addresses F4)
- Current Design: ~90 flat fields including test-only overrides and many
  one-shot flags.
- Proposed Design: `CommandLineState`, `FocusState`, `DetachSplitState`,
  `EditorsState`, `DirOverrides` (test-only, `#[cfg(test)]`-friendly), etc.;
  access via `self.focus.first_interior_id` instead of `self.first_interior_id`.
- Benefits: clearer ownership, less borrow friction, test-only state separated
  from production state, `mod.rs` becomes readable.
- Migration Effort: Medium (lots of field references to update, but each change
  is trivial and compiler-guided).

### S4 -- Collapse the dual notification mechanism to the queue (addresses F6)
- Current Design: channel (rx/tx) + shared queue + per-frame drain, with the
  only sender factory dead.
- Proposed Design: keep the shared `Arc<Mutex<NotificationQueue>>`; post directly;
  drop the channel until a real cross-thread producer exists.
- Benefits: fewer fields, one less per-frame step, removes dead sender API,
  clearer "there is one notification sink".
- Migration Effort: Small.

### S5 -- Standardise Context->shell effects on one mechanism (addresses F5)
- Current Design: `ShellRequest`/`ShellServices.request_*` (unused) coexist with
  the `pending_action` + `apply_*_action` stash (used).
- Proposed Design: delete the unused `ShellRequest` vocabulary and the
  bypassed `ShellServices` fields; document `pending_action` + `apply_*_action`
  as THE Context-effect pattern.
- Benefits: one effect mechanism to learn, less dead API, smaller
  `workspace_context.rs`.
- Migration Effort: Small-to-Medium (delete + adjust the two no-interior arms'
  signatures if needed).

---

## 5. Technical Debt (ranked by priority)

1. Doubled command routing (F1) -- the typed path bypasses `resolve_target`;
   `builtin_workspace_target` is a stub. Highest-value debt because it defeats
   the stated single-dispatch invariant.
2. The `handle_command` ladder (F2) -- legacy linear-intercept pattern the
   project's own rules forbid for new code; the biggest single complexity sink.
3. Oversized files (F3 tests ~11k; render/commands/tab_manager ~2.4-2.7k) --
   violates the project's 400-line rule; pure-refactor debt.
4. `WorkbenchShell` god-struct + kind-state on `TabState` (F4) -- weak cohesion,
   test-only fields in production struct.
5. Unused framework surface: `ShellRequest`/`ShellServices.request_*`,
   `InteriorFocus::none/new` dead-code (F5); notification channel (F6).
6. Speculative/superseded modules: `panel_layout.rs` (whole module unused),
   retained `tab_manager` openers, `resolve_base` ahead of consumer (F7).
7. Wrapper/adapter thinness (not a problem, noted for completeness): the
   `theme_editor_panel.rs`-style thin adapters are GOOD debt-avoidance (orphan
   rule), keep them.

---

## 6. Wiring Template Assessment (current successful patterns)

The best existing example is the Theme Editor Context, mirrored by Menus/Keys/
Kinds/CommandConfigurator/MacroLibrary/EventLog/PluginManager/SearchResults/Help.
Its wiring today:

- Registration/kind: a `TabKind` variant (`tab_state.rs`) and a
  `WorkspaceKind` descriptor mapping in `shell/nav_stack.rs`
  (`descriptor_for_current_context` + `reconstruct_custom`).
- Command: a verb in `handle_command` (`THEME`) that calls `open_theme_editor`,
  which navigates in place via `navigate_to`/`nav_to_kind`.
- Render + focus: the pure render lives in the extracted crate
  (`ff-theme-editor`); a thin `impl WorkspaceContext for ThemeEditorState`
  (`theme_editor_panel.rs`) returns `InteriorFocus { first, last }`; the central
  match arm (`render.rs`) does `mem::take` -> `render_workspace_context` ->
  put-back -> `apply_theme_editor_action`.
- Effect: a `ThemeEditorAction` enum returned by render, stashed on
  `pending_action`, applied by `apply_theme_editor_action` (`commands.rs`).
- Session: persists as a `WorkspaceDescriptor` and restores via
  `reconstruct_context`.
- Focus test: a full-shell `egui_kittest` first-Tab test (per
  workspace-conformance).

Assessment: This Context-wiring pattern is coherent, repeatable, and already
followed by ~10 Contexts. It is the right template and should be the documented
standard (section 8). The ONE weak spot is the Command step: it still means
hand-editing the `handle_command` ladder rather than registering a verb (F1/F2).
Fix that and the template is clean end to end.

---

## 7. Refactoring Roadmap

### Phase 1 -- Safe, minimal-risk simplifications
1. Delete `panel_layout.rs` and the superseded `tab_manager` openers (F7).
   Benefit: removes a whole unused module and dead primitives. Effort: Small.
   Order: first (independent, pure deletion).
2. Collapse the notification channel to the queue (S4/F6). Benefit: fewer
   fields, one less per-frame step. Effort: Small. Order: second.
3. Remove the unused `ShellRequest`/`request_*` surface, standardise on
   `pending_action` (S5/F5). Benefit: one effect mechanism. Effort: Small-Medium.
   Order: third.
4. Split `shell/tests.rs` by feature area (S2/F3). Benefit: conformance + edit
   surface. Effort: Medium, zero behaviour risk. Order: can run in parallel.

### Phase 2 -- Structural improvements
5. Group `WorkbenchShell` fields into sub-structs; separate test-only overrides
   (S3/F4). Benefit: cohesion, readability, less borrow friction. Effort: Medium.
   Order: before S1 so the command table has a clean state surface to touch.
6. Split `render.rs`/`commands.rs`/`tab_manager.rs` to the 400-line rule
   (S2/F3). Benefit: maintainability. Effort: Medium. Order: alongside S3/S1.

### Phase 3 -- Architectural cleanup
7. Convert `handle_command` to a verb dispatch table and implement
   `builtin_workspace_target` so `resolve_target` is the single front door for
   typed, menu, and key seams (S1/F1/F2). Benefit: the framework's core invariant
   becomes true; ordering hazards gone; verbs testable in isolation. Effort:
   Large but incremental (move verb families into the table one at a time).
   Order: last, after state grouping and file splits make the moves mechanical.
8. Move kind-specific tab state off `TabState` behind `TabKind` (F4 tail).
   Benefit: `TabState` stops carrying fields for kinds it is not. Effort: Medium.
   Order: optional follow-on to S3.

Guiding constraint throughout: preserve current behaviour, extensibility, and the
focus-latch framework (F8). Prefer deletion and consolidation over new
abstraction.

---

## 8. Recommended FFWB Core Wiring Standard

This is the canonical, repeatable way to wire a NEW feature into the live stack.
It is grounded in the existing framework (WorkspaceContext +
`render_workspace_context`, `resolve_target`/CommandTarget, `menus/*.toml`,
`WorkspaceDescriptor`, the Workspace Kinds registry) -- it invents nothing new.
It also assumes Phase 3 (S1) has landed so Command Registration is table-based;
until then, the Command step means adding a verb to the ladder (and that is the
one place the standard is temporarily weaker).

### Feature Registration
- If the feature is a new Workspace Context: add a `TabKind` variant
  (`tab_state.rs`) and, if it is navigable/persistable, a `WorkspaceKind` arm in
  `shell/nav_stack.rs` (`descriptor_for_current_context` + `reconstruct_custom`).
- Keep the feature's pure logic in its OWN crate (model + egui render), depending
  only on narrow model crates, NOT on `ff-desktop`. Add it to `ff-desktop`'s
  `Cargo.toml`.

### Command Registration
- Register the verb in the command table (post-S1) so `resolve_target`
  classifies it via `builtin_workspace_target`, OR register a `Function`
  Command_Id in the `CommandRegistry` when the action is a true function target.
- Do NOT add a bespoke `if upper == "..."` intercept (framework-conformance).
  The menu-click, key, and typed paths must all reach the same handler.

### Menu Integration
- Add the option to the relevant `menus/*.toml` (content, not mechanism) with its
  command value. Built-in menus stay code-only (`DEFAULT_*_TOML`); never write a
  default menu to disk. A clicked option routes through `resolve_target` ->
  `dispatch_command_target` -> `open_menu_by_name` (for Menu targets) exactly like
  the typed path.

### Toolbar Integration
- There is no separate toolbar dispatch; a toolbar/affordance MUST invoke the
  command (same `dispatch_bound_command` seam as menu/key), never call the
  handler directly.

### Event Subscription
- For in-process status/results, post to the shared
  `Arc<Mutex<NotificationQueue>>`. Only introduce a channel if a genuine
  background (cross-thread) producer exists (do not pre-wire one -- see F6).

### State Integration
- Hold the feature's UI state in its panel struct; store it on the shell inside
  the appropriate grouped sub-struct (post-S3), not as new flat fields.
- Communicate state-changing intent from render by RETURNING an action enum the
  shell applies in an `apply_<feature>_action` method. Do not mutate the shell
  from inside render. (This is the `pending_action` pattern; it is the standard.)

### View Integration
- Implement `WorkspaceContext` for the panel state: `render(&mut self, ui,
  &mut ShellServices) -> InteriorFocus`. Give the FIRST interactive control a
  stable `egui::Id`; return `InteriorFocus::single(id)` (or `{first,last}`), or
  `InteriorFocus::none()` for a documented no-interior Context.
- In the central-panel match arm, use the owned-panel swap: `mem::take` the panel
  -> `render_workspace_context(ctx, ui, &mut panel)` -> put it back -> apply the
  stashed action. Do NOT hand-write the focus latch; `render_workspace_context`
  (or `apply_interior_focus`) is the ONE latch path.

### Testing Requirements
- Full-shell `egui_kittest` first-Tab test: the first Tab from the command field
  lands exactly on the reported first interior control (no phantom stop) -- per
  workspace-conformance.
- Unit tests for the command handler (post-S1, test the table entry directly) and
  for the `apply_<feature>_action` effect.
- Keep every new/changed source file under 400 non-test lines; split tests before
  they exceed ~200 lines.

### Implementation checklist (follow in order)
1. Create/extend the feature's pure crate (model + egui render returning an
   action enum). Add it to `ff-desktop/Cargo.toml`.
2. Add a `TabKind` variant and, if navigable, the `WorkspaceKind` descriptor
   arms in `shell/nav_stack.rs`.
3. Add the thin `impl WorkspaceContext` adapter in `ff-desktop` returning
   `InteriorFocus` with a stable first-control id.
4. Add the central-panel match arm using the `mem::take` ->
   `render_workspace_context` -> put-back -> `apply_<feature>_action` pattern.
5. Register the command (table entry / Command_Id) so typed, menu, and key seams
   all resolve to it; add the menu option in `menus/*.toml`.
6. Implement `apply_<feature>_action` for the rich shell-side effects.
7. Write the full-shell first-Tab `egui_kittest` test plus handler/effect unit
   tests; run the SCOPED `cargo check/test/clippy -p ff-desktop` and `cargo fmt`.
8. Hand off for the owner's full `verify.ps1` gate.

---

## Conclusions

- The stack is well-layered at the crate boundary; the decomposition goal is
  substantially achieved. The focus-latch framework (F8) and the Context-wiring
  pattern (section 6/8) are genuine assets -- keep them.
- The highest-value work is CONSOLIDATION inside the shell: make `resolve_target`
  the real single front door (F1/S1), turn the `handle_command` ladder into a
  table (F2/S1), and shrink the oversized files and god-struct (F3/F4/S2/S3).
- The highest-value DELETIONS are the speculative surfaces: `panel_layout.rs`,
  the unused `ShellRequest` vocabulary, the dead notification channel, and the
  superseded `tab_manager` openers (F5/F6/F7).
- Of the seven documented framework mechanisms: single-dispatch (1) is only
  partly realised (bypassed by the typed path) -- the top finding; menu-name
  resolution (2), the per-tab Navigation_Stack (3), code-only built-in menus (4),
  the WorkspaceContext focus latch (5), descriptor session persistence (6), and
  the Kinds registry (7) are all genuinely justified by the current wired feature
  set and are earning their keep. The only clearly OVER-built surface relative to
  what is reachable today is the unused `ShellRequest`/`ShellServices` request
  vocabulary inside mechanism 5 and the dead notification channel -- the focus
  latch itself is right-sized.

All recommendations preserve current functionality, extensibility, and
maintainability, and none require a change of architecture.
