TASK21_FRAMEWORK_CHANGE

# Task 21 / Requirement 14 -- Markdown Viewer Shell Integration: Investigation and Plan

Verdict token (line 1): TASK21_FRAMEWORK_CHANGE -- wiring the markdown viewer
into the shell requires ALTERING core framework seams that do NOT exist in
ff-desktop today (no ff-viewers dependency, no Viewer_Registry, no PREVIEW
command in the CommandRegistry, no plugin host). Per framework-conformance.md
this needs EXPRESS OWNER CONFIRMATION before any source is written. Do NOT
improvise it. The OWNER DECISION section below states the exact seams, the
options, their tradeoffs, and a recommendation.

This document is investigation + planning only. No source file was changed.

---

## Part 1 -- Verification of Tasks 13-20 (the "already done" claim)

Scoped reads only (no suites run), per the brief. All claims below are grounded
in files read this session with file:line references.

### 1.1 tasks.md 12-20 are marked [x]

Confirmed in docs/specs/custom-file-viewers/tasks.md:
- Task 12 (ASCII cleanup) ....................... [x]
- Task 13 (render_to_html unit tests) ........... [x]
- Task 14 (Scanner::scan unit tests) ............ [x]
- Task 15 (FileWatcher predicate + debounce) .... [x]
- Task 16 (MdxFileViewer unit tests) ............ [x]
- Task 17 (installer PATH de-dup) ............... [x]
- Task 18 (ff-mdx-app pure helpers) ............. [x]
- Task 19 (ff-mdx-app egui_kittest GUI tests) ... [x]
- Task 20 (rust-standards hardening) ............ [x]
- Task 21 (shell integration) ................... [ ]  with a BLOCKED sub-note.

So 12-20 are all [x]; only Task 21 remains open. This matches the prior report.

### 1.2 TCR CR-CH-049 rows are PASS / MANUAL with stated reasons

docs/quality/TCR.md has a dedicated section "Markdown Viewer family -- CR-CH-049
(custom-file-viewers Req 11-17)" (around line 3205). Spot-checked rows:
- ff-md-viewer Req 11.1-11.4 (render_to_html + extensions + empty input): PASS,
  backed by renderer::tests::* including table_markdown_produces_a_table_element,
  footnote_reference_produces_a_footnote_anchor, strikethrough_produces_a_del_element,
  task_list_item_produces_a_checkbox_input, straight_quotes_become_smart_punctuation,
  empty_input_yields_empty_fragment.
- ff-md-viewer Req 12.1-12.6 (Scanner::scan): PASS, backed by scanner::tests::
  includes_only_md_files, excluded_directories_are_not_descended,
  relative_path_uses_forward_slashes_and_full_path_is_absolute,
  results_are_sorted_lexicographically, unreadable_root_returns_empty_vector.
- ff-md-viewer Req 15.1-15.3 (FileWatcher predicate + debounce): PASS, backed by
  watcher::tests::markdown_path_predicate_accepts_only_md_extension,
  burst_coalesces_to_most_recent_path, empty_burst_coalesces_to_none.
- ff-mdx-app Req 15.4 (reload only the open file): PASS,
  helpers::tests::reload_only_when_changed_file_is_the_open_file.
- ff-mdx-plugin Req 13.1-13.3 (MdxFileViewer): PASS, backed by viewer::tests::
  viewer_key_is_stable, declares_markdown_extensions_and_mime_types,
  can_render_matches_md_and_markdown_suffixes_only.

MANUAL rows (the two the brief calls out):
- Req 16.6 -- the rfd native folder-open and HTML-save dialogs are recorded
  MANUAL with the OS-native-dialog reason (testing.md exception list). The pure
  helpers around them are unit-tested (Task 18 / 19.4).
- Req 17.4 -- the HKCU\Environment registry write is recorded MANUAL with the
  "real OS side effect" reason; the PATH-dedup predicate is factored pure and
  unit-tested (Task 17).

These two MANUAL justifications match the testing.md exception list exactly
(OS-native dialogs; real OS side effects).

### 1.3 Reported tests exist with // Validates annotations

Spot-checked by reading the actual source:
- crates/ff-mdx-plugin/src/viewer.rs: a #[cfg(test)] module with four tests, each
  carrying `// Validates: Requirement 13.x`. CONFIRMED present.
- crates/ff-mdx-plugin/src/lib.rs: plugin_metadata_matches_viewer_display_name_and_mime_types
  with `// Validates: Requirement 13.5`. CONFIRMED.
- crates/ff-viewers/src/{registry.rs,trait_def.rs,plugin_bridge.rs}: house-style
  tests with `// Validates: Requirement X.Y`. CONFIRMED (these back the
  underlying ff-viewers framework the plugin targets).

### 1.4 Gaps noted during verification

1. ASCII violation in ff-viewers source (NOT in the four markdown crates, so
   outside Task 12's scope, but worth flagging). Em dash U+2014 appears in
   crates/ff-viewers/src/plugin_bridge.rs line 1 ("bridge -- integration"),
   command.rs, trait_def.rs comments, etc. documentation.md requires .rs files
   to be plain ASCII. This is a pre-existing ff-viewers issue, not a Task 13-20
   regression; record it but do not fix under this task.
2. Req 14 TCR rows: NOT COVERED (correctly, pending the Task 21 decision). The
   TCR section header comment states this explicitly.
3. The prior report's "scoped-clean across four crates" claim was NOT
   re-executed here (brief says a scoped read is enough). The implementer must
   run the scoped commands in section 1.5 to confirm green before relying on it.

Net: Tasks 13-20 are genuinely implemented, test-first in style, with TCR rows
PASS/MANUAL and the two MANUAL reasons correct. No blocking gap in 13-20.

### 1.5 Exact scoped verification commands the implementer must run

Run per tooling.md (bare cargo leader, one job per invocation; NEVER verify.ps1
or --workspace). From the workspace root:

    cargo fmt
    cargo check -p ff-md-viewer
    cargo test  -p ff-md-viewer
    cargo clippy -p ff-md-viewer
    cargo check -p ff-mdx-plugin
    cargo test  -p ff-mdx-plugin
    cargo clippy -p ff-mdx-plugin
    cargo check -p ff-mdx-app
    cargo test  -p ff-mdx-app
    cargo clippy -p ff-mdx-app
    cargo check -p ff-mdx-installer
    cargo test  -p ff-mdx-installer
    cargo clippy -p ff-mdx-installer

(Each clippy run should be clean under the project's -D warnings policy; run
`cargo clippy -p <crate> -- -D warnings` if confirming the gate locally.)
These four crates do NOT currently require ff-desktop to build, so none of the
above pulls in the shell.

---

## Part 2 -- Task 21 / Requirement 14 investigation against the ACTUAL framework

### 2.1 Does ff-desktop depend on ff-viewers / ff-mdx-plugin today?

NO. crates/ff-desktop/Cargo.toml [dependencies] lists ~40 ff-* crates but
NEITHER ff-viewers NOR ff-mdx-plugin NOR ff-md-viewer appears. (It does depend
on ff-plugin, but see 2.4.)

### 2.2 Is there a PREVIEW command already wired into resolve_target / dispatch?

NO.
- A grep over crates/ff-desktop/**/*.rs for PREVIEW/ViewerRegistry/FileViewer/
  ff-viewers/MdxPlugin found NO command, NO registry, NO viewer reference. The
  only "preview" hits are unrelated: a theme live-preview in commands.rs and a
  help-topic mode keyword string ("preview") in help.rs. Neither is a command.
- The dispatch chain is: ff_command::resolve_target (crates/ff-command/src/
  command_target.rs:267) classifies a bare string into a CommandTarget
  {Menu | CustomWorkspace | Function | Macro | External}; the shell dispatches it
  via WorkbenchShell::dispatch_command_target
  (crates/ff-desktop/src/shell/target_dispatch.rs). A registered Command_Id
  becomes a `Function` target that runs through handle_command.
- The shell builds its CommandRegistry in shell/mod.rs (around line 758,
  `CommandRegistry::new()` then registers e.g. "config.open"). There is NO
  "viewer.preview" registration anywhere. `shell.cmd_registry.contains(config.open)`
  is asserted in tests; no analogous viewer registration exists.

### 2.3 Is there any viewer-to-shell seam (registry construction, PREVIEW handler)?

NO.
- No ViewerRegistry / ViewerPanel field on WorkbenchShell (shell/mod.rs struct
  fields read; none present).
- The ff-viewers PREVIEW command (crates/ff-viewers/src/command.rs) is a
  SELF-CONTAINED handler type `PreviewCommand<'a>` with
  `PREVIEW_COMMAND_ID = "viewer.preview"`. It is NOT a ff_command CommandHandler
  and is NOT registered into any ff_command::CommandRegistry. It operates on a
  ViewerRegistry + ViewerPanel + ContentSelector it is handed directly. So even
  within ff-viewers, "PREVIEW" is not plumbed into the shell's single dispatch
  path -- there is no adapter from ff_command dispatch to PreviewCommand.
- The one shell-side thing that resembles a viewer -- ScrmViewerState in
  crates/ff-desktop/src/scrm_viewer_panel.rs -- is a BESPOKE shell
  WorkspaceContext (it impl's crate::shell::workspace_context::WorkspaceContext),
  NOT a ff-viewers FileViewer, and is reached via the CAPTURE REPLAY command, not
  PREVIEW. It is precedent for "surface a viewer as a shell Context" but it does
  not use ff-viewers at all.

### 2.4 What ff-plugin wiring exists in the shell?

Only cosmetic. crates/ff-desktop/src/plugin_manager_panel.rs imports
`ff_plugin::PluginState` purely as a display enum; the Plugin Manager panel holds
a Vec<(String, PluginState)> of plugin NAMES/states for rendering. There is NO
plugin host, NO PluginContext construction, NO FileForgePlugin instance created,
NO register_viewer call anywhere in ff-desktop. MdxPlugin::initialize/activate
(crates/ff-mdx-plugin/src/lib.rs) is never invoked by the app. The plugin is an
orphan workspace member.

### 2.5 Truth of the reported blocker

CONFIRMED TRUE by direct code reading. ff-desktop has no ff-viewers dependency,
no Viewer_Registry, no viewer.preview command on the single dispatch path, and
no plugin host. The ff-viewers framework (registry, trait, panel, PREVIEW
handler, plugin_bridge) exists and is unit-tested ONLY in isolation; nothing in
the shell constructs or calls it. The MdxFileViewer/MdxPlugin are referenced only
within ff-mdx-plugin itself.

---

## Part 3 -- Classification per framework-conformance.md

Requirement 14 (requirements.md:320) itself is tagged [framework] and demands:
14.1 register MdxPlugin/MdxFileViewer with the shell's ff-viewers Viewer_Registry;
14.2 invoke the viewer ONLY through a registered Command_Id resolved by
resolve_target -> dispatch_command_target (no bespoke intercept); 14.3 command
parity for any menu/toolbar/shortcut affordance.

Mapping each needed step to framework-conformance.md:

- "Register in an EXISTING ff-viewers registry that an EXISTING PREVIEW command
  already dispatches" -> would be ADDITIVE. BUT: neither the registry nor the
  PREVIEW command exists in the shell (2.1-2.3). So the additive precondition is
  FALSE.

- What is actually required instead:
  (a) Add ff-viewers (and ff-mdx-plugin / ff-md-viewer) as NEW ff-desktop
      dependencies. (New cross-crate wiring; by itself a dependency edit, but it
      is the on-ramp to the seam changes below.)
  (b) Construct and own a ViewerRegistry (+ ViewerPanel + ContentSelector) on
      WorkbenchShell and register the built-ins + MdxFileViewer. This is a NEW
      shell subsystem, not an existing seam.
  (c) Make `viewer.preview` reachable through ff_command::resolve_target ->
      dispatch_command_target. The ff-viewers PreviewCommand is NOT a
      ff_command CommandHandler; bridging it means EITHER registering a new
      CommandHandler into the shell's CommandRegistry (so it resolves as a
      `Function` target) OR adding a new adapter. Introducing the command-dispatch
      seam for viewers where none exists touches framework mechanism 1 (single
      command dispatch) directly.
  (d) Decide how the viewer SURFACES: as a DockablePanel (ff-viewers ViewerPanel)
      vs as its own shell WorkspaceContext. If a WorkspaceContext, mechanism 5
      (WorkspaceContext::render -> InteriorFocus via render_workspace_context +
      full-shell first-Tab egui_kittest test) applies.

Per framework-conformance.md "What counts as a framework change (needs express
confirmation)": "A new command-dispatch path or intercept" and introducing a new
shell subsystem/seam qualify. Items (b) and (c) are exactly that -- creating the
viewer-to-shell dispatch seam that does not exist. Therefore:

VERDICT: TASK21_FRAMEWORK_CHANGE. The implementer must NOT improvise the wiring.
Surface the OWNER DECISION below and wait for express confirmation.

(Note: if, AFTER owner approval of a chosen option, the only remaining work were
literally `registry.register_builtin(...)` + a one-line register of the viewer
into an already-wired PREVIEW path, that final step alone would be additive. The
point is that the enabling seam (b)+(c) is the framework change that must be
confirmed first.)

---

## Part 4 -- OWNER DECISION (required before any code)

### The seam that must change
ff-desktop currently has NO viewer subsystem on the single command-dispatch
path. Satisfying Req 14 requires introducing one: a shell-owned ViewerRegistry
and a `viewer.preview` command resolvable through resolve_target ->
dispatch_command_target, plus a decision on how the rendered viewer surfaces in
the shell. This creates a new seam at framework mechanism 1 (and possibly
mechanism 5), which framework-conformance.md reserves for owner confirmation.

### Options

Option A -- Wire PREVIEW into the shell as a registered Function command, surface
via the ff-viewers ViewerPanel (DockablePanel).
- Add ff-viewers + ff-mdx-plugin + ff-md-viewer deps to ff-desktop.
- Own a ViewerRegistry on WorkbenchShell; register built-ins + MdxFileViewer at
  startup (Req 14.1).
- Register a `viewer.preview` CommandHandler into the shell CommandRegistry so a
  typed `PREVIEW ...`, a menu option, and a shortcut all resolve to the same
  `Function` target through resolve_target -> dispatch_command_target
  (Req 14.2, 14.3). The handler adapts to ff-viewers PreviewCommand.
- Render through ff-viewers ViewerPanel docked in the layout.
- Tradeoffs: faithful to the ff-viewers design (ViewerPanel/DockablePanel exists
  for exactly this); reuses the existing PREVIEW action semantics; command parity
  is natural. Cost: ff-viewers ViewerPanel must integrate with ff-layout docking,
  and ff-desktop gains a non-trivial new subsystem; the PreviewCommand ->
  ff_command adapter is new glue. Does NOT by itself require a WorkspaceContext.

Option B -- Surface the markdown viewer as its own shell WorkspaceContext.
- Same registry + command wiring as A, but render the viewer as a shell Context
  (like ScrmViewerState) rather than a dock panel.
- MUST implement WorkspaceContext::render -> InteriorFocus with a stable
  first-control egui::Id dispatched via render_workspace_context, AND add the
  mandatory full-shell first-Tab egui_kittest test (mechanism 5 /
  workspace-conformance.md).
- Tradeoffs: fits the Context/tab model and the focus contract the shell already
  enforces; precedent exists (ScrmViewerState). Cost: more work (focus latch +
  Tab test); conceptually a "Context" for what is really a read-only preview,
  which may duplicate the ViewerPanel concept ff-viewers already provides.

Option C -- Defer Task 21.
- Leave the four markdown crates as covered-and-green (Tasks 13-20) and keep
  Req 14 NOT COVERED with an explicit deferral note, until the broader
  viewer-framework-into-shell integration is scheduled as its own gated change.
- Tradeoffs: zero framework risk now; the standalone ffmdx.exe still delivers the
  user-facing markdown explorer. Cost: the in-shell "invoke like every other
  action" story (Req 14) stays unmet; ff-mdx-plugin remains an orphan.

### Recommendation
Option A, conditional on owner approval, with Option C as the safe fallback if
the owner does not want to grow ff-desktop now.

Rationale: Req 14.1 explicitly names "the ff-viewers Viewer_Registry" and Req 7
already defines ViewerPanel as a DockablePanel -- the ff-viewers design intends a
docked panel, so Option A builds ON that design with the least conceptual
duplication, and keeps the single-dispatch contract (14.2/14.3) intact by
registering one `viewer.preview` Function command that menu/shortcut/typed paths
all share. Option B is only preferable if the owner wants markdown preview to be
a first-class tab/Context; it costs the extra InteriorFocus + first-Tab test and
risks duplicating ViewerPanel. Either A or B is a framework change and needs the
owner's express yes before code. Option C is the correct choice if the owner
wants to keep ff-desktop unchanged for now.

### If (and only if) the owner approves Option A -- TDD outline (do NOT start before approval)
1. Add deps: ff-viewers, ff-mdx-plugin, ff-md-viewer to crates/ff-desktop/
   Cargo.toml. Verify: `cargo check -p ff-desktop`.
2. RED test: a shell unit test asserting `shell.cmd_registry.contains(viewer.preview)`
   after construction (mirrors the existing config.open assertion in
   shell/tests.rs:5661). `// Validates: Requirement 14.2`.
3. GREEN: own a ViewerRegistry on WorkbenchShell; register built-ins +
   MdxFileViewer at startup; register a `viewer.preview` CommandHandler adapting
   to ff-viewers PreviewCommand. Verify: `cargo test -p ff-desktop viewer`.
4. RED test: resolve_target("PREVIEW LIST") -> Function{command_id:"viewer.preview"}
   dispatched via dispatch_command_target reaches the handler (no bespoke
   intercept). `// Validates: Requirement 14.2, 14.1`.
5. RED test: a menu option / shortcut bound to `viewer.preview` dispatches the
   SAME command (command parity). `// Validates: Requirement 14.3`.
6. GREEN each; then flip the Req 14 TCR rows (14.1/14.2/14.3) to PASS.
7. Scoped gate: `cargo fmt`; `cargo check -p ff-desktop`;
   `cargo test -p ff-desktop`; `cargo clippy -p ff-desktop`. Then HAND OFF to the
   owner for the full verify.ps1 gate (never run it from Kiro).
   (If Option B is chosen instead, add: WorkspaceContext::render -> InteriorFocus
   with a stable first-control egui::Id dispatched via render_workspace_context,
   plus the full-shell first-Tab egui_kittest test modelled on
   full_shell_config_first_tab_focuses_filter_field.)

---

## What was NOT changed
No source file was modified. This is investigation and planning only, per the
brief. The verdict is based on code read this session (ff-desktop Cargo.toml and
shell sources, ff-command command_target.rs, ff-desktop target_dispatch.rs,
ff-viewers registry/trait_def/command/plugin_bridge, ff-mdx-plugin lib/viewer),
the spec (requirements.md Req 11-17), tasks.md Tasks 12-21, and the TCR CR-CH-049
section -- not on the prior report alone.
