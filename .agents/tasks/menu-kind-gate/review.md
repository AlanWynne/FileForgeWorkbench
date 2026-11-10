# Review: menu-kind-gate (CR-NR-117) -- APPROVED

Evidence checked: `git diff main...HEAD` (5 docs files, 585 insertions), gate-summary.md,
alias-kind-gate change-log, and spot-checks of the cited code.

1. Layout and table match. Req 23.2-23.6 give `<key> <command> : <description>`, no bracket
   suffix, aligned colon, `X X` kept, calendar at widest row + 4. 23.10-23.13 give the
   Option | Command | Description table after the box, with no calendar or group rows.
   One owner-mock inconsistency (box row `SCRM : SCRM :`, table `Screen Capture and
   Recording`) is resolved by D7a, which edits the POM `S` description so box and table agree.
2. Generic: 23 scope text forbids POM/Settings/SCRM/is_home branches. Settings differs only
   by data (`show_calendar=false`, group line).
3. Id: CR-NR-117 has zero hits on main and on alias-kind-gate (grep returned none);
   alias-kind-gate uses CR-NR-116. Unique.
4. Claims verified: `screen_snapshot.rs::option_row_text` emits `{key} {command} : {description}
   ({command})`; `compose_two_column_body` uses `MENU_COLUMN_GAP`; `defaults.rs` POM `S`
   description is `SCRM : Screen Capture and Recording` (no catalogue key);
   `workspace_status_cells` exists in render_status.rs and common_frame.rs consumes it.
5. Docs only (no .rs in diff). Added lines are ASCII (the only non-ASCII hit, a check mark in
   tasks.md line 513, is pre-existing). Criteria use EARS. Tasks use only [ ]/[x] with titles;
   task 52 (red tests and full-text goldens for POM, Settings, SCRM, user menu in box and
   Markdown flavors) precedes 53 implementation. Verification is compile-only for agents.
6. Framework verdict (none; additive crate-private items) is sound. D1-D11 each carry a
   recommendation. Summary ends with GATE_READY_FOR_APPROVAL.

No blockers.
