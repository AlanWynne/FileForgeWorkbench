# Documentation Character Set -- MANDATORY

## Scope

This rule applies to:
- All files under `docs/` (Markdown and plain text)
- All files under `.amazonq/rules/` (Markdown)
- All Rust source files under `crates/` (`.rs`)

The rules differ slightly between Markdown/text files and Rust source files -- see below.

---

## Markdown and Text Files (`docs/`, `.amazonq/rules/`)

### Allowed non-ASCII

Box-drawing characters (U+2500-U+257F) are ALLOWED in Markdown files.
They render correctly in VS Code, GitHub, and UTF-8 terminals, and they
appear only in fenced code blocks or ASCII-art diagrams -- never in prose
that automated editing tools need to pattern-match.

All other non-ASCII characters are PROHIBITED unless listed as allowed above.

### Prohibited characters

The following characters are explicitly prohibited because they cause
silent pattern-matching failures in automated editing tools (the tool
cannot find `oldStr` when the editor has auto-substituted one of these):

| Character | Unicode | Common name | Substitute |
|-----------|---------|-------------|------------|
| `\u2014` | U+2014 | em dash | `--` or `: ` |
| `\u2013` | U+2013 | en dash | `-` |
| `\u2018` | U+2018 | left single quote | `'` |
| `\u2019` | U+2019 | right single quote | `'` |
| `\u201C` | U+201C | left double quote | `"` |
| `\u201D` | U+201D | right double quote | `"` |
| `\u2026` | U+2026 | ellipsis | `...` |
| `\uFEFF` | U+FEFF | BOM / zero-width no-break space | remove |
| `\u00E2` | U+00E2 | a-circumflex (BOM artefact) | remove |
| `\u2200` | U+2200 | for all (math) | `for all` |
| `\u2227` | U+2227 | logical AND | `AND` |
| `\u2228` | U+2228 | logical OR | `OR` |
| `\u2208` | U+2208 | element of | `in` |
| `\u2209` | U+2209 | not element of | `not in` |
| `\u2264` | U+2264 | less-than or equal | `<=` |
| `\u2265` | U+2265 | greater-than or equal | `>=` |
| `\u2192` | U+2192 | rightwards arrow | `->` |
| `\u2194` | U+2194 | left-right arrow | `<->` |
| `\u21A6` | U+21A6 | maps to | `->` |
| `\u00AC` | U+00AC | not sign | `NOT` |
| `\u2260` | U+2260 | not equal | `!=` |
| `\u2203` | U+2203 | there exists | `exists` |

### Substitution rules

- Em dash: use `--` (double hyphen) or `: ` (colon-space) depending on context.
- En dash: use `-` (hyphen-minus).
- Curly apostrophe or single quote: use `'` (straight apostrophe).
- Curly double quote: use `"` (straight double quote).
- Ellipsis: use `...` (three full stops).
- BOM bytes at the start of a file: remove entirely.
- Math/logic symbols: use the ASCII substitutes in the table above.
  In correctness property sections, rewrite inline:
  - `for all x in S:` instead of `forall x in S:`
  - `AND`, `OR`, `NOT` instead of symbols
  - `->` instead of arrow symbols
  - `<=`, `>=`, `!=` instead of Unicode comparison symbols

---

## Rust Source Files (`crates/`)

Rust source files MUST use plain ASCII only (code points 0x00-0x7F).
Box-drawing characters are NOT allowed in Rust source files.

Section separator comments MUST use plain ASCII only:

```rust
// === Section Name ===================================================
```

Do NOT use Unicode box-drawing characters (`--`, `-`) in Rust source separators.

---

## Enforcement

### Check Markdown/text files (allows box-drawing, flags everything else)

```bash
rg "[^\x00-\x7F\u2500-\u257F]" docs/ .amazonq/rules/ --glob "*.md"
```

Any match is a violation. Fix by replacing with the ASCII substitute above.

### Check Rust source files (strict ASCII only)

```bash
rg "[^\x00-\x7F]" crates/ --glob "*.rs"
```

### Quick check for the most common offenders (em dash, curly quotes, math symbols)

```bash
rg "[\u2013\u2014\u2018\u2019\u201C\u201D\u2026\u2200\u2227\u2228\u2192\u2264\u2265]" docs/ .amazonq/rules/ --glob "*.md"
```

---

## Why These Specific Characters Are Prohibited

The prohibited characters fall into two categories:

**Category 1 -- Editor auto-substitution (em dash, curly quotes, ellipsis):**
Text editors and word processors silently replace typed characters with
typographic equivalents. When an automated editing tool later tries to
find that text using `oldStr` pattern matching, it fails because the
stored text contains the substituted character, not the typed one.
This is the primary source of tool failures in this project.

**Category 2 -- Math/logic symbols (arrows, quantifiers, comparisons):**
These appear in correctness property sections and formal specifications.
They cause the same pattern-matching failures as Category 1 when they
appear in `oldStr` blocks. They also render inconsistently in terminals
that do not fully support Unicode mathematical blocks.

**Box-drawing is allowed because:**
Box-drawing characters appear only in fenced code blocks and ASCII-art
architecture diagrams. They are never embedded in prose that automated
tools need to search or replace. They render correctly in all modern
editors and terminals. Prohibiting them would make architecture diagrams
significantly harder to read with no practical benefit.
