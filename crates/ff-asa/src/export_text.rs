//! Plain text export.
//!
//! Renders the preview as UTF-8 plain text with configurable page separators,
//! preserving spacing and merged content as plain characters.

use crate::config::ExportPageSeparator;
use crate::preview::PreviewElement;

/// Options for text export.
// Validates: Requirement 11.1, 11.3
#[derive(Debug, Clone)]
pub struct TextExportOptions {
    /// How to represent page breaks in the output.
    pub page_separator: ExportPageSeparator,
}

impl Default for TextExportOptions {
    fn default() -> Self {
        Self {
            page_separator: ExportPageSeparator::Dashes,
        }
    }
}

/// Export the preview as plain text.
///
/// Rendering rules:
/// - `DataLine` → plain text content (no bold/underline markers)
/// - `SpacingLine` → empty line
/// - `PageBand` → page separator (dashes or form-feed)
/// - `HaltBand` → `--- PRINTER HALT ---`
// Validates: Requirement 11.1, 11.3–11.5
pub fn export_text(elements: &[PreviewElement], options: &TextExportOptions) -> String {
    let mut output = String::new();

    for element in elements {
        match element {
            PreviewElement::DataLine { content, .. } => {
                output.push_str(&content.plain_text());
                output.push('\n');
            }
            PreviewElement::SpacingLine { .. } => {
                output.push('\n');
            }
            PreviewElement::PageBand { page_number, .. } => match options.page_separator {
                ExportPageSeparator::Dashes => {
                    output.push_str(&format!("--- PAGE {} ---\n", page_number));
                }
                ExportPageSeparator::FormFeed => {
                    output.push('\x0C');
                    output.push('\n');
                }
            },
            PreviewElement::HaltBand { .. } => {
                output.push_str("--- PRINTER HALT ---\n");
            }
        }
    }

    output
}

/// Count page separators in exported text (for verification).
pub fn count_page_separators(text: &str, separator_style: ExportPageSeparator) -> usize {
    match separator_style {
        ExportPageSeparator::Dashes => text
            .lines()
            .filter(|l| l.starts_with("--- PAGE ") && l.ends_with(" ---"))
            .count(),
        ExportPageSeparator::FormFeed => text.chars().filter(|&c| c == '\x0C').count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge::MergedLine;

    fn data_line(text: &str, source_line: usize) -> PreviewElement {
        PreviewElement::DataLine {
            content: MergedLine::from_base(text, source_line),
            band_group: 0,
            page_line: 1,
        }
    }

    #[test]
    // Validates: Requirement 11.1
    fn export_data_lines_as_plain_text() {
        let elements = vec![data_line("HELLO WORLD", 0), data_line("LINE TWO", 1)];
        let text = export_text(&elements, &TextExportOptions::default());
        assert_eq!(text, "HELLO WORLD\nLINE TWO\n");
    }

    #[test]
    // Validates: Requirement 11.4
    fn export_spacing_lines_as_blank_lines() {
        let elements = vec![
            data_line("LINE 1", 0),
            PreviewElement::SpacingLine { band_group: 0 },
            data_line("LINE 2", 1),
        ];
        let text = export_text(&elements, &TextExportOptions::default());
        assert_eq!(text, "LINE 1\n\nLINE 2\n");
    }

    #[test]
    // Validates: Requirement 11.3
    fn export_page_bands_as_dashes() {
        let elements = vec![
            PreviewElement::PageBand {
                page_number: 1,
                is_explicit: true,
            },
            data_line("DATA", 0),
            PreviewElement::PageBand {
                page_number: 2,
                is_explicit: true,
            },
            data_line("MORE", 1),
        ];
        let text = export_text(&elements, &TextExportOptions::default());
        assert!(text.contains("--- PAGE 1 ---"));
        assert!(text.contains("--- PAGE 2 ---"));
    }

    #[test]
    // Validates: Requirement 11.3
    fn export_page_bands_as_formfeed() {
        let elements = vec![
            PreviewElement::PageBand {
                page_number: 1,
                is_explicit: true,
            },
            data_line("DATA", 0),
        ];
        let options = TextExportOptions {
            page_separator: ExportPageSeparator::FormFeed,
        };
        let text = export_text(&elements, &options);
        assert!(text.contains('\x0C'));
    }

    #[test]
    // Validates: Requirement 11.5
    fn export_merged_content_as_plain_chars() {
        let mut merged = MergedLine::from_base("HELLO", 0);
        merged.apply_overprint("HELLO"); // makes it bold
        let elements = vec![PreviewElement::DataLine {
            content: merged,
            band_group: 0,
            page_line: 1,
        }];
        let text = export_text(&elements, &TextExportOptions::default());
        // No bold markers, just plain text
        assert_eq!(text, "HELLO\n");
    }

    #[test]
    fn export_halt_band() {
        let elements = vec![PreviewElement::HaltBand { source_line: 0 }];
        let text = export_text(&elements, &TextExportOptions::default());
        assert_eq!(text, "--- PRINTER HALT ---\n");
    }

    #[test]
    fn count_page_separators_dashes() {
        let text = "--- PAGE 1 ---\nDATA\n--- PAGE 2 ---\nMORE\n";
        assert_eq!(count_page_separators(text, ExportPageSeparator::Dashes), 2);
    }

    #[test]
    fn count_page_separators_formfeed() {
        let text = "\x0C\nDATA\n\x0C\nMORE\n";
        assert_eq!(
            count_page_separators(text, ExportPageSeparator::FormFeed),
            2
        );
    }
}
