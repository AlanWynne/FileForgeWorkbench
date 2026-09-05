# Tasks -- Command Palette

## Task 1. Fuzzy match engine (Req 2)

- [x] 1.1 Create `ff-desktop/src/command_palette/fuzzy.rs` with `fuzzy_match(query, target) -> bool`
  and `fuzzy_score(query, target) -> i32` pure functions
  - Satisfies: Req 2.1, 2.2
- [x] 1.2 Write unit tests: subsequence match, contiguous run scoring, word-boundary bonus,
  case-insensitive, no match returns false
  - Satisfies: Req 2.1, 2.2, 2.5

## Task 2. Palette state and data model (Req 1, 4, 5)

- [x] 2.1 Create `CommandPaletteState` struct: `query: String`, `filtered: Vec<PaletteEntry>`,
  `selected_index: usize`, `open: bool`
  - Satisfies: Req 1.1, 4.3
- [x] 2.2 Add `palette_open: bool` and `palette_state: CommandPaletteState` to `WorkbenchShell`
  - Satisfies: Req 1.1
- [x] 2.3 Add `recent_palette_commands: Vec<String>` (max 10) to `SessionState`; persist in
  `session.toml`
  - Satisfies: Req 5.1, 5.2
- [x] 2.4 Write unit tests: recent commands round-trip through session
  - Satisfies: Req 5.2

## Task 3. Palette rendering (Req 1, 2, 3)

- [x] 3.1 Create `ff-desktop/src/command_palette/render.rs` with `render_command_palette()` --
  centered `egui::Window`, search input, scrollable entry list (max 20 visible)
  - Satisfies: Req 1.1, 3.4
- [x] 3.2 Render each `PaletteEntry` with display name, category label, shortcut (right-aligned);
  highlight matched characters in display name
  - Satisfies: Req 3.1, 3.3
- [x] 3.3 Render detail description area below list for highlighted entry
  - Satisfies: Req 3.2
- [x] 3.4 Render "Recently Used" section header when query is empty
  - Satisfies: Req 5.1
- [x] 3.5 Render "No commands match '<query>'" when filtered list is empty
  - Satisfies: Req 2.6
- [x] 3.6 Render disabled entries with visual disabled style
  - Satisfies: Req 4.5

## Task 4. Palette activation and keyboard handling (Req 1, 4)

- [x] 4.1 Wire Ctrl+Shift+P in `shell/update.rs` to set `palette_open = true` and focus search field
  - Satisfies: Req 1.1
- [x] 4.2 Wire Escape to close palette and restore previous focus
  - Satisfies: Req 1.2
- [x] 4.3 Wire click-outside detection to close palette
  - Satisfies: Req 1.3
- [x] 4.4 Add `View > Command Palette` menu item
  - Satisfies: Req 1.4
- [x] 4.5 Wire Ctrl+Shift+P toggle (close if already open)
  - Satisfies: Req 1.5
- [x] 4.6 Wire Up/Down arrow navigation and Enter execution in palette
  - Satisfies: Req 4.3
- [x] 4.7 On Enter: close palette, dispatch command via `handle_command()`, add to recent list
  - Satisfies: Req 4.1, 4.4
- [x] 4.8 Write unit tests: fuzzy filter reduces list, disabled entry not executed, recent list updated
  - Satisfies: Req 2.1, 4.5, 4.4
