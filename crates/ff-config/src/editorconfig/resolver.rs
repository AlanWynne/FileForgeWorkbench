//! EditorConfig property resolver.
//!
//! Resolves EditorConfig properties for a given file path by traversing
//! parent directories, collecting applicable `.editorconfig` sections,
//! and merging properties with closer files taking priority.

use std::path::Path;

use super::parser::{load_editorconfig_file, matches_pattern, EditorConfigProperties};

/// Merge `source` properties into `target`, overriding only fields that are
/// explicitly set in `source`.
///
/// This implements field-by-field merge: if a field in `source` is `Some`,
/// it overwrites the corresponding field in `target`. Fields that are `None`
/// in `source` are left unchanged in `target`.
fn merge_properties(target: &mut EditorConfigProperties, source: &EditorConfigProperties) {
    if source.indent_style.is_some() {
        target.indent_style = source.indent_style;
    }
    if source.indent_size.is_some() {
        target.indent_size = source.indent_size;
    }
    if source.tab_width.is_some() {
        target.tab_width = source.tab_width;
    }
    if source.end_of_line.is_some() {
        target.end_of_line = source.end_of_line;
    }
    if source.charset.is_some() {
        target.charset = source.charset;
    }
    if source.trim_trailing_whitespace.is_some() {
        target.trim_trailing_whitespace = source.trim_trailing_whitespace;
    }
    if source.insert_final_newline.is_some() {
        target.insert_final_newline = source.insert_final_newline;
    }
}

/// Collect properties from a single `.editorconfig` file for the given file path.
///
/// Iterates sections in order, applying matching sections' properties on top of
/// each other. Per the EditorConfig spec, later sections in the same file that
/// match the same file path override earlier ones (field-by-field).
///
/// `relative_path` is the file's path relative to the directory containing the
/// `.editorconfig` file, using forward slashes as separators.
fn collect_matching_properties(
    sections: &[super::parser::EditorConfigSection],
    relative_path: &str,
) -> EditorConfigProperties {
    let mut result = EditorConfigProperties::default();
    for section in sections {
        if matches_pattern(&section.pattern, relative_path) {
            merge_properties(&mut result, &section.properties);
        }
    }
    result
}

/// Resolve EditorConfig properties for a given file path.
///
/// Performs standard EditorConfig path traversal:
/// 1. Starts from the file's parent directory
/// 2. At each directory level, looks for a `.editorconfig` file
/// 3. For each file found, determines which sections match the target file
/// 4. Collects properties from all matching files
/// 5. Stops when `root = true` is encountered or the filesystem root is reached
///
/// Properties from closer (deeper) files take priority over farther (shallower)
/// files. Within a single file, later sections override earlier ones for the
/// same matched file.
///
/// # Arguments
///
/// * `file_path` — The absolute path of the file to resolve properties for.
///
/// # Returns
///
/// The merged `EditorConfigProperties` combining all applicable sections
/// from all `.editorconfig` files in the path hierarchy.
pub fn resolve_editorconfig(file_path: &Path) -> EditorConfigProperties {
    let file_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(_) => file_path.to_path_buf(),
    };

    let mut merged = EditorConfigProperties::default();

    // Collect properties from each directory level. We walk from the file's
    // directory upward, so closer files are processed first.
    // Since closer files have higher priority, we collect them in order and
    // merge farther-file properties *under* (without overriding) closer ones.
    let mut current_dir = file_path.parent();

    while let Some(dir) = current_dir {
        let editorconfig_path = dir.join(".editorconfig");

        if let Some(ec_file) = load_editorconfig_file(&editorconfig_path) {
            // Compute the file's path relative to this .editorconfig's directory.
            // Use forward slashes for pattern matching regardless of OS.
            let relative_path = file_path
                .strip_prefix(dir)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .replace('\\', "/");

            let file_props = collect_matching_properties(&ec_file.sections, &relative_path);

            // Merge: farther file properties fill in gaps left by closer files.
            // `merged` already has closer-file values, so we only fill in `None` fields.
            merge_properties_under(&mut merged, &file_props);

            // Stop if this file declares root = true
            if ec_file.root {
                break;
            }
        }

        current_dir = dir.parent();
    }

    merged
}

/// Merge `source` properties into `target`, but only for fields that are
/// currently `None` in `target`.
///
/// This is the "lower priority" merge: source fills gaps without overriding
/// existing values. Used when merging farther (shallower) file properties
/// under already-collected closer (deeper) file properties.
fn merge_properties_under(target: &mut EditorConfigProperties, source: &EditorConfigProperties) {
    if target.indent_style.is_none() {
        target.indent_style = source.indent_style;
    }
    if target.indent_size.is_none() {
        target.indent_size = source.indent_size;
    }
    if target.tab_width.is_none() {
        target.tab_width = source.tab_width;
    }
    if target.end_of_line.is_none() {
        target.end_of_line = source.end_of_line;
    }
    if target.charset.is_none() {
        target.charset = source.charset;
    }
    if target.trim_trailing_whitespace.is_none() {
        target.trim_trailing_whitespace = source.trim_trailing_whitespace;
    }
    if target.insert_final_newline.is_none() {
        target.insert_final_newline = source.insert_final_newline;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editorconfig::parser::{Charset, EndOfLine, IndentSize, IndentStyle};
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a file with given content in a directory.
    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
    }

    // Validates: Requirement 6 AC 6.4 — path traversal from file's directory up to root
    #[test]
    fn resolve_walks_up_directory_tree() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        // Create directory structure: root/src/main.rs
        let src_dir = root_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let file_path = src_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // .editorconfig at root level
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*]\nindent_size = 2\n",
        );

        let props = resolve_editorconfig(&file_path);
        assert_eq!(props.indent_size, Some(IndentSize::Value(2)));
    }

    // Validates: Requirement 6 AC 6.4 — traversal stops at root = true
    #[test]
    fn resolve_stops_at_root_true() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        // Create structure: root/project/src/main.rs
        let project_dir = root_dir.join("project");
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let file_path = src_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // .editorconfig at root_dir level (above project) — should NOT be reached
        write_file(
            root_dir,
            ".editorconfig",
            "[*]\nindent_style = tab\ncharset = latin1\n",
        );

        // .editorconfig at project level with root = true — stops here
        write_file(
            &project_dir,
            ".editorconfig",
            "root = true\n\n[*]\nindent_style = space\n",
        );

        let props = resolve_editorconfig(&file_path);
        assert_eq!(props.indent_style, Some(IndentStyle::Space));
        // charset from root_dir's .editorconfig should NOT be applied
        assert_eq!(props.charset, None);
    }

    // Validates: Requirement 6 AC 6.5 — closer files take priority over farther files
    #[test]
    fn resolve_closer_file_takes_priority() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        // Create structure: root/src/main.rs
        let src_dir = root_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let file_path = src_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // .editorconfig at root level: indent_style = tab, indent_size = 4
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*]\nindent_style = tab\nindent_size = 4\ncharset = utf-8\n",
        );

        // .editorconfig at src level: indent_style = space (overrides root's tab)
        write_file(
            &src_dir,
            ".editorconfig",
            "[*]\nindent_style = space\nindent_size = 2\n",
        );

        let props = resolve_editorconfig(&file_path);
        // Closer (src/) wins for indent_style and indent_size
        assert_eq!(props.indent_style, Some(IndentStyle::Space));
        assert_eq!(props.indent_size, Some(IndentSize::Value(2)));
        // charset from root is still applied (not overridden by closer file)
        assert_eq!(props.charset, Some(Charset::Utf8));
    }

    // Validates: Requirement 6 AC 6.5 — within a single file, later sections override
    #[test]
    fn resolve_later_section_overrides_earlier_in_same_file() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let file_path = root_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        // Two sections matching *.rs — later one wins for indent_size
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*.rs]\nindent_size = 2\nindent_style = space\n\n[*.rs]\nindent_size = 4\n",
        );

        let props = resolve_editorconfig(&file_path);
        // Later section overrides indent_size
        assert_eq!(props.indent_size, Some(IndentSize::Value(4)));
        // indent_style from first section remains (not overridden by second)
        assert_eq!(props.indent_style, Some(IndentStyle::Space));
    }

    // Validates: Requirement 6 AC 6.5 — section pattern matching
    #[test]
    fn resolve_only_applies_matching_sections() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let file_path = root_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*.py]\nindent_size = 4\n\n[*.rs]\nindent_size = 2\n\n[Makefile]\nindent_style = tab\n",
        );

        let props = resolve_editorconfig(&file_path);
        // Only [*.rs] matches main.rs
        assert_eq!(props.indent_size, Some(IndentSize::Value(2)));
        // [*.py] and [Makefile] don't match
        assert_eq!(props.indent_style, None);
    }

    // Validates: Requirement 6 AC 6.4, 6.5 — no .editorconfig files returns defaults
    #[test]
    fn resolve_no_editorconfig_files_returns_empty_properties() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let file_path = root_dir.join("main.rs");
        fs::write(&file_path, "").unwrap();

        let props = resolve_editorconfig(&file_path);
        assert_eq!(props, EditorConfigProperties::default());
    }

    // Validates: Requirement 6 AC 6.5 — multi-level merge with three directory levels
    #[test]
    fn resolve_three_level_merge_closer_wins() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        // Structure: root/project/src/lib.rs
        let project_dir = root_dir.join("project");
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let file_path = src_dir.join("lib.rs");
        fs::write(&file_path, "").unwrap();

        // Root level: end_of_line = crlf, charset = utf-8, indent_style = tab
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*]\nend_of_line = crlf\ncharset = utf-8\nindent_style = tab\n",
        );

        // Project level: indent_style = space, indent_size = 4
        write_file(
            &project_dir,
            ".editorconfig",
            "[*]\nindent_style = space\nindent_size = 4\n",
        );

        // Src level: indent_size = 2
        write_file(&src_dir, ".editorconfig", "[*.rs]\nindent_size = 2\n");

        let props = resolve_editorconfig(&file_path);
        // src level wins for indent_size
        assert_eq!(props.indent_size, Some(IndentSize::Value(2)));
        // project level wins for indent_style (closer than root)
        assert_eq!(props.indent_style, Some(IndentStyle::Space));
        // root level provides end_of_line and charset (not overridden)
        assert_eq!(props.end_of_line, Some(EndOfLine::CrLf));
        assert_eq!(props.charset, Some(Charset::Utf8));
    }

    // Validates: Requirement 6 AC 6.5 — merge is field-by-field, not all-or-nothing
    #[test]
    fn resolve_field_by_field_merge_does_not_clobber_unset_fields() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let sub_dir = root_dir.join("sub");
        fs::create_dir_all(&sub_dir).unwrap();
        let file_path = sub_dir.join("test.rs");
        fs::write(&file_path, "").unwrap();

        // Root: defines indent_style and end_of_line
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*]\nindent_style = tab\nend_of_line = lf\n",
        );

        // Sub: defines only indent_size — should not wipe out root's values
        write_file(&sub_dir, ".editorconfig", "[*]\nindent_size = 3\n");

        let props = resolve_editorconfig(&file_path);
        assert_eq!(props.indent_size, Some(IndentSize::Value(3)));
        assert_eq!(props.indent_style, Some(IndentStyle::Tab));
        assert_eq!(props.end_of_line, Some(EndOfLine::Lf));
    }

    // Validates: Requirement 6 AC 6.5 — empty sections don't affect merge
    #[test]
    fn resolve_empty_sections_have_no_effect() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let file_path = root_dir.join("app.rs");
        fs::write(&file_path, "").unwrap();

        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*.rs]\n\n[*]\nindent_size = 4\n",
        );

        let props = resolve_editorconfig(&file_path);
        // Empty [*.rs] section has no properties to contribute
        // [*] section provides indent_size
        assert_eq!(props.indent_size, Some(IndentSize::Value(4)));
    }

    // Validates: Requirement 6 AC 6.4 — traversal continues when root is not set
    #[test]
    fn resolve_continues_past_non_root_editorconfig() {
        let tmp = TempDir::new().unwrap();
        let root_dir = tmp.path();

        let sub_dir = root_dir.join("child");
        fs::create_dir_all(&sub_dir).unwrap();
        let file_path = sub_dir.join("file.rs");
        fs::write(&file_path, "").unwrap();

        // Root level: has root = true, defines charset
        write_file(
            root_dir,
            ".editorconfig",
            "root = true\n\n[*]\ncharset = latin1\n",
        );

        // Child level: no root declaration, defines indent_style
        write_file(&sub_dir, ".editorconfig", "[*]\nindent_style = space\n");

        let props = resolve_editorconfig(&file_path);
        // Child provides indent_style
        assert_eq!(props.indent_style, Some(IndentStyle::Space));
        // Root provides charset (traversal continued past non-root child file)
        assert_eq!(props.charset, Some(Charset::Latin1));
    }

    // Unit test for merge_properties helper
    #[test]
    fn merge_properties_overrides_set_fields_only() {
        let mut target = EditorConfigProperties {
            indent_style: Some(IndentStyle::Tab),
            indent_size: Some(IndentSize::Value(4)),
            ..Default::default()
        };

        let source = EditorConfigProperties {
            indent_style: Some(IndentStyle::Space),
            tab_width: Some(8),
            ..Default::default()
        };

        merge_properties(&mut target, &source);

        assert_eq!(target.indent_style, Some(IndentStyle::Space)); // overridden
        assert_eq!(target.indent_size, Some(IndentSize::Value(4))); // preserved
        assert_eq!(target.tab_width, Some(8)); // added from source
    }

    // Unit test for merge_properties_under helper
    #[test]
    fn merge_properties_under_fills_gaps_only() {
        let mut target = EditorConfigProperties {
            indent_style: Some(IndentStyle::Space),
            ..Default::default()
        };

        let source = EditorConfigProperties {
            indent_style: Some(IndentStyle::Tab),    // should NOT override
            indent_size: Some(IndentSize::Value(2)), // should fill gap
            ..Default::default()
        };

        merge_properties_under(&mut target, &source);

        assert_eq!(target.indent_style, Some(IndentStyle::Space)); // preserved
        assert_eq!(target.indent_size, Some(IndentSize::Value(2))); // filled
    }

    // Unit test for collect_matching_properties
    #[test]
    fn collect_matching_properties_applies_later_sections_over_earlier() {
        use crate::editorconfig::parser::EditorConfigSection;

        let sections = vec![
            EditorConfigSection {
                pattern: "*".to_string(),
                properties: EditorConfigProperties {
                    indent_size: Some(IndentSize::Value(2)),
                    indent_style: Some(IndentStyle::Tab),
                    ..Default::default()
                },
            },
            EditorConfigSection {
                pattern: "*.rs".to_string(),
                properties: EditorConfigProperties {
                    indent_size: Some(IndentSize::Value(4)),
                    ..Default::default()
                },
            },
        ];

        let props = collect_matching_properties(&sections, "main.rs");

        // [*] sets indent_style = tab, indent_size = 2
        // [*.rs] overrides indent_size = 4
        assert_eq!(props.indent_size, Some(IndentSize::Value(4)));
        assert_eq!(props.indent_style, Some(IndentStyle::Tab));
    }
}
