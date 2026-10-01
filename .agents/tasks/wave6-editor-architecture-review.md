# DECOMP Wave 6 -- Editor Architecture Review (advisory, read-only)

Status: ADVISORY ONLY. No gate, no code change. Produced from a read-only
investigation of the main checkout at `c:\workspace\VSC\FileForgeWorkbench`
(branch decomp-and-scrm-wave, Waves 1-5 landed). Line counts are approximate
(2026-09-26 baseline per the spec; re-measure at extraction time).

---

## 1. Summary answer (read this first)

The owner's hypothesis is CONFIRMED with an important twist.

- `editor_panel.rs` (~1650 lines, but only ~770 non-test; the rest is one large
  `#[cfg(test)] mod tests`) is almost entirely (a) the egui RENDER surface for a
  single tab, (b) raw keyboard/mouse INPUT handling, and (c) INLINE edit/undo
  logic that talks straight to `ff-document-model`. It is NOT a composition of
  the editor-aspect crates. It delegates to only three model crates:
  `ff-viewport-scrolling` (cursor/viewport movement + caret policy),
  `ff-exclude-show-filter` (via the local `exclude_manager` adapter, for the
  placeholder display list) and `ff-command-semantics` (prefix line-command
  submission). It uses `ff-document-model` directly for all text I/O.

- The surprise: MOST editor-aspect crates ALREADY EXIST
  (`ff-edit-operations`, `ff-undo-redo`, `ff-caret-selection`,
  `ff-line-commands`, `ff-syntax-highlighting`, `ff-auto-indent`,
  `ff-text-decorations`, `ff-whitespace-guides`, `ff-seqnum`, `ff-wrap`,
  `ff-select`, `ff-clipboard`, `ff-find-and-replace`), but the LIVE editor is
  NOT wired to them. `ff-desktop/Cargo.toml` does not even depend on
  `ff-undo-redo`, `ff-caret-selection`, `ff-line-commands`,
  `ff-syntax-highlighting`, `ff-auto-indent`, `ff-text-decorations`,
  `ff-whitespace-guides`, `ff-seqnum`, `ff-wrap`, `ff-select`, or `ff-clipboard`.
  The editor reimplements insert/backspace/enter/undo inline and only consumes
  `EditProfile` (the CAPS/NULLS/STATS/LOCK/HILITE flags) out of
  `ff-edit-operations`.

Consequence for the choice: splitting `ff-editor-panel` into "aspect sub-crates"
would mostly create crates that DUPLICATE the names/concerns of crates that
already exist but are unused. That is ceremony, not value. The real architectural
debt is the opposite of what Option B implies: the editor is UNDER-integrated,
not under-decomposed.

RECOMMENDATION: a **HYBRID**. Do the mechanical Wave 6 extraction as specced
(Option A: `ff-scroll-amount`, `ff-exclude-manager`, `ff-editor-panel`
render-only crate; `TabState` stays in `ff-desktop`), because it is a safe,
behaviour-preserving refactor that reduces the monolith now. Do NOT invent new
aspect crates. SEPARATELY, record a NON-refactor follow-up (requires the gate)
to progressively WIRE the editor onto the existing aspect crates
(`ff-edit-operations` edit ops + `ff-undo-redo` transactions first). Keep the two
streams apart: Wave 6 is a pure move; the integration is observable behaviour and
must go through the requirements gate.

---

## 2. What `editor_panel.rs` actually contains (breakdown by concern)

Non-test body is ~770 lines; the `#[cfg(test)] mod tests` block is ~880 lines
(roughly half the file). Approximate non-test breakdown by concern:

| Concern | ~lines | Backing crate it delegates to | Inline (no crate used)? |
|---------|-------:|-------------------------------|-------------------------|
| `build_display_list` (interleave exclusion placeholders) | ~55 | `ff-exclude-show-filter` (`ExclusionBlock`) | partial -- logic inline, data type from crate |
| `scroll_by_amount` (page/half/csr/max/lines) | ~65 | `ff-viewport-scrolling` (`ViewportModel`/`CursorModel`) + `crate::scroll_amount::ScrollAmount` | glue only |
| `render` -- viewport sizing / visible_count | ~15 | `ff-viewport-scrolling` | glue |
| `render` -- text input (insert typed chars) | ~35 | NONE -- raw `ff-document-model` insert + `tab.undo_stack.push` | INLINE |
| `render` -- Backspace (char delete + line-join) | ~55 | NONE -- raw `ff-document-model` delete | INLINE |
| `render` -- Enter (split line) | ~20 | NONE -- raw `ff-document-model` insert | INLINE |
| `render` -- arrow-key navigation | ~55 | `ff-viewport-scrolling` (`move_cursor_*`, `CaretPolicyEngine`) | glue |
| `render` -- PageUp/PageDown | ~10 | via `scroll_by_amount` | glue |
| `render` -- Ctrl+Z undo | ~25 | NONE -- pops `tab.undo_stack` (local `UndoEntry`), raw doc edit | INLINE |
| `render` -- mouse-wheel scroll | ~25 | `ff-viewport-scrolling` (`scroll_wheel_vertical`) | glue |
| `render` -- read visible lines (async doc reads) | ~25 | `ff-document-model` | glue |
| `render` -- paint lines/placeholders/prefix/caret/selection | ~150 | egui painter + `ff-exclude-show-filter` block text | egui RENDER |
| `render` -- editable prefix area -> line command submit | ~35 | `ff-command-semantics` (`submit_line_command`) | glue |
| `render` -- mouse click/drag selection (`pos_to_line_col`) | ~70 | NONE -- sets `tab.canvas_selection` tuple inline | INLINE |
| `render` -- Ctrl+C copy selection to clipboard | ~30 | `arboard` directly (+ inline `extract_selected_text`) | INLINE (NOT `ff-clipboard`) |
| `extract_selected_text` / `normalise_selection` / `cursor_byte_position` / `line_char_count` | ~110 | NONE -- raw `ff-document-model` geometry | INLINE |
| `EditorPanel` test shim | ~10 | - | test-only |

Concerns WITH a backing crate it genuinely uses: viewport/scroll (fully),
exclusion display (data type), line-command submission (engine).

Concerns implemented INLINE with no dedicated crate backing (despite a matching
crate existing): character insert/delete/enter, undo (local `UndoEntry` enum, not
`ff-undo-redo`), mouse caret + selection model (a bare `(u64,u64,u64,u64)` tuple
on `TabState`, not `ff-caret-selection` / `ff-edit-operations` `SelectionRange`),
clipboard copy (`arboard` + inline extract, not `ff-clipboard`).

Concerns NOT present at all in the live editor: syntax highlighting, whitespace
guides, auto-indent, sequence numbers, line-wrap toggle, text decorations,
find/replace in-canvas, multi-caret. These are rendered as plain monospace text.

---

## 3. Hypothesis verdict: "most aspects are already crates; the monolith is
mostly render + tab glue" -- TRUE, with a sharpening

TRUE that the monolith is mostly render + input + tab glue (see section 2): the
genuinely reusable model logic it needs is already outsourced to
`ff-viewport-scrolling` and `ff-exclude-show-filter`.

TRUE that most aspects already live in dedicated crates -- confirmed by the
workspace crate inventory: `ff-edit-operations`, `ff-undo-redo`,
`ff-caret-selection`, `ff-line-commands`, `ff-syntax-highlighting`,
`ff-auto-indent`, `ff-text-decorations`, `ff-whitespace-guides`, `ff-seqnum`,
`ff-wrap`, `ff-select`, `ff-clipboard`, `ff-find-and-replace` all exist with
rich, GUI-independent public APIs (e.g. `ff-undo-redo` exposes
`DocumentUndoManager`, transactions, coalescing, save-points, recovery;
`ff-edit-operations` exposes `EditModeManager`, `SelectionContainer`,
`SelectionRange`, BOUNDS; `ff-caret-selection` exposes caret shape/colour/blink
display models).

The SHARPENING (and the key evidence that refutes the Option-B framing): those
aspect crates are NOT consumed by the live editor. Evidence:
- `ff-desktop/Cargo.toml` lists `ff-edit-operations`, `ff-find-and-replace`,
  `ff-exclude-show-filter`, `ff-display-line-mapping`, `ff-viewport-scrolling`
  as deps, but DOES NOT list `ff-undo-redo`, `ff-caret-selection`,
  `ff-line-commands`, `ff-syntax-highlighting`, `ff-auto-indent`,
  `ff-text-decorations`, `ff-whitespace-guides`, `ff-seqnum`, `ff-wrap`,
  `ff-select`, or `ff-clipboard`.
- `editor_panel.rs` imports only: `eframe::egui`, `ff_command_semantics`,
  `ff_document_model`, `ff_exclude_show_filter`, `ff_viewport_scrolling`,
  `tokio`, `crate::exclude_manager`, `crate::tab_state`.
- The only use of `ff-edit-operations` anywhere near the editor is `EditProfile`
  (CAPS/NULLS/STATS/LOCK/HILITE), stored on `TabState` and toggled by shell
  commands -- the edit-OPERATIONS (insert/delete/selection) half of that crate
  is unused by the render path.

So: the monolith is small because the editor does LESS than the spec corpus
describes, not because it already composes a dozen aspect crates. Extracting the
render surface is a clean move; manufacturing aspect crates would mostly re-wrap
existing-but-unused crates.

---

## 4. The `tab_state` boundary

### 4a. Exactly what crosses the boundary

`editor_panel::render` has signature:
```
pub fn render(
    ui: &mut egui::Ui,
    tab: &mut TabState,
    runtime: &Runtime,
    cmd_engine: &mut CommandEngine,
    exclude_manager: &mut ExcludeManager,
    tab_id: TabId,
    scroll_amount: &crate::scroll_amount::ScrollAmount,
) -> Option<String>
```

Through `&mut TabState` it touches: `tab.document` (`DocumentHandle`),
`tab.viewport` (`ff_viewport_scrolling::ViewportModel`), `tab.cursor`
(`ff_viewport_scrolling::CursorModel`), `tab.undo_stack`
(`Vec<crate::tab_state::UndoEntry>`), `tab.prefix_inputs` (`HashMap<u64,String>`),
`tab.canvas_selection` (`Option<(u64,u64,u64,u64)>`), `tab.line_count`,
`tab.is_modified`. It uses `tab_id: TabId` only to call
`exclude_manager.exclusion_blocks(tab_id)` (read-only).

Note the editor does NOT touch `TabManager` -- the shell (`shell/render.rs`)
resolves the active tab (`self.tabs.active_tab_mut()`) and passes `&mut TabState`
in. `TabManager` is only coupled to `exclude_manager` (section 5), not to the
editor render.

### 4b. Why `TabState`/`TabId`/`UndoEntry` are shell-entangled

`TabState` is the shell's whole-Workspace runtime record, far wider than an
editor document view. Beyond the editor fields above it carries: `kind: TabKind`
(14+ non-editor variants: MenuWorkspace/POM, ConfigPanel, FileExplorerPanel,
SearchResults, PluginManager, EventLog, MacroLibrary, ThemeEditor, MenusEditor,
KeysEditor, KindsEditor, HelpContext, ScrmViewer, ...), `menu_workspace:
Option<MenuWorkspaceState>`, `is_home`, `workspace_name`, `is_floating`,
`nav_stack: Vec<ff_session::WorkspaceDescriptor>` (the per-tab Navigation_Stack,
CR-CH-022), `edit_profile`, `line_end_mode`. `TabId` is the shell's tab identity
used by `TabManager`, the layout tree, session persistence, and the exclude
manager keying. `UndoEntry` is the editor's private inverse-op enum but lives on
`TabState` because the shell owns the per-tab stack lifecycle.

Pulling `TabState` into an editor crate would drag the entire shell Workspace
model (menus, nav stack, session descriptors, layout) across the boundary -- it
is correctly shell-owned.

### 4c. Options for crossing the boundary from an editor crate

1. Keep the `render` free function in `ff-desktop` as the adapter; move only the
   PURE helpers (geometry, display-list, scroll math, selection text extraction)
   into `ff-editor-panel`. The adapter keeps its `&mut TabState` signature and
   calls crate functions with primitive/borrowed arguments. LOW risk; matches the
   established "thin adapter keeps shell-entangled glue" pattern used for the
   Editor Context in `shell/render.rs` and for Wave 5.
2. Make the editor crate generic over a trait (e.g. `EditorDocView`) that
   `TabState` implements in `ff-desktop`. Cleaner on paper but the render body
   touches ~8 distinct fields with egui-flavoured access patterns and async doc
   reads via the Tokio runtime; the trait would be wide and leaky, and would
   change call sites. MEDIUM churn, LOW payoff for a behaviour-preserving wave.
3. Extract a shared `ff-tab-state` types crate holding `TabState`. REJECTED:
   `TabState` depends on `MenuWorkspaceState`, `ff_session::WorkspaceDescriptor`,
   `TabKind`, etc. -- it would pull half the shell into a "types" crate and invert
   the dependency graph.

RECOMMENDATION: Option 1. The editor crate exposes pure, testable functions and
free `render` helpers taking borrowed primitives/models (`&mut ViewportModel`,
`&mut CursorModel`, `&Document`, `&[ExclusionBlock]`, `&ScrollAmount`, selection
tuples); `ff-desktop` keeps the thin `render(&mut TabState, ...)` entry that
unpacks `TabState` and calls them. This is exactly what task 22 already proposes
("keeping tab_state-entangled glue in ff-desktop as the adapter"), and it is the
same seam Wave 5 used (`explorer_view` core moved; shell wiring stayed).

---

## 5. The `exclude_manager` coupling -- can it become a clean crate?

Current coupling (confirmed): `exclude_manager.rs` imports
`crate::tab_manager::TabManager` and `crate::tab_state::TabId`. Its command
methods (`exclude_all`, `exclude_text`, `exclude_text_all`, `show_all`,
`show_text`, `reset`) all take `&mut TabManager` and internally call
`tabs.active_tab_mut()`, then `snapshot_lines(tab, runtime)` to build a
`Vec<String>` via the Tokio runtime, keying a per-`TabId`
`ExclusionEngine<ContractionState, TabDocAdapter>`. The read side
(`exclusion_blocks(tab_id)`, `is_excluded(tab_id, ...)`) already takes only
`TabId`.

Public surface:
- `new()`
- `exclusion_blocks(&self, tab_id: TabId) -> Vec<ExclusionBlock>` (read)
- `is_excluded(&self, tab_id: TabId, line_1based: u64) -> bool` (read)
- `exclude_all / exclude_text / exclude_text_all / show_all / show_text / reset`
  -- all `(&mut self, ..., tabs: &mut TabManager, runtime: &Runtime) -> String`

The ONLY thing it needs from `TabManager`+`TabState` is: the active tab's `TabId`,
its `line_count`, and a `Vec<String>` line snapshot. So YES -- the surface can be
refactored to a clean crate.

Proposed seam (behaviour-preserving):
```
// ff-exclude-manager (clean crate: deps ff-exclude-show-filter + ff-display-line-mapping only)
impl ExcludeManager {
    pub fn exclude_all(&mut self, tab_id: TabId, line_count: usize,
                       lines: impl FnOnce() -> Vec<String>) -> String { ... }
    // ...same shape for exclude_text/show_all/.../reset...
}
```
The shell adapter in `ff-desktop` becomes the ONLY place that touches
`TabManager`:
```
// ff-desktop/src/exclude_manager.rs (thin adapter)
pub fn exclude_all(mgr: &mut ExcludeManager, tabs: &mut TabManager, rt: &Runtime) -> String {
    let tab = tabs.active_tab_mut();
    let id = tab.id; let n = tab.line_count as usize;
    mgr.exclude_all(id, n, || snapshot_lines(tab, rt))
}
```
`TabId` itself is a trivial `pub struct TabId(pub u64)`; to avoid dragging
`tab_state` into the crate, EITHER (a) define a crate-local `TabId(u64)` newtype
in `ff-exclude-manager` and `From`-convert at the adapter, OR (b) parameterise the
engine map key as `u64`. Option (b) is simplest and keeps `TabId` entirely in the
shell. The `snapshot_lines` closure keeps the Tokio/async doc read in `ff-desktop`
(the crate stays runtime-free, consistent with the "pure model crate" rule).

Net: `ff-exclude-manager` becomes a clean leaf (no `crate::` refs), the
`TabManager`/`TabId` entanglement collapses into a ~30-line adapter. This is
slightly MORE than the mechanical "move as-is" task 21 implies, but it is the only
way task 21 yields a genuinely clean leaf; the alternative is a crate that still
can't compile without the shell. Recommended to do the small seam refactor as part
of task 21 (still behaviour-preserving: the call sites' observable results are
identical).

---

## 6. Option comparison and recommendation

### Option A -- single `ff-editor-panel` (as specced, tasks 20-22)
- Pros: smallest, safest, behaviour-preserving; proven pattern (Waves 1-5);
  3 reviewable steps; removes ~770 non-test lines from the binary crate; no gate.
- Cons: `ff-editor-panel/src/lib.rs` would be ~500-600 non-test lines, over the
  400-line guideline (same known follow-up already accepted for
  `ff-explorer-view`). Mitigate by an intra-crate `_render.rs`/`_edit.rs`/
  `_geometry.rs` split (still one crate, one adapter, one gate).

### Option B -- aspect-sliced editor crate family
- Pros: in principle maps one crate per editor concern.
- Cons: FATAL for this codebase -- the aspect crates ALREADY EXIST
  (`ff-edit-operations`, `ff-undo-redo`, `ff-caret-selection`, `ff-line-commands`,
  `ff-syntax-highlighting`, `ff-auto-indent`, `ff-text-decorations`,
  `ff-whitespace-guides`, `ff-seqnum`, `ff-wrap`, `ff-select`, `ff-clipboard`,
  `ff-find-and-replace`). New aspect crates would DUPLICATE them. The editor's
  only non-crate-backed logic (inline insert/delete/undo/selection/clipboard)
  should be MIGRATED onto those existing crates, not re-crated under new names --
  and that migration CHANGES OBSERVABLE BEHAVIOUR (real transaction/undo
  semantics, real selection model), so it is a gated requirement stream, not a
  refactor wave. Doing B inside Wave 6 would conflate a safe move with a risky
  behaviour change.
- Ceremony-only crates to explicitly AVOID creating (each would just re-wrap an
  existing crate): `ff-editor-undo` (use `ff-undo-redo`), `ff-editor-caret` /
  `ff-editor-selection` (use `ff-caret-selection` + `ff-edit-operations`),
  `ff-editor-scroll` (use `ff-viewport-scrolling`), `ff-editor-exclude` (use
  `ff-exclude-show-filter`), `ff-editor-clipboard` (use `ff-clipboard`),
  `ff-editor-linecmd` (use `ff-line-commands` / `ff-command-semantics`).

### Option HYBRID (RECOMMENDED)
Do Option A now as a pure refactor (Wave 6 below). Separately open a GATED
requirement stream ("wire the editor onto the existing aspect crates") to retire
the inline insert/delete/undo/selection/clipboard code in favour of
`ff-edit-operations` + `ff-undo-redo` + `ff-caret-selection` + `ff-clipboard`.
Keep them separate: Wave 6 must stay behaviour-preserving and ungated; the
integration is observable behaviour and MUST run the requirements gate
(`workflow.md`). Do NOT create new aspect crates.

CANDIDATE new crates that do NOT already exist and would add value: NONE for the
editor's current feature set. The only genuinely new leaf worth creating is the
one the spec already names -- `ff-scroll-amount` (the `ScrollAmount` enum has zero
`crate::` refs; a clean leaf). Everything else the editor needs is covered by an
existing crate.

### Concrete revised Wave 6 task list (leaves-first, behaviour-preserving, thin
adapters, one crate per reviewable step)

- [ ] 20. Extract `scroll_amount` -> `ff-scroll-amount` (clean leaf; no deps).
      Thin `pub use ff_scroll_amount::*;` adapter in `ff-desktop`; add path dep;
      scoped gate (`-p ff-scroll-amount -p ff-desktop`). (Req 19.1-19.4)
- [ ] 21. Extract `exclude_manager` -> `ff-exclude-manager`, applying the section-5
      seam so it is a CLEAN leaf: command methods take `(tab_id-as-u64,
      line_count, lines-snapshot-closure)` instead of `&mut TabManager`; deps
      `ff-exclude-show-filter` + `ff-display-line-mapping` only. The `TabManager`/
      runtime glue (incl. `snapshot_lines`) stays in `ff-desktop` as a ~30-line
      adapter that preserves the existing call-site results exactly. Scoped gate
      (`-p ff-exclude-manager -p ff-desktop`). (Req 19.1-19.4)
      - If the seam refactor is judged out of scope for a pure move, FALL BACK to
        moving it as-is and recording "not a clean leaf (keeps a TabId newtype)"
        -- but prefer the clean seam.
- [ ] 22. Extract the editor render/scroll/geometry helpers -> `ff-editor-panel`
      (deps: egui/eframe, `ff-document-model`, `ff-viewport-scrolling`,
      `ff-exclude-show-filter`, `ff-command-semantics`, `ff-scroll-amount`). Move
      the PURE functions (`build_display_list`, `scroll_by_amount`,
      `extract_selected_text`, `normalise_selection`, `cursor_byte_position`,
      `line_char_count`, `DisplayRow`) and the paint/input body as free functions
      taking borrowed models + primitives. `ff-desktop` keeps the thin
      `render(&mut TabState, ...)` adapter (section 4c Option 1) that unpacks
      `TabState` and calls the crate. If the crate exceeds ~400 non-test lines,
      split intra-crate into `_geometry.rs` / `_render.rs` / `_input.rs` (one
      crate, one adapter, one gate -- record the >400 follow-up as done for
      `ff-explorer-view`). Scoped gate (`-p ff-editor-panel -p ff-desktop`).
      (Req 19.1-19.4)
- [ ] 23. (NEW, SEPARATE, GATED -- NOT part of this refactor wave) Open a
      requirements-gated stream to wire the editor onto the existing aspect crates
      (`ff-edit-operations` edit ops + `ff-undo-redo` transactions first, then
      `ff-caret-selection`/`ff-clipboard`), retiring the inline insert/delete/
      undo/selection/clipboard logic and the local `UndoEntry`. This CHANGES
      observable undo/selection behaviour -> run the gate per `workflow.md`; do
      NOT fold it into Wave 6.

---

## 7. Framework-conformance check (per .kiro/steering/framework-conformance.md)

- Single command dispatch (CommandTarget): UNCHANGED. The editor's only command
  touch-point is `cmd_engine.submit_line_command(...)` (prefix line commands),
  which already routes through `ff-command-semantics`. The extraction keeps that
  call in the moved code; it does NOT add any `if upper == "..."` intercept or a
  parallel dispatcher. CONFORMANT.
- Navigation (per-tab Navigation_Stack, `navigate_to`): UNCHANGED. The editor
  render does not navigate; `nav_stack` stays on `TabState` in `ff-desktop`. No
  second navigation stack is introduced. CONFORMANT.
- Focus (WorkspaceContext single `InteriorFocus` latch): UNCHANGED and must stay
  so. The Editor Context is the documented `InteriorFocus::none()` no-interior
  case (shell/render.rs FileEditor/Untitled arm already reports
  `apply_interior_focus(ctx, InteriorFocus::none())`). The extraction must keep
  that arm's `InteriorFocus::none()` reporting in `ff-desktop` -- the moved render
  helpers must NOT introduce their own focus ring or egui focus latch. CONFORMANT
  provided the thin adapter retains the existing no-interior reporting.
- Session persistence (WorkspaceDescriptor): UNCHANGED. `TabState` and its
  descriptor stay in `ff-desktop`. CONFORMANT.
- Kinds/title/chrome: UNCHANGED. CONFORMANT.

One explicit guardrail for the implementer: because the editor canvas is a
painted surface (not a focusable widget) with its OWN internal caret model, the
extracted `ff-editor-panel` must remain a pure render/input helper and must NOT
start owning egui keyboard focus or a Tab stop -- that would violate CR-CH-023 /
the single-latch rule. Keep focus reporting (`InteriorFocus::none()`) in the shell
arm.

---

## 8. Conclusions

1. `editor_panel.rs` is ~770 non-test lines = egui render + raw input + inline
   edit/undo/selection/clipboard; it delegates only to `ff-viewport-scrolling`,
   `ff-exclude-show-filter`, and `ff-command-semantics`.
2. Hypothesis CONFIRMED: aspect crates already exist and the monolith is mostly
   render + tab glue -- but the aspect crates are UNUSED by the live editor
   (under-integration, not under-decomposition).
3. `TabState`/`TabId`/`UndoEntry` are correctly shell-owned; cross the boundary
   via a thin `ff-desktop` adapter (keep `render(&mut TabState, ...)` as the
   entry, move pure helpers). Do NOT make a shared TabState crate.
4. `exclude_manager` CAN become a clean leaf by swapping `&mut TabManager` for a
   `(tab-id, line_count, snapshot-closure)` seam; the small refactor is worth it.
5. Choose HYBRID: Option A mechanical extraction now (tasks 20-22, revised
   above), plus a SEPARATE GATED stream to wire the editor onto the existing
   aspect crates. Create NO new aspect crates (`ff-scroll-amount` is the only new
   crate and it is a genuine new leaf).
6. The proposed shape builds ON the framework (command dispatch, navigation,
   focus latch, persistence all unchanged); the only conformance watch-item is
   keeping the Editor Context's `InteriorFocus::none()` reporting in the shell.
