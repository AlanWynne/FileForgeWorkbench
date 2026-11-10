# Common Workspace Frame laid out top to bottom inside one box (fix 87346de)

The fix replaces the legacy slot order, which emitted content first and the chrome after it with the Command line and status bar outside the box, with a dedicated framed renderer (`ff-screen-model/src/render/framed.rs`). It is selected whenever a model carries `header_lines`. The shell helper `apply_common_frame` now also fills the `command_line` slot for non-menu Contexts through a new `screen_snapshot::command_field_value`. A new test file asserts order, containment, single chrome and snapshot/capture equality for a markdown_sections Context (Config) and a table Context (Alias). A previously wrong alias second-tab test was corrected to use START.

Watch for: Config content may be duplicated inside the box (possible, not verified, see Issues).

**Verdict**: APPROVED

## High-level view

The renderer builds one row list in the required order: header lines (window title bar, menu bar, tab bar), the centered workspace title, a rule, the Command line, then body, fields, table grids and buttons, then messages, status bar and key line. It draws a single border around all of it. The title appears once, as a centered row, and no longer as a box caption.

The shell side only fills the existing model slots. The public `ScreenModel` and `Table` types are untouched, `model.rs` is not in the diff, and no `environment.rs` or clipboard/PNG file appears in the changed-file list. The legacy layout is kept for models without `header_lines`, so non-framed callers are unchanged.

The new tests assert absolute positions and containment, not presence. They would fail on the broken layout, and the coder recorded 4 of 5 new order tests red before the fix and green after. Snapshot equals capture byte-for-byte is covered for both Contexts.

<details>
<summary>Issues (2)</summary>

1. **Possible content duplication in Config (possible)**: the framed renderer emits `body_lines`, then `fields`, then `tables`. The user's original Config output showed every row once as box lines and again as markdown tables. If Config populates both body and tables, the framed box may repeat the content. The tests take the first occurrence of the content marker only, so they would not catch this. Check the rendered Config output once, and consider asserting a Config row appears once.
2. **Test comment/count nit (confirmed, non-blocking)**: the commit says 4 of 5 order tests failed on the old layout, which leaves one that passed on the old layout. The recorded detail does not say which one. It is most likely the snapshot==capture test, which holds on any layout, so it is harmless.

</details>

<details>
<summary>Details</summary>

### Would the order and containment assertions fail on the broken layout

Reasoned against the broken layout (content first; title bar, menu, tab bar and key line after it; Command line and status bar below the closing border; title as box caption):

- `command < content` fails, because content came first.
- `title_bar == 1` fails, because the first row under the top border was content or the caption.
- The "every non-blank line starts with a frame glyph" loop fails on the `Command ===>` and status lines that sat outside the box.
- `bottoms[0] == last_non_blank` fails for the same reason.
- `ws_title` lookup uses `inner(l) == title` on a body row, so a title that is only a caption in the top border yields zero hits, and `only_index` panics.
- The centered-title check (`left > 1`, balanced padding) fails on a caption.

These are real positional assertions. The coder's commit records "new order tests FAILED on the old layout (4 of 5 red) then passed". Only the commit message was read, and the suites were not re-run, as instructed.

### Rendered Config and Alias, read against the required layout

Read from `framed::collect_rows`, since the output was not re-generated. The order is title bar, menu bar, tab bar, centered title, rule, `Command ===> <typed>____`, content, blank, messages (only if present), status bar, key lines (only if pfshow on, supplied through `footer_lines`). Each appears once. The bottom border closes the box and nothing follows it. Alias tables render as box grids inside the frame, with the grid glyph asserted. Content lines are left aligned, and the box width is the maximum row width with a floor of 20.

### Constraints

- No public reshape: `Glyphs` and helper functions are widened to `pub(super)` only. `model.rs` is absent from the diff.
- `environment.rs` is absent from the diff (the name-only diff for `*environment.rs` was empty).
- Clipboard and PNG flavor: no files for them are in the changed list.
- File sizes: `text.rs` has 312 non-test lines, `framed.rs` is 168 lines in total, and `common_frame.rs` is small. All are under 400 non-test lines. The test files are test-only.
- The fix commit message contains no semicolon. Other commit messages in the range were not part of this check.
- Docs: new comments are ASCII. No new docs were added in this fix commit.

### Test hygiene

`tests_common_frame_snapshot.rs` previously assumed CONFIG then ALIASES makes two tabs. That contradicts CR-CH-022, which transforms the current tab in place. The change uses START and has a justification comment, and it does not weaken the assertions. Gate-policy is respected: scoped trio and scoped tests ran, and the full gate was left to the owner.

</details>

<details>
<summary>File map</summary>

- `crates/ff-screen-model/src/render/framed.rs`: new one-box renderer, with unit tests.
- `crates/ff-screen-model/src/render/text.rs`: delegates to the framed renderer when `header_lines` is non-empty, and widens helpers to `pub(super)`.
- `crates/ff-screen-model/src/render.rs`: registers the module.
- `crates/ff-desktop/src/screen_snapshot.rs`: `command_field_value` helper.
- `crates/ff-desktop/src/shell/common_frame.rs`: fills the Command line slot for non-menu Contexts.
- `crates/ff-desktop/src/shell/tests_common_frame_order.rs`: new order, containment and equality tests.
- `crates/ff-desktop/src/shell/tests_common_frame_snapshot.rs`: START-based second-tab fix.
- `crates/ff-desktop/src/shell/mod.rs`: registers the test module.

Full diff: `git diff b72733c..87346de` in the worktree.

</details>
