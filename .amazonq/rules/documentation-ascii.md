# Documentation Character Set — MANDATORY

All documentation files in this project MUST use plain ASCII characters only.

## Rule

Every file under `docs/`, `.amazonq/rules/`, and any other Markdown or text
file in the workspace MUST contain only characters in the ASCII range
(code points 0x00 to 0x7F).

## Prohibited Characters

The following characters are explicitly prohibited:

| Character | Unicode | Common name | ASCII substitute |
|-----------|---------|-------------|-----------------|
| `\u2014` | U+2014 | em dash | `--` or `: ` |
| `\u2013` | U+2013 | en dash | `-` |
| `\u2018` | U+2018 | left single quote | `'` |
| `\u2019` | U+2019 | right single quote | `'` |
| `\u201C` | U+201C | left double quote | `"` |
| `\u201D` | U+201D | right double quote | `"` |
| `\u2026` | U+2026 | ellipsis | `...` |
| `\u00E2` | U+00E2 | a-circumflex (BOM artefact) | remove |
| `\u00E2\u0080\u0099` | UTF-8 sequence | curly apostrophe | `'` |
| `\uFEFF` | U+FEFF | BOM / zero-width no-break space | remove |
| Any box-drawing character | U+2500-U+257F | Unicode box lines | use plain `-`, `|`, `+` |

## Substitution Rules

- Em dash (`--`) or a colon-space (`: `) replaces `--`.
- Hyphen-minus (`-`) replaces en dash.
- Straight apostrophe (`'`) replaces curly apostrophes.
- Straight double quote (`"`) replaces curly double quotes.
- Three full stops (`...`) replace the ellipsis character.
- Plain ASCII box-drawing (`-`, `|`, `+`) replaces Unicode box-drawing characters.
- BOM bytes at the start of a file MUST be removed.

## Section Separator Style

Section separator comments in Rust source files MUST use plain ASCII only:

```rust
// === Section Name ===================================================
```

Do NOT use Unicode box-drawing characters. This rule is already stated in
`source-file-size.md` and is repeated here for completeness.

## Enforcement

Before committing any documentation change, verify with:

```bash
rg "[^\x00-\x7F]" docs/ .amazonq/rules/ --glob "*.md"
```

Any match is a violation. Fix by replacing the offending character with its
ASCII substitute listed above.

## Why This Rule Exists

Non-ASCII characters in Markdown and plain-text files cause:

- Pattern-matching failures in automated editing tools.
- Garbled display in terminals that do not support UTF-8.
- BOM artefacts that corrupt file comparisons and diffs.
- Inconsistent rendering across editors and CI pipelines.
- Encoding errors when files are read by Rust `std::fs::read_to_string`
  on systems with non-UTF-8 default encodings.
