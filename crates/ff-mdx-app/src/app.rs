use std::path::PathBuf;

use ff_html_export::export_html_file;
use ff_md_viewer::{render_to_html, FileEntry, FileWatcher, Scanner};

use crate::file_tree::FileTree;
use crate::helpers::{classify_drop, filter_matches, should_reload, DropAction};
use crate::viewer::MarkdownViewer;

/// The standalone Markdown Explorer application state.
pub struct MdxApp {
    folder: Option<PathBuf>,
    files: Vec<FileEntry>,
    filter: String,
    tree: FileTree,
    viewer: MarkdownViewer,
    watcher: Option<FileWatcher>,
    status: String,
}

impl MdxApp {
    /// Create the app, loading `initial_folder` if it is a directory.
    pub fn new(_cc: &eframe::CreationContext, initial_folder: PathBuf) -> Self {
        let mut app = Self {
            folder: None,
            files: Vec::new(),
            filter: String::new(),
            tree: FileTree::default(),
            viewer: MarkdownViewer::default(),
            watcher: None,
            status: "Ready -- open a folder to get started".into(),
        };
        if initial_folder.is_dir() {
            app.load_folder(initial_folder);
        }
        app
    }

    fn load_folder(&mut self, path: PathBuf) {
        self.files = Scanner::scan(&path);
        self.status = format!(
            "{} markdown file(s) in {}",
            self.files.len(),
            path.display()
        );
        self.watcher = FileWatcher::new(&path).ok();
        self.folder = Some(path);
        self.tree.set_files(&self.files);
        self.viewer.clear();
    }

    fn open_file(&mut self, path: PathBuf) {
        self.viewer.load(&path, &self.folder);
        let display = self
            .folder
            .as_ref()
            .and_then(|f| path.strip_prefix(f).ok())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| path.display().to_string());
        self.status = format!("Viewing: {display}");
    }

    fn poll_watcher(&mut self) {
        let Some(w) = &self.watcher else { return };
        if let Ok(changed) = w.changes().try_recv() {
            if should_reload(self.viewer.current_path().map(|p| p.as_path()), &changed) {
                self.viewer.reload();
                self.status = format!("Reloaded: {}", changed.display());
            }
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        let dropped: Vec<egui::DroppedFile> = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            let Some(path) = file.path else { continue };
            match classify_drop(&path) {
                DropAction::OpenFolder(dir) => self.load_folder(dir),
                DropAction::OpenParentOf(file) => {
                    if let Some(parent) = file.parent() {
                        self.load_folder(parent.to_path_buf());
                    }
                }
                DropAction::Ignore => {}
            }
        }
    }

    /// Open the folder picker and load the chosen folder, if any.
    fn open_folder_dialog(&mut self) {
        if let Some(path) = pick_folder(self.folder.clone()) {
            self.load_folder(path);
        }
    }

    /// Rescan the current folder, re-opening the previously viewed file.
    fn refresh_current_folder(&mut self) {
        let Some(f) = self.folder.clone() else { return };
        let current = self.viewer.current_path().cloned();
        self.load_folder(f);
        if let Some(p) = current {
            if p.exists() {
                self.open_file(p);
            }
        }
    }

    /// Export the currently open document to a user-chosen HTML file.
    fn export_current_as_html(&mut self) {
        let Some(src) = self.viewer.current_path().cloned() else {
            return;
        };
        let Some(dst) = save_file("html") else { return };
        let title = src
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Document");
        let md = std::fs::read_to_string(&src).unwrap_or_default();
        let body = render_to_html(&md);
        match export_html_file(&body, title, &dst) {
            Ok(_) => self.status = format!("HTML saved: {}", dst.display()),
            Err(e) => self.status = format!("Export error: {e}"),
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open Folder").clicked() {
                self.open_folder_dialog();
            }
            if ui.button("Refresh").clicked() {
                self.refresh_current_folder();
            }
            ui.separator();
            ui.menu_button("Export", |ui| {
                if ui.button("Export as HTML...").clicked() {
                    ui.close();
                    self.export_current_as_html();
                }
            });
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(concat!("ffmdx v", env!("CARGO_PKG_VERSION"))).weak());
            });
        });
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (open, refresh) = ctx.input_mut(|i| {
            use egui::Key;
            (
                i.consume_key(egui::Modifiers::CTRL, Key::O),
                i.consume_key(egui::Modifiers::NONE, Key::F5),
            )
        });
        if open {
            self.open_folder_dialog();
        }
        if refresh {
            if let Some(f) = self.folder.clone() {
                self.load_folder(f);
            }
        }
    }
}

impl eframe::App for MdxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_watcher();
        self.handle_drops(ctx);

        if self.watcher.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.toolbar(ui);
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.status).weak().small());
        });

        egui::SidePanel::left("file_tree")
            .default_width(280.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter);
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtered: Vec<&FileEntry> = self
                        .files
                        .iter()
                        .filter(|f| filter_matches(&f.relative_path, &self.filter))
                        .collect();
                    if let Some(path) = self.tree.show(ui, &filtered) {
                        self.open_file(path);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.viewer.show(ui);
        });
    }
}

fn pick_folder(start: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(s) = start {
        dialog = dialog.set_directory(s);
    }
    dialog.pick_folder()
}

fn save_file(ext: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter(ext.to_uppercase(), &[ext])
        .save_file()
}

#[cfg(test)]
impl MdxApp {
    /// Number of Markdown files currently loaded (test accessor).
    fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[cfg(test)]
mod tests {
    use super::MdxApp;
    use egui_kittest::Harness;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn pressing_f5_rescans_the_current_folder() {
        // Validates: Requirement 16.4 (F5 rescan; Ctrl+O dialog is MANUAL per 16.6)
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().to_path_buf();
        fs::write(root.join("one.md"), "# one").expect("write one");

        let root_c = root.clone();
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(1000.0, 700.0))
            .build_eframe(move |cc| MdxApp::new(cc, root_c.clone()));
        harness.run();

        assert_eq!(
            harness.state().file_count(),
            1,
            "initial scan finds the one markdown file"
        );

        // Add a second markdown file on disk, then press F5 to rescan.
        fs::write(root.join("two.md"), "# two").expect("write two");
        harness.key_press(egui::Key::F5);
        harness.run();
        harness.run();

        assert_eq!(
            harness.state().file_count(),
            2,
            "F5 rescans and picks up the newly added markdown file"
        );
    }
}
