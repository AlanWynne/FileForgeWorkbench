# Alias Editor concerns (owner, 2026-11-01)

Captured from the owner's `Alias-Editor-concerns.md` (originally written inside
the fix-alias-line-commands worktree; moved here so it survives worktree cleanup).

## The live panel vs the markdown snapshot disagree

The Alias editor renders differently in the live panel than in the markdown
snapshot taken by the ScreenProvider.

### Owner's "actual window" sketch (live panel -- has DUPLICATED chrome)

```
+--------------------------------------------------------------+
| # | FileForge Workbench                               _ [] X |
+--------------------------------------------------------------+
| [ALIASES] |                                                  |
+--------------------------------------------------------------+
|                    Alias Editor                              |   <- title line 1 (context)
+--------------------------------------------------------------+
| Command ===> _______________________________________        |   <- command line 1
+--------------------------------------------------------------+
|           FileForge Workbench Alias Editor                   |   <- title line 2 (heading) REDUNDANT
+--------------------------------------------------------------+
| Command ===> _______________________________________        |   <- command line 2 REDUNDANT
+--------------------------------------------------------------+
| Act | CE    | Command   | Alias | Parameters | Description   |
| ... | FFCMD | SNAPSHOT  | SS    | &1         | note          |
| ... | ...   | ...       | ...   | ...        | note          |
+--------------------------------------------------------------+
```

### Owner's EXPECTATION (single Title line + single Command line)

```
+--------------------------------------------------------------+
| # | FileForge Workbench                               _ [] X |
+--------------------------------------------------------------+
| [ALIASES] |                                                  |
+--------------------------------------------------------------+
|                    Alias Editor                              |   <- ONE title line
+--------------------------------------------------------------+
| Command ===> _______________________________________        |   <- ONE command line
+--------------------------------------------------------------+
| Act | CE    | Command   | Alias | Parameters | Description   |
| ... | FFCMD | SNAPSHOT  | SS    | &1         | note          |
| ... | ...   | ...       | ...   | ...        | note          |
+--------------------------------------------------------------+
```

### Current markdown snapshot (MISSING header + command line)

```
+----------------------------------------------------+
| Alias Editor                                       |
+----------------------------------------------------+
| SS . . . : SNAPSHOT                                |
| Aliases                                            |
| Act  CE  Command  Alias  Parameters  Description   |
|    ffcmd  SNAPSHOT  SS                              |
| ignored line command 'RR' (not enabled); ...       |
+----------------------------------------------------+
```

## What is wrong (owner's list)

1. Extra Title line is redundant -- rendered in the actual panel.
2. Extra Command line is redundant -- rendered in the actual panel.
3. Header and command line are MISSING from the markdown snapshot/explorer.
4. The Down arrow when focused on the Act field does not move focus to the row
   below; Up/Down (and Left/Right) arrows do not change focus between fields at
   all.

## Owner clarification on block line commands (2026-11-01)

RR/RR, CC/CC (and by extension DD/DD, MM/MM) are the ISPF BLOCK variants of the
single-letter line commands: a doubled letter on two rows marks an inclusive
range and the operation applies to the whole block. They should be supported in
all ISPF-style screens. In screens like the Alias editor they are not very
useful, but the PATTERN should support them uniformly. The pattern we build must
be reusable/copyable into other workspaces (Menu, Kinds, future ISPF screens).

Owner confirmed scope:
- Block set = RR / DD / CC / MM (no II).
- Arrow navigation = full: Up/Down between rows AND Left/Right between columns.
- Single R / Rn remain single-row repeat / repeat-n; RR...RR is the separate
  block form. (The `RR` in the current snapshot was rejected as "not enabled" --
  that is the bug: block commands are not yet recognised.)

## Resulting change requests (to be gated before code)

- CR-a: Block line commands (RR/DD/CC/MM range pairs) in the shared
  ispf_table_editor, reusable across ISPF workspaces. Unterminated block = a
  non-blocking "block not closed" notice, no structural change. CC/MM blocks
  still need an A/B destination. Advertised per-host via LineCommandSet. Confirm
  whether ff-line-commands already parses the doubled block form / paired ranges
  (the text editor uses block commands today) before extending its API.
- CR-b: Single-chrome layout -- the shared component owns ONE Command ===> line +
  header; the Alias Context stops drawing its own duplicate title/command line;
  the snapshot ScreenProvider renders the SAME single-chrome layout so live and
  snapshot agree (fixes concerns 1-3).
- CR-c: Full arrow-key focus navigation (Up/Down rows, Left/Right columns) owned
  by the shared component so all hosts inherit it (fixes concern 4).

Decision (owner-approved): bank the already-approved in-cell-Enter fix
(branch fix-alias-line-commands, commit cc1a4bc) to main FIRST, then gate and
build CR-a/b/c as a fresh piece.
