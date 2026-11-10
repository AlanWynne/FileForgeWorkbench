# Task 9.1 TRIAGE -- CR-NR-103 Localisation Phase 2 (ff-desktop i18n, argument-bearing / verb-embedding STATUS/ERROR messages)

READ-ONLY investigation. No code was changed. All file:line citations are from
`crates/ff-desktop/src` and the `ff-*` library crates at the time of this scan.

---

## 1. Summary answer (the key finding)

Task 9 is dominated by ONE recurring site shape: `self.open_error = Some(...)`.
A full scan found **83 assignment sites** (82 production + 1 in a test file,
`tests_session.rs:642`, which is excluded from localisation). The status-bar
segments in `render_status.rs` and the SCROLL error add a handful more.

Classifying each production site:

| Class | Count | Meaning |
|-------|------:|---------|
| USER-FACING-TRANSLATABLE -- own literal/format in ff-desktop | ~48 | Must route through `ff_i18n::t` / `t_args`; interpolated values and embedded verbs become placeables. |
| USER-FACING at a DISPLAY BOUNDARY -- text originates in a `thiserror`/`CommandResult`/`String`-returning helper | ~26 | Req 9.3: localise WHERE SHOWN, not at the error source. The error enums are NOT changed. Many of these are currently best left as pass-through (see note) and only a small set have a stable enough shape to key now. |
| INTERNAL-DIAGNOSTIC / not localisable here | 8 (plus the 1 test) | Pass-through of an already-built `String`/`e.to_string()` whose content is produced elsewhere, or a value that is DATA. |

Status-bar + SCROLL additional sites (`render_status.rs`, `render_command_line.rs`,
`environment_ffedit.rs`): **~8 translatable** (phase labels, `Ln/Col`, `{} lines`,
`CAPS`, degraded-logging tooltip, SCROLL invalid-amount x2).

Requirement 8 (plugin UI + plugin command-environment seam) requires **NO rewrite
of existing files**: the `ff-i18n` seam (`t`/`t_args`) and the per-environment
`AliasTable::overlay_locale` seam already exist and are public; Req 8 is satisfied
by those seams being AVAILABLE to plugin-authored code, which has no concrete
render/registration surface in `ff-plugin` yet. It is a seam-availability item,
not a code-change item (documented as such below).

Recommended sequencing: **9.2 interpolated messages first** (highest value, most
sites, clean placeables), then **9.3 verb-embedding + thiserror display boundary**,
then **9.4 plugin seam (doc/verify only)**. Size into small per-file/per-area
commits mirroring Task 8's cadence (~1 file or 1 cluster per commit).

**Byte-identical constraint (Task 8 convention, confirmed in the shipped `.ftl`
headers):** at `ui.locale=en` the rendered text MUST stay identical. Counts are
passed as **TEXT placeables pre-formatted in Rust** (e.g. `.set("count", n.to_string())`),
NOT Fluent `NUMBER()`, to avoid locale digit grouping. Embedded VERBS are passed
as **non-translated TEXT placeables** (Req 9.2).

---

## 2. Evidence -- the seam and conventions already in place

- Seam API (`crates/ff-i18n/src/lib.rs`, `seam.rs`, `args.rs`):
  - `ff_i18n::t(key) -> String`
  - `ff_i18n::t_args(key, &MessageArgs)` where `MessageArgs::new().set(name, text)`
    (TEXT, incl. embedded verbs) and `.set_number(name, f64)` (numeric).
  - `args.rs` doc comment already states a verb is "supplied as an ordinary
    string argument and is NEVER translated (localization Req 9.2)".
- English `.ftl` catalogues live under `crates/ff-i18n/i18n/en/`, one file per
  area, named `ff-desktop-<area>.ftl`. Existing Task 8 headers explicitly DEFER
  the Task 9 set:
  - `ff-desktop-shell-status.ftl`: "the open_error text are swept/decided in
    area 20 / the Task 9 status pass"; phase labels / `Ln/Col` / `{} lines` /
    `CAPS` and the degraded-logging tooltip are called out as Task 9.
  - `ff-desktop-shell-commandline.ftl`: "the SCROLL invalid-amount error (an
    open_error status message deferred to the Task 9 / area-17 status pass)".
  - `ff-desktop-shell-dialogs.ftl`: shows the ESTABLISHED conventions this triage
    follows -- count passed as a TEXT placeable (`{ $count }`, pre-formatted),
    leading whitespace preserved with a quoted placeable, "External execution
    cancelled." open_error deferred to Task 9.
- Alias overlay seam (`crates/ff-desktop/src/shell/environment.rs`):
  `AliasTable::overlay_locale` already layers a locale's rows onto the English
  base (Req 5), so Req 8.2's plugin-environment aliases reuse it unchanged.

---

## 3. Per-crate / per-target classification tables

Columns: `file:line | current text (abridged) | interpolates? | embeds verb? | classification | proposed ff.ftl key or 'leave' | note`.
Classifications: **UF** = user-facing-translatable (own literal); **UF-DB** =
user-facing at a display boundary (text from a `thiserror`/helper -- Req 9.3);
**INT** = internal/pass-through, leave as plain Rust.
Proposed target file for ALL ff-desktop shell status/error keys unless noted:
a new **`ff-desktop-shell-status-messages.ftl`** (status-bar strings stay in the
existing `ff-desktop-shell-status.ftl`; command-line SCROLL stays in
`ff-desktop-shell-commandline.ftl`).

### 3a. `crates/ff-desktop/src/shell/actions.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| actions.rs:53 | `Lua execution not yet available` | no | no | UF | `...-lua-not-available` | static stub status |
| actions.rs:57 | `Delete failed: {e}` | yes (`e`) | no | UF | `...-delete-failed` | `t_args`, `error`=e.to_string() TEXT placeable |
| actions.rs:84 | `Some(e)` (from `shell_open_file`) | n/a | no | UF-DB | leave (pass-through) | `e` is a `String` already built in ff-desktop; localise at the helper's own message sites, not here |
| actions.rs:112 | `Replaced {} occurrence(s) in {} file(s)` | yes (2 counts) | no | UF | `...-replace-summary` | counts as TEXT placeables `{ $count }`,`{ $files }` |
| actions.rs:118 | `Replace failed: {e}` | yes (`e`) | no | UF | `...-replace-failed` | `t_args` error placeable |

### 3b. `crates/ff-desktop/src/shell/commands_fastpath.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands_fastpath.rs:102 | `Chained path exceeds maximum depth of 4 segments.` | no | no | UF | `...-chain-depth-exceeded` | static |

### 3c. `crates/ff-desktop/src/shell/commands_ladder_a.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands_ladder_a.rs:43 | `msg` = `format_logoff_message()` | yes | no | UF-DB | leave (pass-through) | localise inside `format_logoff_message` (its own site) |
| commands_ladder_a.rs:49 | `Some(e.to_string())` (`CommandResult::Err`) | n/a | no | UF-DB | leave | file.exit error; command-result text (see 3t) |
| commands_ladder_a.rs:74 | `EDIT requires a file path` | no | yes (EDIT) | UF | `...-edit-requires-path` | verb EDIT is in prose at start; keep as embedded non-translated arg `{ $verb }` OR keep literal EDIT in key (see decision D1) |
| commands_ladder_a.rs:80 | `Some(e.to_string())` (file.open) | n/a | no | UF-DB | leave | command-result text |
| commands_ladder_a.rs:146 | `DOCK: the current workspace is not detached` | no | yes (DOCK) | UF | `...-dock-not-detached` | verb DOCK prefix -> `{ $verb }` placeable |

### 3d. `crates/ff-desktop/src/shell/commands_ladder_b.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands_ladder_b.rs:53 | `COMMAND: '{other}' is not a valid position (TOP or BOTTOM)` | yes (`other`) | yes (COMMAND, TOP, BOTTOM) | UF | `...-command-bad-position` | `other` TEXT placeable; COMMAND/TOP/BOTTOM verbs/keywords stay English (in key literal or as args, see D1) |
| commands_ladder_b.rs:106 | `message` from `resolve_reset_bare_target` | yes | no | UF-DB | leave (pass-through) | localise in the resolver's message site |

### 3e. `crates/ff-desktop/src/shell/commands_ladder_c.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| :71 | `UNSPLIT: the Workspace is not split.` | no | yes (UNSPLIT) | UF | `...-unsplit-not-split` | verb prefix |
| :82 | `FOCUS: the Workspace is not split.` | no | yes (FOCUS) | UF | `...-focus-not-split` | verb prefix |
| :116 | `SWAP: tab number {n} out of range (valid: 1 to {count})` | yes (n,count) | yes (SWAP) | UF | `...-swap-out-of-range` | n,count TEXT placeables |
| :122 | `SWAP: invalid argument '{arg}'. Usage: SWAP <n>, SWAP LIST` | yes (arg) | yes (SWAP, LIST) | UF | `...-swap-bad-arg` | arg placeable; verb/usage tokens English |
| :149 | `SUBMIT: JES subsystem not yet available` | no | yes (SUBMIT) | UF | `...-submit-not-available` | verb prefix |
| :162 | `msg` = `Date: {} Time: {} Day: {}` (TIME) | yes (3 vals) | no | UF | `...-time-status` | date/time/day as TEXT placeables (pre-formatted) |
| :177 | `msg` STATUS routing (`STATUS: routing to JES panel (filter: {j})` / `STATUS: routing to JES job status panel`) | yes (j) opt | yes (STATUS) | UF | `...-status-routing-filter` + `...-status-routing` | two keys; filter placeable |
| :184 | `CREATE {dsn}: dataset creation not yet available` | yes (dsn) | yes (CREATE) | UF | `...-create-not-available` | dsn TEXT placeable (DATA) |
| :190 | `REPLACE {dsn}: dataset replace not yet available` | yes (dsn) | yes (REPLACE) | UF | `...-replace-dsn-not-available` | dsn placeable |
| :206 | `Some(e.to_string())` (file.open) | n/a | no | UF-DB | leave | command-result text |
| :223 | `Some(e.to_string())` (file.open) | n/a | no | UF-DB | leave | command-result text |
| :233 | `COMPARE {dsn}: compare view not yet available` | yes (dsn) | yes (COMPARE) | UF | `...-compare-not-available` | dsn placeable |
| :244 | `WORKSPACE OPEN requires a path` | no | yes (WORKSPACE OPEN) | UF | `...-workspace-open-requires-path` | verb phrase English |
| :253 | `WORKSPACE SAVE AS requires a path` | no | yes | UF | `...-workspace-saveas-requires-path` | |
| :262 | `WORKSPACE ADD ROOT requires a path` | no | yes | UF | `...-workspace-addroot-requires-path` | |
| :291 | `WORKSPACE REMOVE ROOT requires a path` | no | yes | UF | `...-workspace-removeroot-requires-path` | |
| :303 | `No active workspace` | no | no | UF | `...-no-active-workspace` | static |
| :306 | `WORKSPACE: unknown subcommand. Use OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT` | no | yes (many) | UF | `...-workspace-unknown-subcommand` | subcommand token list stays English |

### 3f. `crates/ff-desktop/src/shell/commands.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands.rs:263 | `THEME: '{arg}' does not exist` | yes (arg) | yes (THEME) | UF | `...-theme-not-exist` | arg placeable (theme name = DATA) |
| commands.rs:325 | `Some(status.text.clone())` (SyntaxError/StructureError/RuntimeError) | n/a | no | UF-DB | leave (pass-through) | `status.text` originates in language-service/structure engines; localise at those producers if ever, not here |

### 3g. `crates/ff-desktop/src/shell/commands_scrm.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands_scrm.rs:102 | `msg` -- several CAPTURE subcommand results incl. `CAPTURE: unknown subcommand '{other}'.`, `Captured screen {seq}.`, `CAPTURE SCREEN: the active workspace cannot be captured yet.` | yes (other/seq) | yes (CAPTURE) | UF | multiple keys `...-capture-*` | seq as TEXT placeable; status()/list()/purge() returns are DATA from `scrm` (UF-DB -- see note) |
| commands_scrm.rs:287 | `No screen collection to replay.` | no | no | UF | `...-scrm-no-collection` | static |
| commands_scrm.rs:325 | `Snapshot copied to clipboard ({fmt}).` / `Snapshot rendered ({fmt}) but the clipboard was unavailable.` | yes (fmt) | no | UF | `...-snapshot-copied` / `...-snapshot-no-clipboard` | fmt label is DATA placeable |
| commands_scrm.rs:333 | `Some(message)` (snapshot err) | yes | no | UF-DB | leave (pass-through) | message built in `snapshot_text_for_active` |
| commands_scrm.rs:386 | `Unknown PFSHOW argument '{arg}' -- expected ON, OFF, BASE, SHIFT, CTRL, or ALT.` | yes (arg) | yes (PFSHOW + keywords) | UF | `...-pfshow-bad-arg` | arg placeable; keyword list English |

Note: the `scrm.status()`/`list()`/`purge()`/`export()` return strings reach
`open_error` via `msg` at :102. These are SCRM-engine report text; classify UF-DB
and localise (if ever) at the SCRM engine's own rendering, not at this boundary.
See DECISION D3.

### 3h. `crates/ff-desktop/src/shell/commands_session.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| commands_session.rs:278 | `Could not restore: {e}` | yes (e) | no | UF | `...-restore-failed` | error TEXT placeable |

### 3i. `crates/ff-desktop/src/shell/dispatch_ffedit.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| dispatch_ffedit.rs:264 | `Some(summary)` = `edit_profile.display_summary()` | yes | no | UF-DB | leave (pass-through) | summary built in `ff-edit-operations`; a composite profile dump -- localise at source only if ever (DECISION D4) |
| dispatch_ffedit.rs:281 | `Profile is locked -- use LOCK OFF to unlock` | no | yes (LOCK OFF) | UF | `...-profile-locked` | THE canonical Req 9.2 example; `LOCK OFF` as non-translated `{ $verb }` placeable |
| dispatch_ffedit.rs:283 | `Some(e.to_string())` (`ProfileError::UnknownKey`/`UnknownValue`) | n/a | no | UF-DB | leave OR key at boundary | `ProfileError` Display carries a `[profile]` dev prefix (see 3u); Req 9.3 says localise at display -- recommend a small match-at-boundary (DECISION D2) |
| dispatch_ffedit.rs:304 | `HILITE: unknown mode '{keyword}'` | yes (keyword) | yes (HILITE) | UF | `...-hilite-unknown-mode` | keyword placeable |

### 3j. `crates/ff-desktop/src/shell/environment_ffedit.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| environment_ffedit.rs:281 | `SCROLL: '{arg}' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)` | yes (arg) | yes (SCROLL + keywords) | UF | `ff-desktop-shell-commandline-scroll-invalid` | SHARES a key with render_command_line.rs:176 (identical text); arg placeable |

### 3k. `crates/ff-desktop/src/shell/external_adapter.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| external_adapter.rs:107 | `ShellError::ShellDisabled.to_string()` = `[shell] execute: shell access is disabled by configuration` | no | no | UF-DB | key at boundary | Req 9.3 display-boundary: do NOT change `ShellError`; route a fixed key here `...-shell-disabled` (DECISION D2) |
| external_adapter.rs:155 | `Some(e.to_string())` (external err) | n/a | no | UF-DB | leave | runtime spawn error |
| external_adapter.rs:176 | `'{}' finished (exit {}). See the Output panel.` | yes (program, code) | no | UF | `...-external-finished` | program(DATA)+code TEXT placeables |
| external_adapter.rs:185 | `Some(e.to_string())` (external err) | n/a | no | UF-DB | leave | runtime spawn error |

### 3l. `crates/ff-desktop/src/shell/kinds_editor.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| kinds_editor.rs:193 | `Some(e)` from `persist_kind_config` | yes | no | UF-DB | leave (pass-through) | `e` String from kind-config persistence; localise at source if ever |

### 3m. `crates/ff-desktop/src/shell/nav_stack.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| nav_stack.rs:297 | `Some(e.to_string())` | n/a | no | UF-DB | leave | nav error pass-through |

### 3n. `crates/ff-desktop/src/shell/render_body.rs` / `render_body_arms.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_body.rs:60 | `Some(err)` | yes | no | UF-DB | leave | `err` String produced upstream |
| render_body_arms.rs:102 | `Some(e)` | yes | no | UF-DB | leave | pass-through |

### 3o. `crates/ff-desktop/src/shell/render_command_line.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_command_line.rs:176 | `SCROLL: '{}' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)` | yes (text) | yes (SCROLL) | UF | `ff-desktop-shell-commandline-scroll-invalid` | the SCROLL error named in the brief; SHARE key with environment_ffedit.rs:281 |

### 3p. `crates/ff-desktop/src/shell/render_nav_ops.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| :84 | `New file/folder: {e}` | yes (e) | no | UF | `...-new-file-failed` | error placeable |
| :101 | `Create failed: {e}` | yes (e) | no | UF | `...-create-failed` | |
| :129 | `Paste: {e}` | yes (e) | no | UF | `...-paste-failed` | |
| :179 | `Paste: {errors} file(s) could not be copied` | yes (count) | no | UF | `...-paste-count-failed` | count TEXT placeable |
| :251 | `Delete: {e}` | yes (e) | no | UF | `...-delete-op-failed` | |
| :264 | `Delete failed: {e}` | yes (e) | no | UF | `...-delete-op-failed2` (or share w/ actions.rs:57) | |
| :337 | `Rename: {e}` | yes (e) | no | UF | `...-rename-failed` | |
| :344 | `Rename: {e}` | yes (e) | no | UF | `...-rename-failed` (share) | second branch, same text |
| :356 | `Rename failed: {e}` | yes (e) | no | UF | `...-rename-failed2` | |

### 3q. `crates/ff-desktop/src/shell/render_nav.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_nav.rs:160 | `Some(e)` | yes | no | UF-DB | leave | pass-through String |
| render_nav.rs:167 | `Some(e)` | yes | no | UF-DB | leave | pass-through |
| render_nav.rs:196 | `Some(e)` | yes | no | UF-DB | leave | pass-through |

### 3r. `crates/ff-desktop/src/shell/render_tab_bar.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_tab_bar.rs:216 | `Maximum 16 floating windows already open.` | no | no | UF | `ff-desktop-shell-tab-bar-max-floating` | static; target existing `ff-desktop-shell-tab-bar.ftl` |

### 3s. `crates/ff-desktop/src/shell/render_theme.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_theme.rs:72 | `Theme changed to {} for this session, but could not be saved: {e}` | yes (mode, e) | no | UF | `...-theme-mode-save-failed` | mode label + error placeables |
| render_theme.rs:127 | `Theme '{name}' applied for this session, but could not be saved: {e}` | yes (name, e) | no | UF | `...-theme-name-save-failed` | name(DATA)+error placeables |
| render_theme.rs:133 | `Theme '{name}' could not be loaded` | yes (name) | no | UF | `...-theme-load-failed` | name placeable |

### 3t. `crates/ff-desktop/src/shell/render.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render.rs:154 | `Some(e.to_string())` (file.open via ShellRequest::OpenFile) | n/a | no | UF-DB | leave | command-result text |
| render.rs:157 | `ShellRequest::Status(message)` | yes | no | UF-DB | leave (pass-through) | `message` built by the Context that emitted the request; localise at that Context |

### 3u. `crates/ff-desktop/src/shell/target_dispatch.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| target_dispatch.rs:153 | `Command '{}' is not defined.` | yes (id) | no (id is identifier DATA) | UF | `...-command-not-defined` | id is a command_id (stable identifier) -> non-translated DATA placeable (Req 4.2) |
| target_dispatch.rs:269 | `Custom workspace target '{other}' is not yet runnable via a command binding.` | yes (other) | no | UF | `...-custom-ws-not-runnable` | other=workspace_kind string (identifier) -> non-translated placeable |
| target_dispatch.rs (Macro arm) | `Macro targets are not yet runnable via a command binding.` | no | no | UF | `...-macro-not-runnable` | static |

### 3v. `crates/ff-desktop/src/shell/update.rs` / `update_startup.rs` / `update_dialogs.rs` / `update_dialogs_catalog.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| update.rs:97 | `Some(e)` | yes | no | UF-DB | leave | pass-through String |
| update_startup.rs:29 | `Some(e)` | yes | no | UF-DB | leave | pass-through |
| update_startup.rs:118 | `Workspace '{}' not found -- starting without workspace` | yes (path) | no | UF | `...-workspace-not-found` | path=DATA placeable |
| update_dialogs.rs:212 | `External execution cancelled.` | no | no | UF | `...-external-cancelled` | static (named in dialogs.ftl header as Task 9) |
| update_dialogs_catalog.rs:61 | `Some(e)` from `execute_delete` | yes | no | UF-DB | leave | pass-through |

### 3w. `crates/ff-desktop/src/shell/workspace_io.rs`

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| workspace_io.rs:42 | `Cannot open workspace: {e}` | yes (e) | no | UF | `...-ws-open-failed` | error placeable |
| workspace_io.rs:92 | `No active workspace to save` | no | no | UF | `...-ws-none-to-save` | static |
| workspace_io.rs:100 | `No workspace file path set -- use WORKSPACE SAVE AS <path>` | no | yes (WORKSPACE SAVE AS) | UF | `...-ws-no-path` | verb phrase English |
| workspace_io.rs:113 | `Cannot save workspace: {e}` | yes (e) | no | UF | `...-ws-save-failed` | error placeholder |

### 3x. Status bar -- `crates/ff-desktop/src/shell/render_status.rs` (target: existing `ff-desktop-shell-status.ftl`)

| file:line | current text | interp? | verb? | class | key | note |
|---|---|---|---|---|---|---|
| render_status.rs ~110 | phase labels `RUNNING` / `STARTING` / `SHUTTING DOWN` / `TERMINATED` | no | no (status mnemonics) | UF | `...-status-phase-running` etc (4 keys) | uppercase status mnemonics -- prose, translatable per Req 4.1 |
| render_status.rs ~137 | `Ln {line}, Col {col}` | yes (line,col) | no | UF | `...-status-line-col` | line,col TEXT placeables (byte-identical: pre-format, no NUMBER()) |
| render_status.rs ~156 | `{} lines` | yes (count) | no | UF | `...-status-line-count` | count TEXT placeable |
| render_status.rs ~166 | `CAPS` | no | no | UF | `...-status-caps` | mode indicator prose |
| render_status.rs ~198 | hover `Logging degraded: {reason}` | yes (reason) | no | UF | `...-status-log-degraded-hover` | reason=DATA placeable; the argument-bearing degraded tooltip the status .ftl header defers to Task 9 |

Note: `format_session_start()` ("Started: HH:MM"), `active_profile_label()`,
`encoding_label()`, zoom `text`, and the modified-dot glyph are DATA/already-owned
elsewhere; the modified dot `\u{25cf}` and `CAPS`/banner/`LOG!` glyph/banner are
handled (banner+LOG! done in Task 8). Only the five rows above are Task 9 here.

---

## 4. `thiserror` enums reaching a DISPLAY boundary (Req 9.3 -- localise at display, do NOT change the enum)

These error types' `Display` text can surface into `open_error`/UI via a
`.to_string()` pass-through. Per Req 9.3 the FIX is at the display SITE, keyed
through the seam; the enums themselves are LEFT UNCHANGED.

| Crate | Type | Reaches user via | Display shape | Recommendation |
|---|---|---|---|---|
| `ff-shell` | `ShellError` (`crates/ff-shell/src/error.rs`) | external_adapter.rs:107 (`ShellDisabled`), :155/:185 (`e.to_string()`) | `[shell] ...` dev-prefixed | Key the KNOWN fixed variant (`ShellDisabled`) at the boundary (external_adapter.rs:107). Runtime spawn errors (:155/:185) stay pass-through (INT-leaning UF-DB) -- content is OS/runtime text. |
| `ff-edit-operations` | `ProfileError` (`.../profile.rs`) | dispatch_ffedit.rs:283 (`e.to_string()`) | `[profile] ...` dev-prefixed | `Locked` is ALREADY intercepted and keyed (:281). For `UnknownKey`/`UnknownValue`, match at the boundary and route keys -- do NOT translate the `[profile]` Display. |
| `ff-edit-operations` | `EditError` (`.../error.rs`), `ProfilePersistError` (`.../profile_persistence.rs`) | indirectly via kinds/profile persistence pass-through sites | dev-prefixed | Keep pass-through for now; revisit only if a specific message becomes user-prominent. |
| (ff-desktop-local) | `shell_open_file -> Result<(), String>`, `CommandResult::Err(String)`, catalog/kind/search `Result<_, String>` | many `Some(e)` / `Some(e.to_string())` sites | plain String | These are NOT thiserror; the text is already composed in-crate. Localise at the STRING-PRODUCING site (the helper), not at the boundary. Listed as UF-DB "leave (pass-through)" above. |

Do NOT add `ff_i18n` as a dependency of the library error crates; the seam is
GUI-free but the enums must stay source-language per Req 9.3 / Req 4.

---

## 5. Command-ladder / dispatch `format!` error sites that render to the user

Covered in the tables above: `commands_ladder_a/b/c.rs`, `commands_fastpath.rs`,
`commands.rs`, `commands_scrm.rs`, `dispatch_ffedit.rs`, `target_dispatch.rs`.
The characteristic pattern is a verb-prefixed message (`SWAP:`, `WORKSPACE ...`,
`THEME:`, `HILITE:`, `PFSHOW`, `COMMAND:`): **the verb/keyword stays English**
(either as a literal retained INSIDE the English `.ftl` value, or as a
`{ $verb }` non-translated placeable -- see DECISION D1), and only the prose and
interpolated DATA localise.

---

## 6. Requirement 8 seam points (plugin UI + plugin command environment) -- READ-ONLY identification

- `crates/ff-i18n` exposes `t` / `t_args` publicly and is GUI-free (Req 3.5), so
  Req 8.1 (plugin-authored UI localises through the SAME seam) is satisfied by
  the seam being a normal public dependency a plugin crate can call. There is
  currently NO plugin-authored render surface in `crates/ff-plugin` (the
  `FileForgePlugin` trait in `traits.rs` has lifecycle + `CommandRegistration`
  but no `render(ui)` hook), so there is nothing to re-route -- this is a
  seam-AVAILABILITY item, not a code change.
- `crates/ff-desktop/src/plugin_manager_panel.rs` is the CORE plugin-manager UI
  and is ALREADY fully localised through `ff_i18n::t` (Task 8). It is not a
  Task 9 target.
- Req 8.2 (plugin-authored command environment, CR-CH-053 Req 9.4) reuses the
  EXISTING `AliasTable::overlay_locale` seam in
  `crates/ff-desktop/src/shell/environment.rs` -- a plugin environment's
  per-locale alias rows load into that environment's `AliasTable` with no new
  mechanism. No existing file needs editing to enable this; it is proven by the
  overlay seam already shipping.
- Req 8.3 (no new capability) is a non-code constraint.

Conclusion for Task 9.4: a DOCUMENTATION + VERIFICATION task (confirm the seam is
reachable and add a test that a mock plugin env's locale overlay resolves), not a
rewrite.

---

## 7. DECISION-NEEDED list (ambiguous sites -- owner call, with recommendation)

- **D1 -- verb retention style (verb-prefixed messages).** For messages like
  `SWAP: tab number ...`, `WORKSPACE OPEN requires a path`, `THEME: '{arg}' ...`:
  keep the verb as a LITERAL inside the English `.ftl` value, OR pass it as a
  `{ $verb }` non-translated placeable?
  **Recommendation:** keep the verb as a LITERAL in the English `.ftl` value for
  single-verb-prefix messages (simpler, byte-identical, verb never translated
  because it is baked into the English string and a translator is instructed not
  to touch it). RESERVE the `{ $verb }` placeable for the ONE case Req 9.2 calls
  out where the verb sits MID-SENTENCE and would move under translation
  (`Profile is locked -- use LOCK OFF to unlock`). This matches the Req 9.2
  example exactly and keeps the churn minimal.

- **D2 -- thiserror display-boundary keying depth.** `ShellError::ShellDisabled`
  and `ProfileError::UnknownKey/UnknownValue` currently surface their
  `[shell]`/`[profile]`-prefixed Display to the user.
  **Recommendation:** key the fixed-shape `ShellDisabled` at external_adapter.rs:107
  and match `ProfileError` variants at dispatch_ffedit.rs:283 to route clean
  keys. Leave truly open-ended runtime/OS error pass-throughs (`e.to_string()`
  from spawn/IO) as-is this gate -- their content is not a stable translatable
  message. (This splits ~26 UF-DB sites into ~4 "key at boundary" + ~22 "leave".)

- **D3 -- SCRM engine report text** (`scrm.status()/list()/purge()/export()` via
  commands_scrm.rs:102). These are multi-line report dumps from `ff-scrm`.
  **Recommendation:** classify UF-DB and DEFER (leave pass-through) this gate;
  they are diagnostic-report-shaped and low-value to translate now.

- **D4 -- `edit_profile.display_summary()`** (dispatch_ffedit.rs:264). A composite
  profile dump from `ff-edit-operations`.
  **Recommendation:** DEFER (leave) -- it is a structured key/value dump; localising
  it means threading the seam into the library's formatter, out of scope for a
  status-message gate.

- **D5 -- pass-through `Some(e)` String sites** (render_nav*, render_body*,
  update*, kinds_editor, nav_stack, actions.rs:84). The String is built upstream.
  **Recommendation:** leave at the boundary; if/when the UPSTREAM producers are
  extracted (future per-crate extraction work), the text localises there. Do not
  double-handle at the display boundary.

---

## 8. Totals per class (production sites)

- `open_error = Some(...)` production sites scanned: **82** (+1 test, excluded).
- **UF (own literal/format, route now):** ~48 sites.
- **UF-DB (display boundary):** ~26 sites -> recommended split: ~4 "key at
  boundary" (D2) + ~22 "leave pass-through this gate".
- **INT / DATA pass-through (leave):** the ~22 above behave as INT for THIS gate.
- Status bar (`render_status.rs`): **5** translatable rows (4 phase-label keys
  counted as one cluster + Ln/Col + line-count + CAPS + degraded hover).
- SCROLL invalid-amount: **2 sites sharing 1 key**
  (`render_command_line.rs:176`, `environment_ffedit.rs:281`).
- Req 8 (plugin seam): **0 code-change sites** (seam-availability + a verify test).

Net NEW translatable keys to author in `.ftl`: roughly **55-60** (48 own-literal
+ ~4 boundary + 5 status + 1 scroll shared + a few SCRM/capture), grouped into
the files below.

---

## 9. Recommended IMPLEMENTATION SEQUENCE for Task 9 (small per-area commits, Task 8 cadence)

Each numbered item = one small commit (one source file or one tight cluster +
its `.ftl` keys), compile-verified scoped (`cargo check -p ff-desktop`,
`cargo clippy -p ff-desktop`, `cargo fmt`) with the authoritative full gate left
to the owner (per testing.md execution split). Every commit must keep the
`ui.locale=en` rendering BYTE-IDENTICAL (TEXT placeables, no `NUMBER()`).

### Phase 9.2 -- interpolated (argument-bearing) messages, no embedded verb first
1. Create `ff-desktop-shell-status-messages.ftl`; wire the simplest file-op
   cluster: `render_nav_ops.rs` (9 sites) -- all `{e}`/count placeables.
2. `workspace_io.rs` (4 sites) + `commands_session.rs:278`.
3. `render_theme.rs` (3 sites) -- mode/name/error placeables.
4. `actions.rs` (4 own-literal sites: :53, :57, :112, :118).
5. `external_adapter.rs:176` + `update_startup.rs:118` + `commands_scrm.rs`
   capture/snapshot own-literals (:102 subset, :287, :325).

### Phase 9.2b -- status bar + SCROLL (own cluster, argument-bearing)
6. `render_status.rs` -- phase labels, `Ln/Col`, `{} lines`, `CAPS`, degraded
   hover -> existing `ff-desktop-shell-status.ftl`.
7. SCROLL invalid-amount shared key -> `ff-desktop-shell-commandline.ftl`,
   applied at BOTH `render_command_line.rs:176` and `environment_ffedit.rs:281`.

### Phase 9.3 -- verb-embedding messages + thiserror display boundary
8. Verb-prefixed command-ladder cluster A: `commands_ladder_c.rs` (18 sites --
   SWAP/SUBMIT/TIME/STATUS/CREATE/REPLACE/COMPARE/WORKSPACE/UNSPLIT... per D1).
9. Verb-prefixed cluster B: `commands_ladder_a.rs` (EDIT/DOCK),
   `commands_ladder_b.rs:53` (COMMAND position), `commands.rs:263` (THEME),
   `commands_fastpath.rs:102`, `render_tab_bar.rs:216`, `target_dispatch.rs`
   (3 sites), `update_dialogs.rs:212`.
10. The Req 9.2 canonical mid-sentence verb: `dispatch_ffedit.rs:281`
    (`Profile is locked -- use { $verb } ...`) + `:304` HILITE +
    `commands_scrm.rs:386` PFSHOW.
11. thiserror DISPLAY-BOUNDARY keys (D2): `external_adapter.rs:107`
    (`ShellError::ShellDisabled`) and `dispatch_ffedit.rs:283`
    (`ProfileError::UnknownKey/UnknownValue`) -- key at the boundary; enums
    unchanged.

### Phase 9.4 -- plugin seam (doc + verify, no rewrite)
12. Add a test proving a mock plugin command-environment's per-locale alias rows
    resolve through `AliasTable::overlay_locale`, and document that plugin UI
    calls `ff_i18n::t`/`t_args` directly. No existing file edited for behaviour.

### Deferred (owner decisions D3/D4/D5)
- SCRM engine report text, `display_summary()`, and upstream-built `Some(e)`
  pass-throughs: leave this gate; revisit under the per-crate extraction follow-on.

---

## 10. Conclusions

- Task 9 is well-bounded: ~48 clean own-literal `open_error` sites plus the 5
  status-bar rows and the shared SCROLL key are the real work; they all fit the
  existing `t`/`t_args` seam with TEXT placeables and need no framework change
  (Req 10.1).
- The ~26 display-boundary sites mostly stay pass-through this gate; only
  `ShellError::ShellDisabled` and the two `ProfileError` variants are worth keying
  at the boundary, honouring Req 9.3 (localise at display, not at source) without
  touching the error enums (Req 4.2).
- Verbs/keywords/identifiers (`SWAP`, `WORKSPACE`, `LOCK OFF`, command_ids,
  workspace_kind strings) stay English -- as baked-in literals in the English
  value, except the one Req 9.2 mid-sentence case which uses a `{ $verb }`
  non-translated placeable.
- Requirement 8 needs no code rewrite: the seam and the alias overlay already
  exist and are public; 9.4 is a verify/doc task.
- Byte-identical English rendering is preserved by pre-formatting all counts in
  Rust and passing them as TEXT placeables, matching the Task 8 convention
  recorded in the shipped `.ftl` headers.
