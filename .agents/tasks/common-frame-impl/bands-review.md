# CR-NR-115 common-frame ruled-band snapshot rendering

The branch redraws the SNAPSHOT/CAPTURE frame as ruled bands with cell dividers and junction glyphs, in two commits (7f1c84c bands, 2116336 status split and single-cell key line). The golden fixtures, read literally line by line, match the authoritative layout. Full-text golden tests, a SNAPSHOT == CAPTURE byte check and an ASCII-fallback golden all exist.

Watch for: the status band in the Config and Alias fixtures is `Last | Prof` only, not the longer example in the task text (confirmed, see Issue 1). This comes from the owner revision recorded in commit 2116336 and tested for the Editor Context.

**Verdict**: APPROVED

## High-level view

The renderer (ff-screen-model render/framed.rs) lays out title bar, menu bar, tab bar, centred title, Command line, content, optional message, status and optional key line as bands. Each band is separated by a rule, and the rule junctions are derived from the cell dividers of the adjacent bands. The shell side (common_frame.rs) only composes the cell text and constants.

The fixtures are real rendered text compared in full. Both Unicode and ASCII goldens are present, and SNAPSHOT is checked equal to CAPTURE.

## Issues (1)

<details>
<summary>Issues (1)</summary>

1. **Status band content differs from the task's example string (informational, confirmed)** -- Config and Alias goldens show `Last: 11:02 | Prof: default` only. The `Ln 1, Col 1 | UTF-8 | Lines N` cells are now workspace-owned and appear for the Editor Context (test `editor_context_status_text_keeps_ln_col_encoding_and_line_count`). This follows the owner revision in 2116336. Confirm that revision is intended, no code change needed.

</details>

<details>
<summary>Details</summary>

## Band-by-band reading of the fixtures

Fixtures read: `config_pfshow_on.txt` (141 wide, 36 lines), `alias_pfshow_on.txt`, `config_message_pfshow_off.txt` (51 wide), `alias_message_pfshow_off.txt` and `alias_message_pfshow_off_ascii.txt` (65 wide). A line-length scan found every line in each fixture has one uniform width, so no band is ragged.

Top border: `┌───┬──...──┐`, with the tee over the `#` cell boundary.
Title bar: `│ # │ FileForge Workbench ... _ [] X │`. The `#` cell is leading, and the window controls are right-aligned with one space before the border. Confirmed.
Rule after title: `├───┴──────┬──────┬───...┤`. The `┴` sits at column 4, directly under the `#` divider. The `┬` at columns 11 and 18 are the Settings and Help dividers of the menu band. Confirmed by column counting.
Menu band: `│ Settings │ Help │ SCRM │` with `|`-style dividers (box vertical). Confirmed.
Rule after menu: `├──────────┴───┬──┴───...┤`. The `┴` at columns 11 and 18 close the menu dividers, and the `┬` at column 15 opens the tab divider. Alias variant lands at column 15 as well (`>[ALIASES] x<` is wider, the divider moves accordingly and the junctions follow). Confirmed.
Tab band: `│ >[CONFIG] x< │ [POM] x │`. The active tab keeps the `>[TAG] x<` marker, the other tab is a separate cell. Confirmed.
Rule after tab bar: `├──────────────┴────...┤`, the `┴` matches the tab divider. Confirmed.
Centred title band: a single cell, no internal divider, text centred (Configuration, Alias Editor). Confirmed.
Rule after title: plain rule, no junctions. Confirmed.
Command line: `│ Command ===> ____...____ │`. Underscores fill to one space before the border, with no rule after it, and the field holds only underscores (the triggering `ss` is not included). Confirmed.
Content rows: unchanged. The Alias table grid stays inside the frame.
Rule after last content row: it follows immediately (`vfs.default_provider` then a rule, the Alias grid `└` row then a rule). There is no blank row before the status bar. Confirmed.
Message band (message fixtures only): `├───┤ / │ info one │ / ├───...┤`, between the content closing rule and the status band, with its own rules and no junctions. It is absent in the pfshow-on fixtures that carry no message. Confirmed.
Status band: `│ Last: 11:02 │ Prof: default │`. The label reads `Prof:`, and the `┬` above and `┴` below sit at column 14, matching the divider. Confirmed.
Rule after status: present in the pfshow-on fixtures, and the key line is a single cell and the last band, followed by `└──┘`. Confirmed. In the pfshow-off fixtures the status band is the last band, followed by the bottom border (the bottom border serves as the closing rule, junction `┴` correct).
ASCII golden: uses only `+ - |` and the same junction positions (`+---+------+------+`, `+----------+----+-+`). Confirmed, no box-drawing glyph present.

## Test quality (part b)

`tests_common_frame_golden.rs` compares complete text with `assert_eq!(snapshot_text(&h), fixture(...))` for Config pfshow on, Alias pfshow on, Config message pfshow off, Alias message pfshow off, and the ASCII fallback (`alias_snapshot_ascii_fallback_matches_golden_fixture`, via `RenderOptions { unicode_box: false }`). These are not substring checks. `snapshot_equals_capture_byte_for_byte_for_config_and_alias` asserts snapshot text equals capture text for both Contexts. Determinism is handled by a fixed timestamp, the default profile and pruning to exactly two tabs. A fixed-width harness gives a 141-column frame for the pfshow-on fixtures.

Not tested: a golden for a table Context in a multi-segment status band, and a pfshow-on plus message combination. Neither is required by the task.

## Constraints

No framework change: the diff touches ff-screen-model render, shell/common_frame.rs, render_status.rs, shell/mod.rs (test module registration), tests and docs. No dispatch, navigation, focus or persistence change.
No public ScreenModel/Table reshape: model.rs and table.rs are absent from the diff stat. The only render.rs change is 7 lines, and text.rs adds the glyph accessors `tee_down`, `tee_up`, `cross`.
shell/environment.rs: not in the diff stat. Untouched.
Clipboard (CR-CH-079 dual representation, PNG flavor): no clipboard or PNG file in the diff stat. Unchanged.
File size: framed.rs 293 non-test lines, text.rs 327, render.rs 8, common_frame.rs 188, render_status.rs 331. All under 400.
ASCII: the changed source is ASCII, except render_status.rs, whose only non-ASCII is the box-drawing section-separator comments at lines 15 and 103 (`// ── Key label bar`, `// ── Status bar`). These are not in the added lines as far as the diff stat of this file (92 lines) shows, and are pre-existing (likely). Not a blocker, possible cleanup when next touching the file. The fixtures contain box-drawing by design, as test data.
Commit messages: both describe the files and areas changed, contain no semicolons, and record what was run (cargo check, clippy -D warnings, fmt --check, scoped tests for ff-screen-model and ff-desktop with pass counts). I did not re-run anything.

</details>

## File map

<details>
<summary>Files</summary>

- ff-screen-model render/framed.rs: ruled-band layout with cell dividers and junctions.
- ff-screen-model render/text.rs, render.rs, lib.rs: junction glyphs and the right-align marker.
- ff-desktop shell/common_frame.rs: composes band cells and constants.
- ff-desktop shell/render_status.rs: common versus workspace-owned status cells.
- ff-desktop shell/golden/*.txt: five golden fixtures.
- tests_common_frame_golden.rs, render/framed_tests.rs and two updated tests: golden, equality and unit coverage.
- docs/specs and change-log: requirement, design and log updates.

Full diff: `git diff 87346de...HEAD` in the worktree.

</details>
