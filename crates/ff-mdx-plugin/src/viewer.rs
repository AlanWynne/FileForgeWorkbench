use ff_md_viewer::render_to_html;
use ff_viewers::trait_def::FileViewer;

/// A [`FileViewer`] that renders Markdown content to HTML.
pub struct MdxFileViewer;

impl FileViewer for MdxFileViewer {
    fn viewer_key(&self) -> &str {
        "mdx-markdown"
    }

    fn display_name(&self) -> &str {
        "Markdown Viewer"
    }

    fn description(&self) -> &str {
        "Renders Markdown (.md) files to HTML using pulldown-cmark"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["md", "markdown"]
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["text/markdown", "text/x-markdown"]
    }

    fn can_render(&self, uri: &str, _content_sample: &[u8]) -> bool {
        uri.ends_with(".md") || uri.ends_with(".markdown")
    }

    fn render(&self, content: &[u8]) -> String {
        render_to_html(&String::from_utf8_lossy(content))
    }

    fn on_content_changed(&mut self, _new_content: &[u8]) {
        // Stateless renderer -- nothing to invalidate
    }
}

#[cfg(test)]
mod tests {
    use super::MdxFileViewer;
    use ff_viewers::trait_def::FileViewer;

    #[test]
    fn viewer_key_is_stable() {
        // Validates: Requirement 13.1
        assert_eq!(MdxFileViewer.viewer_key(), "mdx-markdown");
        assert!(!MdxFileViewer.display_name().is_empty());
        assert!(!MdxFileViewer.description().is_empty());
    }

    #[test]
    fn declares_markdown_extensions_and_mime_types() {
        // Validates: Requirement 13.2
        assert_eq!(MdxFileViewer.supported_extensions(), &["md", "markdown"]);
        assert_eq!(
            MdxFileViewer.supported_mime_types(),
            &["text/markdown", "text/x-markdown"]
        );
    }

    #[test]
    fn can_render_matches_md_and_markdown_suffixes_only() {
        // Validates: Requirement 13.3
        let v = MdxFileViewer;
        assert!(v.can_render("file:///notes.md", b""));
        assert!(v.can_render("file:///notes.markdown", b""));
        assert!(!v.can_render("file:///notes.txt", b""));
        assert!(!v.can_render("file:///notes", b""));
    }

    #[test]
    fn render_on_invalid_utf8_does_not_panic_and_returns_html() {
        // Validates: Requirement 13.4
        let v = MdxFileViewer;
        // Invalid UTF-8 bytes followed by a heading.
        let content = [0xff, 0xfe, b'#', b' ', b'H', b'i'];
        let html = v.render(&content);
        assert!(html.contains("Hi"), "lossy-decoded content is rendered");
    }
}
