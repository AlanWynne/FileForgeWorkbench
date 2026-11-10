# Stream SCRM Phase 2B -- picture snapshot, format picker, multi-flavor clipboard

Phase 2B adds the picture/binary SNAPSHOT formats (PNG and PDF), a no-argument format picker, and a true multi-flavor clipboard that places a PNG picture and the text rendering on the OS clipboard together. The work is spread across three crates on the right seams: `ff-screen-model` gains a dependency-free text-to-bitmap rasteriser plus a PNG encoder; `ff-scrm` gains a thin `export_screen_pdf` wrapper that reuses the existing selectable-text Collection PDF path; and `ff-desktop` gains a `SnapshotClipboard` abstraction, a split-out `commands_snapshot.rs`, and a picker overlay that dispatches the chosen format back through the single command front door. Both the PNG and the clipboard text/picture derive from the `ScreenModel` text render -- there is no egui pixel grab anywhere in the path. New dependencies (`png`, `clipboard-win`) are pinned to exact versions.

Watch for: `update_dialogs.rs` is pushed to ~433 non-test lines by the picker arm, over the 400-line steering limit (confirmed). The Windows `CF_DIB`/`Bitmap` clipboard write passes a full BMP file including the 14-byte `BITMAPFILEHEADER`, which CF_DIB does not expect -- a possible Windows-runtime correctness issue that the deterministic tests cannot catch (possible). Neither blocks the behavioral goals of the phase.

**Verdict**: APPROVED

## High-level view

The logical-model-first constraint is honoured cleanly. `rasterise_screen` calls the crate's own `render(model, PlainText, {unicode_box:false})` and draws each character with a fixed 5x7 bitmap font at a fixed cell size, so the PNG is a deterministic raster of the exact text the clipboard also carries -- never a GUI capture. The clipboard payload carries the raster, its PNG bytes, and the text, all built from one `ScreenModel`.

The picker routes through the single front door. Bare `SNAPSHOT` sets a flag; the overlay lists every format's verb and, on selection, calls `dispatch_command_string("SNAPSHOT <verb>")` -- the same prelude -> `resolve_target` -> ladder path a typed command takes. There is no bespoke intercept and no second dispatcher. A picked format is byte-identical to the typed form, and existing explicit verbs (TEXT/ANSI/MARKDOWN/MD/HTML/YAML) are untouched; PNG/PDF are added as new args.

The multi-flavor clipboard is a genuine capability addition behind a small trait. The C.4a finding is recorded accurately: `arboard`'s `set()` is single-flavor per transaction, so the former `set_text`-only path could not offer image and text together. The Windows `OsClipboard` opens the clipboard once via `clipboard-win` and writes `CF_UNICODETEXT` + the bitmap in one transaction; non-Windows falls back to `arboard` behind the same trait. The "offers both flavors" behavior is tested against a fake backend, never a real OS clipboard.

The PDF path is a true reuse. `export_screen_pdf` wraps the single screen in a one-capture `ScreenCollection` and calls the existing `export_pdf`, so the snapshot PDF is the same hand-written Base-14 Courier selectable-text document as a Collection export, with no new PDF infrastructure.

Dependency pinning and TDD are in order. `png = "=0.18.1"` and `clipboard-win = "=5.4.3"` are exact pins; the clipboard-win pin is correctly scoped to `target.'cfg(windows)'`. Tests carry `// Validates:` annotations, are deterministic, and the prior bare-SNAPSHOT assertion was rewritten to the picker contract rather than weakened. File-size discipline holds everywhere except `update_dialogs.rs`.

<details>
<summary>Issues (2)</summary>

1. **update_dialogs.rs over 400 lines** — the picker arm pushes the file to ~433 non-test lines, over the steering limit; the new overlay arm (and ideally the existing dialog cluster) should be split into a `_snapshot` / per-dialog helper. Non-blocking but a documented limit violation.
2. **Windows CF_DIB header (possible)** — `OsClipboard::place` on Windows hands `formats::Bitmap.write_clipboard` a full BMP file including the 14-byte `BITMAPFILEHEADER`; CF_DIB expects the DIB starting at the `BITMAPINFOHEADER`. Verify on a real Windows clipboard that rich targets (Word/PowerPoint) accept the paste; if not, strip the file header for the DIB write.

</details>

<details>
<summary>Details</summary>

### Logical-model-first: PNG rasterises the text render, not egui

`rasterise_screen` (crates/ff-screen-model/src/raster/mod.rs) renders the model to plain text with the ASCII-box fallback, then draws each glyph from a column-major 5x7 bitmap font (`raster/font.rs`) at a fixed `CELL_W`/`CELL_H`. Output size is a pure function of the text (`raster_height_tracks_text_row_count` asserts height tracks the rendered row count), and two renders of the same model produce identical PNG bytes (`raster_is_deterministic`). The crate has no egui dependency at all, so a pixel grab is structurally impossible here. `RasterImage` deliberately keeps raw RGBA so the clipboard layer can build a DIB without re-decoding the PNG.

The font module notes its glyph bitmaps are public-domain factual data; the fallback-glyph path (`non_ascii_maps_to_fallback`) guarantees `glyph` is total for any `char`, and `render` maps the `Png`/`Pdf` variants to `text::render_plain` so the renderer stays total while the binary formats are produced as bytes elsewhere.

### Picker dispatches through the one front door

`handle_snapshot` (commands_snapshot.rs) treats a blank arg as "open picker" (sets `show_snapshot_picker`, clears `open_error`, returns) rather than defaulting to a format. The overlay in `update_dialogs.rs` iterates `SnapshotFormat::all()`, shows each `verb()`, and on a click calls `self.dispatch_command_string(format!("SNAPSHOT {verb}"))`. `dispatch_command_string` (dispatch.rs) is the documented single shell-side entry: prelude -> `ff_command::resolve_target` -> ladder. So the picker produces the identical dispatch to a typed `SNAPSHOT PNG`, which `snapshot_pdf_writes_single_screen_pdf_via_command_path` exercises directly. `SnapshotFormat::all()` and `verb()`/`parse_arg` round-trip is asserted (`all_formats_verbs_round_trip`).

The picker is a plain modal overlay, not a Workspace Context; the code comments this and defers full dialog capture to Phase 2E. Given it is not a Context, the master-screen-directive focus-latch/ScreenProvider obligations do not attach to it in 2B -- a reasonable scoping call, explicitly recorded.

### Multi-flavor clipboard behind a trait

`SnapshotPayload::from_model` builds the raster, its PNG, and carries the caller-supplied text, so both flavors come from one model (Req 23.4). `SnapshotClipboard::place` returns `OfferedFlavors { image, text }` so a test can assert both were offered without an OS clipboard; `FakeClipboard` captures the payload and reports flavors from payload contents. `payload_from_model_has_both_flavors`, `place_offers_both_image_and_text_flavors`, and `raster_to_bmp_is_well_formed_dib` cover the seam. For the PNG format the text flavor is the plain-text render so a plain-text target still has something to paste; for a text format the requested rendering is the text flavor -- both documented.

The Windows implementation opens once with `Clipboard::new_attempts(10)` (which empties the clipboard) and writes Unicode text then the bitmap in the same open, which is the correct shape for a simultaneous offer. The one caveat is the DIB header (see Issues): `raster_to_bmp` emits a complete BMP file (`"BM"` + `BITMAPFILEHEADER` + `BITMAPINFOHEADER` + bottom-up BGR rows), and that whole buffer is passed to `formats::Bitmap`. Whether `clipboard-win`'s `Bitmap` setter expects a full BMP (and strips the file header) or a bare DIB is version-specific; the deterministic BMP test only checks the file is well-formed, not that Windows accepts it as CF_DIB. Worth a manual paste check on Windows before relying on the picture flavor in rich targets.

### PDF reuse is thin and selectable

`export_screen_pdf` (ff-scrm/src/pdf.rs) wraps the screen in a one-capture `ScreenCollection` and calls the existing `export_pdf`, threading a timestamp for determinism. `single_screen_pdf_reuses_collection_path_with_selectable_text` confirms the output starts with `%PDF-1.7`, contains ` Tj` text operators and `/BaseFont /Courier` (selectable text, not an image), and has the same title+index+page layout as a Collection export; `single_screen_pdf_is_deterministic` pins reproducibility. No new PDF code path or dependency.

### Dependency pinning and file-size discipline

Both new deps are exact-pinned: `png = "=0.18.1"` (ff-screen-model) and `clipboard-win = "=5.4.3"` (ff-desktop, correctly under `target.'cfg(windows)'.dependencies`), with rationale comments citing Req 23.7. `Cargo.lock` reflects the clipboard-win bump. New files sit well under 400 non-test lines (clipboard_backend.rs 195, commands_snapshot.rs 171, raster/mod.rs ~130, font.rs ~130 of mostly const data, pdf.rs 225). `commands_scrm.rs` shrank because the SNAPSHOT family moved to `commands_snapshot.rs` (pure movement). The one exception is `update_dialogs.rs`, pushed to ~433 lines by the 46-line picker arm (Issues #1). Functions are small; `raster_to_bmp` at ~45 lines slightly exceeds the ~40 guideline but is a single linear buffer builder.

</details>

<details>
<summary>File map</summary>

- `crates/ff-screen-model/src/lib.rs` -- `SnapshotFormat` gains `Png`/`Pdf` + `is_binary`/`all`/`verb`; re-exports raster API.
- `crates/ff-screen-model/src/raster/mod.rs` -- new text-to-RGBA rasteriser + PNG encoder.
- `crates/ff-screen-model/src/raster/font.rs` -- new dependency-free 5x7 ASCII bitmap font.
- `crates/ff-screen-model/src/render.rs` -- maps `Png`/`Pdf` to the plain-text basis to stay total.
- `crates/ff-screen-model/Cargo.toml` -- adds `png = "=0.18.1"`.
- `crates/ff-scrm/src/pdf.rs` -- new `export_screen_pdf` thin wrapper + 2 tests.
- `crates/ff-desktop/src/clipboard_backend.rs` -- new `SnapshotClipboard` trait, `SnapshotPayload`, `OsClipboard` (Windows clipboard-win / non-Windows arboard), `raster_to_bmp`, tests.
- `crates/ff-desktop/src/shell/commands_snapshot.rs` -- new SNAPSHOT command family (picker branch, clipboard path, PDF path).
- `crates/ff-desktop/src/shell/commands_scrm.rs` -- SNAPSHOT methods removed (moved out).
- `crates/ff-desktop/src/shell/update_dialogs.rs` -- picker overlay arm (+46 lines; now over 400).
- `crates/ff-desktop/src/shell/state.rs` / `construct.rs` -- `show_snapshot_picker` field + init.
- `crates/ff-desktop/src/shell/helpers.rs` -- `snapshot_format_label` gains PNG/PDF.
- `crates/ff-desktop/src/shell/mod.rs` / `main.rs` -- module wiring.
- `crates/ff-desktop/src/shell/tests_command.rs` -- prior bare-SNAPSHOT test rewritten to picker contract.
- `crates/ff-desktop/src/shell/tests_snapshot_picture.rs` -- new shell-wiring tests (picker open, parity, PDF via command, full-shell dialog).
- `crates/ff-desktop/Cargo.toml` -- adds `clipboard-win = "=5.4.3"` under cfg(windows).
- `crates/ff-i18n/i18n/{en,af,de,fr}/*.ftl` -- picker title/body + multi-flavor/PDF status keys.
- `Cargo.lock` -- clipboard-win 5.4.1 -> 5.4.3, png added.

Full diff: `git -C .worktrees/scrm-2b-picture-clipboard diff main...HEAD`

</details>
