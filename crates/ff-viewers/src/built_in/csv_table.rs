//! CSV Table Viewer — renders CSV/TSV files as formatted table grids.
//!
//! Displays tabular data with column headers (from the first row), aligned
//! columns, row numbering, and horizontal scrolling support for wide tables.

use crate::trait_def::FileViewer;

/// Built-in viewer for CSV/TSV files as formatted table grids.
///
/// Renders CSV content with column headers from the first row, aligned columns,
/// and row numbering.
pub struct CsvTableViewer {
    /// Cached display name.
    display_name: String,
    /// Delimiter character (default: `,`).
    delimiter: char,
    /// Whether the first row is treated as headers.
    has_header: bool,
}

impl CsvTableViewer {
    /// Create a new CSV Table Viewer instance with default settings.
    pub fn new() -> Self {
        Self {
            display_name: "CSV Table".to_string(),
            delimiter: ',',
            has_header: true,
        }
    }

    /// Parse CSV content into rows of fields.
    fn parse_rows(&self, content: &[u8]) -> Vec<Vec<String>> {
        let text = String::from_utf8_lossy(content);
        text.lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                line.split(self.delimiter)
                    .map(|field| field.trim().to_string())
                    .collect()
            })
            .collect()
    }

    /// Calculate the maximum width for each column.
    fn column_widths(&self, rows: &[Vec<String>]) -> Vec<usize> {
        if rows.is_empty() {
            return vec![];
        }

        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut widths = vec![0usize; max_cols];

        for row in rows {
            for (i, field) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(field.len());
                }
            }
        }

        // Ensure minimum column width of 3
        for w in &mut widths {
            *w = (*w).max(3);
        }

        widths
    }
}

impl Default for CsvTableViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl FileViewer for CsvTableViewer {
    fn viewer_key(&self) -> &str {
        "csv-table"
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        "Renders CSV/TSV files as a formatted table grid with column headers and row numbering"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["csv", "tsv"]
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["text/csv"]
    }

    fn can_render(&self, uri: &str, _content_sample: &[u8]) -> bool {
        let uri_lower = uri.to_lowercase();
        self.supported_extensions()
            .iter()
            .any(|ext| uri_lower.ends_with(&format!(".{ext}")))
    }

    fn render(&self, content: &[u8]) -> String {
        let rows = self.parse_rows(content);
        if rows.is_empty() {
            return "(empty CSV)".to_string();
        }

        let widths = self.column_widths(&rows);
        let mut output = String::new();

        // Row number column width
        let row_num_width = rows.len().to_string().len().max(3);

        // Separator line
        let separator = {
            let mut sep = String::new();
            sep.push_str(&"-".repeat(row_num_width + 2));
            sep.push('+');
            for (i, w) in widths.iter().enumerate() {
                sep.push_str(&"-".repeat(w + 2));
                if i < widths.len() - 1 {
                    sep.push('+');
                }
            }
            sep.push('\n');
            sep
        };

        let mut row_iter = rows.iter().enumerate();

        // Header row (if has_header is true and we have data)
        if self.has_header {
            if let Some((_, header)) = row_iter.next() {
                output.push_str(&format!("{:>width$}", "#", width = row_num_width));
                output.push_str(" | ");
                for (i, field) in header.iter().enumerate() {
                    let col_width = widths.get(i).copied().unwrap_or(3);
                    output.push_str(&format!("{:<width$}", field, width = col_width));
                    if i < header.len() - 1 {
                        output.push_str(" | ");
                    }
                }
                output.push('\n');
                output.push_str(&separator);
            }
        } else {
            output.push_str(&separator);
        }

        // Data rows
        for (idx, row) in row_iter {
            let row_num = if self.has_header { idx } else { idx + 1 };
            output.push_str(&format!("{:>width$}", row_num, width = row_num_width));
            output.push_str(" | ");
            for (i, field) in row.iter().enumerate() {
                let col_width = widths.get(i).copied().unwrap_or(3);
                output.push_str(&format!("{:<width$}", field, width = col_width));
                if i < row.len() - 1 {
                    output.push_str(" | ");
                }
            }
            output.push('\n');
        }

        output
    }

    fn on_content_changed(&mut self, _new_content: &[u8]) {
        // Stub: no internal state to update for now.
    }

    fn configure(&mut self, config: &toml::Value) {
        if let Some(table) = config.as_table() {
            if let Some(delim) = table.get("delimiter").and_then(|v| v.as_str()) {
                if let Some(ch) = delim.chars().next() {
                    self.delimiter = ch;
                }
            }
            if let Some(header) = table.get("has_header").and_then(|v| v.as_bool()) {
                self.has_header = header;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_key_is_csv_table() {
        // Validates: Requirement 4 AC 4
        let viewer = CsvTableViewer::new();
        assert_eq!(viewer.viewer_key(), "csv-table");
    }

    #[test]
    fn supported_extensions_include_csv_and_tsv() {
        // Validates: Requirement 4 AC 4
        let viewer = CsvTableViewer::new();
        let exts = viewer.supported_extensions();
        assert!(exts.contains(&"csv"));
        assert!(exts.contains(&"tsv"));
    }

    #[test]
    fn supported_mime_types_include_text_csv() {
        // Validates: Requirement 4 AC 4
        let viewer = CsvTableViewer::new();
        assert!(viewer.supported_mime_types().contains(&"text/csv"));
    }

    #[test]
    fn can_render_matches_csv_extension() {
        let viewer = CsvTableViewer::new();
        assert!(viewer.can_render("file:///data.csv", b""));
        assert!(viewer.can_render("file:///data.tsv", b""));
        assert!(!viewer.can_render("file:///data.txt", b""));
    }

    #[test]
    fn render_produces_table_with_headers() {
        let viewer = CsvTableViewer::new();
        let content = b"Name,Age,City\nAlice,30,NYC\nBob,25,LA";
        let output = viewer.render(content);
        assert!(output.contains("Name"));
        assert!(output.contains("Age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("---")); // separator
    }

    #[test]
    fn render_handles_empty_content() {
        let viewer = CsvTableViewer::new();
        let output = viewer.render(b"");
        assert_eq!(output, "(empty CSV)");
    }

    #[test]
    fn configure_updates_delimiter() {
        // Validates: Requirement 10 AC 4
        let mut viewer = CsvTableViewer::new();
        let config = toml::Value::Table({
            let mut map = toml::map::Map::new();
            map.insert(
                "delimiter".to_string(),
                toml::Value::String("\t".to_string()),
            );
            map
        });
        viewer.configure(&config);
        assert_eq!(viewer.delimiter, '\t');
    }
}
