# Alias Workspace Kind gate -- owner FINAL decisions (2026-11-05)

All seven gate decisions settled, plus the CE-case fix and the seed scope.

## The seven decisions (a)-(g)
- (a) Combo cells: YES. CE and Command render as `[value v]` per cell, empty combo `[ v]`, plain header; empty text field `____`.
- (b) Filter row: YES. Omitted from the snapshot; an active filter shows as a `Filter: <ce>/<root>` status cell.
- (c) Grid-in-frame: YES, with the normative PRINCIPLE: the FRAME owns the band border; the Context does NOT draw or overwrite it. The Context DECLARES its column offsets; the frame renders the border rule AND the column-junction glyphs (tee-down/cross/tee-up) at those offsets. Top content rule carries the grid's column junctions; the rule above the status bar carries the status cells' junctions (they differ in x-position, which is correct). Same mechanism already used for the status-bar cells (CR-NR-115). A Context never gains border control -- it only contributes column positions. Write this as the normative rule so Menus/Kinds follow it.
- (d) Status segment: YES, wording `Aliases: n | Rows 1-m visible`, appended AFTER the common `Last | Prof | Zoom`. "visible" = the filtered set.
- (e) Value fidelity: mirror stored values; `(&1)`/`note` are hint placeholders, empty stays `____`. (Settled earlier.)
- (f) Scratch row: YES, ALWAYS show one trailing empty row (live AND snapshot). Shared-component change -> Menus/Kinds inherit it.
- (g) Clipboard: YES, box first then ONE `### Aliases` markdown table; no Property/Value table.

## CE case (from ce-case-investigation.md)
DISPLAY-ONLY fix: CE is already stored canonically UPPERCASE (FFCMD.toml/FFEDIT.toml), matching is case-insensitive, and CE+Command are already constrained dropdowns. Three render sites show `env.label()` (lowercase) instead of `env_name()` (uppercase). Fix: display `FFCMD`/`FFEDIT` (uppercase) in the editor combo (rows.rs), the snapshot table (screen_providers_tables.rs), and the filter host (host.rs). Storage and dropdown constraints already correct.

## Seed row
- Exactly one English seed TODAY per CR-CH-116: `X -> EXIT`, Description = "Exit the application". CONFIRMED, editable normal row, restored by Reset bare.
- SS is the owner's user alias.

## Seed SCOPE decision: OPTION B (owner 2026-11-05)
Ship a deliberate built-in seed alias set, all with descriptions, all restored by Reset bare:
- X  -> EXIT           "Exit the application"
- SS -> SNAPSHOT       (SCRM snapshot shortcut -- description TBD, e.g. "Screen snapshot to clipboard")
- CS -> CAPTURE START  (e.g. "Start screen capture / recording")
- CT -> CAPTURE STOP   (e.g. "Stop and save screen capture")
This EXTENDS the CR-CH-116 seed (currently X->EXIT only) and what Reset bare rebuilds.

## SEQUENCING
1. Alias DISPLAY gate (CR-NR-116, this gate): merge docs + implement the rendering (a)-(g) + CE uppercase fix. Does NOT change the seed set.
2. Seed-set change (small follow-up to CR-CH-116): add SS/CS/CT seed rows + descriptions to the first-run write + Reset-bare rebuild. Its own small gated change in the config-model area, NOT folded into the Alias display gate. Confirm the exact SCRM descriptions when authoring.
Note: with Option B, the Alias editor's default count becomes 4 seeds (X/SS/CS/CT) once the seed-set change lands; the Alias display gate's fixtures use the CURRENT seed (X->EXIT) + SS-as-user-alias until then. The gate fixture count stays correct for its own point in time; the seed-set change updates fixtures when it lands.
