# New requirement: snapshot clipboard flavors each carry BOTH box-drawn + property/value tables

Owner-confirmed 2026-11-02. Screen-snapshot clipboard (SCRM) area.

## Current behaviour (owner-observed, Config snapshot)
- Paste into Word (rich/HTML+image target): PNG image -- WORKING; box-drawn
  layout (frame + data) -- WORKING; property/value tables -- MISSING.
- Paste into Notepad (plain-text target): property/value markdown tables --
  WORKING; box-drawn layout -- MISSING.

So each flavor currently carries ONE of the two text representations; each is
missing the other.

## Desired end state (symmetric -- each flavor carries BOTH representations)
- Word / rich (HTML + image): PNG image + box-drawn layout + property/value
  tables (as HTML tables, like the markdown tables render).
- Notepad / plain-text: box-drawn layout + property/value tables (as
  markdown-style tables).

Common new requirement: the clipboard's text content (both the rich/HTML flavor
and the plain-text flavor) must contain BOTH the box-drawn layout AND the
property/value pairs rendered as tables. The rich flavor also keeps the PNG
image (already working). Nothing about the image changes.

## To settle during the gate (investigation, do not assume)
- How the current clipboard FLAVORS are structured (CF_DIB image, CF_HTML,
  CF_UNICODETEXT/plain) and which renderer emits box vs tables today -- so we add
  the missing representation to the right flavor rather than duplicating.
- Ordering within a single paste: box-first-then-tables or tables-first-then-box,
  and how box + tables sit together in the HTML flavor. Record the chosen order
  as a criterion.
- Whether this is Config-specific or applies to ALL screen snapshots that have
  both a box layout and tabular content (prefer the general rule, in the shared
  screen-model/clipboard path, so all editors inherit it -- consistent with the
  reusability principle).

## Process
- New requirement -> requirements gate (workflow.md) BEFORE code. Docs only:
  criteria (EARS, ASCII), design (flavor structure + ordering), tasks (TDD-first).
- Area: docs/specs/screen-snapshot-scrm/ (+ whatever clipboard/screen-model spec
  owns the flavors). Log the CR in docs/status/change-log.md with the next id.
- Build ON existing SCRM clipboard flavors; no new framework mechanism unless the
  gate surfaces one for explicit owner confirmation.
