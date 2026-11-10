# Iteration Charts / Structograms from Source Code -- Discussion

This document captures a design discussion about building a language-neutral tool
that turns a source file into a horizontal, boxed "iteration chart" style design
diagram, rendered as SVG, and ultimately published as a per-program web page.

## 1. What this kind of chart is called

The sketch under discussion is a set of named routine boxes laid out
horizontally, where a `do NAME` line inside one box is linked by a connector to
another box that holds that routine's body, and every box ends in an `Exit` line.

This maps onto several established program-design conventions:

| Convention | Layout | What a box holds | Call shown as |
|---|---|---|---|
| Nassi-Shneiderman / structogram (ISO 5807) | vertical nesting | control structures as nested boxes | a "process" sub-box |
| Chapin chart | vertical nesting | a style of Nassi-Shneiderman diagram | sub-box |
| Structure chart (Yourdon/Constantine) | top-down tree | a module name | arrow/line to child module |
| Jackson Structure Diagram (JSP) | tree | data/program structure | child node |
| Warnier-Orr | horizontal, left-to-right | braces of refinements | nested bracket |
| HIPO (Hierarchy plus Input-Process-Output) | hierarchy + per-process IPO | input/process/output | reference number |
| The "iteration chart" in this discussion | horizontal panels | per-routine pseudo-code | `do NAME` line linked to NAME's panel |

Summary of the naming question:

- The boxed-pseudo-code feel is closest to **Nassi-Shneiderman structograms**
  (the most formal, ISO 5807).
- The horizontal, left-to-right call-linkage feel is closest to
  **Warnier-Orr diagrams** and **structure charts**.
- The specific drawing is a hybrid: per-routine pseudo-code panels plus call
  connectors between them. "Iteration chart" appears to be a house/shop term for
  this hybrid; there is no single universal name that matches it exactly.

## 2. The original sketch

```
+---------------------------------------+ +-+---------------------------------------+
|main A                                 | | |function BA                            |
+---------------------------------------+ | +---------------------------------------+
|Start of program                       | | |....                                   |
|....                                   | | |....                                   |
|....                                   | | |....                                   |
|....                                   | | |....                                   |
+---------------------------------------+ | |....                                   |
|do BA while a ne b                     |-+ |....                                   |
+---------------------------------------+   |Exit function                          |
|....                                   |   +---------------------------------------+
|....                                   | +-+---------------------------------------+
|do read-file                           | | |function CA                            |
+---------------------------------------+ | +---------------------------------------+
|do CA UNTIL e EQ d                     |-+ |....                                   |
+---------------------------------------+   |....                                   |
|....                                   |   |....                                   |
|....                                   |   |....                                   |
|....                                   |   |....                                   |
|....                                   |   |....                                   |
|....                                   |   |do read-file                           |
|....                                   |   |Exit function                          |
|Exit Program                           |   +---------------------------------------+
+---------------------------------------+
+---------------------------------------+
|read-file                              |
+---------------------------------------+
|....                                   |
|....                                   |
|....                                   |
|....                                   |
|Exit function                          |
+---------------------------------------+
```

Key features of the style:

- One panel per routine (function / procedure / `main`).
- Each panel has a header row (the routine name) and a footer row (`Exit ...`).
- Body rows are either filler (`....`) or a control line (`do NAME while ...`,
  `do NAME until ...`, `do NAME`).
- A control line that invokes another routine is linked by a connector to that
  routine's panel.

## 3. Reducing many languages to one pseudo-code

The core decision: do not pseudo-code "from the language." Pseudo-code from a
small, language-independent intermediate representation (IR) that every language
front-end lowers into.

```
source file --[language front-end]--> common IR --[pseudo lowering]--> panels --[SVG]--> diagram
```

The IR is intentionally tiny -- only the constructs an iteration chart shows:

| IR node | Means | Pseudo-code line |
|---|---|---|
| `Routine{name, params, body}` | a function/method/procedure | panel header + `Exit <name>` footer |
| `Seq[...]` | straight-line statements | one `....` line (or a short label) each |
| `Call{callee}` | invoking another routine | `do <callee>` |
| `Loop{kind, cond, body}` | while / for / until / loop | `do <body> while <cond>` or `... until ...` |
| `If{cond, then, else}` | branch | `if <cond>` / `else` sub-rows |
| `Return` | exit point | `Exit <name>` |

Everything a language can express collapses into those ~6 shapes. A Rust
`for x in it`, a Python `for x in it:`, a C `for(;;)`, and a COBOL
`PERFORM UNTIL` all become the same `Loop` node. That is what makes the output
language-neutral: the diagram vocabulary is fixed, and each front-end maps its
syntax onto these shapes and discards everything else (types, generics,
operators, expression detail).

### Front-ends -- two tiers

1. **Tree-sitter front-end (general path).** Tree-sitter has maintained grammars
   for 40+ languages behind one uniform API. Write one IR-lowering pass per
   language that recognizes that language's function node, loop node, call node,
   etc. Adding a language = add its grammar + a node-kind mapping table. This
   buys breadth cheaply.
2. **Native-parser front-end (optional, higher fidelity).** For a language where
   precision matters, use its real parser (e.g. `syn` for Rust). Emits the same
   IR, with better accuracy.

Both tiers emit identical IR, so lowering and rendering never know which
language the panels came from.

## 4. Agreed constraints

- **Language-neutral**, not Rust-only.
- **Granularity: one diagram per source file.** The IR is really
  `File{ routines: [...] }`.
- **Output: SVG** (clean, scalable connectors; far better than ASCII for the
  non-crossing connector routing).

## 5. Honest limitations

- **Cross-file calls** render as a `do NAME` with no panel to link to (or a stub
  panel); the connector only draws when the callee lives in the same file.
- **Call detection is heuristic** -- "is this identifier a call?" is answered
  structurally (a call-expression node), not semantically. Dynamic dispatch,
  function pointers, macros, and method chains are approximated. Acceptable for a
  design document.
- **Filler lines (`....`)** represent unnamed statements; how aggressively to
  summarize them is a rendering choice.

## 6. Open decisions (to settle before building v1)

1. **Launch language set for v1** -- proposal: Rust via `syn` (one high-fidelity
   front-end) plus one tree-sitter language (Python) to prove the neutral path.
2. **If/branch rendering** -- draw conditionals as indented sub-rows, or leave as
   plain `....` lines for v1?
3. **Filler lines** -- collapse consecutive unnamed statements into a single
   `....`, or emit one `....` per statement to preserve rough body length?
