# Source File Size and Structure Rules

## Maximum File Size

**No source file (`.rs`) may exceed 400 lines**, excluding the `#[cfg(test)]` test module.

If a file's non-test code exceeds 400 lines, it MUST be split before adding further
implementation. The split is a REFACTOR (no gate required) provided no observable
behaviour changes.

## Why 400 Lines

- Fits comfortably in one editor screen with context
- Forces single-responsibility at the file level
- Keeps AI-assisted editing reliable — pattern matching and targeted edits fail on
  large files with repeated structures
- Consistent with the 40-line function limit: ~10 well-sized functions + docs + imports

## Splitting Rules

When a file exceeds the limit, split by concern in this order of preference:

1. **Extract a `_state.rs`** — data structures, `Default` impls, pure data methods
2. **Extract a `_render.rs`** — all `egui` rendering functions
3. **Extract a `_commands.rs`** — command dispatch / handler logic
4. **Extract a `_dialogs.rs`** — modal dialog rendering and state
5. **Extract a `_tests.rs`** — test module (use `#[path]` or `mod tests;` with a
   separate file when the test module alone exceeds 200 lines)

The `mod.rs` (or the primary file) becomes a thin coordinator that re-exports public
items and calls into the sub-modules.

## Comment Separator Style

Section separator comments MUST use plain ASCII only:

```rust
// === Section Name ===================================================
```

Do NOT use Unicode box-drawing characters (`──`, `─`). They cause pattern-matching
failures in automated editing tools and display inconsistently across terminals.

Existing Unicode separators MUST be replaced with ASCII equivalents whenever a file
is touched for another reason.

## Module Structure for `ff-desktop`

The `ff-desktop` binary follows this module layout:

```
src/
  main.rs                  — entry point, startup wiring only
  shell/
    mod.rs                 — WorkbenchShell struct, new(), re-exports
    state.rs               — fields, FocusStop, helper types
    commands.rs            — handle_command(), is_shell_command()
    render.rs              — update(), render_* methods
    dialogs.rs             — render_dialogs() — all modal dialog dispatch
  files_panel/
    mod.rs                 — FilesPanelState, re-exports
    state.rs               — data model, add_dataset, load_entries
    render.rs              — render(), render_catalog_tree(), render_content_area()
    context.rs             — context menu types and item lists
  dataset_alloc_dialog.rs  — <= 400 lines (currently borderline — split tests if needed)
  catalog_manager_dialog.rs
  ... (other dialogs remain single-file while under limit)
```

## Enforcement

Before submitting any implementation task:

```
rg --count-matches '' crates/ff-desktop/src/**/*.rs | awk -F: '$2 > 400'
```

Any file over 400 non-test lines blocks the task until split.
