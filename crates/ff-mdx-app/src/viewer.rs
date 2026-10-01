use std::path::{Path, PathBuf};

use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::helpers::relative_title;

/// Renders a single selected Markdown file and tracks the open path.
#[derive(Default)]
pub struct MarkdownViewer {
    current_path: Option<PathBuf>,
    markdown: String,
    cache: CommonMarkCache,
    title: String,
}

impl MarkdownViewer {
    /// Load `path` for display, computing its title relative to `root`.
    pub fn load(&mut self, path: &Path, root: &Option<PathBuf>) {
        self.current_path = Some(path.to_path_buf());
        self.markdown =
            std::fs::read_to_string(path).unwrap_or_else(|e| format!("Error reading file: {e}"));
        self.title = relative_title(path, root);
        self.cache = CommonMarkCache::default();
    }

    /// Re-read the currently open file from disk.
    pub fn reload(&mut self) {
        if let Some(path) = self.current_path.clone() {
            self.markdown = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| format!("Error reading file: {e}"));
            self.cache = CommonMarkCache::default();
        }
    }

    /// Clear the open file, returning to the empty state.
    pub fn clear(&mut self) {
        self.current_path = None;
        self.markdown.clear();
        self.title.clear();
        self.cache = CommonMarkCache::default();
    }

    /// The path of the currently open file, if any.
    pub fn current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    /// Render the viewer area into `ui`.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.current_path.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Open a folder and select a markdown file")
                        .size(16.0)
                        .weak(),
                );
            });
            return;
        }

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(self.title.clone()).weak());
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown);
            });
    }
}

#[cfg(test)]
mod tests {
    use super::MarkdownViewer;
    use std::cell::RefCell;
    use std::fs;
    use std::rc::Rc;
    use tempfile::TempDir;

    #[test]
    fn empty_state_placeholder_shown_when_no_file_selected() {
        // Validates: Requirement 16.3
        use egui_kittest::kittest::Queryable;
        use egui_kittest::Harness;
        let viewer = Rc::new(RefCell::new(MarkdownViewer::default()));
        let viewer_c = Rc::clone(&viewer);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(600.0, 400.0))
            .build_ui(move |ui| {
                viewer_c.borrow_mut().show(ui);
            });
        harness.run();

        assert!(
            harness
                .query_by_label("Open a folder and select a markdown file")
                .is_some(),
            "empty-state placeholder is shown"
        );
    }

    #[test]
    fn loaded_file_shows_relative_path_title() {
        // Validates: Requirement 16.3
        use egui_kittest::kittest::Queryable;
        use egui_kittest::Harness;
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().to_path_buf();
        let file = root.join("readme.md");
        fs::write(&file, "# Hello").expect("write file");

        let viewer = Rc::new(RefCell::new(MarkdownViewer::default()));
        viewer.borrow_mut().load(&file, &Some(root));

        let viewer_c = Rc::clone(&viewer);
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(600.0, 400.0))
            .build_ui(move |ui| {
                viewer_c.borrow_mut().show(ui);
            });
        harness.run();

        assert!(
            harness.query_by_label("readme.md").is_some(),
            "loaded state shows the relative-path title"
        );
        assert!(
            harness
                .query_by_label("Open a folder and select a markdown file")
                .is_none(),
            "placeholder is not shown once a file is loaded"
        );
    }
}
