//! GOLDEN tests for the framed (common-frame) renderer (CR-NR-115, screen-
//! snapshot-scrm Requirement 30.18-30.24). Each test compares the FULL rendered
//! text, line for line, against a fixed expected fixture (no presence/position
//! shortcuts). Rule lines are built by `rule`, a tiny oracle independent of the
//! renderer: a left glyph, fill glyphs, junction glyphs at fixed line columns,
//! and a right glyph.

use crate::model::ScreenModel;
use crate::render::framed::{render_framed, RIGHT_ALIGN};
use crate::render::RenderOptions;

/// Inner content width of the synthetic model below (its widest band, the
/// status bar, is 41 characters). A full line is `WIDTH + 4` characters.
const WIDTH: usize = 41;

/// Build one rule line of `WIDTH + 4` characters: `left`, fill, `right`, with the
/// given `(column, glyph)` junctions (columns are line indexes).
fn rule(left: char, fill: char, right: char, junctions: &[(usize, char)]) -> String {
    let mut s = String::new();
    s.push(left);
    for col in 1..WIDTH + 3 {
        match junctions.iter().find(|(c, _)| *c == col) {
            Some((_, g)) => s.push(*g),
            None => s.push(fill),
        }
    }
    s.push(right);
    s
}

/// The synthetic model: title bar (`#` cell + right-aligned controls), a menu
/// band, a two-tab band, the centered title, the Command line (field empty),
/// two content rows, optionally ONE message, a status band and optionally a
/// key-line band.
fn model(with_message: bool, with_footer: bool) -> ScreenModel {
    let mut m = ScreenModel::new("Configuration");
    m.header_lines = vec![
        format!("# | FileForge Workbench{RIGHT_ALIGN}_ [] X"),
        "Settings | Help | SCRM".into(),
        ">[CONFIG] x< | [CONFIG] x".into(),
    ];
    m.command_line = "____".into();
    m.body_lines = vec!["content one".into(), "content two".into()];
    if with_message {
        m.messages = vec!["a message".into()];
    }
    m.status_bar.text = "Last: 11:02 | Prof: default | Ln 1, Col 1".into();
    if with_footer {
        m.footer_lines = vec!["Base | F1 Help | F12 Retrieve".into()];
    }
    m
}

fn lines(out: &str) -> Vec<String> {
    out.lines().map(str::to_string).collect()
}

// Validates: screen-snapshot-scrm Requirement 30.18, 30.19, 30.20, 30.21, 30.22,
// 30.23, 30.29 -- ASCII fallback golden: ruled bands, '#' cell + right-aligned
// controls, cell dividers, Command line filled with underscores to the border,
// NO blank row before the status bar, message band, key line last.
#[test]
fn ascii_golden_full_frame_with_message_and_key_line() {
    let out = render_framed(&model(true, true), RenderOptions { unicode_box: false });
    let plain = rule('+', '-', '+', &[]);
    let plain_bottom = plain.clone();
    let expected: Vec<String> = vec![
        rule('+', '-', '+', &[(4, '+')]),
        "| # | FileForge Workbench            _ [] X |".into(),
        rule('+', '-', '+', &[(4, '+'), (11, '+'), (18, '+')]),
        "| Settings | Help | SCRM                    |".into(),
        rule('+', '-', '+', &[(11, '+'), (15, '+'), (18, '+')]),
        "| >[CONFIG] x< | [CONFIG] x                 |".into(),
        rule('+', '-', '+', &[(15, '+')]),
        "|               Configuration               |".into(),
        plain.clone(),
        "| Command ===> ____________________________ |".into(),
        plain.clone(),
        "| content one                               |".into(),
        "| content two                               |".into(),
        plain,
        "| a message                                 |".into(),
        rule('+', '-', '+', &[(14, '+'), (30, '+')]),
        "| Last: 11:02 | Prof: default | Ln 1, Col 1 |".into(),
        rule('+', '-', '+', &[(14, '+'), (30, '+')]),
        "| Base | F1 Help | F12 Retrieve             |".into(),
        plain_bottom,
    ];
    assert_eq!(lines(&out), expected, "full text:\n{out}");
}

// Validates: screen-snapshot-scrm Requirement 30.21, 30.22, 30.23 -- with NO
// message and NO key line the content rule runs straight into the status band
// (no blank row) and the status band closes with the bottom border.
#[test]
fn ascii_golden_without_message_and_key_line() {
    let out = render_framed(&model(false, false), RenderOptions { unicode_box: false });
    let got = lines(&out);
    let n = got.len();
    assert_eq!(
        got[n - 3],
        rule('+', '-', '+', &[(14, '+'), (30, '+')]),
        "content closing rule carries the status dividers below it"
    );
    assert_eq!(
        got[n - 4],
        "| content two                               |",
        "last content row is directly above that rule"
    );
    assert_eq!(
        got[n - 2],
        "| Last: 11:02 | Prof: default | Ln 1, Col 1 |",
        "status band directly after the rule (no blank row)"
    );
    assert_eq!(
        got[n - 1],
        rule('+', '-', '+', &[(14, '+'), (30, '+')]),
        "status closing border carries the dividers"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.18, 30.20, 30.22 -- Unicode
// golden: corner, tee-down, tee-up glyphs at the divider columns, horizontal
// and vertical box glyphs for rules and cell dividers.
#[test]
fn unicode_golden_full_frame_with_message_and_key_line() {
    let out = render_framed(&model(true, true), RenderOptions::default());
    let (h, tl, tr, el, er) = ('\u{2500}', '\u{250C}', '\u{2510}', '\u{251C}', '\u{2524}');
    let (bl, br, dn, up) = ('\u{2514}', '\u{2518}', '\u{252C}', '\u{2534}');
    let expected: Vec<String> = vec![
        rule(tl, h, tr, &[(4, dn)]),
        "\u{2502} # \u{2502} FileForge Workbench            _ [] X \u{2502}".into(),
        rule(el, h, er, &[(4, up), (11, dn), (18, dn)]),
        "\u{2502} Settings \u{2502} Help \u{2502} SCRM                    \u{2502}".into(),
        rule(el, h, er, &[(11, up), (15, dn), (18, up)]),
        "\u{2502} >[CONFIG] x< \u{2502} [CONFIG] x                 \u{2502}".into(),
        rule(el, h, er, &[(15, up)]),
        "\u{2502}               Configuration               \u{2502}".into(),
        rule(el, h, er, &[]),
        "\u{2502} Command ===> ____________________________ \u{2502}".into(),
        rule(el, h, er, &[]),
        "\u{2502} content one                               \u{2502}".into(),
        "\u{2502} content two                               \u{2502}".into(),
        rule(el, h, er, &[]),
        "\u{2502} a message                                 \u{2502}".into(),
        rule(el, h, er, &[(14, dn), (30, dn)]),
        "\u{2502} Last: 11:02 \u{2502} Prof: default \u{2502} Ln 1, Col 1 \u{2502}".into(),
        rule(el, h, er, &[(14, up), (30, up)]),
        "\u{2502} Base | F1 Help | F12 Retrieve             \u{2502}".into(),
        rule(bl, h, br, &[]),
    ];
    assert_eq!(lines(&out), expected, "full text:\n{out}");
}

// Validates: screen-snapshot-scrm Requirement 30.20 -- a divider that continues
// through a rule (same column above and below) draws the cross junction.
#[test]
fn cross_junction_where_dividers_align_across_a_rule() {
    let mut m = ScreenModel::new("T");
    m.header_lines = vec!["aa | bb".into(), "cc | dd".into()];
    let out = render_framed(&m, RenderOptions::default());
    let got = lines(&out);
    // Line 0 top border, 1 band, 2 rule between the two bands.
    assert!(
        got[2].contains('\u{253C}'),
        "cross between aligned bands:\n{out}"
    );
}
