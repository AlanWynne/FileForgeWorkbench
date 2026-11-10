# Menu Kind gate (CR-NR-117) -- implementation review

Scope: the code commit `d37f41b` (after the docs reconciliation commit `3412be9`)
on branch `menu-kind-gate` in `.worktrees/menu-kind-gate`, reviewed against the
approved menu-workspace Requirement 23 (23.1-23.22), the design section "Menu Kind
snapshot content (CR-NR-117)", and decisions D1-D11. Docs were reviewed in the
prior gate step; this pass reviews the implementation and the generated goldens.
Build/test results were read from the commit message, not rerun.

## Verdict: APPROVED

No blockers found.

## What was checked

### Generated goldens vs the owner's expected layout and decisions
- `menu_pom_message_pfshow_on.txt` and `menu_pom_clipboard_markdown.md`: content
  rows `0 SETTINGS`, `3 HELP`, `S SCRM`, `X X`, colon aligned to the widest key
  and command, calendar on the right starting at widest-row + 4 cells, `S SCRM`
  description now `Screen Capture and Recording` (D7a). `X X` shows both columns
  (D5, 23.5). Frame-owned rule sits between the `Command ===>` row and the first
  option row with no blank row (owner Option 1); the first row below the rule is
  `0 SETTINGS`. Matches the design's full target POM layout.
- `menu_settings_pfshow_off.txt` / `.md`: no calendar, `group_separator = "line"`
  emits one rule row (as wide as the widest option row) between the Core group and
  `R RESET BARE`; the rule is in the box but NOT in the table (23.7, 23.12).
- `menu_scrm_pfshow_off.txt` / `.md`: mixed 1-2 letter keys aligned (`S`, `CS`,
  `CT`, `CR`, `H`, `X`), calendar shown (show_calendar defaults true; D10, 23.6).
- `menu_user_reports_pfshow_off.txt` / `.md`: a `menus/reports.toml` fixture with
  `group_headers = true` (headers `Standard`/`Admin` precede their groups),
  `group_separator = "space"` (blank row between groups), a disabled option
  (appears as a row, 23.2), and a literal `|` in a description that the table
  renderer escapes to `\|` (23.12). Genericity is exercised: the user menu uses
  the same provider.
- Every Markdown golden has the fenced box FIRST, then the single
  `Option | Command | Description` table (23.11); the table carries no calendar,
  no Time/Day of Year, no group rule and no group header (23.12). Table Command
  cells equal the displayed surfaces and the box rows (D11, 23.11).

### Builder (screen_snapshot_menu.rs) and wiring
- No menu-name / `is_home` branch in `format_menu_rows`, `compose_with_calendar`,
  `menu_table_section`, or `menu_row_displays`. One generic path for POM,
  Settings, SCRM and user menus.
- Rows are built ONCE: `menu_workspace_screen_model_full` derives both the box
  body and the one `markdown_sections` table from the same `rows` list (23.13).
  `fields` and `tables` are left empty, so the generic Markdown/HTML path cannot
  emit a second Property|Value copy (23.13); a guard test pins this.
- `menu_row_displays` fills rows from `menu_option_mnemonic_surfaces()`,
  `menu_option_command_surfaces()` and `localised_option_description`, the SAME
  sources the live renderer reads (23.2). The capture path builds the chrome from
  the ACTIVE tab's `mw`, and both surface helpers also read the active tab, so the
  surfaces are positionally aligned with `mw.menu.options`; a count-mismatch falls
  back to `rows_from_menu` (stored values). No alignment defect.
- Load-error and empty menus emit no option rows and no table (23.9), tested.

### Built-in content (defaults.rs)
- POM `S` description changed to `Screen Capture and Recording` (23.15, D7a); the
  `defaults.rs` SCRM assertion updated in lock-step; no mnemonic/command/group/
  order change (23.18).
- SCRM `X` gains `show_in_menu_bar = false` (23.16, D8); a new test asserts every
  SCRM option is bar-visible except `X`. The SCRM `X` still appears as a snapshot
  option row and table row (show_in_menu_bar affects the menu bar only).
- Built-in menus remain the compiled `DEFAULT_*_TOML` constants; no write-to-disk
  was added (mechanism 4 preserved).

### Framework and constraints
- No public `ScreenModel` / `Table` / `WorkspaceContext` reshape. `MenuRowDisplay`
  is crate-private and `rows` is an additive field on the crate-private
  `MenuScreenChrome` (23.20). Commit records `git diff --stat` on
  `crates/ff-screen-model` as empty.
- `shell/environment.rs`, `shell/common_frame.rs` and `shell/render_status.rs` are
  not in the commit's file set; the frame rule between the Command line and the
  content is drawn by the shared framed renderer, not by the Menu Kind builder
  (23.1). The live `render_menu_workspace` path is untouched (23.19).
- Non-test line counts: screen_snapshot.rs ~233, screen_snapshot_menu.rs ~171,
  screen_provider.rs ~275, defaults.rs ~248 -- all under 400. Tests live in a
  sibling `screen_snapshot_menu/tests.rs` and the shell test modules.
- Source files are ASCII (scan clean); Unicode box-drawing appears only in golden
  fixtures, which is permitted test data.

### Tests and coverage
- Builder unit tests cover no-suffix rows, colon alignment, `X X`, calendar column
  = widest + 4, no right padding without calendar, line/space/none separators,
  group headers, single table section, table excludes calendar/group rows, empty
  fields/tables, pipe escaping, load-error/empty, box-then-table order, HTML
  header, and displayed-surface as the single source.
- Shell goldens (box + Markdown) for POM, Settings, SCRM and the user menu, plus a
  status-cell guard (Last | Prof | Zoom only, 23.14) and a displayed-surface
  alignment check. A shared `assert_golden` helper was factored out and the two
  existing menu-frame goldens were rewired to it.
- Recorded results in the commit message: `cargo check --tests`, `clippy --tests
  -- -D warnings`, `fmt --check` all returncode 0; scoped test groups pass (menu
  330, screen_snapshot 26, snapshot 90, common_frame 28, capture 64,
  tests_common_frame_menu 3). The owner runs the authoritative full gate.

### Renumbering
- screen-snapshot-scrm criterion 30.31 appears exactly once; this code commit
  touched that spec only for the Req 22.3 / 25.2 SCRM-description wording in
  lock-step with 23.15. The 30.29/30.30 ownership and 30.31 uniqueness were
  settled in the docs reconciliation step.

## Notes (non-blocking)
- The design's illustrative fixtures showed the command column in title case
  (e.g. `0 Settings`), while the goldens show the displayed command surface in the
  stored case (e.g. `0 SETTINGS`). The design states the golden files are
  authoritative and Req 23.2 / D11 require the displayed surface, so the goldens
  are correct; the illustrative text was not binding.
