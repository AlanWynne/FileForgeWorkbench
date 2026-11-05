//! Tests for drawing a model's `tables` as frame-ruled BANDS inside the framed
//! (common-frame) renderer (CR-NR-116, screen-snapshot-scrm Requirement
//! 30.32-30.40). A table is NOT a nested box: its header and data rows are
//! ordinary frame bands, its top rule merges with the frame rule under the
//! Command line, and its closing rule merges with the rule above the status
//! band. These tests assert the band/rule structure, column alignment, cell
//! discreteness, coalescing, and the ASCII fallback.

use crate::model::{MarkdownSection, ScreenModel, Table};
use crate::render::framed::render_framed;
use crate::render::{render, RenderOptions};
use crate::SnapshotFormat;

/// A framed model carrying `header_lines` (so the framed path is taken), a
/// Command line, a status band, and the caller's content.
fn framed(tables: Vec<Table>, body: Vec<String>) -> ScreenModel {
    let mut m = ScreenModel::new("Alias Editor");
    m.header_lines = vec!["Settings | Help".into()];
    m.command_line = "____".into();
    m.body_lines = body;
    m.tables = tables;
    m.status_bar.text = "Last: 11:02 | Prof: default".into();
    m
}

/// A small Alias-like table: 4 columns, 2 data rows, using the provider cell
/// conventions (`[value v]`, `[ v]`, `____`).
fn alias_table() -> Table {
    Table {
        caption: None,
        headers: vec!["Act".into(), "CE".into(), "Command".into(), "Alias".into()],
        rows: vec![
            vec![
                "____".into(),
                "[FFCMD v]".into(),
                "SNAPSHOT".into(),
                "SS".into(),
            ],
            vec![
                "____".into(),
                "[ v]".into(),
                "________".into(),
                "________".into(),
            ],
        ],
    }
}

fn lines(m: &ScreenModel, unicode: bool) -> Vec<String> {
    render_framed(
        m,
        RenderOptions {
            unicode_box: unicode,
        },
    )
    .lines()
    .map(str::to_string)
    .collect()
}

fn command_index(got: &[String]) -> usize {
    got.iter()
        .position(|l| l.contains("Command ===>"))
        .expect("command line")
}

/// True when `line` is made only of frame rule / junction / corner glyphs.
fn is_rule(line: &str) -> bool {
    !line.is_empty()
        && line.chars().all(|c| {
            "\u{2500}\u{252C}\u{2534}\u{253C}\u{251C}\u{2524}\u{250C}\u{2510}\u{2514}\u{2518}+-"
                .contains(c)
        })
}

fn assert_no_consecutive_rules(got: &[String]) {
    for pair in got.windows(2) {
        assert!(
            !(is_rule(&pair[0]) && is_rule(&pair[1])),
            "two consecutive rules:\n{}",
            got.join("\n")
        );
    }
}

// Validates: screen-snapshot-scrm Requirement 30.32, 30.34 -- a framed table
// draws a top rule (tee-down per divider, merged with the Command-line frame
// rule), a header band, a header/data rule (cross per divider), one band per
// data row, and the closing rule (tee-up per divider), with NO blank spacer row
// and NO caption row inside the box.
#[test]
fn framed_table_draws_header_band_rules_and_data_bands_unicode() {
    let got = lines(&framed(vec![alias_table()], Vec::new()), true);
    let c = command_index(&got);
    let dn = '\u{252C}';
    let cross = '\u{253C}';
    let up = '\u{2534}';
    // The single rule under the Command line carries the header tee-down
    // junctions (merged top rule).
    assert!(
        got[c + 1].contains(dn),
        "merged top rule has tee-down junctions:\n{}",
        got.join("\n")
    );
    assert!(is_rule(&got[c + 1]));
    // Header band directly after the merged top rule (no blank/caption row).
    assert!(got[c + 2].contains("Act") && got[c + 2].contains("Command"));
    // Header/data rule with crosses.
    assert!(
        got[c + 3].contains(cross),
        "header/data rule has crosses:\n{}",
        got.join("\n")
    );
    // Two data bands.
    assert!(got[c + 4].contains("SNAPSHOT") && got[c + 4].contains("[FFCMD v]"));
    assert!(got[c + 5].contains("[ v]"));
    // Closing rule carries tee-up junctions; no caption anywhere.
    let closing = &got[c + 6];
    assert!(
        is_rule(closing) && closing.contains(up),
        "closing tee-up rule"
    );
    assert!(
        !got.iter().any(|l| l.contains("caption")),
        "no caption row inside the box"
    );
    assert_no_consecutive_rules(&got);
}

// Validates: screen-snapshot-scrm Requirement 30.33 -- column width is the
// widest of the header and all its cells, so dividers align across the header
// and every data band (every band line shares the same width).
#[test]
fn framed_table_columns_align_across_header_and_data_bands() {
    let got = lines(&framed(vec![alias_table()], Vec::new()), true);
    let band_lines: Vec<&String> = got
        .iter()
        .filter(|l| l.contains("Act") || l.contains("SNAPSHOT") || l.contains("[ v]"))
        .collect();
    assert_eq!(band_lines.len(), 3, "header + 2 data bands");
    let v = '\u{2502}';
    // The interior divider columns line up across the header and data bands.
    let divider_cols = |line: &str| -> Vec<usize> {
        line.char_indices()
            .filter(|(_, ch)| *ch == v)
            .map(|(i, _)| i)
            .collect()
    };
    let first = divider_cols(band_lines[0]);
    for b in &band_lines {
        assert_eq!(
            divider_cols(b),
            first,
            "dividers align:\n{}",
            got.join("\n")
        );
    }
    // 4 columns => 3 interior dividers + 2 frame borders = 5 verticals.
    assert_eq!(first.len(), 5);
}

// Validates: screen-snapshot-scrm Requirement 30.32, 30.37 -- a table with NO
// rows draws its header band and rules only.
#[test]
fn framed_table_with_no_rows_draws_header_band_and_rules_only() {
    let empty = Table {
        caption: None,
        headers: vec!["Act".into(), "Alias".into()],
        rows: Vec::new(),
    };
    let got = lines(&framed(vec![empty], Vec::new()), true);
    let c = command_index(&got);
    assert!(is_rule(&got[c + 1]), "top rule");
    assert!(
        got[c + 2].contains("Act") && got[c + 2].contains("Alias"),
        "header band"
    );
    assert!(is_rule(&got[c + 3]), "closing rule after the header band");
    assert!(!got.iter().any(|l| l.contains("SNAPSHOT")));
    assert_no_consecutive_rules(&got);
}

// Validates: screen-snapshot-scrm Requirement 30.37 -- a table with NO columns
// is omitted entirely (no extra rule, no band).
#[test]
fn framed_table_with_no_columns_is_omitted() {
    let empty = Table {
        caption: None,
        headers: Vec::new(),
        rows: Vec::new(),
    };
    let got = lines(&framed(vec![empty], vec!["body line".into()]), true);
    // Only the frame-owned boundary rule after the Command line and the closing
    // rule before the status band exist; the body line renders normally.
    assert!(got.iter().any(|l| l.contains("body line")));
    assert_no_consecutive_rules(&got);
}

// Validates: screen-snapshot-scrm Requirement 30.37 -- two tables (or a table
// after body lines) lay out one after another with the adjacent rules coalesced
// (never two consecutive rules) and no blank spacer row.
#[test]
fn framed_two_tables_coalesce_adjacent_rules() {
    let got = lines(
        &framed(vec![alias_table(), alias_table()], Vec::new()),
        true,
    );
    assert_no_consecutive_rules(&got);
    // The first table's SNAPSHOT appears twice (both tables), proving both drew.
    assert_eq!(got.iter().filter(|l| l.contains("SNAPSHOT")).count(), 2);
    // No blank content row between them.
    assert!(
        !got.iter().any(
            |l| l.trim_matches(|c| c == '\u{2502}' || c == ' ').is_empty()
                && l.contains('\u{2502}')
        ),
        "no blank spacer band:\n{}",
        got.join("\n")
    );
}

// Validates: screen-snapshot-scrm Requirement 30.37 -- a table following
// body_lines lays out after them with a single coalesced rule between.
#[test]
fn framed_table_after_body_lines_single_rule_between() {
    let got = lines(
        &framed(vec![alias_table()], vec!["content line".into()]),
        true,
    );
    assert_no_consecutive_rules(&got);
    let body = got
        .iter()
        .position(|l| l.contains("content line"))
        .expect("body");
    let header = got
        .iter()
        .position(|l| l.contains("Act"))
        .expect("header band");
    assert!(body < header, "body line before the table header band");
    // Exactly one rule between the body line and the header band.
    let between = &got[body + 1..header];
    assert_eq!(between.iter().filter(|l| is_rule(l)).count(), 1);
    assert!(between.iter().all(|l| is_rule(l)), "only a rule between");
}

// Validates: screen-snapshot-scrm Requirement 30.36 -- a cell containing ` | `
// stays in its cell (not re-split on the band separator), and control chars
// (tab, CR, LF) render as a single space and are never read as the right-align
// marker.
#[test]
fn framed_table_cell_is_discrete_and_control_chars_become_space() {
    let t = Table {
        caption: None,
        headers: vec!["H".into()],
        rows: vec![vec!["a | b\tc\rd\ne".into()]],
    };
    let got = lines(&framed(vec![t], Vec::new()), true);
    let data = got.iter().find(|l| l.contains("a | b")).expect("data band");
    // The ' | ' inside the cell is preserved and the control chars became spaces
    // (so the cell reads "a | b c d e"), with no tab/CR/LF leaking through.
    assert!(
        data.contains("a | b c d e"),
        "cell discrete, controls spaced:\n{data}"
    );
    assert!(!data.contains('\t') && !data.contains('\r'));
    // Exactly the two frame borders -- the in-cell ' | ' is NOT a divider.
    assert_eq!(
        data.matches('\u{2502}').count(),
        2,
        "no spurious divider:\n{data}"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.38 -- the ASCII fallback
// (unicode_box=false) yields the equivalent band layout using only + - |.
#[test]
fn framed_table_ascii_fallback_uses_plus_dash_pipe() {
    let got = lines(&framed(vec![alias_table()], Vec::new()), false);
    let c = command_index(&got);
    assert!(
        got[c + 1].contains('+'),
        "ASCII top rule uses '+':\n{}",
        got.join("\n")
    );
    assert!(got[c + 2].contains('|'), "ASCII header band uses '|'");
    assert!(got.iter().any(|l| l.contains("SNAPSHOT")));
    // No box-drawing glyphs leak into the ASCII fallback.
    assert!(!got
        .iter()
        .any(|l| l.contains('\u{2502}') || l.contains('\u{253C}')));
    assert_no_consecutive_rules(&got);
}

// Validates: screen-snapshot-scrm Requirement 30.39 -- the LEGACY render path
// (a model WITHOUT `header_lines`) is UNCHANGED: a table is still drawn as the
// below-the-frame `render_table_grid` box (its own corners, a caption on its
// own line), NOT as frame bands. Guards that the framed band layout did not
// leak into the legacy path.
#[test]
fn legacy_render_plain_still_draws_below_frame_grid_box() {
    let mut m = ScreenModel::new("Alias Editor");
    // No header_lines -> legacy render_plain path.
    m.tables.push(Table {
        caption: Some("My Aliases".into()),
        headers: vec!["CE".into(), "Command".into()],
        rows: vec![vec!["[FFCMD v]".into(), "SNAPSHOT".into()]],
    });
    let out = render(&m, SnapshotFormat::PlainText, RenderOptions::default());
    // The caption is drawn on its own line (framed bands never draw a caption).
    assert!(
        out.contains("My Aliases"),
        "legacy caption line present:\n{out}"
    );
    // The grid is its OWN box: a grid top rule carrying a tee-down (U+252C) and
    // a grid bottom rule carrying a tee-up (U+2534), which the framed band grid
    // never draws (its rules merge with the frame).
    assert!(out.contains('\u{252C}'), "legacy grid top tee-down:\n{out}");
    assert!(
        out.contains('\u{2534}'),
        "legacy grid bottom tee-up:\n{out}"
    );
    assert!(out.contains("[FFCMD v]") && out.contains("SNAPSHOT"));
}

// Validates: screen-snapshot-scrm Requirement 30.40 -- a model's
// `markdown_sections` still drive the Markdown flavor as box-first then the
// sections (the framed table-band change does not touch the Markdown path), and
// a model WITHOUT markdown_sections is unchanged.
#[test]
fn markdown_sections_still_render_box_first_then_sections() {
    let mut m = ScreenModel::new("Alias Editor");
    m.body_lines = vec!["Aliases: 1".into()];
    m.markdown_sections = vec![MarkdownSection {
        heading: Some("Aliases".into()),
        table: Table {
            caption: None,
            headers: vec!["CE".into(), "Command".into(), "Alias".into()],
            rows: vec![vec!["FFCMD".into(), "SNAPSHOT".into(), "SS".into()]],
        },
    }];
    let out = render(&m, SnapshotFormat::Markdown, RenderOptions::default());
    let box_idx = out.find('\u{250C}').expect("fenced box body present");
    let table_idx = out
        .find("| CE | Command | Alias |")
        .expect("section table present");
    assert!(
        box_idx < table_idx,
        "box before the section table (Req 30.40)"
    );
    assert!(out.contains("### Aliases"));

    // Without markdown_sections the Markdown path is unchanged (box only, no
    // section table).
    let mut plain = ScreenModel::new("Plain");
    plain.body_lines = vec!["just a line".into()];
    let out2 = render(&plain, SnapshotFormat::Markdown, RenderOptions::default());
    assert!(out2.contains('\u{250C}'));
    assert!(!out2.contains("| CE | Command | Alias |"));
}
