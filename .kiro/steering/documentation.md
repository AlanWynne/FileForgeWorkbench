---
inclusion: always
---

# Documentation Character Set -- MANDATORY

## Scope
- All files under `docs/` (Markdown and plain text)
- All steering files under `.kiro/steering/` and any rules under `.amazonq/rules/`
- All Rust source files under `crates/` (`.rs`)

Rules differ slightly between Markdown/text and Rust source -- see below.

## Markdown and Text Files

### Allowed non-ASCII
Box-drawing characters (U+2500-U+257F) are ALLOWED in Markdown. They render
correctly in VS Code, GitHub, and UTF-8 terminals, and appear only in fenced
code blocks or ASCII-art diagrams, never in prose that automated tools must
pattern-match. The TCR status emoji (checkmark, cross, white square, red circle)
are also allowed in `docs/quality/TCR.md` tables.

All other non-ASCII characters are PROHIBITED unless listed as allowed above.

### Prohibited characters
These cause silent pattern-matching failures in automated editing tools (the
tool cannot find `oldStr` when the editor has auto-substituted the character):

| Unicode | Name | Substitute |
|---------|------|------------|
| U+2014 | em dash | `--` or `: ` |
| U+2013 | en dash | `-` |
| U+2018 | left single quote | `'` |
| U+2019 | right single quote | `'` |
| U+201C | left double quote | `"` |
| U+201D | right double quote | `"` |
| U+2026 | ellipsis | `...` |
| U+FEFF | BOM / zero-width no-break space | remove |
| U+00E2 | a-circumflex (BOM artefact) | remove |
| U+2200 | for all | `for all` |
| U+2227 | logical AND | `AND` |
| U+2228 | logical OR | `OR` |
| U+2208 | element of | `in` |
| U+2209 | not element of | `not in` |
| U+2264 | less-than or equal | `<=` |
| U+2265 | greater-than or equal | `>=` |
| U+2192 | rightwards arrow | `->` |
| U+2194 | left-right arrow | `<->` |
| U+21A6 | maps to | `->` |
| U+00AC | not sign | `NOT` |
| U+2260 | not equal | `!=` |
| U+2203 | there exists | `exists` |

### Substitution rules
- Em dash: `--` or `: ` by context. En dash: `-`.
- Curly single/double quotes: straight `'` / `"`. Ellipsis: `...`.
- BOM bytes at file start: remove entirely.
- Math/logic symbols: use the ASCII substitutes above. In correctness-property
  sections rewrite inline: `for all x in S:`, `AND`/`OR`/`NOT`, `->`, `<=`/`>=`/`!=`.

## Rust Source Files
Plain ASCII only (0x00-0x7F). Box-drawing is NOT allowed in `.rs` files.
Section separators use ASCII only:
```rust
// === Section Name ===================================================
```

## Enforcement
```bash
# Markdown/text (allows box-drawing, flags everything else)
rg "[^\x00-\x7F\u2500-\u257F]" docs/ .kiro/steering/ .amazonq/rules/ --glob "*.md"

# Rust source (strict ASCII only)
rg "[^\x00-\x7F]" crates/ --glob "*.rs"

# Common offenders (em dash, curly quotes, math symbols)
rg "[\u2013\u2014\u2018\u2019\u201C\u201D\u2026\u2200\u2227\u2228\u2192\u2264\u2265]" docs/ .kiro/steering/ --glob "*.md"
```
Any match (outside allowed ranges) is a violation. Fix with the ASCII substitute.

## Why these characters
1. **Editor auto-substitution** (em dash, curly quotes, ellipsis): editors
   silently replace typed characters with typographic equivalents; automated
   `oldStr` matching then fails. This is the primary source of tool failures here.
2. **Math/logic symbols** (arrows, quantifiers, comparisons): appear in
   correctness-property sections and cause the same failures, plus inconsistent
   rendering in terminals without full Unicode math support.
3. **Box-drawing is allowed** because it appears only in fenced code blocks and
   diagrams, never in searchable prose, and renders correctly everywhere.