//! Property tests for ff-screen-model.
//!
//! Feature: screen-snapshot-scrm, Property 1: every renderer preserves all
//! visible field text as selectable characters (the "capture is copy/paste-able
//! text, not a raster" constraint).
//!
//! Validates: screen-snapshot-scrm Requirement 1.1, 1.2.

use ff_screen_model::{render, Field, RenderOptions, ScreenModel, SnapshotFormat};
use proptest::prelude::*;

/// Build a ScreenModel from simple, printable label/value pairs. We restrict the
/// alphabet to printable ASCII excluding characters that a structured format
/// legitimately transforms (`<`, `>`, `&` are HTML-escaped; `|` is
/// Markdown-escaped; `"` and `\` are YAML-escaped), so the "contains" assertion
/// tests preservation rather than escaping mechanics (those have their own unit
/// tests).
fn model_from(pairs: Vec<(String, String)>, title: String) -> ScreenModel {
    let mut m = ScreenModel::new(title);
    for (label, value) in pairs {
        m = m.with_field(Field::new(label, value));
    }
    m
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // Feature: screen-snapshot-scrm, Property 1.
    #[test]
    fn every_format_preserves_field_values(
        title in "[A-Za-z0-9 ]{1,20}",
        pairs in proptest::collection::vec(
            ("[A-Za-z0-9 ]{1,12}", "[A-Za-z0-9. ]{1,20}"),
            0..8,
        ),
    ) {
        let model = model_from(pairs.clone(), title.clone());
        for fmt in [
            SnapshotFormat::PlainText,
            SnapshotFormat::Ansi,
            SnapshotFormat::Markdown,
            SnapshotFormat::Html,
            SnapshotFormat::Yaml,
        ] {
            let out = render(&model, fmt, RenderOptions::default());
            // Title always present.
            prop_assert!(out.contains(&title), "format {:?} dropped title", fmt);
            // Every non-empty value present as selectable text.
            for (_label, value) in &pairs {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    prop_assert!(
                        out.contains(trimmed),
                        "format {:?} dropped value {:?}",
                        fmt,
                        trimmed
                    );
                }
            }
        }
    }
}
