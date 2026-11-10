# New requirement: every screen snapshot must include the window title bar + tab-tag line

Owner-confirmed 2026-11-03. SCRM / screen-snapshot area. GENERAL -- applies to
ALL screen snapshots (every workspace/Context), not just Alias.

## Statement
Every screen snapshot's box-drawn layout MUST always begin with, at the top:
1. The application WINDOW TITLE BAR line, e.g.:
   `# | FileForge Workbench                                      _ [] X`
2. The TAB-TAG line for the active workspace, e.g. `[ALIASES]` (and `[CONFIG]`,
   `[THEME]`, etc. -- the active Context's `[XXX]` tab tag, per CR-NR-090 /
   CR-CH-045 tag-vs-title distinction).

Currently NO snapshot produces either of these lines. They are app-level chrome
that frames every workspace, so the snapshot must reproduce them for every
Context.

## Observed (Alias snapshot, main 9a75275)
Box starts straight at the "Alias Editor" title -- no window title bar, no
"[ALIASES]" tag line. Same absence expected for every other Context's snapshot.

## Notes / to settle during the gate
- This is GENERAL: implement in the shared snapshot/box renderer so every
  ScreenProvider/Context inherits the two header lines, NOT per-editor.
- Exact text/format of the title bar line (app name, the window-control glyphs
  "_ [] X") and the tag line -- confirm the canonical rendering (ASCII-only per
  documentation-charset.md; the "_ [] X" are plain ASCII).
- Where the active tab tag comes from (the Workspace Kind / tab-header tag,
  CR-NR-090 / CR-CH-045) so the correct "[XXX]" is emitted per Context.
- Which flavors: the box-drawn block appears in BOTH the plain-text (Notepad) and
  the rich/HTML (Word) flavors per CR-CH-079, so the two header lines ride along
  in both wherever the box is rendered.
- This is part of the broader snapshot-FIDELITY gap (see
  alias-editor-concerns/snapshot-fidelity-gap.md): the box also misses column
  separators, dropdown affordances, blank/underscored cells, scratch row, full
  width, and places the Command ===> line at the bottom instead of the top. The
  title-bar + tag lines are the FIRST, clearly-scoped, general slice of that gap.

## Process
- New requirement -> requirements gate before code. General, in screen-snapshot-scrm.
- Strong candidate to be folded into / sequenced with the WORKSPACE-ARCHITECTURE
  ALIGNMENT (one model for chrome/title/command-line/snapshot across all
  workspaces) the owner asked for before building the next major plugins.
