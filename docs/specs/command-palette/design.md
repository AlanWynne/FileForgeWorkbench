# Design -- Command Palette

## Architectural Decisions

### 1. No new crate -- implemented in ff-desktop

The Command Palette is a pure UI concern. It reads from the existing `CommandRegistry`
(already accessible in `ff-desktop` via the shell state) and dispatches via the existing
`handle_command()` path. No new library crate is needed.

### 2. Modal overlay rendering

The palette is rendered as an `egui::Window` with `movable(false)`, `resizable(false)`,
centered on the screen. It is drawn on top of all other panels by being rendered last in
the `update()` call. A `palette_open: bool` flag on `WorkbenchShell` controls visibility.

### 3. Fuzzy matching algorithm

A simple subsequence scorer is sufficient:
- Check if query is a subsequence of the target (all query chars appear in order).
- Score = sum of: +10 per contiguous run, +5 per word-boundary match, -1 per gap.
- Implemented as a pure function in `palette_fuzzy.rs` with no external dependencies.

### 4. Command list source

`CommandRegistry::list_all()` returns `Vec<CommandMetadata>` sorted by display name.
The palette filters this list on every keystroke (the list is small -- typically < 200
commands -- so O(n) filtering per frame is acceptable).

### 5. Recent commands persistence

`recent_palette_commands: VecDeque<String>` (Command_IDs, max 10) is added to
`SessionState` in `ff-session` and round-trips through `session.toml`.

## Module Layout (ff-desktop)

```
ff-desktop/src/
  shell/
    mod.rs          -- add palette_open field
    render.rs       -- call render_command_palette() when palette_open
  command_palette/
    mod.rs          -- CommandPaletteState, open/close
    render.rs       -- render_command_palette()
    fuzzy.rs        -- fuzzy_match(), fuzzy_score()  [pure functions]
```
