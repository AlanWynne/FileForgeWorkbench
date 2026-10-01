# DECOMP Wave 6 -- Implementation Plan

Scope: HYBRID-approved MECHANICAL extraction (Option A). Behaviour-preserving,
leaves-first, thin adapters left in ff-desktop. THREE tasks: 20, 21, 22.

All paths below are inside the worktree:
`c:/workspace/VSC/FileForgeWorkbench/.worktrees/decomp-wave6`.

## Constraints that bind EVERY task

- Behaviour-preserving: NO observable behaviour change. Pure refactor -- NO
  requirements gate (no `docs/specs` edits, no new criteria). Each moved item is
  moved as-is; signatures only change where a clean seam demands it (Task 21),
  and that change is internal plumbing, not observable behaviour.
- rust-standards: no `.rs` file over 400 NON-TEST lines; no `unwrap()`/`expect()`
  in library code (tests may `expect("why")`); `thiserror` for library errors
  (none are introduced here -- these modules return `String`/`Option`, so no new
  error type is needed); ASCII-only in `.rs` files (box-drawing is NOT allowed in
  `.rs` -- replace the existing `-- ... --` separators with ASCII
  `// === ... ===` when a file is created from moved content).
- documentation: ASCII only.
- Each new crate carries `version/edition/authors/license` from
  `[workspace.package]` via `.workspace = true`, matching the ff-theme-editor
  manifest pattern.
- Per-task SCOPED gate ONLY (NEVER `--workspace`, NEVER `verify.ps1`). For a task
  adding crate `<new>`:
  - `cargo fmt -p <new> -p ff-desktop -- --check`
  - `cargo clippy -p <new> -p ff-desktop --tests -- -D warnings`
  - `cargo nextest run -p <new> -p ff-desktop`  (nextest, NOT plain `cargo test`
    -- B048 env-var process isolation).
- Every MOVED test keeps its `// Validates: Requirement X.Y` annotation verbatim.
  Weaken no test.

## DEFERRED (OUT OF SCOPE -- do not do here)

A full re-architecture of the editor onto editor-aspect crates is a SEPARATE
gated stream (CR-NR-099), to be run when editor testing begins. Wave 6 does NOT
create editor-aspect crates, does NOT invert the dependency graph, and does NOT
make moved functions generic over the tab type. `TabState`/`TabId`/`UndoEntry`
and `TabManager` are the SHELL RUNTIME MODEL and STAY in ff-desktop in all three
tasks. Leave a one-line note in `ff-editor-panel`'s crate doc comment pointing at
CR-NR-099 for the deferred refactor.

## Verified facts from exploration (ground truth)

- `scroll_amount.rs`: ZERO `crate::` references. Public items: `ScrollAmount`
  (enum + `parse`, `display`, `display_string`, `to_line_count`). Consumers of
  `crate::scroll_amount::ScrollAmount`: `nav_manager.rs`, `shell/commands.rs`,
  `shell/render.rs`, `editor_panel.rs`, `shell/mod.rs`
  (`pub(crate) use crate::scroll_amount::ScrollAmount;`), `shell/tests.rs`. True
  clean leaf.
- `exclude_manager.rs`: couples to `crate::tab_manager::TabManager`,
  `crate::tab_state::TabId`, and free fn
  `snapshot_lines(tab: &crate::tab_state::TabState, runtime: &Runtime) -> Vec<String>`
  (reads `tab.line_count`, pulls lines via `ff_document_model::LineNumber` under
  `runtime.block_on`). Depends on `ff-display-line-mapping` (`ContractionState`)
  and `ff-exclude-show-filter` (`DocumentAccess`, `ExcludeArgs`, `ExcludeScope`,
  `ExclusionBlock`, `ExclusionEngine`, `ResetVariant`, `ShowArgs`). Internal
  `TabDocAdapter { lines: Vec<String> }` implements `DocumentAccess`.
  `ExcludeManager` is used from `shell/mod.rs` (field + `::new()`) and
  `editor_panel.rs` (`.exclusion_blocks(tab_id)`). The engine is only rebuilt
  when `needs_rebuild` (missing or stale line_count) -- `snapshot_lines` is
  called ONLY on rebuild (laziness that MUST be preserved).
- `editor_panel.rs`: 1512 lines total; NON-TEST code ends at ~line 775. Only
  external entry point is `editor_panel::render(...)` called from
  `shell/render.rs`. Couples to `crate::scroll_amount::ScrollAmount`,
  `crate::exclude_manager::ExcludeManager`, `crate::tab_state::{TabId, TabState,
  UndoEntry}`. Already uses `eframe::egui`, `ff_command_semantics::CommandEngine`,
  `ff_document_model::{Document, BytePosition, LineNumber, new_document}`,
  `ff_exclude_show_filter::ExclusionBlock`,
  `ff_viewport_scrolling::{CaretPolicyEngine, CursorModel, ViewportModel}`,
  `tokio::runtime::Runtime`, `arboard`.
- `main.rs` declares `mod scroll_amount; mod exclude_manager; mod editor_panel;`.
- ff-theme-editor is the proven extraction pattern (pure crate, re-export shim in
  ff-desktop).

---

# TASK 20 -- scroll_amount -> ff-scroll-amount (TRUE CLEAN LEAF)

Verbatim move of a zero-`crate::`-reference leaf. New crate has NO dependencies
(no egui, no serde -- `ScrollAmount` uses none).

- [ ] 20.1 Create crate `crates/ff-scroll-amount` with a manifest modelled on
      ff-theme-editor (workspace-inherited package fields; NO `[dependencies]`
      needed; add `[dev-dependencies] pretty_assertions = { workspace = true }`
      only if the moved tests use it -- they do NOT, so dev-deps may be omitted).
      Files: `crates/ff-scroll-amount/Cargo.toml`.
      Verify: `cargo check -p ff-scroll-amount` (after 20.2) -- builds.

- [ ] 20.2 Move `scroll_amount.rs` into the new crate VERBATIM as
      `crates/ff-scroll-amount/src/lib.rs`, including its full `#[cfg(test)] mod
      tests`. Replace any non-ASCII (the `===>` arrow glyph in the doc comment is
      ASCII `===>`, fine; confirm no em-dash/curly quotes) and keep ASCII section
      style. Keep every `// Validates: Requirement 19.x` annotation.
      Files: `crates/ff-scroll-amount/src/lib.rs`.
      Verify: `cargo nextest run -p ff-scroll-amount` -- all ScrollAmount tests
      pass in the new crate.

- [ ] 20.3 Add `"crates/ff-scroll-amount"` to the `members` list in the workspace
      `Cargo.toml`, and add `ff-scroll-amount = { path = "../ff-scroll-amount" }`
      to `crates/ff-desktop/Cargo.toml` `[dependencies]`.
      Files: `Cargo.toml`, `crates/ff-desktop/Cargo.toml`.
      Verify: `cargo check -p ff-scroll-amount` -- resolves in the workspace.

- [ ] 20.4 Replace the body of `crates/ff-desktop/src/scroll_amount.rs` with a
      re-export shim so EVERY existing `crate::scroll_amount::...` path keeps
      resolving: `//! Re-export shim ...` + `pub use ff_scroll_amount::*;`. Keep
      the `mod scroll_amount;` declaration in `main.rs` unchanged. (This file now
      contains no tests -- the tests moved with the module in 20.2.) This keeps
      `shell/mod.rs`'s `pub(crate) use crate::scroll_amount::ScrollAmount;` and
      all `crate::scroll_amount::ScrollAmount` consumers compiling unchanged.
      Files: `crates/ff-desktop/src/scroll_amount.rs`.
      Verify: `cargo check -p ff-desktop` -- compiles with the shim.

- [ ] 20.5 Run the Task 20 scoped gate:
      `cargo fmt -p ff-scroll-amount -p ff-desktop -- --check`;
      `cargo clippy -p ff-scroll-amount -p ff-desktop --tests -- -D warnings`;
      `cargo nextest run -p ff-scroll-amount -p ff-desktop`.
      Verify: all three clean; the ff-desktop `scroll_amount_*` shell tests
      (`scroll_amount_defaults_to_page`, `scroll_command_*`,
      `scroll_amount_retained_across_commands`, `command_up/down_with_scroll_*`)
      still pass unchanged.

---

# TASK 21 -- exclude_manager -> ff-exclude-manager (CLEAN-SEAM REFACTOR)

NOT a verbatim move. The new crate operates on a plain `u64` tab id, a
`line_count`, and a lazily-supplied line snapshot. The `TabManager` / Tokio
runtime / `TabState` glue (the current `snapshot_lines` + `active_tab_mut`
plumbing) becomes a thin ADAPTER staying in ff-desktop.

## CHOSEN SEAM SIGNATURE (and why)

The crate's `ExcludeManager` keeps the per-tab `HashMap<u64, ExclusionEngine<
ContractionState, TabDocAdapter>>` and the private `TabDocAdapter { lines:
Vec<String> }` (both move into the crate -- `TabDocAdapter` has no shell
coupling). Its command methods take the tab identity and content explicitly:

```
pub fn exclude_all   (&mut self, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn exclude_text  (&mut self, text: &str, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn exclude_text_all(&mut self, text: &str, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn show_all      (&mut self, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn show_text     (&mut self, text: &str, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn reset         (&mut self, variant: ResetVariant, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> String
pub fn exclusion_blocks(&self, tab_id: u64) -> Vec<ExclusionBlock>   // unchanged shape, u64 id
pub fn is_excluded(&self, tab_id: u64, doc_line_1based: u64) -> bool // unchanged shape, u64 id
```

The private `engine_for` becomes
`fn engine_for(&mut self, tab_id: u64, line_count: usize, lines: impl FnOnce() -> Vec<String>) -> &mut ExclusionEngine<...>`
and calls the closure ONLY when `needs_rebuild` -- preserving the current
rebuild-only snapshot laziness exactly.

**Chosen: a lazy closure `impl FnOnce() -> Vec<String>`, NOT an eager
`Vec<String>`.** Reasons:
1. Behaviour preservation: the current code calls `snapshot_lines` ONLY inside
   the `needs_rebuild` branch. An eager `Vec<String>` parameter would force the
   caller to snapshot the whole document on EVERY exclude/show/reset invocation
   even when the engine is already fresh -- that is extra work per command, a
   behaviour change. The closure keeps the snapshot gated on rebuild, identical
   to today.
2. Crate purity: the closure carries the Tokio `Runtime` + `TabState` read on
   the ADAPTER side, so `ff-exclude-manager` depends on NEITHER `tokio` NOR
   `ff-document-model` NOR any shell type. The crate stays a pure logic crate
   over `ff-display-line-mapping` + `ff-exclude-show-filter`.
3. `TabId` is a newtype over `u64`; the adapter passes `tab.id.0` (or an
   accessor), so the crate never sees `TabId`.

`exclusion_blocks` and `is_excluded` keep their current `#[allow(dead_code)]`
only if still unused after the move; `exclusion_blocks` IS used by
`editor_panel::render` (via the adapter re-export), so drop the `allow` there if
clippy now sees it as used -- adjust to keep `clippy -D warnings` clean.

## The ff-desktop ADAPTER (stays, ~30-40 lines)

`crates/ff-desktop/src/exclude_manager.rs` becomes the adapter module that OWNS
the `crate::exclude_manager::ExcludeManager` path. It:
- `pub use ff_exclude_manager::ExcludeManager;` so `shell/mod.rs`'s
  `use crate::exclude_manager::ExcludeManager;` and `editor_panel`'s use keep
  resolving unchanged.
- Keeps the `snapshot_lines(tab: &crate::tab_state::TabState, runtime: &Runtime)
  -> Vec<String>` free fn (moves nowhere -- it reads `TabState`/`Document` under
  the runtime).
- The shell call sites that today call `mgr.exclude_all(&mut tabs, &rt)` etc.
  (search `self.exclude_manager.` across `shell/`) are rewritten to pass
  `tab.id.0`, `tab.line_count as usize`, and a closure
  `|| snapshot_lines(tab, &self.runtime)`. The cleanest placement is small
  adapter methods on the shell OR free helper fns in this module; choose shell
  methods only if they already exist, otherwise inline the three args at each
  call site (there are a handful). KEEP the exact same returned `String`.

NOTE for implementer: confirm every current caller of the old
`ExcludeManager::{exclude_all, exclude_text, exclude_text_all, show_all,
show_text, reset}` (grep `exclude_manager\.` / `self.exclude_manager` under
`crates/ff-desktop/src/shell/`) and update each to the new 3-extra-arg form via
the adapter. `editor_panel`'s `exclude_manager.exclusion_blocks(tab_id)` call
takes `tab_id` which is a `TabId`; pass `tab_id.0` after the move (Task 22
touches this file anyway; keep the two tasks' edits consistent).

## Tests

- Pure-logic tests MOVE to the crate but MUST NOT build a `TabManager`. The four
  message/behaviour tests currently build a real `TabManager` via
  `make_tabs` -> those STAY in the ff-desktop adapter's test module (they
  exercise the adapter end-to-end). In the CRATE, add equivalent pure tests that
  drive the new signature directly with an inline `|| vec!["..."]` closure and a
  literal `line_count`/`tab_id`, keeping the SAME `// Validates: Requirement
  21.5` annotations. (The `exclusion_blocks_returns_correct_count` test calls the
  private `engine_for` + `engine.exclude_line` -- its crate-side equivalent uses
  the crate's own `engine_for`.)
- The adapter test that builds a `TabManager` stays in ff-desktop and is updated
  to call through the adapter (same `// Validates:` annotations).

## Steps

- [ ] 21.1 Create crate `crates/ff-exclude-manager` with manifest deps
      `ff-display-line-mapping = { path = "../ff-display-line-mapping" }` and
      `ff-exclude-show-filter = { path = "../ff-exclude-show-filter" }`;
      dev-deps `pretty_assertions = { workspace = true }`. Workspace-inherited
      package fields.
      Files: `crates/ff-exclude-manager/Cargo.toml`.
      Verify: `cargo check -p ff-exclude-manager` (after 21.2) -- builds.

- [ ] 21.2 Author `crates/ff-exclude-manager/src/lib.rs`: move `TabDocAdapter`
      and `ExcludeManager` with the CHOSEN seam signatures above (u64 tab id,
      `line_count`, `impl FnOnce() -> Vec<String>` lines; `engine_for` calls the
      closure only on rebuild). Crate doc comment states it is the pure
      exclude/show/reset engine store with NO shell/runtime dependency. ASCII
      separators. Add the crate-side pure tests (Req 21.5 equivalents) using
      inline closures.
      Files: `crates/ff-exclude-manager/src/lib.rs`.
      Verify: `cargo nextest run -p ff-exclude-manager` -- pure tests pass.

- [ ] 21.3 Add `"crates/ff-exclude-manager"` to workspace `Cargo.toml` members
      and `ff-exclude-manager = { path = "../ff-exclude-manager" }` to
      `crates/ff-desktop/Cargo.toml`.
      Files: `Cargo.toml`, `crates/ff-desktop/Cargo.toml`.
      Verify: `cargo check -p ff-exclude-manager`.

- [ ] 21.4 Rewrite `crates/ff-desktop/src/exclude_manager.rs` as the thin
      adapter: `pub use ff_exclude_manager::ExcludeManager;`, keep
      `snapshot_lines(tab, runtime)`, and keep the `TabManager`-building test
      (`make_tabs` + the four Req 21.5 message tests) updated to call through the
      new signature via the adapter. Keep `mod exclude_manager;` in `main.rs`.
      Files: `crates/ff-desktop/src/exclude_manager.rs`.
      Verify: `cargo check -p ff-desktop`.

- [ ] 21.5 Update every shell call site of the exclude/show/reset methods
      (`crates/ff-desktop/src/shell/**`) and `editor_panel`'s
      `exclusion_blocks(tab_id)` to the new argument form (`tab.id.0`,
      `tab.line_count as usize`, `|| snapshot_lines(tab, runtime)`). No observable
      behaviour change -- same returned `String`.
      Files: shell files found by grep `exclude_manager\.`/`self.exclude_manager`;
      `crates/ff-desktop/src/editor_panel.rs` (coordinate with Task 22).
      Verify: `cargo check -p ff-desktop` -- compiles; the EXCLUDE/SHOW/RESET
      shell tests still pass.

- [ ] 21.6 Run the Task 21 scoped gate:
      `cargo fmt -p ff-exclude-manager -p ff-desktop -- --check`;
      `cargo clippy -p ff-exclude-manager -p ff-desktop --tests -- -D warnings`;
      `cargo nextest run -p ff-exclude-manager -p ff-desktop`.
      Verify: all clean; crate pure tests + ff-desktop adapter `TabManager` tests
      both pass; EXCLUDE/SHOW/RESET behaviour unchanged.

---

# TASK 22 -- editor_panel -> ff-editor-panel (HARDEST)

FIXED: `TabState`/`TabId`/`UndoEntry` STAY in ff-desktop. NO shared-types crate,
NO dependency inversion, NO generics over the tab type. KEEP the shell-entangled
`render` (and anything touching `&mut TabState` / `CommandEngine` / live
`ExcludeManager` / `Runtime`) as a THIN ADAPTER in ff-desktop
(`crate::editor_panel`). MOVE only the PURE helpers into `ff-editor-panel`.

## MOVE to ff-editor-panel (pure -- no TabState/TabId/UndoEntry)

| Item | Signature touches | Notes |
|------|-------------------|-------|
| `DisplayRow` enum | `ExclusionBlock`, `String` | pub(crate) today -> make `pub` in crate |
| `build_display_list` | `&[String]`, `&[ExclusionBlock]` | pure |
| `scroll_by_amount` | `&mut ViewportModel`, `&mut CursorModel`, `&ScrollAmount`, `bool` | uses ff-scroll-amount + ff-viewport-scrolling; private today -> `pub` in crate so the adapter can call it |
| `line_char_count` | `&Document`, `u64` | pure; `pub(crate)` -> `pub` in crate (adapter calls it) |
| `extract_selected_text` | `&Document`, `(u64,u64,u64,u64)` | pure |
| `normalise_selection` | four `u64` | pure |
| `cursor_byte_position` | `&Document`, `u64`, `u64` | pure |
| `BASE_FONT_SIZE_PT`, `BASE_LINE_HEIGHT_PX`, `PREFIX_COLS`, `PREFIX_WIDTH`, `GUTTER_CHAR_WIDTH` | consts | geometry; move with the render helpers (GUTTER_CHAR_WIDTH referenced by a moved test) |

Visibility bump: items that are `pub(crate)` or private in ff-desktop but are
called by the ff-desktop adapter across the crate boundary must become `pub` in
`ff-editor-panel`. `GUTTER_CHAR_WIDTH` keeps `#[allow(dead_code)]` if still only
used by a test.

## STAY as ff-desktop adapter (`crate::editor_panel`, touches TabState / shell)

| Item | Why it stays |
|------|--------------|
| `pub fn render(ui, tab: &mut TabState, runtime, cmd_engine, exclude_manager, tab_id: TabId, scroll_amount)` | THE shell entry point; mutates `TabState`, drives `CommandEngine`, live `ExcludeManager`, `Runtime`. Calls the moved pure helpers via `ff_editor_panel::*`. |
| `pub struct EditorPanel` + `impl EditorPanel::new_empty()` | returns `TabState` (`crate::tab_state`) -- test-only wrapper. |

`render` is the only non-test code that must stay. After the pure helpers move,
the adapter `render` keeps its body verbatim EXCEPT that calls to the moved
helpers are now `ff_editor_panel::build_display_list(...)`,
`ff_editor_panel::scroll_by_amount(...)`, `ff_editor_panel::line_char_count(...)`,
`ff_editor_panel::extract_selected_text(...)`,
`ff_editor_panel::normalise_selection(...)`,
`ff_editor_panel::cursor_byte_position(...)`, and the geometry consts are read
from `ff_editor_panel::` (or re-exported). The `DisplayRow` match in `render`
uses `ff_editor_panel::DisplayRow`.

## 400-line check (NON-TEST lines)

- `ff-editor-panel` non-test code = the moved pure helpers. Estimated well under
  400 lines (build_display_list ~45, scroll_by_amount ~55, extract_selected_text
  ~40, normalise_selection ~10, cursor_byte_position ~12, line_char_count ~6,
  consts ~10, DisplayRow ~10, doc comments). ONE file `src/lib.rs` is fine. IF it
  exceeds 400 non-test lines once assembled, split by concern:
  `src/geometry.rs` (consts + `line_char_count`), `src/display_list.rs`
  (`DisplayRow` + `build_display_list`), `src/scroll.rs` (`scroll_by_amount`),
  `src/selection.rs` (`extract_selected_text`, `normalise_selection`,
  `cursor_byte_position`), with `src/lib.rs` re-exporting. Decide AFTER assembly
  by measuring; prefer a single lib.rs if it fits.
- ff-desktop adapter `editor_panel.rs` non-test code after the move = `render`
  (~380 non-test lines in the current file minus the ~180 lines of pure helpers
  that leave) PLUS the `EditorPanel` wrapper. This should land UNDER 400 non-test
  lines. MEASURE after the move with
  `rg --count-matches '' crates/ff-desktop/src/editor_panel.rs` minus the test
  module; if `render` alone still exceeds 400 non-test lines, split the ADAPTER
  by concern (`editor_panel/mod.rs` re-export + `editor_panel/render.rs` for
  `render` + `editor_panel/input.rs` for the key/text/mouse event blocks that
  `render` currently inlines) -- a mechanical same-behaviour split. Keep
  `crate::editor_panel::render` resolving (module path unchanged).

## Test ownership split

MOVE to `ff-editor-panel` tests (pure -- no TabState):
- `build_display_list_*` (5 tests: no_exclusions, single_block, two_blocks,
  placeholder_text_contains_count, excluded_lines_not_in_output) -- Req 6.x.
- `normalise_selection_*` (3 tests) -- Req 13.4.
- `extract_selected_text_*` (4 tests: single_line, multi_line, empty, reversed)
  -- Req 20.1/20.3/20.4 (use `ff_document_model::new_document`, which is a crate
  dep).
- `scroll_by_amount_*` (all Req 14.1-14.8 tests at the file tail) -- use
  `ViewportModel`/`CursorModel`/`ScrollAmount` only.
- The viewport/cursor-only tests that use ONLY `ViewportModel`/`CursorModel`/
  `CaretPolicyEngine` (`arrow_down_advances_cursor_line`,
  `arrow_up_retreats_cursor_line`, `arrow_down_at_last_line_is_noop`,
  `arrow_up_at_first_line_is_noop`, `arrow_left_retreats_cursor_column`,
  `arrow_right_advances_cursor_column`, `page_down_*`, `page_up_*`,
  `arrow_down_scrolls_viewport_when_cursor_leaves_visible_area`) MAY move to the
  crate IF they do not import `crate::tab_state`. CHECK each: those that only
  construct `ViewportModel`/`CursorModel` move; any that construct `TabState`
  stay. (These are really viewport-crate behaviour; keeping them wherever they
  compile cleanly is fine -- do NOT weaken or duplicate.)

STAY in ff-desktop adapter tests (need TabState/TabId/UndoEntry/`new_document`
+ TabState):
- `typed_character_inserts_into_document`, `backspace_deletes_character_before_cursor`,
  `enter_key_splits_line_in_insert_mode`, `new_tab_top_line_is_one`,
  `tab_with_content_has_correct_line_count`, `mouse_click_sets_cursor_to_clicked_line_and_column`,
  `ctrl_z_undoes_last_insert`, `cursor_line_is_tracked_for_highlight`,
  `backspace_at_column_1_joins_line_to_previous`,
  `backspace_at_column_1_on_first_line_is_noop`, `cursor_column_is_tracked_for_caret_bar`,
  `prefix_*` (6 tests -- use `CommandEngine` and/or `TabState.prefix_inputs`),
  `tab_state_has_prefix_inputs_map`, `new_tab_has_no_canvas_selection`,
  `canvas_selection_can_be_set_and_cleared`, `canvas_selection_cleared_on_tab_switch`.
  These reference `super::cursor_byte_position` etc.; update those to
  `ff_editor_panel::cursor_byte_position` after the move (the adapter can also
  re-export the pure helpers so `super::` keeps resolving -- prefer re-export to
  minimise test churn).

Every moved test keeps its `// Validates: Requirement X.Y` annotation.

## Steps

- [ ] 22.1 Create crate `crates/ff-editor-panel` manifest. Deps:
      `ff-scroll-amount = { path = "../ff-scroll-amount" }`,
      `ff-exclude-manager = { path = "../ff-exclude-manager" }`,
      `ff-document-model = { path = "../ff-document-model" }`,
      `ff-viewport-scrolling = { path = "../ff-viewport-scrolling" }`,
      `ff-exclude-show-filter = { path = "../ff-exclude-show-filter" }`,
      `egui = { workspace = true }`. (Needs `ff-exclude-show-filter` for
      `ExclusionBlock` in `DisplayRow`; `egui` only if a moved helper references
      egui types -- the listed pure helpers do NOT use egui, so OMIT egui unless
      assembly shows a need. ff-scroll-amount is needed for `scroll_by_amount`'s
      `ScrollAmount` param.) dev-deps: `tokio = { workspace = true }` and
      `pretty_assertions = { workspace = true }` (moved tests use
      `runtime.block_on`). NO dependency on ff-desktop.
      Files: `crates/ff-editor-panel/Cargo.toml`.
      Verify: `cargo check -p ff-editor-panel` (after 22.2) -- builds.

- [ ] 22.2 Author `ff-editor-panel` library: move the pure helpers + consts +
      `DisplayRow` listed in the MOVE table, as `pub`. Single `src/lib.rs` if
      under 400 non-test lines; otherwise split per the 400-line plan above.
      Crate doc comment: pure GUI-independent editor-panel helpers; note the full
      editor refactor is DEFERRED to CR-NR-099. Move the pure tests
      (build_display_list, normalise_selection, extract_selected_text,
      scroll_by_amount, and viewport-only tests that compile without TabState)
      keeping `// Validates:` annotations. ASCII separators.
      Files: `crates/ff-editor-panel/src/lib.rs` (+ split files if needed).
      Verify: `cargo nextest run -p ff-editor-panel` -- moved pure tests pass.

- [ ] 22.3 Add `"crates/ff-editor-panel"` to workspace `Cargo.toml` members and
      `ff-editor-panel = { path = "../ff-editor-panel" }` to
      `crates/ff-desktop/Cargo.toml`.
      Files: `Cargo.toml`, `crates/ff-desktop/Cargo.toml`.
      Verify: `cargo check -p ff-editor-panel`.

- [ ] 22.4 Rewrite `crates/ff-desktop/src/editor_panel.rs` as the adapter: keep
      `render` and `EditorPanel`; replace in-body calls to the moved helpers with
      `ff_editor_panel::...`; re-export the pure helpers
      (`pub use ff_editor_panel::{build_display_list, scroll_by_amount,
      line_char_count, extract_selected_text, normalise_selection,
      cursor_byte_position, DisplayRow, GUTTER_CHAR_WIDTH, PREFIX_WIDTH, ...}`) so
      existing `super::`/`crate::editor_panel::` test and shell references resolve
      unchanged. Keep the STAY tests here, updated to use the re-exports. Keep
      `mod editor_panel;` in `main.rs`. If `render` + wrapper exceed 400 non-test
      lines, split the adapter into `editor_panel/{mod.rs,render.rs,input.rs}`
      (mechanical, same behaviour) and keep `crate::editor_panel::render`
      resolving.
      Files: `crates/ff-desktop/src/editor_panel.rs` (or `editor_panel/` dir).
      Verify: `cargo check -p ff-desktop` -- `shell/render.rs`'s
      `editor_panel::render(...)` call still resolves and compiles.

- [ ] 22.5 Measure non-test line counts of every new/edited `.rs`
      (`rg --count-matches '' <file>` minus the test module span). Split any file
      over 400 non-test lines per the 400-line plan. No behaviour change.
      Files: `crates/ff-editor-panel/**`, `crates/ff-desktop/src/editor_panel*`.
      Verify: no `.rs` over 400 non-test lines; `cargo check -p ff-editor-panel
      -p ff-desktop` still clean.

- [ ] 22.6 Run the Task 22 scoped gate:
      `cargo fmt -p ff-editor-panel -p ff-desktop -- --check`;
      `cargo clippy -p ff-editor-panel -p ff-desktop --tests -- -D warnings`;
      `cargo nextest run -p ff-editor-panel -p ff-desktop`.
      Verify: all clean; the shell editor render path + all moved and stayed
      editor tests pass; no observable behaviour change.

---

# Ordering / dependency notes

- Task 20 FIRST (ff-scroll-amount is a dep of ff-editor-panel).
- Task 21 SECOND (ff-exclude-manager is a dep of ff-editor-panel).
- Task 22 LAST (depends on both 20 and 21).
- Each task leaves the workspace buildable and its scoped gate green before the
  next begins.

# Hand-off reminder

Kiro runs ONLY the scoped `-p <crate> -p ff-desktop` checks per task, then hands
off: the OWNER runs the full `verify.ps1` gate manually outside Kiro. Never run
`--workspace` or `verify.ps1` from Kiro.

# Gaps / assumptions

- Assumes `TabId` is a public newtype `TabId(pub u64)` (confirmed by
  `TabId(0)`/`tab.id` usage); the adapter passes `tab.id.0`. If `.0` is not
  public, add a `pub fn id(&self) -> u64` accessor on the adapter side only (no
  crate change). CONFIRM at implementation time.
- Assumes `ExclusionBlock` is `Copy` (it is used by-value in the current
  `DisplayRow::Placeholder { block: blocks[block_idx] }`); moving `DisplayRow`
  needs `ff-exclude-show-filter` as a crate dep (listed).
- The exact set of viewport-only tests that can move vs stay is decided at
  implementation time by whether each imports `crate::tab_state`; the default is
  "stay if it touches TabState".
