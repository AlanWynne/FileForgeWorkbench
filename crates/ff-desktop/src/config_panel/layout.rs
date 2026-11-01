//! Pure, GUI-free layout helpers for the Config View tree render
//! (configuration-system Requirement 22). These compute the PER-GROUP
//! vertically-aligned Value_Column position over a single group's
//! currently-visible key labels.
//!
//! SUPERSEDES the box-drawing-connector approach (commit a3059de): egui's
//! native `CollapsingHeader` now owns hierarchy, indentation, and the
//! expand/collapse expander, so the obsolete connector glyphs (U+2500-U+257F)
//! and the former single GLOBAL `value_column_width` are gone. Alignment is now
//! PER GROUP: within each `CollapsingHeader` body the `:`/value column is
//! computed over THAT GROUP'S visible keys only, which is robust against egui's
//! per-header indentation (strict global pixel alignment across groups fights
//! that indentation and is deliberately NOT attempted -- design delta D10/D11).
//!
//! The mechanism is monospace CHARACTER-COUNT padding: building the left
//! segment as one monospace string and right-padding to the max visible
//! key-label width inside the group makes equal char counts map to equal pixel
//! widths, so the `:` separator and value widget land in the same column on
//! every row of that group (the `primary_option_menu/calendar_header.rs` idiom).
//!
//! Everything here is pure and deterministic so it is unit-tested independently
//! of egui (AC 22.12).

/// The number of monospace CHARACTER cells the `:` column must sit at WITHIN A
/// GROUP: the max char-count of the key text over the group's currently VISIBLE
/// key rows, plus one padding space. Group-header rows do not participate.
/// Returns 0 for the empty set (a group with no visible key rows).
///
/// Validates: Requirement 22.5, 22.6, 22.12
pub fn value_column_width(visible_keys_in_group: &[String]) -> usize {
    visible_keys_in_group
        .iter()
        .map(|k| k.chars().count())
        .max()
        .map(|w| w + 1)
        .unwrap_or(0)
}

/// Build the left segment of a key row as ONE monospace string: the key text
/// right-padded to `column` cells, then `" : "`. Appending the value widget
/// after this lands the `:` in the same column on every row of the group. The
/// connector prefix is GONE -- egui's `CollapsingHeader` owns the indentation.
/// Pure.
///
/// Validates: Requirement 22.5
pub fn aligned_left_segment(key: &str, column: usize) -> String {
    let pad = column.saturating_sub(key.chars().count());
    format!("{}{} : ", key, " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 22.12 -- empty group yields a zero-width column.
    #[test]
    fn value_column_width_empty_group_is_zero() {
        let keys: Vec<String> = Vec::new();
        assert_eq!(value_column_width(&keys), 0);
    }

    // Validates: Requirement 22.12 -- single key: column = key width + 1.
    #[test]
    fn value_column_width_single_key() {
        let keys = vec!["accessibility.reduce_motion".to_string()];
        let expected = keys[0].chars().count() + 1;
        assert_eq!(value_column_width(&keys), expected);
    }

    // Validates: Requirement 22.5, 22.12 -- within a group the column is the max
    // over the group's visible key labels (the widest key decides).
    #[test]
    fn value_column_width_takes_widest_key_in_group() {
        let keys = vec![
            "editor.tab_size".to_string(),
            "editor.trim_trailing_whitespace".to_string(),
            "editor.line_endings".to_string(),
        ];
        let widest = keys.iter().map(|k| k.chars().count()).max().unwrap();
        assert_eq!(value_column_width(&keys), widest + 1);
    }

    // Validates: Requirement 22.6, 22.12 -- a filtered subset recomputes from
    // only the keys passed (a filtered-out long key cannot widen the column).
    #[test]
    fn value_column_width_filtered_subset_ignores_absent_keys() {
        let full = vec![
            "catalogs.default_mainframe_root".to_string(),
            "catalogs.level".to_string(),
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

    // Validates: Requirement 22.5 -- the aligned segment right-pads the key to
    // the column then appends " : "; the ':' lands at the same cell for keys of
    // differing length within the group.
    #[test]
    fn aligned_left_segment_pads_to_column_and_aligns_colon() {
        let short = "editor.tab_size";
        let long = "editor.trim_trailing_whitespace";
        let keys = vec![short.to_string(), long.to_string()];
        let column = value_column_width(&keys);

        let seg_short = aligned_left_segment(short, column);
        let seg_long = aligned_left_segment(long, column);

        let colon_char_short = seg_short.split(" : ").next().unwrap().chars().count();
        let colon_char_long = seg_long.split(" : ").next().unwrap().chars().count();
        assert_eq!(
            colon_char_short, colon_char_long,
            "the ':' must align to the same column for both keys in the group"
        );
        assert_eq!(colon_char_short, column);
    }

    // Validates: Requirement 22.5 -- the widest key needs a single pad cell (the
    // ':' sits exactly one cell past it, from the +1 in value_column_width).
    #[test]
    fn aligned_left_segment_widest_key_single_pad() {
        let key = "catalogs.default_mainframe_root";
        let keys = vec![key.to_string()];
        let column = value_column_width(&keys);
        let seg = aligned_left_segment(key, column);
        assert_eq!(column, key.chars().count() + 1);
        let before_colon = seg.split(" : ").next().unwrap();
        assert_eq!(before_colon.chars().count(), column);
        assert!(before_colon.ends_with(' '));
    }
}
