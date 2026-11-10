# Proposal: Source-to-SVG Iteration Charts, Published as a Per-Program Website

## 1. Goal

Build a tool that:

1. Takes a single source file (any supported language).
2. Produces a horizontal, boxed "iteration chart" (per-routine pseudo-code panels
   with call connectors) as an **SVG** diagram.
3. Publishes each SVG into a **web page indexed on the program name**.
4. Aggregates those pages into a **website** where the design documents are
   browsable per program name.

One diagram per source file is the fixed granularity.

## 2. Pipeline

```
source file
   |
   v
[language front-end]   tree-sitter (general) or native parser (syn, etc.)
   |
   v
common IR              File{ routines: [Routine{Seq|Call|Loop|If|Return}] }
   |
   v
[pseudo lowering]      IR -> panel model (pseudo-code lines + call links)
   |
   v
[layout]               panels -> columns + geometry + connector routing
   |
   v
[SVG renderer]         geometry -> standalone .svg
   |
   v
[site generator]       .svg + metadata -> per-program HTML page + index
   |
   v
static website
```

## 3. Component design

### 3.1 Common IR

The language-neutral core (see discussion.md for the full rationale). A minimal
node set: `Routine`, `Seq`, `Call`, `Loop`, `If`, `Return`. Every front-end
lowers to exactly this. Rendering and site generation are language-agnostic.

### 3.2 Language front-ends

- `LanguageFrontend` trait: `fn lower(&self, source: &str, path: &Path) -> File`.
- **Tree-sitter driver** + per-language node-kind mapping tables (breadth).
- **Native front-ends** where fidelity matters (`syn` for Rust).
- File extension selects the front-end.

### 3.3 Pseudo-code lowering

Walks the IR and emits per-routine panel models:

- Header row = routine name (`main A`, `function BA`, ...).
- Body rows = `....` filler, `do NAME`, `do NAME while <cond>`,
  `do NAME until <cond>`, and (optionally) `if <cond>` / `else` sub-rows.
- Footer row = `Exit <name>` / `Exit Program`.
- Each `do NAME` records a **call link** to the panel for NAME when NAME is
  defined in the same file.

### 3.4 Layout

The genuinely hard stage:

- Assign panels to columns (e.g. `main` in the left column, callees to the
  right).
- Compute panel/row geometry from text metrics.
- Route connectors from a `do NAME` row's right edge to NAME's panel header,
  minimizing crossings. SVG makes this tractable (orthogonal polylines with
  routing channels between columns).

### 3.5 SVG renderer

- Emits a standalone, self-contained `.svg` (embedded font-family, no external
  assets).
- Monospace text, box-drawing rectangles for panels and rows, polyline
  connectors with arrowheads.
- Deterministic output (stable ordering) so diffs are meaningful in version
  control.

### 3.6 Site generator and indexing

**Program name** is the index key. Derivation, in order of preference:

1. An explicit directive/front-matter in the source or a sidecar
   (`// @program: PAYROLL`).
2. The declared module/package/class name.
3. The file stem.

Collisions (two files claiming the same program name) are resolved by
appending the relative path, and reported as a warning.

Per build, the generator emits:

```
site/
  index.html                 -- master index: program name -> page, searchable
  programs/
    <program-name>.html      -- one page per program
  svg/
    <program-name>.svg       -- the diagram
  assets/                    -- css, client-side search index (json)
```

- `programs/<name>.html` embeds the SVG inline (so it scales and is
  selectable/linkable), plus metadata: source path, language, routine count,
  generation timestamp, and a link to the raw `.svg`.
- `index.html` lists every program alphabetically with a client-side filter box;
  a small JSON search index powers name lookup without a server.
- The whole output is a **static site** -- deployable to any static host
  (GitHub Pages, S3, an internal file share). No runtime server required.

### 3.7 CLI / batch mode

```
diagramgen render <file>                 # single file -> one .svg
diagramgen site   <dir> --out site/      # walk a tree -> full website
```

Batch mode walks a directory, renders one diagram per source file, derives the
program name for each, and writes the indexed site. Incremental mode (only
re-render changed files, keyed on content hash) is a later optimization.

## 4. Suggested Rust workspace layout

```
diagramgen/
  ir            -- common IR types
  frontend-core -- LanguageFrontend trait + extension dispatch
  frontend-ts   -- tree-sitter driver + per-language mapping tables
  frontend-syn  -- native Rust front-end (syn)
  pseudo        -- IR -> panel model
  layout        -- columns, geometry, connector routing
  render-svg    -- panel model + geometry -> SVG
  site          -- SVG + metadata -> HTML pages + index + assets
  cli           -- wiring, render/site subcommands
```

## 5. Phased delivery

- **Phase 1 -- single file to SVG.** IR + one front-end (Rust via `syn`) +
  pseudo lowering + layout + SVG renderer. `diagramgen render file.rs` produces a
  diagram matching the sketch (panels, `do NAME while/until`, `Exit`, connectors
  for same-file calls).
- **Phase 2 -- language neutrality.** Add the tree-sitter front-end and a second
  language (Python) with no change to lowering/layout/render. Proves the IR is
  the only coupling point.
- **Phase 3 -- website.** Site generator: per-program pages, master index with
  client-side search, static output. `diagramgen site <dir>` builds the whole
  site.
- **Phase 4 -- polish.** Conditional sub-rows, incremental rebuilds, cross-file
  call stubs, program-name directives, theming.

## 6. Risks and limits

- **Connector routing** is the main engineering risk; mitigated by SVG and
  column-based channel routing.
- **Call detection is heuristic** (structural, not semantic); dynamic dispatch,
  macros, and method chains are approximated.
- **Program-name derivation** can collide across files; handled by path
  qualification + warning.
- **Tree-sitter mapping drift** -- grammar node-kind names change between grammar
  versions; pin grammar versions per language.

## 7. Open decisions

Same three as in discussion.md, repeated here as the gate to Phase 1:

1. v1 language set (proposal: Rust + Python).
2. If/branch rendering (sub-rows vs plain filler).
3. Filler-line policy (collapse vs one-per-statement).
