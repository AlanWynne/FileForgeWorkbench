# Shell-owned menu-bar dropdown focus walk fixes Down/Enter off-by-one

The menu-bar dropdown keyboard navigation was double-advancing: the first Down landed on item 1 (Aliases) instead of item 0 (Config), so Enter on the "first" item invoked the wrong option. The root cause was a focus traversal conflict, not command routing -- egui 0.33's own `MenuBar` navigation moves focus on the arrow keys, and the shell's manual walk stacked on top of it. The fix makes the shell the single owner of the dropdown focus walk: it consumes both arrow keys before egui's `MenuBar` renders, tracks a clamped selection index (`FocusState::menu_dropdown_index`), renders each dropdown item with a stable `egui::Id`, and forces the selected id into focus *after* the `MenuBar` runs so the shell's request is the frame's last. Enter/Space still fires the button's `clicked()`, so dispatch stays on the single front door (`dispatch_command_string`). The change also extracts the dropdown rendering and menu-peek/dynamic/SCRM helpers into a new `render_menu_bar.rs` to keep `render_chrome.rs` under the 400-line rule.

Watch for: the arrow-key consume in `render_menu_bar_into_ui` runs unconditionally every frame the bar renders, not just when a dropdown is open (possible) -- worth confirming it does not swallow arrow keys from a focused editor/list. Otherwise the fix is tightly scoped and directly validated.

**Verdict**: APPROVED

## High-level view

The diagnosis the commit records is a focus-traversal conflict, not command routing: `resolve_target`/`builtin_workspace_target_for` already classified Config -> config and Aliases -> aliases, and the bug was focus double-advancement. Activation stays on the single dispatch seam -- no bespoke dispatch branch is added; Enter flows through `clicked()` -> `dispatch_command_string`, identical to a mouse click, which is the no-intercept constraint the task names.

The focus walk is shell-owned and deterministic. Arrow keys are consumed before egui's `MenuBar` renders so egui never also navigates; the shell advances/retreats a clamped index, stashes the selected item's stable id, and the bar renderer forces it into focus after the `MenuBar` has run. The index resets on close, so the first Down after reopening lands on item 0. Items carry stable ids salted by top-level command plus index -- the stable-`egui::Id` the focus-latch path requires.

Five full-shell `egui_kittest` tests pin the behavior: first Down focuses item 0, a second Down advances, Up returns, Enter on item 0 opens ConfigPanel (the exact failing assertion from the bug report), and Down,Down,Enter opens AliasEditor (the off-by-one in the opposite direction). The `// Validates:` annotations map to the new EARS criteria 17.5a/b/c.

`requirements.md` gains EARS sub-criteria 17.5a/b/c plus an implementation note; the refactor keeps `render_chrome.rs` at 223 lines and the new `render_menu_bar.rs` at 295, both under 400 non-test lines. The one open question is whether consuming Down/Up unconditionally at the top-panel level starves arrow navigation elsewhere -- the broad suite passing (1060 + the new 5) is suggestive but not conclusive.

<details>
<summary>Issues (2)</summary>

1. **Unconditional arrow consume** (possible) -- `render_menu_bar_into_ui` consumes ArrowDown/ArrowUp every frame the bar renders, with no gate on whether a dropdown is open or focus is in the bar. Confirm a focused editor, list, or other arrow-key consumer in the central panel is not starved of arrow input (egui input-routing/focus priority may already make this safe; the passing suite is suggestive but not conclusive).
2. **Shared single-dropdown walk state** (confirmed, non-blocking) -- `menu_dropdown_index`/`menu_dropdown_focus_target` are single values shared across all top-level dropdowns. This is correct because egui opens at most one dropdown at a time; noted only so a future multi-open affordance would need to revisit it.

</details>

<details>
<summary>Details</summary>

### Root-cause diagnosis and the ownership flip

The commit message states the diagnosis cleanly: `dispatch_command_string("Config")` already opened ConfigPanel and the target resolver already classified the verbs correctly, so the defect was purely in focus traversal. egui 0.33 moves focus on the arrows inside its own `MenuBar::ui()` call, which runs before the per-dropdown content closure -- so an attempt to also move focus from inside the closure stacked on egui's move and produced the one-off-too-far landing (item 1 instead of item 0), and egui's own Up escaped the dropdown entirely.

The fix inverts ownership. `render_menu_bar_into_ui` consumes both arrow keys up front (so egui's `MenuBar` sees none), the dropdown render advances/retreats a clamped `menu_dropdown_index` and stashes the selected item id in `menu_dropdown_focus_target`, and after the `MenuBar` returns the bar renderer forces that id into focus. Because that forced request is the frame's last focus mutation, egui cannot drift it. The re-assert is gated on `focused() != target`, so a settled dropdown stops requesting focus and does not spin repaints -- a sensible guard against a repaint loop in the harness and in production.

### Dispatch stays on the single front door

The activation path is unchanged from a mouse click: a focused item's Enter/Space makes egui fire `clicked()`, and the handler calls `self.dispatch_command_string(&item.command)` then `ui.close()`. No `if upper == "..."` intercept, no second dispatcher, no navigation-stack shortcut. This is the behavior the task and framework-conformance.md demand, and it is why the two Enter tests can assert on the resulting active-tab `KindTag` (ConfigPanel / AliasEditor) rather than on an internal routing flag.

### Stable ids and the item ordering that pins the bug

Each item is rendered via `ui.push_id(("mb_dropdown_item", top_command, j), ...)` and its real `response.id` is captured, so the id both is stable across frames and is the id egui actually focuses -- the round-trip the workspace-conformance rule requires. The Settings dropdown order comes from `DEFAULT_SETTINGS_TOML`: Config (`C`) then Aliases (`A`), confirming item 0 = Config and item 1 = Aliases. That ordering is exactly what makes the two Enter tests meaningful: one Down must reach Config and only a second Down may reach Aliases.

### The unconditional arrow consume

```rust
let nav = MenuDropdownNav {
    down: ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)),
    up:   ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)),
};
```

This runs every frame the menu bar panel renders, before egui's `MenuBar`, with no gate on whether any dropdown is open or whether focus is in the bar. The captured `nav` is only acted on inside an open dropdown's content closure, so a swallowed-but-unused arrow when no dropdown is open is a silent consume. Whether that matters depends on egui's input routing: the top panel renders before the central panel, so in principle an editor or list in the central panel could be starved of Down/Up. The regression suite (1060 prior + 5 new) passing is suggestive that existing arrow-driven behavior still works, but it does not conclusively cover every arrow consumer. Gating the consume on "a dropdown is open this frame" (the state the renderer already tracks via `menu_dropdown_open_this_frame`, available from the previous frame) would remove the ambiguity.

</details>

<details>
<summary>Files changed</summary>

- `crates/ff-desktop/src/shell/render_menu_bar.rs` (new, 295 lines) -- dropdown item rendering, the shell-owned Down/Up focus walk, and the relocated `peek_menu_options` / `dynamic_menu_options` / `scrm_menu_bar_options` helpers.
- `crates/ff-desktop/src/shell/render_chrome.rs` (223 lines) -- consumes arrows before the `MenuBar`, forces the stashed focus target after it, resets the walk index on close; the moved helpers are deleted here.
- `crates/ff-desktop/src/shell/state_groups.rs` -- adds `menu_dropdown_focus_target`, `menu_dropdown_index`, `menu_dropdown_open_this_frame` to `FocusState`.
- `crates/ff-desktop/src/shell/mod.rs` -- registers the `render_menu_bar` module and the `tests_menu_bar_keynav` test module.
- `crates/ff-desktop/src/shell/tests_menu_bar_keynav.rs` (new) -- five full-shell `egui_kittest` tests with `// Validates:` annotations for Req 17.5a/b/c.
- `crates/ff-desktop/src/menu_workspace/defaults.rs` -- doc-comment path update (`render_chrome` -> `render_menu_bar`).
- `docs/specs/menu-workspace/requirements.md` -- EARS criteria 17.5a/b/c plus the B-fix implementation note.

Full diff: `git -C .worktrees/fix-menubar-keynav diff ed75bf90cca411331209186c6643b04cbbc1d00b...HEAD`

</details>
