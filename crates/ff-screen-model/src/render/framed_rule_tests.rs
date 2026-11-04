//! Tests for the frame-owned Content_Area boundary rules (workspace-framework
//! Requirement 7.30-7.32, screen-snapshot-scrm Requirement 30.29, 30.30).
//!
//! The synthetic "grid" content below stands in for a table drawn in the
//! frame's own rules: its first body line is a grid TOP rule and its last is a
//! grid BOTTOM rule, built from the glyph set of the render mode. No live
//! workspace emits such content yet; the merge is specified generically.

use crate::model::ScreenModel;
use crate::render::framed::render_framed;
use crate::render::RenderOptions;

/// Inner content width (the status band, 41 characters, is the widest row).
const WIDTH: usize = 41;
/// Content-relative column of the two grid dividers (line columns are +2).
const GRID_CUTS: [usize; 2] = [6, 14];
/// A grid body row whose `|`-style dividers sit at `GRID_CUTS`.
const GRID_ROW: &str = "aaaaa | bbbbb | c";

/// Glyph set of a render mode: (h, left tee, right tee, tee-down, tee-up, v).
struct Set {
    h: char,
    el: char,
    er: char,
    dn: char,
    up: char,
    v: char,
}

fn set(unicode: bool) -> Set {
    if unicode {
        Set {
            h: '\u{2500}',
            el: '\u{251C}',
            er: '\u{2524}',
            dn: '\u{252C}',
            up: '\u{2534}',
            v: '\u{2502}',
        }
    } else {
        Set {
            h: '-',
            el: '+',
            er: '+',
            dn: '+',
            up: '+',
            v: '|',
        }
    }
}

/// One expected frame rule line with `(line column, glyph)` junctions.
fn rule(s: &Set, junctions: &[(usize, char)]) -> String {
    let mut out = String::new();
    out.push(s.el);
    for col in 1..WIDTH + 3 {
        out.push(
            junctions
                .iter()
                .find(|(c, _)| *c == col)
                .map_or(s.h, |(_, g)| *g),
        );
    }
    out.push(s.er);
    out
}

/// A grid rule as BODY content: `glyph` at the grid cuts, `h` elsewhere.
fn grid_rule(s: &Set, glyph: char) -> String {
    (0..20)
        .map(|i| if GRID_CUTS.contains(&i) { glyph } else { s.h })
        .collect()
}

/// A full content line (`v`, pad, text padded to WIDTH, pad, `v`).
fn row(s: &Set, text: &str) -> String {
    format!("{} {text:<WIDTH$} {}", s.v, s.v)
}

/// Grid column junctions on a frame rule (line columns).
fn grid_junctions(glyph: char) -> Vec<(usize, char)> {
    GRID_CUTS.iter().map(|c| (c + 2, glyph)).collect()
}

fn base(body: Vec<String>, with_message: bool) -> ScreenModel {
    let mut m = ScreenModel::new("Configuration");
    m.header_lines = vec!["Settings | Help | SCRM".into()];
    m.command_line = "____".into();
    m.body_lines = body;
    if with_message {
        m.messages = vec!["a message".into()];
    }
    m.status_bar.text = "Last: 11:02 | Prof: default | Ln 1, Col 1".into();
    m
}

fn render(m: &ScreenModel, unicode: bool) -> Vec<String> {
    let out = render_framed(
        m,
        RenderOptions {
            unicode_box: unicode,
        },
    );
    out.lines().map(str::to_string).collect()
}

fn command_index(got: &[String]) -> usize {
    got.iter()
        .position(|l| l.contains("Command ===>"))
        .expect("command line")
}

fn is_rule(line: &str) -> bool {
    !line.is_empty()
        && line.chars().all(|c| {
            "\u{2500}\u{252C}\u{2534}\u{253C}\u{251C}\u{2524}\u{250C}\u{2510}\u{2514}\u{2518}+-"
                .contains(c)
        })
}

fn assert_no_consecutive_rules(got: &[String]) {
    let text = got.join("\n");
    for pair in got.windows(2) {
        assert!(
            !(is_rule(&pair[0]) && is_rule(&pair[1])),
            "two consecutive rules:\n{text}"
        );
    }
}

fn body(rows: &[String]) -> Vec<String> {
    rows.to_vec()
}

fn check_plain(unicode: bool) {
    let s = set(unicode);
    let got = render(
        &base(body(&["content one".into(), "content two".into()]), false),
        unicode,
    );
    let c = command_index(&got);
    assert_eq!(
        got[c + 1],
        rule(&s, &[]),
        "one frame rule after Command:\n{}",
        got.join("\n")
    );
    assert_eq!(got[c + 2], row(&s, "content one"));
    assert_no_consecutive_rules(&got);
}

// Validates: workspace-framework Requirement 7.30, 7.31; screen-snapshot-scrm
// Requirement 30.29 -- plain content gets ONE frame rule after the Command line.
#[test]
fn plain_content_gets_one_frame_rule_after_command_line() {
    check_plain(true);
}

// Validates: workspace-framework Requirement 7.30; screen-snapshot-scrm
// Requirement 30.29, 30.23 -- ASCII fallback uses `+ - +`.
#[test]
fn ascii_plain_content_gets_one_frame_rule_after_command_line() {
    check_plain(false);
}

fn check_top_merge(unicode: bool) {
    let s = set(unicode);
    let rows = [
        grid_rule(&s, s.dn),
        GRID_ROW.to_string(),
        GRID_ROW.to_string(),
    ];
    let got = render(&base(body(&rows), false), unicode);
    let c = command_index(&got);
    let text = got.join("\n");
    assert_eq!(
        got[c + 1],
        rule(&s, &grid_junctions(s.dn)),
        "merged top rule:\n{text}"
    );
    assert_eq!(
        got[c + 2],
        row(&s, GRID_ROW),
        "first grid row follows:\n{text}"
    );
    assert_no_consecutive_rules(&got);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.29 -- frame rule and grid top rule MERGE into ONE rule with
// the grid's top-tee junctions.
#[test]
fn grid_top_rule_merges_with_frame_rule_into_one_rule() {
    check_top_merge(true);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.29, 30.23 -- ASCII equivalent of the top merge.
#[test]
fn ascii_grid_top_rule_merges_with_frame_rule_into_one_rule() {
    check_top_merge(false);
}

fn check_bottom_merge(unicode: bool, with_message: bool) {
    let s = set(unicode);
    let rows = [GRID_ROW.to_string(), grid_rule(&s, s.up)];
    let got = render(&base(body(&rows), with_message), unicode);
    let c = command_index(&got);
    let text = got.join("\n");
    // Frame rule, then the (unmerged) first content row, then the merged rule.
    assert_eq!(got[c + 1], rule(&s, &[]), "top frame rule:\n{text}");
    assert_eq!(got[c + 2], row(&s, GRID_ROW));
    let mut junctions = grid_junctions(s.up);
    if !with_message {
        // The status band's dividers continue below the merged rule.
        junctions.extend([(14, s.dn), (30, s.dn)]);
    }
    assert_eq!(
        got[c + 3],
        rule(&s, &junctions),
        "merged bottom rule:\n{text}"
    );
    assert_eq!(got.iter().filter(|l| l.contains(GRID_ROW)).count(), 1);
    assert_no_consecutive_rules(&got);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.30 -- a grid bottom rule merges with the frame rule before
// the status band into ONE rule with bottom-tee junctions.
#[test]
fn grid_bottom_rule_merges_with_frame_rule_before_status_band() {
    check_bottom_merge(true, false);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.30 -- same, when a message band follows the merged rule.
#[test]
fn grid_bottom_rule_merges_with_frame_rule_before_message_band() {
    check_bottom_merge(true, true);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.30, 30.23 -- ASCII equivalents of the bottom merge.
#[test]
fn ascii_grid_bottom_rule_merges_with_frame_rule() {
    check_bottom_merge(false, false);
    check_bottom_merge(false, true);
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.29, 30.30 -- a grid with BOTH rules merges at both ends and
// never emits two consecutive rules.
#[test]
fn grid_with_top_and_bottom_rules_merges_at_both_boundaries() {
    let s = set(true);
    let rows = [
        grid_rule(&s, s.dn),
        GRID_ROW.to_string(),
        grid_rule(&s, s.up),
    ];
    let got = render(&base(body(&rows), false), true);
    assert_no_consecutive_rules(&got);
    let c = command_index(&got);
    assert_eq!(got[c + 1], rule(&s, &grid_junctions(s.dn)));
    assert_eq!(got[c + 2], row(&s, GRID_ROW));
}

// Validates: workspace-framework Requirement 7.32; screen-snapshot-scrm
// Requirement 30.29, 30.30 -- a SELF-BOXED grid (its own corners, which are
// `+` in the ASCII fallback) is ordinary content and is NOT merged.
#[test]
fn self_boxed_ascii_grid_is_not_merged_with_frame_rules() {
    let rows = [
        "+-----+-----+".to_string(),
        "| a   | b   |".to_string(),
        "+-----+-----+".to_string(),
    ];
    let got = render(&base(body(&rows), false), false);
    let c = command_index(&got);
    let text = got.join("\n");
    assert_eq!(
        got[c + 1],
        rule(&set(false), &[]),
        "plain frame rule:\n{text}"
    );
    assert_eq!(got[c + 2], row(&set(false), "+-----+-----+"));
    assert_eq!(
        got.iter().filter(|l| l.contains("| a   | b   |")).count(),
        1
    );
}

// Validates: workspace-framework Requirement 7.30; screen-snapshot-scrm
// Requirement 30.29 -- with NO content the single frame rule is not doubled.
#[test]
fn empty_content_does_not_double_the_frame_rule() {
    let got = render(&base(Vec::new(), false), true);
    assert_no_consecutive_rules(&got);
}
