//! Pure, GUI-free layout helpers for the Config View tree render
//! (configuration-system Requirement 22). These compute the GLOBAL
//! vertically-aligned Value_Column position across EVERY currently-visible row
//! of the whole panel -- the header rows (`Filter`, `Unsaved edits`), the
//! bracketed group-header rows, and the indented child key rows all share one
//! `:` column.
//!
//! SUPERSEDES both the box-drawing-connector approach (commit a3059de) AND the
//! PER-GROUP value column of commit 128d8d2. The owner explicitly asked for a
//! SINGLE GLOBAL aligned `:` column across the whole panel (not per group), with
//! the group headers flush-left as `[Group] : expanded|collapsed` and the child
//! rows indented exactly one level under their group. There are no box-drawing
//! glyphs and no `. . .` dot-leaders -- the alignment is a single padded `:`
//! column only.
//!
//! The mechanism is monospace CHARACTER-COUNT padding: every row's LEFT-LABEL is
//! built as one monospace string (a group's left-label is its bracketed name; a
//! child's left-label is the one-level indent + its full key; a header row's
//! left-label is its caption). Right-padding each left-label to the max visible
//! left-label width makes equal char counts map to equal pixel widths, so the
//! `:` separator and the value/state widget land in the same column on every row
//! of the panel (the `primary_option_menu/calendar_header.rs` idiom).
//!
//! Everything here is pure and deterministic so it is unit-tested independently
//! of egui (AC 22.12).

/// The number of monospace spaces a child key row is indented beneath its group
/// header (exactly one level, AC 22.4). The indent is baked INTO the child's
/// left-label string (not egui container indentation) so the global `:` column
/// is computed over the real on-screen left-label width.
pub const CHILD_INDENT: usize = 4;

/// Build a group header's LEFT-LABEL: the bracketed display name, flush-left
/// (no indent), e.g. `"[Editor]"`. Pure.
///
/// Validates: Requirement 22.3, 22.4
pub fn group_left_label(display_name: &str) -> String {
    format!("[{}]", display_name)
}

/// Build a child key row's LEFT-LABEL: one level of indent then the full key,
/// e.g. `"    editor.tab_size"`. Pure.
///
/// Validates: Requirement 22.4
pub fn child_left_label(key: &str) -> String {
    format!("{}{}", " ".repeat(CHILD_INDENT), key)
}

/// The number of monospace CHARACTER cells the GLOBAL `:` column must sit at:
/// the max char-count over EVERY currently-visible left-label (header rows,
/// bracketed group rows, and indented child rows), plus one padding space.
/// Returns 0 for the empty set.
///
/// This is the GLOBAL width across the whole panel (AC 22.5, superseding the
/// per-group width of commit 128d8d2): the caller collects every visible
/// left-label -- the header captions, each `group_left_label`, and each
/// `child_left_label` of an expanded group -- and passes them all here.
///
/// Validates: Requirement 22.5, 22.6, 22.12
pub fn global_key_column_width(visible_left_labels: &[String]) -> usize {
    visible_left_labels
        .iter()
        .map(|label| label.chars().count())
        .max()
        .map(|w| w + 1)
        .unwrap_or(0)
}

/// Build the left segment of a row as ONE monospace string: the left-label
/// right-padded to `column` cells, then `" : "`. Appending the value widget
/// after this lands the `:` in the same GLOBAL column on every row. Pure.
///
/// Validates: Requirement 22.5
pub fn aligned_left_segment(left_label: &str, column: usize) -> String {
    let pad = column.saturating_sub(left_label.chars().count());
    format!("{}{} : ", left_label, " ".repeat(pad))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 22.12 -- the empty set yields a zero-width column.
    #[test]
    fn global_key_column_width_empty_is_zero() {
        let labels: Vec<String> = Vec::new();
        assert_eq!(global_key_column_width(&labels), 0);
    }

    // Validates: Requirement 22.12 -- a single label: column = width + 1.
    #[test]
    fn global_key_column_width_single_label() {
        let labels = vec![child_left_label("accessibility.reduce_motion")];
        let expected = labels[0].chars().count() + 1;
        assert_eq!(global_key_column_width(&labels), expected);
    }

    // Validates: Requirement 22.5, 22.12 -- the GLOBAL width is the max over ALL
    // visible left-labels: header captions, bracketed groups, and indented
    // children together (the widest of the whole panel decides).
    #[test]
    fn global_key_column_width_takes_widest_over_whole_panel() {
        let labels = vec![
            "Filter".to_string(),
            "Unsaved edits".to_string(),
            group_left_label("Editor"),
            child_left_label("editor.tab_size"),
            child_left_label("editor.trim_trailing_whitespace"),
            group_left_label("Catalogs"),
            child_left_label("catalogs.default_mainframe_root"),
        ];
        let widest = labels.iter().map(|l| l.chars().count()).max().unwrap();
        assert_eq!(global_key_column_width(&labels), widest + 1);
        // The widest here is an indented child, not a group or header row.
        assert_eq!(
            widest,
            child_left_label("catalogs.default_mainframe_root")
                .chars()
                .count()
        );
    }

    // Validates: Requirement 22.6 -- a collapsed group contributes only its
    // header row (not its children), so collapsing can narrow the global column.
    #[test]
    fn global_key_column_width_excludes_hidden_children() {
        let expanded = vec![
            group_left_label("Catalogs"),
            child_left_label("catalogs.default_mainframe_root"),
        ];
        // Collapsed: only the bracketed group header remains visible.
        let collapsed = vec![group_left_label("Catalogs")];
        let expanded_w = global_key_column_width(&expanded);
        let collapsed_w = global_key_column_width(&collapsed);
        assert!(
            collapsed_w < expanded_w,
            "hiding a wide child narrows the global column"
        );
    }

    // Validates: Requirement 22.5 -- the aligned segment right-pads the label to
    // the column then appends " : "; the ':' lands at the same cell for a header
    // row, a group row, and a child row (GLOBAL alignment).
    #[test]
    fn aligned_left_segment_aligns_colon_globally() {
        let header = "Unsaved edits".to_string();
        let group = group_left_label("Editor");
        let child = child_left_label("editor.trim_trailing_whitespace");
        let labels = vec![header.clone(), group.clone(), child.clone()];
        let column = global_key_column_width(&labels);

        let seg_header = aligned_left_segment(&header, column);
        let seg_group = aligned_left_segment(&group, column);
        let seg_child = aligned_left_segment(&child, column);

        let colon_header = seg_header.split(" : ").next().unwrap().chars().count();
        let colon_group = seg_group.split(" : ").next().unwrap().chars().count();
        let colon_child = seg_child.split(" : ").next().unwrap().chars().count();
        assert_eq!(colon_header, column);
        assert_eq!(colon_group, column);
        assert_eq!(colon_child, column);
    }

    // Validates: Requirement 22.4 -- a child row is indented exactly one level
    // (CHILD_INDENT spaces) beneath its flush-left group header.
    #[test]
    fn child_is_indented_one_level_under_flush_left_group() {
        let group = group_left_label("Editor");
        let child = child_left_label("editor.tab_size");
        assert!(!group.starts_with(' '), "group header is flush-left");
        assert!(
            child.starts_with(&" ".repeat(CHILD_INDENT)),
            "child row is indented one level"
        );
        assert!(
            !child.starts_with(&" ".repeat(CHILD_INDENT + 1)),
            "child row is indented EXACTLY one level"
        );
    }
}
