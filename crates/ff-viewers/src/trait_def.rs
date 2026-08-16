//! FileViewer trait definition.
//!
//! The core trait that all viewer implementations must implement. Defines methods
//! for rendering, supported content types, panel integration, and refresh behaviour.
//! Viewers are always read-only.

/// The core trait that all viewer implementations must implement.
///
/// `FileViewer` defines the contract for content viewers in the FileForgeWorkbench
/// platform. Viewers display file content in specialised visual representations
/// without modifying the underlying document.
///
/// # Object Safety
///
/// This trait is object-safe, allowing the platform to store viewers as trait
/// objects (`Box<dyn FileViewer>`).
///
/// # Read-Only Guarantee
///
/// The `render` method receives content as an immutable byte slice (`&[u8]`).
/// Only `on_content_changed` and `configure` take `&mut self` for internal
/// state updates.
///
/// # Implementor Responsibilities
///
/// Implementors do NOT manage their own panel lifecycle — the platform's
/// `ViewerPanel` wrapper handles docking, visibility, and focus.
pub trait FileViewer: Send + Sync {
    /// Returns the unique ViewerKey identifier string.
    ///
    /// This must match the key used during registration and remain stable
    /// for the lifetime of the viewer.
    fn viewer_key(&self) -> &str;

    /// Returns a human-readable display name (1 to 128 characters).
    ///
    /// Used in the `PREVIEW LIST` output and status bar display.
    fn display_name(&self) -> &str;

    /// Returns a brief description of what this viewer renders.
    ///
    /// Used in the `PREVIEW LIST` output and help text.
    fn description(&self) -> &str;

    /// Returns file extensions this viewer handles (without leading dot).
    ///
    /// Examples: `["lst", "rpt", "spool"]`, `["csv", "tsv"]`
    fn supported_extensions(&self) -> &[&str];

    /// Returns MIME types this viewer handles.
    ///
    /// Examples: `["text/csv"]`, `["image/png", "image/jpeg"]`
    fn supported_mime_types(&self) -> &[&str];

    /// Returns whether this viewer can render the given resource.
    ///
    /// Uses URI metadata (e.g., file extension, path patterns) and/or a
    /// content sample for sniffing. This is the last-resort matching method
    /// in the priority chain.
    fn can_render(&self, uri: &str, content_sample: &[u8]) -> bool;

    /// Renders the content into a text-based representation.
    ///
    /// Content is received as an immutable byte slice — no mutation is possible.
    /// The returned string contains the rendered output for display.
    ///
    /// # Read-Only Contract
    ///
    /// This method takes `&self` — it cannot modify the viewer's internal state
    /// or the provided content. The platform enforces this at the type level.
    fn render(&self, content: &[u8]) -> String;

    /// Called when the underlying document changes, allowing the viewer to
    /// refresh its internal state (e.g., re-parse, update cached render data).
    ///
    /// This is the only method (along with `configure`) that takes `&mut self`.
    fn on_content_changed(&mut self, new_content: &[u8]);

    /// Optional configuration method.
    ///
    /// Called during initialization and when the `[viewers.<key>]` configuration
    /// section changes at runtime. Default implementation is a no-op.
    fn configure(&mut self, _config: &toml::Value) {
        // Default: no-op — viewers that don't need configuration skip this.
    }
}

/// Compile-time assertion that `FileViewer` is object-safe.
///
/// This function is never called at runtime; it exists solely to produce a
/// compile error if the trait loses object safety.
fn _assert_object_safe(_: &dyn FileViewer) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal test viewer for verifying trait compilation and default behaviour.
    struct TestViewer;

    impl FileViewer for TestViewer {
        fn viewer_key(&self) -> &str {
            "test-viewer"
        }

        fn display_name(&self) -> &str {
            "Test Viewer"
        }

        fn description(&self) -> &str {
            "A test viewer for unit tests"
        }

        fn supported_extensions(&self) -> &[&str] {
            &["txt"]
        }

        fn supported_mime_types(&self) -> &[&str] {
            &["text/plain"]
        }

        fn can_render(&self, _uri: &str, _content_sample: &[u8]) -> bool {
            true
        }

        fn render(&self, content: &[u8]) -> String {
            String::from_utf8_lossy(content).to_string()
        }

        fn on_content_changed(&mut self, _new_content: &[u8]) {
            // no-op for test
        }
    }

    #[test]
    fn trait_object_can_be_constructed() {
        // Validates: Requirement 2 AC 2 — trait is object-safe
        let viewer: Box<dyn FileViewer> = Box::new(TestViewer);
        assert_eq!(viewer.viewer_key(), "test-viewer");
        assert_eq!(viewer.display_name(), "Test Viewer");
    }

    #[test]
    fn default_configure_is_no_op() {
        // Validates: Requirement 2 AC 1 — configure has default no-op
        let mut viewer = TestViewer;
        let config = toml::Value::Table(toml::map::Map::new());
        viewer.configure(&config); // Should not panic
    }

    #[test]
    fn render_takes_immutable_self_and_immutable_content() {
        // Validates: Requirement 2 AC 3, AC 4; Requirement 8 AC 1
        // The type system enforces this — render takes &self and &[u8]
        let viewer = TestViewer;
        let content = b"hello world";
        let output = viewer.render(content);
        assert_eq!(output, "hello world");
        // content is still accessible — not consumed or modified
        assert_eq!(content, b"hello world");
    }

    #[test]
    fn on_content_changed_takes_mut_self() {
        // Validates: Requirement 2 AC 3 — only on_content_changed is &mut self
        let mut viewer = TestViewer;
        viewer.on_content_changed(b"new content");
    }

    #[test]
    fn viewer_metadata_methods_return_expected_values() {
        // Validates: Requirement 2 AC 1
        let viewer = TestViewer;
        assert_eq!(viewer.supported_extensions(), &["txt"]);
        assert_eq!(viewer.supported_mime_types(), &["text/plain"]);
        assert!(viewer.can_render("file:///test.txt", b"sample"));
        assert_eq!(viewer.description(), "A test viewer for unit tests");
    }
}
