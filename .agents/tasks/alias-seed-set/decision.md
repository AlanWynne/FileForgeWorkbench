# Alias seed-set expansion -- owner decision 2026-11-05 (FINAL)

Follow-up to the config-model change CR-CH-116. Today Reset bare rebuilds the
FFCMD alias seed with ONLY X -> EXIT, so a user's SCRM shortcut aliases (SS etc.)
vanish on Reset bare. The owner wants the SCRM shortcut set to be built-in seed
aliases that are written on first run and restored by Reset bare.

## The seed set (X + the four SCRM shortcuts), all FFCMD, en seed
| surface | canonical (command) | description |
|---------|---------------------|-------------|
| X  | EXIT          | Exit the application          (already shipped, CR-CH-116) |
| SS | SNAPSHOT      | Screen snapshot to clipboard  |
| CS | CAPTURE START | Start screen capture          |
| CT | CAPTURE STOP  | Stop and save screen capture  |
| CR | CAPTURE REPLAY| Replay a saved capture        |

(Descriptions are the orchestrator's proposed wording; owner may tweak.)

## Scope (small, additive to CR-CH-116)
- Add the four SCRM shortcut rows (SS/CS/CT/CR) with descriptions to:
  1. the FIRST-RUN write seed (the Config_Rebuild_Helper default for FFCMD.toml), and
  2. the RESET-BARE rebuild (same helper, same source).
- The rows are normal editable user-override rows (visible in the Alias editor,
  restorable by Reset bare) -- same treatment as the X -> EXIT seed.
- Update the X-only seed tests/goldens to the five-row set (first-run writes 5
  rows; Reset bare restores 5; the Alias editor default count becomes Aliases: 5).
- FFCMD environment, aliases/<locale>/FFCMD.toml, locale = active ui.locale (en default).

## NOT in scope
- No framework change (additive seed rows only; same write-through model).
- Does NOT touch the Alias display implementation (alias-kind-impl) -- that renders
  whatever aliases exist; this just changes which aliases ship.
- Localised seed sets for non-en locales are out of scope here (en only, matching
  the current X->EXIT seed).

## Sequencing
Author as its own small gated change in the config-model area AFTER the Alias
display implementation (alias-kind-impl) merges, to avoid two branches editing the
alias area at once. Owner noted SS currently missing after a Reset bare; until this
lands, the user can re-add SS by hand in the Alias editor (persists as a user alias).
