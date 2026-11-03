# toml-tree

A small, standalone Rust tool that reads a TOML file and prints it as an ASCII
tree: each `[Section]` is a branch with an item count, and each key is a leaf
with its value aligned in a right-hand column.

It is intentionally OUTSIDE the FileForge Workbench cargo workspace (its
`Cargo.toml` carries an empty `[workspace]` table), so building it never touches
the product build.

## Usage

```
cargo run --manifest-path tools/rust/toml-tree/Cargo.toml -- <path-to-toml>
```

Example against the bundled fixture:

```
cargo run --manifest-path tools/rust/toml-tree/Cargo.toml -- tools/rust/toml-tree/sample.toml
```

Output (ASCII sketch):

```
File: "tools/rust/toml-tree/sample.toml"
|
+-+ [Accessibility]          : 1 Item
| +-reduce_motion            : false
|
+-+ [Catalogs]               : 2 Items
...
+-+ [Server]                 : 3 Items
  +- host                    : localhost
  +- [http]                  : 3 Items
  |  +- port                 : 8080
  |  +- tls                  : false
  |  +- [limits]             : 2 Items
  |     +- max_connections   : 128
  |     +- timeout_seconds   : 30
  +- [routes]                : 2 Items
     +- [0]                  : 2 Items
     |  +- path              : /
     |  +- method            : GET
     +- [1]                  : 2 Items
        +- path              : /api
        +- method            : POST
```

(The real output uses Unicode box-drawing glyphs for the branches; the ASCII
sketch above is only an approximation for this README.)

## Inputs and outputs

- Input: one TOML file path, given as the single CLI argument.
- Output: the rendered tree on stdout. Nothing is written to disk and no file
  is modified. Errors (missing file, invalid TOML, missing argument) go to
  stderr with a non-zero exit code.

## Notes

- Top-level tables (`[Section]`) are rendered in document order.
- Bare top-level `key = value` pairs that appear before any section are grouped
  into a leading unnamed section.
- Values must be valid TOML. Unquoted bare words (e.g. `level = info`) are NOT
  valid TOML and will fail to parse -- quote them (`level = "info"`).

## Recursive nesting

The tree expands to arbitrary depth:

- A nested sub-table (e.g. `[Server.http]`, which TOML nests as table `http`
  inside table `Server`) renders as its own indented child branch labelled
  `[http]`, not collapsed inline as `{ ... }`.
- An array-of-tables (e.g. `[[Server.routes]]`) renders as a branch whose
  children are indexed element nodes `[0]`, `[1]`, ...; each element expands into
  its own fields. Scalar arrays (e.g. `[1, 2, 3]`) still render inline on the
  right.
- Every level draws correct box-drawing connectors: a tee for a child with a
  following sibling, an L-corner for the last child, and a vertical pipe
  continuation under any ancestor that still has later siblings. The right-hand
  value/count column stays aligned across the whole tree.
- A flat section (only scalar leaves) keeps its original two-level appearance
  unchanged; the expanded layout appears only where real nesting is present.
