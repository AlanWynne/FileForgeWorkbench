use pulldown_cmark::{html, Options, Parser};

/// Convert markdown text to an HTML fragment string.
///
/// The parser enables the CommonMark extensions tables, footnotes,
/// strikethrough, task lists, and smart punctuation. An empty input yields an
/// empty fragment.
pub fn render_to_html(markdown: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION;
    let parser = Parser::new_ext(markdown, opts);
    let mut out = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut out, parser);
    out
}

#[cfg(test)]
mod tests {
    use super::render_to_html;

    #[test]
    fn table_markdown_produces_a_table_element() {
        // Validates: Requirement 11.2, 11.3 -- ENABLE_TABLES
        let md = "| a | b |\n|---|---|\n| 1 | 2 |\n";
        let html = render_to_html(md);
        assert!(html.contains("<table>"), "expected a <table>, got: {html}");
    }

    #[test]
    fn footnote_reference_produces_a_footnote_anchor() {
        // Validates: Requirement 11.2, 11.3 -- ENABLE_FOOTNOTES
        let md = "text[^1]\n\n[^1]: the note\n";
        let html = render_to_html(md);
        assert!(
            html.contains("footnote"),
            "expected a footnote anchor, got: {html}"
        );
    }

    #[test]
    fn strikethrough_produces_a_del_element() {
        // Validates: Requirement 11.2, 11.3 -- ENABLE_STRIKETHROUGH
        let html = render_to_html("~~gone~~");
        assert!(html.contains("<del>"), "expected a <del>, got: {html}");
    }

    #[test]
    fn task_list_item_produces_a_checkbox_input() {
        // Validates: Requirement 11.2, 11.3 -- ENABLE_TASKLISTS
        let html = render_to_html("- [x] done\n");
        assert!(
            html.contains("type=\"checkbox\""),
            "expected a checkbox list item, got: {html}"
        );
    }

    #[test]
    fn straight_quotes_become_smart_punctuation() {
        // Validates: Requirement 11.2, 11.3 -- ENABLE_SMART_PUNCTUATION
        let html = render_to_html("\"quoted\"");
        // Smart punctuation replaces straight double quotes with curly quotes
        // (non-ASCII in the OUTPUT html, which is allowed -- this is runtime
        // data, not a source-file literal).
        assert!(
            !html.contains("\"quoted\""),
            "smart punctuation should transform straight quotes, got: {html}"
        );
    }

    #[test]
    fn empty_input_yields_empty_fragment() {
        // Validates: Requirement 11.4
        assert!(render_to_html("").trim().is_empty());
    }
}
