use ff_md_viewer::render_to_html;
use ff_viewers::trait_def::FileViewer;

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
        // Stateless renderer — nothing to invalidate
    }
}
