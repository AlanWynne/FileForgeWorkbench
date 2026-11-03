//! toml-tree: render a TOML file as an ASCII tree.
//!
//! Usage:
//!     toml-tree <path-to-toml>
//!
//! Reads a TOML document and prints each top-level table (section) as a
//! branch of a tree, with its keys as leaves and their values aligned in a
//! right-hand column. Mirrors the layout:
//!
//!     File: "<path>"
//!     |
//!     +-+ [Section]                        : N Items
//!     | +-+first_key                       : value
//!     |   +middle_key                      : value
//!     |   +last_key                        : value
//!     |
//!     +-+ [LastSection]                    : N Items
//!       +-first_key                        : value

use std::process::ExitCode;

use toml::Value;

/// A node in the rendered tree.
///
/// A `Leaf` is a scalar key/value pair (or a key whose value is an empty array,
/// which still renders inline as `[]` on the right). A `Branch` is a table or a
/// non-empty array that expands into its own indented child branches.
enum Node {
    Leaf {
        key: String,
        value: String,
    },
    Branch {
        /// Display label, already bracketed (e.g. `[Server]`, `[http]`, `[0]`).
        label: String,
        children: Vec<Node>,
    },
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: toml-tree <path-to-toml>");
            return ExitCode::from(2);
        }
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };

    let doc: Value = match text.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: cannot parse {path} as TOML: {e}");
            return ExitCode::from(1);
        }
    };

    let sections = collect_nodes(&doc);
    print!("{}", render(&path, &sections));
    ExitCode::SUCCESS
}

/// Pull the top-level tables out of the document in order, recursing into
/// nested sub-tables and arrays-of-tables. Non-table top-level keys (bare
/// key/value pairs before any `[section]`) are gathered into a leading unnamed
/// section so the current behaviour is preserved.
fn collect_nodes(doc: &Value) -> Vec<Node> {
    let mut sections = Vec::new();
    let Some(table) = doc.as_table() else {
        return sections;
    };

    let mut root_leaves = Vec::new();
    for (key, val) in table {
        if expands(val) {
            sections.push(node_for(&format!("[{key}]"), val));
        } else {
            root_leaves.push(Node::Leaf {
                key: key.clone(),
                value: render_value(val),
            });
        }
    }

    if !root_leaves.is_empty() {
        sections.insert(
            0,
            Node::Branch {
                label: "[]".to_string(),
                children: root_leaves,
            },
        );
    }
    sections
}

/// A value "expands" into its own branch when it is a table or a NON-EMPTY
/// array (of any element type - tables, scalars, or nested arrays). An empty
/// array stays inline as `[]`; only scalars and empty arrays remain inline.
fn expands(v: &Value) -> bool {
    match v {
        Value::Table(_) => true,
        // Any non-empty array expands into indexed child nodes. An empty array
        // deliberately stays inline (rendered as `[]`) rather than becoming an
        // empty branch.
        Value::Array(a) => !a.is_empty(),
        _ => false,
    }
}

/// Build a branch node for an expanding value, recursing into children.
///
/// - A table becomes a branch whose children are its keys (leaves for scalars,
///   nested branches for sub-tables / non-empty arrays).
/// - A non-empty array becomes a branch whose children are indexed nodes `[0]`,
///   `[1]`, ... one per element: a table/array element recurses into its own
///   branch, while a scalar element lands in the catch-all arm and becomes a
///   leaf whose key is the `[i]` label and whose value is the scalar.
fn node_for(label: &str, v: &Value) -> Node {
    match v {
        Value::Table(t) => {
            let children = t
                .iter()
                .map(|(k, val)| {
                    if expands(val) {
                        node_for(&format!("[{k}]"), val)
                    } else {
                        Node::Leaf {
                            key: k.clone(),
                            value: render_value(val),
                        }
                    }
                })
                .collect();
            Node::Branch {
                label: label.to_string(),
                children,
            }
        }
        Value::Array(a) => {
            let children = a
                .iter()
                .enumerate()
                .map(|(i, elem)| node_for(&format!("[{i}]"), elem))
                .collect();
            Node::Branch {
                label: label.to_string(),
                children,
            }
        }
        // A scalar array element reaches here: the `[i]` label becomes the leaf
        // key and the scalar becomes its right-hand value. Also the defensive
        // path for a non-expanding value passed directly.
        other => Node::Leaf {
            key: label.to_string(),
            value: render_value(other),
        },
    }
}

/// Render a scalar/compound value the way it should appear after the colon.
/// Only scalars and empty arrays (rendered `[]`) reach this for inline display;
/// tables and non-empty arrays are expanded into branches before rendering. The
/// Array/Table arms remain for the defensive catch-all in `node_for`.
fn render_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Datetime(d) => d.to_string(),
        Value::Array(a) => {
            let parts: Vec<String> = a.iter().map(render_value).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Table(t) => {
            let parts: Vec<String> = t
                .iter()
                .map(|(k, v)| format!("{k} = {}", render_value(v)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
    }
}

/// One rendered line. `Row` lines carry a left label and a right value/count
/// and participate in colon alignment; `Raw` lines (the header, root pipe, and
/// section spacers) are emitted verbatim.
enum Line {
    Row { left: String, right: String },
    Raw(String),
}

/// Build the full rendered tree as a single string (ends with a newline).
///
/// Two passes: traverse the node tree into `Line`s (so connector decisions are
/// made once, during traversal), then compute the colon column from the widest
/// `Row` left label and emit. This keeps the right-hand column aligned across
/// every depth of the tree.
fn render(path: &str, sections: &[Node]) -> String {
    let mut lines = vec![
        Line::Raw(format!("File: \"{path}\"")),
        Line::Raw("|".to_string()),
    ];

    let last_section = sections.len().saturating_sub(1);
    for (si, section) in sections.iter().enumerate() {
        let is_last_section = si == last_section;
        draw_section(section, is_last_section, &mut lines);
    }

    // Colon column: widest Row left label, plus one space of breathing room.
    let label_width = lines
        .iter()
        .filter_map(|l| match l {
            Line::Row { left, .. } => Some(left.chars().count()),
            Line::Raw(_) => None,
        })
        .max()
        .unwrap_or(0)
        + 1;

    let mut out = String::new();
    for line in &lines {
        match line {
            Line::Raw(s) => {
                out.push_str(s);
                out.push('\n');
            }
            Line::Row { left, right } => {
                out.push_str(&pad_to(left, label_width));
                out.push_str(right);
                out.push('\n');
            }
        }
    }
    out
}

/// Draw a top-level section (table or array-of-tables). Flat sections (only
/// scalar leaves) use the legacy quirky layout verbatim; sections that contain
/// real nesting expand with the clean recursive layout.
fn draw_section(section: &Node, is_last_section: bool, lines: &mut Vec<Line>) {
    let Node::Branch { label, children } = section else {
        // A bare top-level leaf cannot occur (they are grouped into the unnamed
        // root section), but handle it defensively.
        if let Node::Leaf { key, value } = section {
            lines.push(Line::Row {
                left: key.clone(),
                right: format!(": {value}"),
            });
        }
        return;
    };

    // Section header: unchanged glyphs for every section, flat or nested.
    let branch = if is_last_section {
        "\u{2514}\u{2500}\u{252c} "
    } else {
        "\u{251c}\u{2500}\u{252c} "
    };
    lines.push(Line::Row {
        left: format!("{branch}{label}"),
        right: count_suffix(children.len()),
    });

    // Gutter beneath this section: a pipe if more sections follow, else blank.
    let gutter = if is_last_section { "  " } else { "| " };

    if children.iter().all(|c| matches!(c, Node::Leaf { .. })) {
        // Legacy layout: preserve the exact two-level look.
        draw_legacy_leaves(children, gutter, lines);
    } else {
        // Real nesting: clean recursive layout.
        draw_children(children, gutter, lines);
    }

    // Blank spacer line between sections (and after the last one).
    lines.push(Line::Raw(gutter.trim_end().to_string()));
}

/// Legacy leaf drawing: the first leaf joins the section node with an
/// L-corner+tee and later leaves hang beneath it. Byte-identical to the
/// original two-level renderer.
fn draw_legacy_leaves(leaves: &[Node], gutter: &str, lines: &mut Vec<Line>) {
    let last = leaves.len().saturating_sub(1);
    for (li, leaf) in leaves.iter().enumerate() {
        let Node::Leaf { key, value } = leaf else {
            continue;
        };
        let is_first = li == 0;
        let is_last = li == last;
        let connector = match (is_first, is_last) {
            (true, true) => "\u{2514}\u{2500}",          // only leaf
            (true, false) => "\u{2514}\u{2500}\u{252c}", // first of several
            (false, true) => "  \u{2514}",               // last of several
            (false, false) => "  \u{251c}",              // middle
        };
        lines.push(Line::Row {
            left: format!("{gutter}{connector}{key}"),
            right: format!(": {value}"),
        });
    }
}

/// Clean recursive child drawing used wherever real nesting expands. Each child
/// gets a tee unless it is the last child (which gets an L-corner); a branch
/// child recurses with the gutter extended by a vertical pipe (`| `) when
/// siblings follow, or blank spaces when it is the last child.
fn draw_children(children: &[Node], gutter: &str, lines: &mut Vec<Line>) {
    let last = children.len().saturating_sub(1);
    for (ci, child) in children.iter().enumerate() {
        let is_last = ci == last;
        let connector = if is_last {
            "\u{2514}\u{2500} "
        } else {
            "\u{251c}\u{2500} "
        };
        match child {
            Node::Leaf { key, value } => {
                lines.push(Line::Row {
                    left: format!("{gutter}{connector}{key}"),
                    right: format!(": {value}"),
                });
            }
            Node::Branch {
                label,
                children: grandchildren,
            } => {
                lines.push(Line::Row {
                    left: format!("{gutter}{connector}{label}"),
                    right: count_suffix(grandchildren.len()),
                });
                let deeper = if is_last {
                    format!("{gutter}   ")
                } else {
                    format!("{gutter}|  ")
                };
                draw_children(grandchildren, &deeper, lines);
            }
        }
    }
}

/// The right-hand count suffix for a branch header: `: N Item` / `: N Items`.
fn count_suffix(count: usize) -> String {
    let unit = if count == 1 { "Item" } else { "Items" };
    format!(": {count} {unit}")
}

/// Pad-to-width: append spaces until the string reaches `width` columns.
fn pad_to(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        format!("{s} ")
    } else {
        format!("{s}{}", " ".repeat(width - len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(toml_src: &str) -> String {
        let doc: Value = toml_src.parse().expect("test TOML should parse");
        let nodes = collect_nodes(&doc);
        render("sample.toml", &nodes)
    }

    // Validates: flat two-level sections render byte-identical to the legacy
    // layout (first leaf joins with the L-corner+tee, later leaves hang under
    // it), so currently-working files are unchanged.
    #[test]
    fn flat_sections_render_unchanged() {
        let out = tree("[A]\nx = 1\n\n[B]\np = 2\nq = 3\n");
        let expected = "\
File: \"sample.toml\"
|
\u{251c}\u{2500}\u{252c} [A] : 1 Item
| \u{2514}\u{2500}x   : 1
|
\u{2514}\u{2500}\u{252c} [B] : 2 Items
  \u{2514}\u{2500}\u{252c}p  : 2
    \u{2514}q  : 3

";
        assert_eq!(out, expected);
    }

    // Validates: a nested sub-table ([Server.http] under [Server]) renders as
    // its own indented child branch rather than being collapsed inline.
    #[test]
    fn nested_subtable_expands_to_child_branch() {
        let out = tree("[Server]\nhost = \"localhost\"\n\n[Server.http]\nport = 8080\n");
        // The http sub-table must appear as a bracketed child branch with its
        // own leaf, not inline as `{ port = 8080 }`.
        assert!(
            out.contains("[http]"),
            "sub-table should be a child branch:\n{out}"
        );
        assert!(
            out.contains("port"),
            "sub-table leaf should be present:\n{out}"
        );
        assert!(
            !out.contains("{ port"),
            "sub-table must not render inline:\n{out}"
        );
    }

    // Validates: an array-of-tables ([[routes]]) expands to indexed sub-nodes,
    // one child branch per element, not a single inline [...] line.
    #[test]
    fn array_of_tables_expands_to_indexed_nodes() {
        let out = tree("[[routes]]\npath = \"/\"\n\n[[routes]]\npath = \"/api\"\n");
        assert!(
            out.contains("[routes]"),
            "array-of-tables should be a branch:\n{out}"
        );
        assert!(
            out.contains("[0]"),
            "first element should be an indexed node:\n{out}"
        );
        assert!(
            out.contains("[1]"),
            "second element should be an indexed node:\n{out}"
        );
        assert!(
            !out.contains("[{"),
            "array-of-tables must not render inline:\n{out}"
        );
    }

    // Validates: deep multi-level nesting draws correct connectors - a tee for a
    // child with a following sibling, an L-corner for the last child, and a
    // vertical pipe continuation under an ancestor that still has siblings.
    #[test]
    fn deep_nesting_draws_correct_connectors() {
        let out = tree(
            "[Server]\nhost = \"localhost\"\n\n[Server.http]\nport = 8080\n\n[Server.http.limits]\nmax = 128\n",
        );
        // host has a following sibling branch -> tee connector.
        assert!(
            out.lines().any(|l| l.contains("\u{251c}\u{2500} host")),
            "host should use a tee (has following sibling):\n{out}"
        );
        // http is the last child of Server -> L-corner, and continuation under
        // it is spaces (no trailing pipe).
        assert!(
            out.lines().any(|l| l.contains("\u{2514}\u{2500} [http]")),
            "http should use an L-corner (last child):\n{out}"
        );
        // limits is the last child of http -> L-corner, nested two deep.
        assert!(
            out.lines().any(|l| l.contains("[limits]")),
            "limits sub-table should be present at depth 2:\n{out}"
        );
        // Right-hand value column stays aligned: every data line's colon is at
        // the same column.
        // Visible column (char count) of the aligned ": " on each data line.
        let colon_cols: Vec<usize> = out
            .lines()
            .filter(|l| !l.starts_with("File:"))
            .filter(|l| l.contains(": "))
            .map(|l| {
                let byte = l.find(": ").unwrap();
                l[..byte].chars().count()
            })
            .collect();
        assert!(
            colon_cols.windows(2).all(|w| w[0] == w[1]),
            "colon column should be aligned across depth: {colon_cols:?}\n{out}"
        );
    }

    // Validates: a non-empty scalar array expands into indexed element leaves
    // ([0]/[1]/[2] with values 1/2/3) rather than rendering inline as a single
    // `: [..]` value.
    #[test]
    fn scalar_array_expands_to_indexed_leaves() {
        let out = tree("[S]\nnums = [1, 2, 3]\n");
        assert!(
            out.contains("[nums]"),
            "scalar array should be a child branch:\n{out}"
        );
        assert!(
            !out.contains(": [1, 2, 3]"),
            "scalar array must not render inline:\n{out}"
        );
        // Each element becomes an indexed leaf: `[0] : 1`, `[1] : 2`, `[2] : 3`.
        for (i, v) in [(0, 1), (1, 2), (2, 3)] {
            assert!(
                out.lines()
                    .any(|l| l.contains(&format!("[{i}]")) && l.contains(&format!(": {v}"))),
                "element [{i}] should be a leaf with value {v}:\n{out}"
            );
        }
    }

    // Validates: a nested array (array of arrays) expands recursively - the
    // outer array becomes a branch of indexed element branches, each of which
    // expands into its own indexed scalar leaves.
    #[test]
    fn nested_array_expands_recursively() {
        let out = tree("[S]\ngrid = [[1, 2], [3, 4]]\n");
        assert!(
            out.contains("[grid]"),
            "outer array should be a child branch:\n{out}"
        );
        // Outer elements [0] and [1] are themselves branches, each with their
        // own [0]/[1] scalar leaves holding 1/2 and 3/4.
        assert!(
            out.lines().any(|l| l.contains("[0]") && l.contains(": 1")),
            "inner element 1 should be a leaf:\n{out}"
        );
        assert!(
            out.lines().any(|l| l.contains("[1]") && l.contains(": 4")),
            "inner element 4 should be a leaf:\n{out}"
        );
        assert!(
            !out.contains(": [["),
            "nested array must not render inline:\n{out}"
        );
    }

    // Validates: an empty array stays inline as `: []` (the chosen behaviour) -
    // only non-empty arrays expand.
    #[test]
    fn empty_array_stays_inline() {
        let out = tree("[S]\nempty = []\n");
        assert!(
            out.contains(": []"),
            "empty array should stay inline as []:\n{out}"
        );
        assert!(
            !out.contains("[empty]"),
            "empty array must not expand into a branch:\n{out}"
        );
    }
}
