use std::path::{Path, PathBuf};

use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

#[derive(Default)]
pub struct MarkdownViewer {
    current_path: Option<PathBuf>,
    markdown: String,
    cache: CommonMarkCache,
    title: String,
}

impl MarkdownViewer {
    pub fn load(&mut self, path: &Path, root: &Option<PathBuf>) {
        self.current_path = Some(path.to_path_buf());
        self.markdown = std::fs::read_to_string(path)
            .unwrap_or_else(|e| format!("Error reading file: {e}"));
        self.title = root
            .as_ref()
            .and_then(|r| path.strip_prefix(r).ok())
            .map(|p| p.display().to_string().replace('\\', "/"))
            .unwrap_or_else(|| path.display().to_string());
        self.cache = CommonMarkCache::default();
    }

    pub fn reload(&mut self) {
        if let Some(path) = self.current_path.clone() {
            self.markdown = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| format!("Error reading file: {e}"));
            self.cache = CommonMarkCache::default();
        }
    }

    pub fn clear(&mut self) {
        self.current_path = None;
        self.markdown.clear();
        self.title.clear();
        self.cache = CommonMarkCache::default();
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.current_path.as_ref()
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        if self.current_path.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("📄  Open a folder and select a markdown file")
                        .size(16.0)
                        .weak(),
                );
            });
            return;
        }

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("📄  {}", self.title)).weak());
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown);
            });
    }
}
