# Alias snapshot fidelity gap (owner, 2026-11-02)

Owner compared the CURRENT Alias snapshot (main 8e40709, BEFORE the unmerged
ispf-editor-impl single-chrome branch lands) against the expected fuller layout.

## What we get now (main, thin snapshot -- pre CR-CH-077 merge)
```
# Alias Editor
+--------------------------------------------------+
| Alias Editor                                     |
+--------------------------------------------------+
|                                                  |
| SS . . . : SNAPSHOT                              |
|                                                  |
| Aliases                                          |
| Act  CE  Command  Alias  Parameters  Description |
|   ffcmd  SNAPSHOT  SS                            |
+--------------------------------------------------+
```

## What the owner EXPECTS (fuller, faithful to the live panel)
```
+----------------------------------------------------------------------------------+
|# | FileForge Workbench                                                  _ [] X |
+----------------------------------------------------------------------------------+
| [ALIASES] |                                                                      |
+----------------------------------------------------------------------------------+
|                    Alias Editor                                                  |
+----------------------------------------------------------------------------------+
| Command ===> ___________________________________________________                 |
+----------------------------------------------------------------------------------+
|                                                                                  |
| Act  | CE     | Command          | Alias            | Parameters      | Description |
|------|[---- ]v|[---------------]v|------------------|-----------------|------------ |
| ____ | FFCMD_ | SNAPSHOT         | SS               | &*              | note        |
| ____ | ____   | ________________ | ________________ | _______________ | note        |
+----------------------------------------------------------------------------------+
```

## Gap analysis vs the approved ispf-editor-impl branch (CR-CH-076/077/078)
The branch (reviewed/approved, NOT yet merged) DELIVERS:
- Single chrome: one "Alias Editor" title (no duplicate).
- The `Command ===>` line AND the column header now appear in the snapshot
  (Req 10.3 / 7.12).

The branch likely does NOT yet deliver (owner's expected sketch is richer):
1. The app WINDOW title bar line ("FileForge Workbench ... _ [] X") in the snapshot.
2. The `[ALIASES]` tab-tag line in the snapshot.
3. Column-rule `|` separators between columns.
4. The dropdown-affordance row (`[---- ]v` / `[-----------]v`) under the header for
   the CE and Command combo columns.
5. Fixed-width field cells (aligned underscored blanks) for each column.
6. The always-present empty scratch row at the bottom.
(2-6 are snapshot-fidelity: how faithfully the ScreenProvider reproduces the live
panel.)

## Decision needed
- Sequencing: merge the approved branch FIRST (single-chrome + command-line-in-
  snapshot), then compare the real merged snapshot to this sketch and log the
  residual gap. The residual gap (window chrome, tab tag, separators, dropdown
  affordances, fixed-width cells, scratch row) is a WORKSPACE-MODEL / snapshot-
  fidelity question and likely belongs in the upcoming workspace-architecture
  alignment pass rather than a one-off Alias patch.
- Also note: `&*` vs `&1` in Parameters is data-dependent (owner's live alias
  value), not a rendering bug.

## Blocking item
Owner still needs to run the full `cargo gate` on
.worktrees/ispf-editor-impl before the branch can merge.

## UPDATE 2026-11-03 -- confirmed with a real Notepad paste (post ISPF merge, main 9a75275)
The clipboard CR-CH-079 duality WORKS: the Notepad paste now has the box block
FIRST then the property/value markdown table. So box+tables ordering is correct.

The REMAINING problem is the FIDELITY of the box-drawn block itself -- the Alias
ScreenProvider emits a THIN/COLLAPSED box, not a faithful mirror of the live
panel. Confirmed missing vs the owner's expected box:
- Window title-bar line ("# | FileForge Workbench ... _ [] X") -- MISSING.
- "[ALIASES]" tab-tag line -- MISSING.
- "Alias Editor" title -- present.
- Command ===> line INSIDE the box -- present but at the BOTTOM ("Command ===> ss"),
  owner expects it near the TOP under the title.
- Column header with "|" separators -- MISSING (columns are space-runs).
- Dropdown-affordance row ("[---- ]v" / "[----------]v" for CE and Command combos)
  -- MISSING.
- Fixed-width underscored field cells (____, FFCMD_, etc.) -- MISSING (bare values).
- Empty scratch row at bottom -- MISSING.
- Full panel width -- MISSING (box is narrow).
- EXTRA/odd: current box shows "SS . . . : SNAPSHOT" (ISPF dot-leader prompt
  style) which is NOT in the expected grid layout.

So the Alias box-drawn snapshot does NOT reproduce the live grid. This is a
WORKSPACE-MODEL / snapshot-fidelity decision set (what chrome belongs in a box
snapshot: window bar? tab tag? title? command-line position? is the box a
faithful grid mirror of the live widget layout with column rules, dropdown
affordances, blank cells, scratch row?) -- it should be decided ONCE in the
workspace-architecture alignment and applied to every workspace (Alias, Menu,
Kinds, Config, ...), NOT patched for Alias alone.

RECOMMENDATION: fold this into the workspace-architecture alignment pass rather
than a one-off Alias ScreenProvider patch.
