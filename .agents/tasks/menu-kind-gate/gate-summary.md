# Menu Kind gate summary (CR-NR-117) -- docs only, pending owner approval

Worktree: `.worktrees/menu-kind-gate`, branch `menu-kind-gate`. No `.rs` changed.
Id check: CR-NR-116 is used by `alias-kind-gate`; main's highest is CR-NR-115 /
CR-CH-115; CR-NR-117 is free in both.

## Criteria added

- `docs/specs/menu-workspace/requirements.md` Requirement 23, criteria 23.1-23.22:
  - 23.1-23.9 content-area rows (live displayed texts, no bracket suffix, aligned
    colon, X X kept, derived calendar column, group rules, no title repeat,
    load-error/empty).
  - 23.10-23.13 clipboard table (`Option | Command | Description` via
    `markdown_sections`, box then table, no calendar in table, single
    representation).
  - 23.14 status cells empty.
  - 23.15-23.18 built-in content (POM `S` description, SCRM `X` flag, optional
    description normalisation, code-only preserved).
  - 23.19-23.22 non-regression, no framework type change, full-text goldens,
    placement unchanged.
- `docs/specs/screen-snapshot-scrm/requirements.md` criterion 30.31 (renumbered from 30.29 at rebase; shared:
  per-Kind content owned by the Kind spec; per-Kind table headers).
- design.md: section "Menu Kind snapshot content"; tasks.md: Tasks 52-55.
- change-log.md: CR-NR-117 entry.

## Investigation verdicts

1. `(<command>)` suffix: snapshot-only, `screen_snapshot.rs::option_row_text`
   (tests encode it). Live panel has no bracket and no colon.
2. Alignment: snapshot pads nothing; live pads key (4) and command (widest, 4..24).
3. Calendar: `compose_two_column_body` gap 3; live gap 32 px (about 4 cells).
   Recommend derived column = widest padded row + 4.
4. Clipboard table: missing (builder sets only body_lines); add via
   `markdown_sections`; renderers already take arbitrary headers.
5. Single representation: build rows once; fields/tables stay empty.
6. Status cells: `workspace_status_cells` already empty for Menu; keep.
7. Oddities: `S SCRM : SCRM :` is TOML CONTENT (POM `S` description starts
   `SCRM :`, no catalogue override, so live shows it too); `X X` is correct data
   (key X, command X; owner's `Settings` in the X row was a slip). Mixed styles
   are content. SCRM `X` lacks `show_in_menu_bar=false`. Settings `W` and SCRM
   1-2 letter keys unchanged; colon alignment absorbs them.
- Everything is generic: one provider for POM, Settings, SCRM and user menus.
  Settings differs only by data (`show_calendar=false`, group line separator).

## Framework flag

NONE. Built on common frame (mechanism 8), `markdown_sections`/`Table`
(CR-CH-079), code-only built-ins (mechanism 4, content edit), existing status
cells. Additive crate-private items only: `MenuRowDisplay`, `rows` on
`MenuScreenChrome`, sibling `screen_snapshot_menu.rs`. Live render untouched.

## Decision points and recommendations

- D1 colon row form: keep owner's form.
- D2 table header: `Description`.
- D3 calendar column: derived (widest row + 4 cells), not a fixed column.
- D4 group rules in the box: yes (Settings has a Core/Recovery line); not in table.
- D5 `X X`: show both columns.
- D6 status cells: none.
- D7a POM `S` description: change to `Screen Capture and Recording`.
- D7b full description normalisation: Option 1 (D7a only) now; Option 2 table of
  proposed texts for every option is in design.md; touches ff-i18n catalogues.
- D8 SCRM `X` `show_in_menu_bar=false`: yes.
- D9 placement (Settings in place, SCRM/user menus new tab): not changed here;
  separate CR to make all in place (framework mechanism 3).
- D10 calendar always shown when `show_calendar`: keep (logical screen).
- D11 table Command cell: displayed surface (equals live and box).

## Tests planned

Full-text goldens (box + Markdown flavor) for POM, Settings, SCRM and a user
menu; builder unit tests; status-cell and no-fields/tables guards; displayed
surface fidelity test. Agents compile-verify only; owner runs the full gate.

GATE_READY_FOR_APPROVAL
