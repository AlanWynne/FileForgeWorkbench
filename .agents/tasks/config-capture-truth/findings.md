# Config Capture Truth -- Investigation Findings

READ-ONLY investigation. No code was changed.

## Summary answer (the truth)

The fix in commit `63807fe` **did land, with real content, on the current
`main` HEAD, and it is on the correct clipboard path.** The source on the
checked-out tree no longer contains the dot-leader logic or the duplicate
`Settings` table for the Config Context. Therefore the output the user still
sees can ONLY come from a **STALE BINARY** -- the app the user is running was
built BEFORE the fix was committed and has not been rebuilt.

Hard proof of staleness:

- Commit `63807fe` was committed at **2026-11-02 15:02:52 +02:00**
  (`git show -s --format=%cI HEAD`).
- The only built binary, `target\debug\ffwb.exe`, has
  `LastWriteTime = 2026-11-02 14:42:12` and `CreationTime = 2026-11-02 14:42:10`
  -- roughly **20 minutes BEFORE the fix commit**. There is no `target\release\ffwb.exe`.
- The binary therefore was compiled from the PARENT tree (`63807fe~1`), whose
  `config_panel_screen_model()` builds flat `fields` (dot-leader source) AND
  pushes `config_table()` (the `Settings` block) -- which is EXACTLY the output
  the user pasted.

Recommended fix: **rebuild and relaunch the app** (`cargo build -p ff-desktop`,
or the release profile if that is what the user runs, then restart `ffwb.exe`).
No source change is required; the merged source is already correct.

---

## Evidence

### Q5 -- git history: is 63807fe the current HEAD, and did it change content?

`git log --oneline -5` (read from `_ignore/logs/git-head-check.txt`):

```
63807fe fix(scrm): config capture mirrors visible render (no dot-leaders, no duplicate table)
7a28007 fix(configuration-system): global-aligned flush-left Config tree + duplicate-render guard (Req 22)
...
```

`git show 63807fe --stat` (read from `_ignore/logs/git-show-stat.txt`) -- a real,
non-empty commit:

```
 crates/ff-desktop/src/config_panel/mod.rs          |   3 +
 .../src/shell/screen_providers_config.rs           | 173 ++++++++++++------
 .../src/shell/screen_providers_tables.rs           |  65 -------
 .../src/shell/tests_config_snapshot_fidelity.rs    | 200 +++++++++++++++++----
 .../src/shell/tests_editor_snapshot_tables.rs      |  47 ++---
 docs/specs/master-screen-directive/requirements.md |  12 ++
 6 files changed, 322 insertions(+), 178 deletions(-)
```

`git status -sb`: `## main...origin/main [ahead 5]`. Neither
`screen_providers_config.rs` nor `screen_providers_tables.rs` appears in the
`M` (modified) list, so the checked-out working-tree copy of those two files is
byte-identical to what `63807fe` committed. The fix is present on disk.

### Q1 -- what does the CURRENT config model builder do?

`crates/ff-desktop/src/shell/screen_providers_config.rs`,
`WorkbenchShell::config_panel_screen_model()` (current main). It builds ONLY
`model.body_lines`, never `fields`, never `tables`:

- Header rows pushed as aligned body lines (no dot-leaders), e.g.
  `model.body_lines.push(format!("{}{}", aligned_left_segment(&filter_caption, column), p.filter));`
- Each visible tree row pushed once via `config_capture_row_line(row, column)`,
  which emits `"[Group] : expanded|collapsed"` for a namespace and
  `"key : value"` for a key -- a single global aligned `:` column.

A grep of that file for `\.fields\.push | Table { | tables.push | with_field`
returned **no matches**. There is no flat-field path and no table push left in
the Config provider. (`config_capture_row_line` and `config_key_display_value`
are the only helpers, both body-line based.)

### Q2 -- does config_table() still exist / is an equivalent invoked for Config?

- `grep config_table` across `crates/**/*.rs`: the only hits are in `ff-config`
  (`arb_config_table`, `toml_table_to_config_table`, `flatten_config_table`,
  `config_table_is_ordered_map`) -- all UNRELATED to the screen capture.
- The former `WorkbenchShell::config_table()` builder in
  `screen_providers_tables.rs` is **gone** (the `--stat` shows 65 deletions in
  that file).
- In `screen_providers_tables.rs` the only remaining `Setting`/`Value` table is
  `caption: Some("Theme settings")` -- that belongs to the **Theme editor**
  model, not Config. The Config Context has no table builder and no call site.

### Q3 -- the ACTUAL clipboard copy-screen path, end to end

The Config Context is wired to the SAME model the fix edited; the clipboard path
does not use a different provider:

1. `ff-screen-model::render` dispatch (`crates/ff-screen-model/src/render.rs`):
   `PlainText/Ansi -> text::render_plain`, `Markdown -> markdown::render_markdown`.
2. The clipboard "copy screen" family:
   - Bare `SNAPSHOT` -> `commands_snapshot.rs::handle_snapshot` ->
     `snapshot_composite_to_clipboard` -> `active_screen_capture()` ->
     `stash_composite_request`, which renders `SnapshotFormat::Markdown` for the
     `CF_UNICODETEXT` flavor and `SnapshotFormat::PlainText` for the CF_HTML
     `<pre>` body.
   - `CAPTURE SCREEN` -> `commands_scrm.rs::handle_capture` -> `active_screen_model()`.
   - Both call `active_screen_capture()` in
     `crates/ff-desktop/src/shell/screen_provider.rs`, where
     `KindTag::ConfigPanel => ScreenCapture::Model(Box::new(self.config_panel_screen_model()))`.
   There is ONE config model builder and both copy paths use it. (The literal
   command `ss` is not a built-in verb or alias in the source; the user's pasted
   content is unmistakably a Config screen capture, so it reached the capture
   path -- most likely a user alias or the SNAPSHOT/CAPTURE SCREEN affordance.)

Where each part of the user's paste is emitted (the renderers, which are
UNCHANGED and correct -- the bug is purely the data they were handed):

- **Dot-leaders** come from `crates/ff-screen-model/src/render/text.rs`,
  `render_plain`:
  ```rust
  if !model.fields.is_empty() {
      lines.push(String::new());
      for f in &model.fields {
          if f.label.is_empty() { lines.push(f.value.clone()); }
          else { lines.push(format!("{} . . . : {}", f.label, f.value)); }
      }
  }
  ```
  i.e. a NON-EMPTY `model.fields` is formatted as `{label} . . . : {value}`.
- **The duplicate `Settings` table** comes from the immediately following loop in
  the same file:
  ```rust
  for table in &model.tables {
      lines.push(String::new());
      if let Some(cap) = &table.caption { lines.push(cap.clone()); }
      if !table.headers.is_empty() { lines.push(table.headers.join("  ")); }
      for row in &table.rows { lines.push(row.join("  ")); }
  }
  ```
  i.e. a NON-EMPTY `model.tables` (caption `Settings`, headers
  `Setting / Value / State`) renders as the second block.
- The Markdown version the user pasted into chat is `markdown::render_markdown`,
  which fences `render_plain` output (so the box + dot-leaders + Settings block
  appear inside the fence) and would ALSO append a `| Property | Value |` table
  from `fields`. Both Markdown and plain-text flavors derive from the same
  model.

Crucially: the CURRENT `config_panel_screen_model()` leaves `fields` and
`tables` EMPTY, so with a correctly rebuilt binary neither branch above fires
for the Config Context. The renderers are correct; they faithfully render
whatever the model carries.

### Q4 -- stale binary vs source still containing the bug

The merged source on `main` does **NOT** contain the `. . .` dot-leader data
path or the duplicate table for Config (Q1/Q2). Per the brief's own logic, that
leaves only the stale-binary explanation -- and the binary timestamp confirms it:

- `target\debug\ffwb.exe` LastWriteTime `2026-11-02 14:42:12`.
- HEAD commit time `2026-11-02 15:02:52`.
- Binary is ~20 minutes OLDER than the fix. No release binary exists.

For completeness, the PARENT source (`git show 63807fe~1:.../screen_providers_config.rs`,
saved to `_ignore/logs/old-config-provider.txt`) is what the stale binary was
built from, and it contains BOTH offending constructs:

```rust
model = model.with_field(Field::new("Filter", p.filter.clone()));
model = model.with_field(Field::new("Unsaved edits", ...));
// ... each row as with_field(Field::new(format!("[{}]", ns_display_name(ns)), state))
// ... each key as with_field(Field::new(key.clone(), value))
model.tables.push(self.config_table());   // the Settings / Setting Value State block
```

`with_field(...)` populates `model.fields` (-> dot-leaders), and
`model.tables.push(self.config_table())` populates `model.tables` (-> the
`Settings` block). This is a byte-for-byte match to the user's paste, confirming
the running binary is the pre-fix build.

---

## Conclusions

1. **WHERE the dot-leaders come from:** `ff-screen-model/src/render/text.rs`
   `render_plain`, the `format!("{} . . . : {}", f.label, f.value)` branch, fed
   by a non-empty `model.fields`. In the PRE-FIX `config_panel_screen_model()`
   those fields were populated via `with_field(...)`. The post-fix function no
   longer populates `fields`.
2. **WHERE the duplicate `Settings` table comes from:** the `for table in
   &model.tables` loop in the same `render_plain`, fed by the now-removed
   `model.tables.push(self.config_table())`. The post-fix function no longer
   populates `tables`, and `config_table()` was deleted.
3. **Did the fix touch the actual clipboard path?** YES. The clipboard copy
   (SNAPSHOT composite and CAPTURE SCREEN) resolves the Config Context through
   `active_screen_capture()` -> `config_panel_screen_model()` -- the exact
   function the fix rewrote. The fix is on the correct path, not a sibling.
4. **Single most likely explanation:** **STALE BINARY.** The merged source is
   correct; the running `ffwb.exe` was built ~20 minutes before the fix was
   committed and was never rebuilt. It is NOT fix-on-wrong-path and NOT
   fix-not-landed.

## Recommendation (no code change needed)

- **Rebuild and relaunch.** Run the project build for the desktop app
  (`cargo build -p ff-desktop`, or the `--release` profile if that is what the
  user launches), then restart the application. The Config SNAPSHOT / CAPTURE
  SCREEN clipboard text will then be the single body-line tree (flush-left
  `[Group] : expanded`, indented `key : value`, one aligned `:` column, no
  dot-leaders, no second `Settings` table).
- **Verify the fix (optional, the authoritative check is the owner's run):** the
  fidelity tests already exist -- `cargo test -p ff-desktop config_` /
  `tests_config_snapshot_fidelity.rs` assert no `. . .` dot-leaders and no
  duplicate `Settings` block in `config_panel_screen_model()`'s `body_lines`.
  The commit message records 71 passing `config_` tests.
- No source edit is warranted. If the user had ALSO reported the bug persisting
  after a confirmed rebuild, the next suspects would be the renderers
  (`text.rs`/`markdown.rs`) still emitting from `fields`/`tables` for some other
  Context -- but for Config the model no longer carries either, so a rebuild is
  the fix.

## Files read (evidence trail)

- `crates/ff-desktop/src/shell/screen_providers_config.rs` (current)
- `crates/ff-desktop/src/shell/screen_provider.rs` (capture dispatch)
- `crates/ff-desktop/src/shell/commands_scrm.rs`, `commands_snapshot.rs`
- `crates/ff-desktop/src/shell/commands_ladder_b.rs` (SNAPSHOT verb)
- `crates/ff-screen-model/src/render.rs`, `render/text.rs`, `render/markdown.rs`
- `crates/ff-desktop/src/shell/screen_providers_tables.rs` (Theme-only table)
- git: HEAD log, `show 63807fe --stat`, `show -s %cI HEAD`, `status -sb`,
  `show 63807fe~1:.../screen_providers_config.rs`
- binary timestamp: `target\debug\ffwb.exe`
