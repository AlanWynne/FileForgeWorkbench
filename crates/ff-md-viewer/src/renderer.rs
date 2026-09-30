use pulldown_cmark::{html, Options, Parser};

/// Convert markdown text to an HTML fragment string.
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
