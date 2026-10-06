# B082 Investigation -- Theme Rework Not Visible At Runtime

**Status:** READ-ONLY investigation complete. No source edited, nothing built.

## VERDICT

**Hypothesis A (STALE BINARY) -- CONFIRMED. Hypothesis B (wiring gap) -- REFUTED.**

The CR-CH-056 theme rework is correctly wired into the source for all three
seams the brief asked about (apply seam, default theme, Theme Editor render arm).
The reason the running app looks unchanged and shows the old Theme Editor is that
**the user is running a pre-rework `ffwb.exe`** that was built before the rework
code existed. A plain `cargo build -p ff-desktop` fixes it with **no code change**.

This is a direct repeat of **B007** (docs/status/bugs.md: "stale debug binary ...
Fixed by `cargo build`. No code change required.") and of the stale-binary note
already recorded against **B039**.

## Decisive evidence: binary predates the code

Binary mtimes (verified via `Get-Item`):

| Binary | LastWriteTime |
|--------|---------------|
| `target/debug/ffwb.exe`   | **2026-10-02 17:09** |
| `target/release/ffwb.exe` | **2026-09-07 14:00** |

Git state of the theme rework (`git log` + `git status --short`):

- The last COMMITTED theme change is `be1ec7e` -- "CR-CH-056 **Phase 2** (Task 30)
  -- Solarized Default Dark/Light ...". There is no committed Phase 3/4/5.
- Phases 3-5 are present only as **uncommitted working-tree changes**. `git status`
  shows modified `crates/ff-theme/src/{lib,loader,palette,serialiser,discovery,
  contrast}.rs`, modified `crates/ff-theme-editor/src/lib.rs`, modified
  `crates/ff-desktop/src/theme_editor_panel.rs`, modified
  `crates/ff-desktop/src/shell/render_theme.rs` + `render.rs` + `update.rs`-area
  files, and NEW untracked files `crates/ff-theme-editor/src/editable_surface.rs`,
  `crates/ff-theme/src/{format_version,loader_parse}.rs`.

Since the rework (even Phase 2's Solarized delegation, plus all of Phase 3-5)
lives in commits/working-tree dated at or after 2026-10-02 and the Phase-3/4/5
work is still uncommitted, the 2026-10-02 debug binary and the 2026-09-07 release
binary **cannot contain the rework**. Whichever one the user launched will show
the old Catppuccin chrome and the old Theme Editor. The owner's full
`ffwb-gate.ps1` run compiles and TESTS the workspace (9613 tests) but does not
leave a refreshed dev `ffwb.exe` in `target/debug` -- so a CLEAN gate and a stale
runtime binary coexist exactly as the brief anticipated.

## Seam checks (Hypothesis B) -- all PASS in current source

### 1. Apply seam -- PASS
`crates/ff-desktop/src/shell/render_theme.rs:26-30`, `apply_theme`:
```rust
pub(super) fn apply_theme(&self, ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    self.palette.chrome_style.apply_to_egui(&mut style);
    ctx.set_style(style);
}
```
This is the wholesale `apply_to_egui(&mut Style)` + `ctx.set_style(...)` the
CR-CH-056 design (Req 23.5) calls for -- it installs the chrome Style into the
egui Context, not just an internal palette. It is CALLED every frame at
`crates/ff-desktop/src/shell/update.rs:253` (`self.apply_theme(ctx);`), i.e. at
and after the first frame, and after any palette swap (`set_theme` /
`set_active_theme` mutate `self.palette`, which the next frame's `apply_theme`
pushes). No gap here.

### 2. Default theme -- PASS
`crates/ff-theme/src/defaults.rs:37,46`: `dark_palette()` /`light_palette()` now
delegate to `crate::defaults_solarized::solarized_dark_palette()` /
`solarized_light_palette()`; `default_palette_for_mode` (line 113) returns them.
The former Catppuccin builders were removed (comment at lines 154-158).
So a fresh default install resolves to Solarized Dark/Light -- a correctly built
binary WOULD look different from the old Catppuccin, which is exactly what the
user is NOT seeing (because the binary is stale).

### 3. Theme Editor render arm -- PASS
`crates/ff-desktop/src/theme_editor_panel.rs` is a thin adapter that re-exports
the rebuilt crate: `pub use ff_theme_editor::{render, EditableToken,
ThemeEditorAction, ThemeEditorState};` and holds only the `WorkspaceContext` impl
(dispatched via the single focus-latch path, per workspace-conformance). The
central-panel arm renders through that `WorkspaceContext` dispatch, not an inline
old editor. The crate it points at IS the Phase-3 rebuild:
`crates/ff-theme-editor/src/lib.rs` header documents "CR-CH-056 (Phase 3): the
editable surface is now DERIVED from the egui `Style`/`Visuals` chrome fields ...
and B081 is fixed", declares `mod editable_surface;`, and the NEW
`crates/ff-theme-editor/src/editable_surface.rs` builds the derived surface. No
older editor implementation remains in the live path.

## Recommended fix

**Rebuild the dev binary -- no code change, no requirements gate.** This is a
BUG fix of the stale-binary class (same as B007), not new work:

```
cargo build -p ff-desktop
```

Then launch the freshly built `target/debug/ffwb.exe` (or `cargo run -p
ff-desktop`). The app will come up in Solarized Dark/Light and the Theme Editor
Context will be the rebuilt derived-surface editor with the B081 Save fix.

Two operational notes for the owner:
1. The Phase 3-5 rework is still **uncommitted** (working tree). The passing gate
   validated the working tree, but the changes should be committed so the state
   that tests CLEAN is the state that is preserved.
2. If the user was launching the **release** binary (2026-09-07), build release
   too: `cargo build --release -p ff-desktop`. Confirm which binary the user runs.

## Falls-under / classification

- B082 is a stale-binary runtime bug (Medium), resolved by rebuild. It maps to no
  code defect under any CR-CH-056 criterion -- Req 23.5 (apply seam), the
  Solarized default (Req 23.7/23.8), and Req 20 (Theme Editor) are all satisfied
  in source. Recommend B082 be marked VERIFIED/FIXED-by-rebuild once the user
  confirms a freshly built binary shows Solarized + the new editor, mirroring
  B007's resolution.
