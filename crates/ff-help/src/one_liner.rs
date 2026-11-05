//! One-line-description extractor (CR-NR-119, context-help Req 21).
//!
//! Derives a command's single-source-of-truth one-liner from the first
//! paragraph of its help topic's `## Description` section. This keeps the short
//! description (used by the alias editor, command palette, and status lines) in
//! ONE place -- the command's Markdown help file -- with no duplicated strings.

/// Maximum char count of a valid one-line description. A longer first paragraph
/// is rejected (returns `None`) so the derived one-liner stays a true one-liner.
const MAX_ONE_LINE_CHARS: usize = 200;

/// Extract the one-line description from a help topic body.
///
/// Scans `body` for a heading whose text equals `Description` (case-insensitive),
/// collects the lines until the next heading or end of input, takes the FIRST
/// blank-line-delimited non-empty paragraph, strips inline Markdown to plain
/// text, and collapses internal whitespace to single spaces.
///
/// Returns `None` when there is no `Description` section, its first paragraph is
/// empty, or the resulting text exceeds `MAX_ONE_LINE_CHARS` characters.
pub fn extract_one_line_description(body: &str) -> Option<String> {
    let mut in_description = false;
    let mut paragraph: Vec<&str> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        let is_heading = trimmed.starts_with('#');

        if !in_description {
            if is_heading && heading_text(trimmed).eq_ignore_ascii_case("Description") {
                in_description = true;
            }
            continue;
        }

        // In the Description section: stop at the next heading.
        if is_heading {
            break;
        }

        if trimmed.is_empty() {
            // A blank line ends the first paragraph (if we have collected any).
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }

        paragraph.push(trimmed);
    }

    if paragraph.is_empty() {
        return None;
    }

    let joined = paragraph.join(" ");
    let stripped = strip_inline_markup(&joined);
    let collapsed = collapse_whitespace(&stripped);

    if collapsed.is_empty() || collapsed.chars().count() > MAX_ONE_LINE_CHARS {
        return None;
    }

    Some(collapsed)
}

/// Return the text of a heading line, stripping leading `#` and spaces.
fn heading_text(line: &str) -> &str {
    line.trim_start_matches('#').trim()
}

/// Collapse runs of whitespace to a single space and trim the ends.
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip the inline Markdown used in descriptions to plain text:
/// `` `code` `` -> code, `**bold**` / `*italic*` -> text,
/// `[label](target)` -> label.
fn strip_inline_markup(text: &str) -> String {
    let no_links = strip_links(text);
    let mut out = String::with_capacity(no_links.len());
    let mut chars = no_links.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '`' => {} // drop code ticks
            '*' => {
                // drop a second '*' of a bold pair as well
                if chars.peek() == Some(&'*') {
                    chars.next();
                }
            }
            '_' => {}
            other => out.push(other),
        }
    }
    out
}

/// Replace `[label](target)` link syntax with just `label`.
fn strip_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let (before, after_open) = rest.split_at(open);
        out.push_str(before);
        let after_open = &after_open[1..]; // skip '['
        match after_open.find(']') {
            Some(close) => {
                let label = &after_open[..close];
                let tail = &after_open[close + 1..];
                // Drop a following "(...)" target if present.
                if let Some(stripped) = strip_leading_paren_group(tail) {
                    out.push_str(label);
                    rest = stripped;
                } else {
                    // Not a link: keep the bracket literally.
                    out.push('[');
                    out.push_str(label);
                    out.push(']');
                    rest = tail;
                }
            }
            None => {
                // Unbalanced '[': emit it literally and stop scanning.
                out.push('[');
                out.push_str(after_open);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

/// If `tail` begins with a `(...)` group, return the remainder after it.
fn strip_leading_paren_group(tail: &str) -> Option<&str> {
    let bytes = tail.as_bytes();
    if bytes.first() != Some(&b'(') {
        return None;
    }
    tail.find(')').map(|close| &tail[close + 1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: context-help Requirement 21.3
    #[test]
    fn happy_path_returns_first_description_paragraph() {
        let body = "## Syntax\n\n```\nEXIT\n```\n\n## Description\n\nTerminates the application immediately.\n\n## See Also\n";
        assert_eq!(
            extract_one_line_description(body).as_deref(),
            Some("Terminates the application immediately.")
        );
    }

    // Validates: context-help Requirement 21.3
    #[test]
    fn inline_markup_is_stripped_to_plain_text() {
        let body =
            "## Description\n\nRuns `code` with **bold** and a [link](cmd:SAVE) reference.\n";
        assert_eq!(
            extract_one_line_description(body).as_deref(),
            Some("Runs code with bold and a link reference.")
        );
    }

    // Validates: context-help Requirement 21.3
    #[test]
    fn multi_paragraph_description_returns_first_paragraph_only() {
        let body = "## Description\n\nFirst paragraph sentence.\n\nSecond paragraph is ignored.\n";
        assert_eq!(
            extract_one_line_description(body).as_deref(),
            Some("First paragraph sentence.")
        );
    }

    // Validates: context-help Requirement 21.4
    #[test]
    fn missing_description_returns_none() {
        let body = "## Syntax\n\n```\nEXIT\n```\n\n## See Also\n\n- something\n";
        assert_eq!(extract_one_line_description(body), None);
    }

    // Validates: context-help Requirement 21.4
    #[test]
    fn empty_description_returns_none() {
        let body = "## Description\n\n## See Also\n";
        assert_eq!(extract_one_line_description(body), None);
    }

    // Validates: context-help Requirement 21.4
    #[test]
    fn first_paragraph_over_200_chars_returns_none() {
        let long = "word ".repeat(60); // 300 chars
        let body = format!("## Description\n\n{long}\n");
        assert_eq!(extract_one_line_description(&body), None);
    }

    // Validates: context-help Requirement 21.3
    #[test]
    fn extracts_cmd_save_sentence_from_session_help_shape() {
        // Modelled on help/commands/session.help.md cmd:SAVE, including the
        // wrapped multi-line first paragraph.
        let body = "## Syntax\n\n```\nSAVE\n```\n\n## Description\n\nWrites the current buffer to its backing file. A backup may be created depending\non configuration.\n\n## See Also\n\n- [CANCEL](cmd:CANCEL), [END](cmd:END)\n";
        assert_eq!(
            extract_one_line_description(body).as_deref(),
            Some(
                "Writes the current buffer to its backing file. A backup may be created depending on configuration."
            )
        );
    }

    // Validates: context-help Requirement 21.3
    #[test]
    fn description_heading_match_is_case_insensitive() {
        let body = "## DESCRIPTION\n\nCase insensitive heading match.\n";
        assert_eq!(
            extract_one_line_description(body).as_deref(),
            Some("Case insensitive heading match.")
        );
    }
}
