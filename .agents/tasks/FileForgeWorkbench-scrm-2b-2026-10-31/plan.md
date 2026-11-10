# Implementation Plan -- SCRM Phase 2B THREE-FLAVOR Clipboard SNAPSHOT

Rework the bare `SNAPSHOT` clipboard payload from a single text-rasterised PNG
into a THREE-FLAVOR Windows clipboard transaction:

1. **Picture** (`CF_DIB` + `CF_DIBV5`) built from a REAL screenshot of the
   FileForge application WINDOW (egui/eframe window pixels), NOT a raster of the
   text render.
2. **Text** (`CF_UNICODETEXT`) = Markdown-fenced render using proper Unicode
   box-drawing glyphs (U+2500 family), NOT ASCII `+/-/|`.
3. **Composite HTML** (`CF_HTML`) embedding the window screenshot as a base64
   `data:image/png;base64,...` `<img>` AND the box-drawn text in a monospace
   `<pre>`, so Word shows BOTH.

All three are placed in ONE clipboard-open transaction. ANSI stays off the
clipboard. Explicit `SNAPSHOT <FORMAT>` file outputs are unchanged. No format
picker. `ff-screen-model` stays egui-free; the live screenshot acquisition lives
only in `ff-desktop` (shell).

## WORKTREE (authoritative; work here, do NOT create a new one)
`c:/workspace/apps/FileForgeWorkbench/.worktrees/scrm-2b-picture-clipboard`
BRANCH `scrm-2b`. No merge, no push. Relative paths resolve against the PARENT
workspace, so every path below is written worktree-relative and MUST be joined
onto the worktree root.

> IMPORTANT reconciliation note for the implementer: the editor tools' repo-wide
> search (`grep_search`, `file_search`) and the workspace root
> `c:/workspace/apps/FileForgeWorkbench` are a DIFFERENT checkout (a branch
> WITHOUT the Phase 2B code). The Phase 2B SNAPSHOT code
> (`crates/ff-desktop/src/clipboard_backend.rs`,
> `crates/ff-screen-model/src/raster/`) exists ONLY in the worktree. Read and
> edit files via the absolute worktree path. Do NOT trust a repo-wide grep that
> returns "no matches" -- confirm by reading the worktree file directly.

---

## CRITICAL FACTS (verified against the worktree)

- **Pinned GUI:** `egui = "0.33"` (resolved **0.33.3**) and `eframe = "0.33"`
  (features `default_fonts`, default features ON => **glow** renderer), from the
  workspace `Cargo.toml [workspace.dependencies]`. `egui_kittest = "0.33"`
  (feature `eframe`) is the dev harness. The app runs the **glow** renderer
  (`main.rs` NativeOptions uses `..Default::default()`, no `Renderer::Wgpu`).
- **egui 0.33 window-screenshot API (authoritative, from the egui 0.33.0
  example):**
  - Request: `ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()))`.
    `UserData` can tag the request so the event can be matched to it.
  - Receive (ASYNC -- arrives on a LATER frame): read raw input events
    `ctx.input(|i| for event in &i.raw.events { if let egui::Event::Screenshot { image, user_data, .. } = event { ... } })`.
    `image` is `std::sync::Arc<egui::ColorImage>` of the WINDOW's rendered
    pixels. `image.as_raw()` yields RGBA8 bytes (row-major, top-left origin);
    `image.width()` / `image.height()` give pixel dimensions;
    `image.region(&rect, Some(pixels_per_point))` crops.
  - Native screenshot is supported on BOTH glow and wgpu since the egui 0.24
    viewport refactor (eframe CHANGELOG: the 0.30 entry only ADDED web support).
    So the app's default **glow** renderer CAN screenshot its own window --
    this work is NOT blocked.
- **How SNAPSHOT is dispatched today:** the single front door reaches the
  fallback ladder. `SNAPSHOT` is a `FALLBACK_LADDER_ROOTS` entry in
  `crates/ff-desktop/src/command_config/mod.rs`. `commands_ladder_b.rs` matches
  `"SNAPSHOT"` / `"SNAPSHOT "` and calls `self.handle_snapshot(arg)`
  (`shell/commands_snapshot.rs`). Bare arg -> `snapshot_composite_to_clipboard()`
  (the method to rework); explicit `<FORMAT>` -> file / PDF paths (unchanged).
- **Existing Phase 2B DIB byte-layout + PNG encoding (reuse verbatim):**
  - PNG: `ff_screen_model::encode_png(&RasterImage) -> Vec<u8>` using
    `png = "=0.18.1"` (RGBA8, 8-bit). `RasterImage { width:u32, height:u32,
    rgba:Vec<u8> }` is row-major top-left RGBA8. (`crates/ff-screen-model/src/raster/mod.rs`.)
  - DIB: `clipboard_backend::raster_to_bmp(&RasterImage)` builds a full 24-bit
    bottom-up BGR BMP file (14-byte `BITMAPFILEHEADER` + 40-byte
    `BITMAPINFOHEADER` + padded pixels). `raster_to_dib` strips the 14-byte file
    header (CF_DIB = 8, begins at `BITMAPINFOHEADER`). `raster_to_dib_v5`
    rebuilds a 124-byte `BITMAPV5HEADER` + the same BGR pixels (CF_DIBV5 = 17).
    These take a `RasterImage` and are INDEPENDENT of how the RGBA was produced,
    so they work unchanged on the window-screenshot RGBA.
  - Transaction: `OsClipboard::place` opens the Windows clipboard once via
    `clipboard_win::Clipboard::new_attempts(10)` (empties on open), then
    `raw::set_without_clear` for each format: `CF_UNICODETEXT`(13), `CF_DIB`(8),
    `CF_DIBV5`(17), and a registered `"PNG"`. `clipboard-win = "=5.4.3"`
    (Windows-only target dep) exposes `raw::register_format` /
    `raw::set_without_clear` -- so adding `CF_HTML` needs NO new dependency.
- **Can the installed egui/eframe screenshot its own window? YES.** egui/eframe
  0.33.3 on the glow renderer supports `ViewportCommand::Screenshot`. NOT a
  blocker.
- **Part B root cause (confirmed):** `crates/ff-screen-model/src/render/markdown.rs`
  line ~19 hardcodes `RenderOptions { unicode_box: false }` for the fenced code
  block, so the Markdown text flavor (and thus the composite text) uses ASCII
  `+/-/|`. The Unicode glyphs already exist in `render/text.rs::Glyphs`
  (`'\u{250C}'` etc., built from char constants -- NO raw box glyphs in source,
  satisfying the ASCII-only steering). Fixing Part B = render the fenced block
  with `unicode_box: true`.
- **Clipboard text flavor is Markdown from a MODEL:** today
  `snapshot_composite_to_clipboard` renders `SnapshotFormat::Markdown` from the
  model and builds `SnapshotPayload::from_model(&model, text)`, which internally
  RASTERISES the text (`rasterise_screen`) for the picture. The rework replaces
  that rasterised picture with the real window screenshot RGBA while KEEPING the
  model-derived Markdown text.

---

## Design decisions (made here; grounded in the code above)

- **D1 -- Two-phase async SNAPSHOT.** Bare `SNAPSHOT` cannot place the picture
  synchronously because the window screenshot arrives a later frame. Decision:
  `snapshot_composite_to_clipboard` becomes a REQUEST step that (a) resolves the
  active `ScreenCapture` model NOW (while `&self` context is valid), renders the
  Markdown text NOW, stashes a small `PendingSnapshot { text: String }` on the
  shell, and (b) calls `ctx.send_viewport_cmd(ViewportCommand::Screenshot(..))`.
  The screenshot request needs the egui `ctx`, which `handle_command` does not
  carry; so the request is DEFERRED: the command sets a
  `snapshot_request_pending` flag + stashed text, and the frame loop
  (`shell/update.rs`) issues the viewport command and drains the Screenshot
  event. This mirrors the existing deferred-action pattern in `update.rs`
  (e.g. `pending_menu_option`, `pending_tab_actions`). Rationale: keeps the
  command path non-blocking and the egui coupling in the frame loop only.
- **D2 -- Payload assembly is pure and unit-testable.** A new
  `SnapshotPayload::from_window_capture(image: RasterImage, text: String)`
  builds `{ image, png, text }` directly from a provided RGBA image (NOT
  rasterised from the model), so payload assembly is testable with a STUB image.
  `SnapshotPayload::from_model` is retained for the `SNAPSHOT PNG` file path
  (still a text raster, per Req 23.2 for the FILE output) and for existing tests.
- **D3 -- egui ColorImage -> RasterImage adapter is a thin shell module.** A new
  `crate::screenshot` module (ff-desktop) converts `Arc<egui::ColorImage>` to
  `ff_screen_model::RasterImage` (copy RGBA bytes, set width/height). This is the
  ONLY egui-touching new code; it lives in the shell, keeping ff-screen-model
  egui-free. Kept < 40 lines.
- **D4 -- CF_HTML builder is its own pure module.**
  `crate::clipboard_html::build_cf_html(png: &[u8], pre_text: &str) -> String`
  builds the HTML fragment (base64 `<img>` + `<pre>`) AND the CF_HTML header with
  correct BYTE offsets (`Version:0.9`, `StartHTML`, `EndHTML`, `StartFragment`,
  `EndFragment`, with `<!--StartFragment-->` / `<!--EndFragment-->` markers).
  Offsets are byte counts into the UTF-8 payload. Pure fn, unit-tested by parsing
  the header offsets back and asserting they point AT the fragment markers.
  Base64 is implemented in-module (tiny, no new dep) to avoid adding a crate.
- **D5 -- Keep `clipboard-win = "=5.4.3"`.** CF_HTML needs only
  `raw::register_format("HTML Format")` + `raw::set_without_clear`, both present
  in 5.4.3. No new pin. The HTML text payload is written as UTF-8 bytes (CF_HTML
  is defined as UTF-8).
- **D6 -- Transient overlay timing.** The window screenshot captures whatever the
  GPU rendered that frame, INCLUDING any open modal/overlay. Decision: document
  this behaviour (a SNAPSHOT taken while an overlay is up captures the overlay);
  do not add overlay suppression in this phase. Record in the module doc + the
  requirements note. (Matches Req 27.2 intent that the frontmost surface is
  captured; full per-overlay model capture is out of scope here.)
- **D7 -- Non-Windows fallback.** The `#[cfg(not(windows))]` `OsClipboard::place`
  (arboard image+text) is unchanged; CF_HTML is Windows-only. The window
  screenshot path is cross-platform (egui), so non-Windows still gets the real
  picture + text, just no CF_HTML flavor.

---

## Ordered implementation items

- [ ] 1. Part B: emit Unicode box-drawing in the Markdown fenced block.
      Change `render_markdown` so the fenced code block renders with
      `RenderOptions { unicode_box: true }` (was `false`). Keep the triple-
      backtick fence and the property/table sections unchanged. Update the
      existing `markdown_has_heading_code_block_and_table` test and add a test
      asserting the fenced body contains a U+2500-family glyph (build the
      expected char from `'\u{2500}'` etc. in the TEST -- no raw box glyph in
      source) and contains NO ASCII `+`/`|` frame char in the fenced region.
      Files: `crates/ff-screen-model/src/render/markdown.rs`
      Verify: `cargo test -p ff-screen-model render::markdown` -- new/updated
      tests pass; `cargo clippy -p ff-screen-model -- -D warnings` clean.

- [ ] 2. Add `SnapshotPayload::from_window_capture` and keep `from_model`.
      Add `pub fn from_window_capture(image: RasterImage, text: String) -> Self`
      that sets `png = ff_screen_model::encode_png(&image)` and stores the given
      image+text (NO model rasterisation). Document that `from_model` stays for
      the text-raster FILE path. Add a unit test building a 2x1 stub
      `RasterImage` and asserting `png` has the PNG signature and `text` round-
      trips.
      Files: `crates/ff-desktop/src/clipboard_backend.rs`
      Verify: `cargo test -p ff-desktop clipboard_backend` -- passes;
      `cargo check -p ff-desktop`.

- [ ] 3. Add the CF_HTML builder module.
      New module `crate::clipboard_html` with
      `build_cf_html(png: &[u8], pre_text: &str) -> String` producing the HTML
      fragment (`<img src="data:image/png;base64,....">` + `<pre
      style="font-family:monospace">{escaped pre_text}</pre>`) wrapped with the
      CF_HTML header and `<!--StartFragment-->`/`<!--EndFragment-->` markers at
      correct UTF-8 BYTE offsets. Include an in-module base64 encoder and HTML
      escaping. Keep < 400 lines; functions < 40 lines. Declare `mod clipboard_html;`
      in `main.rs`. Add unit tests that: parse the header's `StartHTML/EndHTML/
      StartFragment/EndFragment` integers back out and assert each offset indexes
      the expected byte (fragment markers / document bounds); assert the
      `data:image/png;base64,` prefix is present; assert the box-drawn text is
      inside the `<pre>`.
      Files: `crates/ff-desktop/src/clipboard_html.rs`, `crates/ff-desktop/src/main.rs`
      Verify: `cargo test -p ff-desktop clipboard_html` -- offset round-trip test
      passes; `cargo clippy -p ff-desktop -- -D warnings` clean.

- [ ] 4. Extend `SnapshotClipboard::place` to offer CF_HTML as a third flavor.
      Add `html: Option<String>` to `SnapshotPayload` (built by the shell from
      `build_cf_html`; `None` on non-Windows or when absent). In the
      `#[cfg(windows)] OsClipboard::place`, after the text + DIB writes, when
      `payload.html` is `Some`, `raw::register_format("HTML Format")` and
      `raw::set_without_clear(html_fmt, html.as_bytes())`. Extend `OfferedFlavors`
      with an `html: bool`. Update the `FakeClipboard` test fake and assertions
      to confirm all three flavors are reported when present.
      Files: `crates/ff-desktop/src/clipboard_backend.rs`
      Verify: `cargo test -p ff-desktop clipboard_backend` -- the fake reports
      image+text+html; `cargo check -p ff-desktop`.

- [ ] 5. Add the egui ColorImage -> RasterImage adapter.
      New thin module `crate::screenshot` with
      `fn color_image_to_raster(image: &egui::ColorImage) -> ff_screen_model::RasterImage`
      copying `image.as_raw()` RGBA bytes and dimensions. The ONLY new egui-
      touching code; < 40 lines. Declare `mod screenshot;` in `main.rs`. Unit
      test: build a small `egui::ColorImage` from known RGBA and assert the
      RasterImage has matching width/height/bytes.
      Files: `crates/ff-desktop/src/screenshot.rs`, `crates/ff-desktop/src/main.rs`
      Verify: `cargo test -p ff-desktop screenshot` -- passes.

- [ ] 6. Add pending-snapshot shell state.
      Add a `PendingSnapshot { text: String }` plus a request flag to the shell
      state struct (a grouped field, not a loose god-struct field where a group
      fits). Add `pub(super)` setter used by the command path and a take/clear
      used by the frame loop. Follow the existing `pending_menu_option` /
      `pending_tab_actions` conventions.
      Files: `crates/ff-desktop/src/shell/state.rs` (and the struct literal in
      `crates/ff-desktop/src/shell/construct.rs` to initialise it)
      Verify: `cargo check -p ff-desktop`.

- [ ] 7. Rework `snapshot_composite_to_clipboard` into the REQUEST phase.
      Change it to: resolve the active `ScreenCapture` (handle `OptOut` with the
      existing error), render the `SnapshotFormat::Markdown` text (now Unicode-
      boxed via item 1), store `PendingSnapshot { text }` and set the request
      flag; do NOT build a payload or touch the clipboard here. Remove the
      synchronous `from_model` + `place` call from the bare path. Keep the status
      message deferred to the assembly step (item 8).
      Files: `crates/ff-desktop/src/shell/commands_snapshot.rs`
      Verify: `cargo check -p ff-desktop`; `cargo test -p ff-desktop commands_snapshot`
      (adjust any test that asserted synchronous placement on bare SNAPSHOT --
      it now asserts the pending request is set).

- [ ] 8. Issue the viewport screenshot and assemble the payload in the frame loop.
      In `shell/update.rs::update`, near the other deferred-action drains: (a) if
      the snapshot request flag is set, clear it and
      `ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()))`,
      leaving `PendingSnapshot` in place; (b) ALWAYS scan
      `ctx.input(|i| i.raw.events)` for `egui::Event::Screenshot { image, .. }`,
      and when one arrives AND a `PendingSnapshot` is waiting: convert via
      `screenshot::color_image_to_raster`, build
      `SnapshotPayload::from_window_capture(raster, pending.text)`, build
      `payload.html = Some(clipboard_html::build_cf_html(&payload.png, &pre_text))`
      where `pre_text` is the box-drawn body (reuse the same Markdown/box text, or
      the plain Unicode-framed body) , then
      `OsClipboard.place(&payload)` and set the success/partial/failure status
      message (reusing the existing `ff-desktop-shell-msg-scrm-snapshot-*` i18n
      keys). Clear `PendingSnapshot`. Document D6 (overlay-in-frame) in the module
      doc.
      Files: `crates/ff-desktop/src/shell/update.rs`
      Verify: `cargo check -p ff-desktop`; `cargo clippy -p ff-desktop -- -D warnings`
      clean. (Live window-paste is the owner's manual Word check -- a headless CI
      cannot assert the Win32 clipboard contract.)

- [ ] 9. Full compile + scoped tests + format.
      Files: (none)
      Verify: `cargo check -p ff-screen-model -p ff-desktop`;
      `cargo clippy -p ff-screen-model -p ff-desktop -- -D warnings`;
      `cargo fmt -p ff-screen-model -p ff-desktop -- --check`;
      `cargo test -p ff-screen-model`; `cargo test -p ff-desktop clipboard_backend clipboard_html screenshot commands_snapshot`.
      The authoritative full `cargo gate` + manual Word paste is the owner's run.

---

## Digest

### `crates/ff-screen-model/src/render/markdown.rs`
- Block: `render_markdown(model: &ScreenModel) -> String`, line ~19:
  `let body = text::render_plain(model, RenderOptions { unicode_box: false });`
  -> change to `unicode_box: true`.
- Test: `#[cfg(test)] mod tests` -- update `markdown_has_heading_code_block_and_table`;
  add a Unicode-glyph-present / no-ASCII-frame test (expected glyphs built from
  `'\u{2500}'` constants in the test).
- Verified: `render_plain`'s `Glyphs::for_options(true)` already emits U+250C/
  2500/2502/etc. from char constants (`render/text.rs`). `RenderOptions::default()`
  is `unicode_box: true`.

### `crates/ff-desktop/src/clipboard_backend.rs`
- Block: `pub struct SnapshotPayload { image: RasterImage, png: Vec<u8>, text: String }`
  -> ADD `pub html: Option<String>`. Update `from_model` to set `html: None`.
- Block: ADD `pub fn from_window_capture(image: RasterImage, text: String) -> Self`
  (png = `encode_png(&image)`, `html: None`).
- Block: `pub struct OfferedFlavors { image: bool, text: bool }` -> ADD
  `html: bool`; `both()` unchanged (keep its image&&text meaning).
- Block: `#[cfg(windows)] impl SnapshotClipboard for OsClipboard::place` -- after
  the CF_DIB/CF_DIBV5 writes, add the CF_HTML write guarded by `payload.html`,
  via `raw::register_format("HTML Format")` + `raw::set_without_clear`.
- Reuse verbatim: `raster_to_bmp`, `raster_to_dib`, `raster_to_dib_v5`,
  `unicode_text_bytes`, constants `CF_DIB=8`, `CF_DIBV5=17`, `CF_UNICODETEXT=13`.
- Test: `#[cfg(test)] mod tests` with `FakeClipboard` -- extend to carry/assert
  `html`; add `from_window_capture` payload test (stub RasterImage).
- Verified: `clipboard-win = "=5.4.3"` exposes `raw::register_format` /
  `raw::set_without_clear` (used already for the `"PNG"` format).

### `crates/ff-desktop/src/clipboard_html.rs` (NEW)
- Block: `pub fn build_cf_html(png: &[u8], pre_text: &str) -> String` + private
  `base64_encode(&[u8]) -> String` + `html_escape(&str) -> String` + the CF_HTML
  header writer computing byte offsets.
- Test: offset round-trip (parse `StartHTML/EndHTML/StartFragment/EndFragment`,
  assert they index the markers / bounds); `data:image/png;base64,` present;
  `<pre>` contains the box-drawn text.
- Verified: CF_HTML is UTF-8; offsets are byte counts; `clipboard-win` raw API
  writes arbitrary bytes for a registered format.

### `crates/ff-desktop/src/screenshot.rs` (NEW)
- Block: `pub fn color_image_to_raster(image: &egui::ColorImage) -> ff_screen_model::RasterImage`
  (copy `image.as_raw()` + dimensions). ONLY egui-touching new code.
- Test: known small `egui::ColorImage` -> RasterImage byte/dim equality.
- Verified: `egui::ColorImage::as_raw()` yields RGBA8; `RasterImage` is
  `{width:u32,height:u32,rgba:Vec<u8>}` row-major RGBA8.

### `crates/ff-desktop/src/shell/commands_snapshot.rs`
- Block: `fn snapshot_composite_to_clipboard(&mut self)` -- rework from
  synchronous `from_model` + `place` into: resolve model, render Unicode-boxed
  Markdown text, stash `PendingSnapshot`, set request flag. OptOut handling and
  the single/PDF/file paths (`snapshot_single_format_to_file`, `snapshot_to_pdf`,
  `handle_snapshot` dispatch) UNCHANGED.
- Test: `#[cfg(test)]` snapshot tests -- adjust the bare-SNAPSHOT assertion to
  the pending-request contract.
- Verified: reached from `commands_ladder_b.rs:73 self.handle_snapshot(arg)` via
  the single front door; `handle_snapshot` branches bare -> this method.

### `crates/ff-desktop/src/shell/update.rs`
- Block: `impl eframe::App for WorkbenchShell { fn update(&mut self, ctx, _frame) }`
  -- add (a) request -> `ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()))`,
  (b) drain `ctx.input(|i| i.raw.events)` for `Event::Screenshot { image, .. }`
  -> convert, `from_window_capture`, build CF_HTML, `OsClipboard.place`, set
  status, clear pending. Place near existing deferred-action drains
  (`pending_menu_option`, `pending_tab_actions`).
- Test: none at this layer (live screenshot requires a GPU window; owner manual
  Word paste is authoritative). Compile-verify only.
- Verified: `update` already uses `ctx.send_viewport_cmd(...)` and
  `ctx.input(|i| ...)`; the frame loop is the right place for the async event.

### `crates/ff-desktop/src/shell/state.rs` + `construct.rs`
- Block: shell state struct -- ADD a grouped `PendingSnapshot { text: String }`
  option + request flag; initialise in the `construct.rs` struct literal.
- Verified: mirrors `pending_menu_option` / `pending_tab_actions` in the same
  struct.

### `crates/ff-desktop/src/main.rs`
- Block: `mod` list -- ADD `mod clipboard_html;` and `mod screenshot;` (keep
  alphabetical near `mod clipboard_backend;`).

### Build/test commands (discovered)
- Per-change scoped loop (gate-policy.md): `cargo check -p <crate>`,
  `cargo clippy -p <crate> -- -D warnings`, `cargo fmt -p <crate> -- --check`,
  plus quick scoped `cargo test -p <crate> <name>`. Crates touched:
  `ff-screen-model`, `ff-desktop`.
- Authoritative full `cargo gate` (fmt + clippy + build + whole suite) is the
  OWNER's manual run (testing.md execution split). An agent compile-verifies and
  runs scoped tests only; do NOT run the whole suite.
- What passes today: the worktree's last `cargo gate` was CLEAN (4652/4652),
  so the baseline is green before these changes.

### Known limitations / follow-ups
- A SNAPSHOT taken while a modal/overlay is open captures the overlay in the
  window screenshot (D6); overlay suppression is out of scope here and documented.
- The real "picture pastes into Word" proof is the owner's manual paste; headless
  CI cannot assert the Win32 clipboard contract or the live window screenshot.
- The requirements file's Req 23.2 still says the picture is a raster of the text
  render; the owner's findings (authoritative task spec) reverse this FOR THE
  CLIPBOARD PICTURE. The `SNAPSHOT PNG` FILE output remains the text raster
  (unchanged). Flag for a requirements edit if the owner wants the FILE PNG to
  also be a window screenshot.
