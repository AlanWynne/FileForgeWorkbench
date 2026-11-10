# SCRM Phase 2B review-finding fixes: file split and clipboard-win BMP contract

The fix commit (`8949c83`) resolves the two pre-merge review findings on Stream SCRM Phase 2B. Finding 1 was a steering violation: `shell/update_dialogs.rs` had grown to 433 non-test lines, over the 400-line rule in `rust-standards.md`, after the SNAPSHOT format-picker arm was added. Finding 2 questioned whether the BMP bytes handed to the Windows clipboard writer matched the `CF_DIB` contract (the review suspected a 14-byte file header needed stripping). The coder split the picker overlay into a new module and, for the clipboard, investigated the actual clipboard-win 5.4.3 contract rather than applying the suspected fix. The task explicitly allowed either "strip the file header" or "keep the full BMP with a comment citing the crate's documented behaviour"; the coder took the second path because the crate's writer parses a full BMP file.

Watch for: the clipboard-win 5.4.3 source could not be read on this build host (the Windows-only dependency is not vendored in the local cargo registry), so the "set_bitmap expects a full BMP file" claim is verified against the crate's documented behaviour and the code/test, not against the crate source directly (likely). Everything else is confirmed from the diff and the worktree files.

**Verdict**: APPROVED

## High-level view

The picker split is a clean cut-and-move: the entire overlay block was lifted out of `update_dialogs.rs` into `shell/update_dialogs_snapshot.rs` as `WorkbenchShell::render_snapshot_picker_overlay`, and the call site in `render_overlays` now invokes it in the exact position the inline arm occupied. The moved code is byte-for-byte the same logic (bare SNAPSHOT opens the window, a selectable-label click dispatches `SNAPSHOT <verb>` through `dispatch_command_string`, Escape/Cancel closes). The only added behaviour is an early `return` when the picker is not shown, which is equivalent to the old `if self.show_snapshot_picker` guard. Resulting non-test line counts are 394 and 68, both independently confirmed under 400.

The clipboard finding resolves to a documentation-and-test fix with no byte-layout change. The coder determined that clipboard-win 5.4.3's `formats::Bitmap` setter (`set_bitmap`) consumes a complete BMP file: it reads the 14-byte `BITMAPFILEHEADER`, then the 40-byte `BITMAPINFOHEADER`, and uses `bfOffBits` to find the pixels before `CreateDIBitmap`, publishing a `CF_BITMAP` handle (Windows auto-synthesises `CF_DIB`/`CF_DIBV5` for rich targets). So the existing `raster_to_bmp` output, which is already a full BMP file, is exactly what the writer wants; stripping the file header would have broken it. The review's bare-DIB premise was wrong for this crate.

The deterministic test was strengthened and renamed to pin the full-BMP contract: it asserts `bfOffBits == 54` and that the `BITMAPINFOHEADER` size field (`40 = 0x28,0,0,0`) begins at offset 14, not offset 0. This is the proof the task asked for, in the full-BMP direction the task permitted.

The dependency pins (`clipboard-win = "=5.4.3"` in `ff-desktop/Cargo.toml`, `png = "=0.18.1"` in `ff-screen-model/Cargo.toml`) are untouched, as is the logical-model-first design and the single-dispatch routing.

<details>
<summary>Issues (1)</summary>

1. **clipboard-win contract not source-verified** — the "set_bitmap expects a full BMP file including the BITMAPFILEHEADER" claim rests on the crate's documented behaviour and the code/test, not a read of the 5.4.3 source (not vendored on this non-Windows host). It matches clipboard-win 5.x's known behaviour and the test pins the layout; the owner's manual Windows paste check is the final confirmation. Non-blocking.

</details>

<details>
<summary>Details</summary>

## Picker overlay extraction is a verbatim move

`update_dialogs.rs` previously held the SNAPSHOT format-picker inline inside `render_overlays`. The commit replaces that block with a single call, `self.render_snapshot_picker_overlay(ctx)`, positioned exactly where the arm ran (right before the catalog-strip comment that ends the method). The new `shell/update_dialogs_snapshot.rs` carries the identical body: the `egui::Window` with the title/body labels, the `SnapshotFormat::all()` loop building selectable labels, the Cancel button, and the post-window handling where a chosen verb sets `show_snapshot_picker = false` and calls `self.dispatch_command_string(&format!("SNAPSHOT {verb}"))`, while Cancel or Escape closes without a snapshot.

The one structural change is that the old `if self.show_snapshot_picker { ... }` wrapper became an early `if !self.show_snapshot_picker { return; }` at the top of the new method. These are behaviourally identical: no window renders and no dispatch happens unless the picker is showing. Command parity is preserved because the dispatch still goes through the single front door (`dispatch_command_string` -> `resolve_target`), so a picked format is indistinguishable from the typed `SNAPSHOT <verb>`. The module is registered in `shell/mod.rs` alphabetically next to `update_dialogs`.

Line counts verified in the worktree: `update_dialogs.rs` is 394 lines total with no `#[cfg(test)]` module (it ends at `build_palette_entries`), so 394 non-test; `update_dialogs_snapshot.rs` is 68 lines, also no test module. Both under 400.

## Clipboard write matches the actual clipboard-win 5.4.3 contract

`raster_to_bmp` builds a complete BMP file: `"BM"` signature, a 14-byte `BITMAPFILEHEADER` whose pixel offset (`bfOffBits`) is set to `FILE_HEADER + INFO_HEADER = 54`, a 40-byte `BITMAPINFOHEADER` (24-bit, bottom-up, BI_RGB), then padded BGR rows. The Windows `place()` path hands this whole buffer to `formats::Bitmap.write_clipboard(&bmp)` unchanged.

The coder's documented contract: clipboard-win 5.4.3's `set_bitmap` reads the file header and info header off the front of the buffer and uses `bfOffBits` to locate the pixels, returning `ERROR_INCORRECT_SIZE` if the buffer is shorter than file+info headers. There is no bare-`CF_DIB` setter in 5.4.3; it publishes `CF_BITMAP`. A header-stripped DIB (what the review suspected was needed) would be mis-parsed. This is captured in doc comments on `raster_to_bmp` and at the `place()` call site, so the write is not re-flagged as a layout bug. The task's acceptance criterion offered this exact alternative ("keep the full BMP ... OR document the full-BMP contract"), and this is the correct reading for this crate.

I could not read the clipboard-win 5.4.3 source on this host to independently confirm the byte-by-byte parse, since the Windows-only dependency is not vendored in the local cargo registry. The claim is consistent with the crate's known 5.x behaviour (it parses a BMP file and calls `CreateDIBitmap`), and the test pins the layout the code produces; the owner's manual Windows paste is the end-to-end check.

## The test proves the full-BMP layout deterministically

`raster_to_bmp_is_full_bmp_file_for_clipboard_win_setter` (renamed from `raster_to_bmp_is_well_formed_dib`) rasterises a sample model and asserts: the `"BM"` signature at offset 0; `bfOffBits == 54` read from offset 10; and the `BITMAPINFOHEADER` size field `[0x28,0,0,0]` at offset 14 (not offset 0), with a comment noting the bare-DIB layout the review suspected would have placed those bytes at offset 0. It also re-checks 24-bpp at offset 28 and positive (bottom-up) height at offset 22. This is deterministic (no clipboard, no OS), satisfying the task's requirement that a unit test assert the info-header start, in the full-BMP framing the task permitted.

## Pins and design untouched

`clipboard-win = "=5.4.3"` in `crates/ff-desktop/Cargo.toml` and `png = "=0.18.1"` in `crates/ff-screen-model/Cargo.toml` are unchanged; neither Cargo.toml appears in the fix commit's diff. The multi-flavor `SnapshotClipboard` trait, the `SnapshotPayload::from_model` logical-model-first construction, and the single-dispatch picker routing are all unchanged by the commit.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/update_dialogs.rs` — removed the inline SNAPSHOT picker block; replaced with a call to `render_snapshot_picker_overlay`. Now 394 non-test lines.
- `crates/ff-desktop/src/shell/update_dialogs_snapshot.rs` — new; holds `WorkbenchShell::render_snapshot_picker_overlay` (the verbatim picker logic). 68 lines.
- `crates/ff-desktop/src/shell/mod.rs` — registered the new module.
- `crates/ff-desktop/src/clipboard_backend.rs` — added doc comments citing the clipboard-win 5.4.3 full-BMP contract on `raster_to_bmp` and the Windows `place()` call; strengthened and renamed the BMP-layout test. No byte-layout change.

Full diff: `git -C .worktrees/scrm-2b-picture-clipboard show 8949c83`.

</details>
