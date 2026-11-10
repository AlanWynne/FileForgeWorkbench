# POM snapshot review (owner, 2026-11-04, main a6aacae)

OWNER RULE: everything below that was found on POM applies to ALL Menu Kind
workspaces (POM, Settings, user menus) -- fix it generically on the shared menu
path, never as a POM-specific branch.

## Common-frame DEFECTS (CR-NR-115, on main) -- menu path not covered by goldens
1. Key line is ABOVE the status bar and INSIDE the content area with no rule.
   Required (Req 30 / 7): ... content, [message band], status band, key line LAST.
2. Command line shows the triggering command `ss` and the underscores stop short
   of the right border. Both were fixed for non-menu Contexts only
   (common_frame_command_field_text); the menu path still uses the old helper.
3. No golden test covers a Menu Context (POM). Add one.

## Common-frame DECISIONS (need owner answer)
- Zoom: recommend a COMMON status segment, always shown: `Zoom: 0`, `Zoom: +14`,
  `Zoom: -3` (check whether negative zoom exists). Owner suggested a bare number
  with F1 help on it; status segments are deliberately non-focusable (B055), so
  recommend hover tooltip / click-to-help instead of F1 focus. PENDING.
- Snapshot's own confirmation message ("Snapshot copied to clipboard: picture +
  Markdown.") in the message band: owner's expected layout shows it. It is
  produced after capture, so including it is self-referential. PENDING.

## Menu Kind items (for the Menu per-Kind gate, after Alias)
- Description repeats the command in brackets, e.g. `(Settings)`, not on screen:
  remove.
- Align the `:` column across options (pad option labels).
- Clipboard: add an `Option | Command | Description` markdown table for every
  Menu Kind snapshot. (Owner's example X row said `Settings`; assumed `X`.)
- Calendar column position per owner expected layout.

## Owner expected POM layout
(see the owner message of 2026-11-04; status band `Last | Prof | Zoom`, message
band with the snapshot confirmation, key line last)
