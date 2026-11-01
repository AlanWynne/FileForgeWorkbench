//! Pure, GUI-free layout helpers for the Config View tree re-skin
//! (configuration-system Requirement 22, design delta D5/D6/D9). These compute
//! the box-drawing tree-connector prefixes and the single vertically-aligned
//! Value_Column position over the currently-visible key labels.
//!
//! The mechanism is monospace CHARACTER-COUNT padding (egui::Grid is rejected,
//! design D9): building the left segment as one monospace string and padding to
//! the max visible key-label width makes equal char counts map to equal pixel
//! widths, so the `:` separator and value widget land in the same column on
//! every row (the `primary_option_menu/calendar_header.rs` idiom).
//!
//! Everything here is pure and deterministic so it is unit-tested independently
//! of egui (AC 22.12).

/// The connector kind for a key row in the strictly two-level tree
/// (group -> key). A key is either a non-last child of its group (`Tee`) or the
/// last child (`Last`); there are no intermediate levels, so only these two
/// connectors are ever drawn (AC 22.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Connector {
    /// Branch-tee connector for every child except the last.
    Tee,
    /// L-connector for the LAST child of a group.
    Last,
}

/// The four-cell box-drawing prefix for a non-last child.
/// Code points: U+251C (BOX DRAWINGS LIGHT VERTICAL AND RIGHT, "tee"),
/// U+2500 (BOX DRAWINGS LIGHT HORIZONTAL), U+2500 again, then an ASCII space.
const TEE_PREFIX: &str = "\u{251c}\u{2500}\u{2500} ";

/// The four-cell box-drawing prefix for the last child.
/// Code points: U+2514 (BOX DRAWINGS LIGHT UP AND RIGHT, "L"),
/// U+2500 (BOX DRAWINGS LIGHT HORIZONTAL), U+2500 again, then an ASCII space.
const LAST_PREFIX: &str = "\u{2514}\u{2500}\u{2500} ";

/// The four-cell connector prefix (three box-drawing glyphs' worth plus a
/// trailing space). Both forms are the SAME monospace width (4 cells), so they
/// do not perturb the Value_Column computation.
pub fn connector_prefix(c: Connector) -> &'static str {
    match c {
        Connector::Tee => TEE_PREFIX,
        Connector::Last => LAST_PREFIX,
    }
}

/// The label text drawn to the LEFT of the `:` column for a key row: the
/// connector prefix followed by the full key path. Pure; no egui.
pub fn key_label(connector: Connector, key: &str) -> String {
    format!("{}{}", connector_prefix(connector), key)
}

/// The number of monospace CHARACTER cells the `:` column must sit at: the max
/// char-count of `key_label` over the CURRENTLY VISIBLE key rows, plus one
/// padding space. Group-header rows do not participate. Returns 0 for the empty
/// set (no visible key rows).
pub fn value_column_width(visible_key_labels: &[String]) -> usize {
    visible_key_labels
        .iter()
        .map(|l| l.chars().count())
        .max()
        .map(|w| w + 1)
        .unwrap_or(0)
}

/// Build the FULL left segment of a key row as ONE monospace string: the
/// `key_label` padded on the right to `column` cells, then `" : "`. Appending
/// the value widget after this lands the `:` in the same column on every row.
/// Pure.
pub fn aligned_left_segment(connector: Connector, key: &str, column: usize) -> String {
    let label = key_label(connector, key);
    let pad = column.saturating_sub(label.chars().count());
    format!("{}{} : ", label, " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 22.4 -- the two connector prefixes are the correct
    // box-drawing glyphs and are equal (four-cell) width.
    #[test]
    fn connector_prefixes_are_correct_and_equal_width() {
        assert_eq!(
            connector_prefix(Connector::Tee),
            "\u{251c}\u{2500}\u{2500} "
        );
        assert_eq!(
            connector_prefix(Connector::Last),
            "\u{2514}\u{2500}\u{2500} "
        );
        // Both prefixes are the same monospace width so they do not perturb the
        // Value_Column (design D4).
        assert_eq!(
            connector_prefix(Connector::Tee).chars().count(),
            connector_prefix(Connector::Last).chars().count()
        );
        assert_eq!(connector_prefix(Connector::Tee).chars().count(), 4);
    }

    // Validates: Requirement 22.4 -- key_label prepends the connector to the key.
    #[test]
    fn key_label_prepends_connector() {
        assert_eq!(
            key_label(Connector::Tee, "editor.tab_size"),
            "\u{251c}\u{2500}\u{2500} editor.tab_size"
        );
        assert_eq!(
            key_label(Connector::Last, "editor.trim_trailing_whitespace"),
            "\u{2514}\u{2500}\u{2500} editor.trim_trailing_whitespace"
        );
    }

    // Validates: Requirement 22.12 -- empty set yields a zero-width column.
    #[test]
    fn value_column_width_empty_set_is_zero() {
        let labels: Vec<String> = Vec::new();
        assert_eq!(value_column_width(&labels), 0);
    }

    // Validates: Requirement 22.12 -- single key row: column = label width + 1.
    #[test]
    fn value_column_width_single_group_single_key() {
        let labels = vec![key_label(Connector::Last, "accessibility.reduce_motion")];
        let expected = labels[0].chars().count() + 1;
        assert_eq!(value_column_width(&labels), expected);
    }

    // Validates: Requirement 22.5, 22.12 -- multiple groups: column is the max
    // over ALL visible key labels (one shared column for the panel).
    #[test]
    fn value_column_width_multiple_groups_takes_max() {
        let labels = vec![
            key_label(Connector::Last, "ui.x"),
            key_label(Connector::Tee, "editor.trim_trailing_whitespace"),
            key_label(Connector::Last, "logging.level"),
        ];
        // Widest is the editor key; connector width is equal for all so the raw
        // key length decides.
        let widest = labels.iter().map(|l| l.chars().count()).max().unwrap();
        assert_eq!(value_column_width(&labels), widest + 1);
    }

    // Validates: Requirement 22.6, 22.12 -- a filtered subset recomputes from
    // only the labels passed (a collapsed/filtered-out long key cannot widen it).
    #[test]
    fn value_column_width_filtered_subset_ignores_absent_labels() {
        let full = vec![
            key_label(Connector::Tee, "catalogs.default_mainframe_root"),
            key_label(Connector::Last, "logging.level"),
        ];
        let subset = vec![full[1].clone()];
        let full_width = value_column_width(&full);
        let subset_width = value_column_width(&subset);
        assert!(
            subset_width < full_width,
            "a narrower visible subset yields a narrower column"
        );
        assert_eq!(subset_width, full[1].chars().count() + 1);
    }

    // Validates: Requirement 22.5 -- the aligned segment right-pads the label to
    // the column then appends " : "; the ':' lands at the same cell for keys of
    // differing length.
    #[test]
    fn aligned_left_segment_pads_to_column_and_aligns_colon() {
        let short = "editor.tab_size";
        let long = "editor.trim_trailing_whitespace";
        let labels = vec![
            key_label(Connector::Tee, short),
            key_label(Connector::Last, long),
        ];
        let column = value_column_width(&labels);

        let seg_short = aligned_left_segment(Connector::Tee, short, column);
        let seg_long = aligned_left_segment(Connector::Last, long, column);

        // Both segments pad the label to `column` cells before " : ", so the
        // ':' sits at the same character index in both.
        let colon_short = seg_short.find(" : ").unwrap();
        let colon_long = seg_long.find(" : ").unwrap();
        // find() returns a BYTE index; the padded region is ASCII spaces but the
        // connector is multi-byte, so compare by char position of the ':'.
        let colon_char_short = seg_short[..colon_short].chars().count();
        let colon_char_long = seg_long[..colon_long].chars().count();
        assert_eq!(
            colon_char_short, colon_char_long,
            "the ':' must align to the same column for both keys"
        );
        assert_eq!(colon_char_short, column);
    }

    // Validates: Requirement 22.5 -- the widest label needs no padding (the ':'
    // sits exactly one cell past it, from the +1 in value_column_width).
    #[test]
    fn aligned_left_segment_widest_label_single_pad() {
        let key = "catalogs.default_mainframe_root";
        let labels = vec![key_label(Connector::Last, key)];
        let column = value_column_width(&labels);
        let seg = aligned_left_segment(Connector::Last, key, column);
        // label is column-1 cells wide; one pad space then " : ".
        let label_len = key_label(Connector::Last, key).chars().count();
        assert_eq!(column, label_len + 1);
        let before_colon = seg.split(" : ").next().unwrap();
        assert_eq!(before_colon.chars().count(), column);
        assert!(before_colon.ends_with(' '));
    }
}
