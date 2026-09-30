use std::path::PathBuf;

use ff_html_export::export_html_file;
use ff_md_viewer::{render_to_html, FileEntry, FileWatcher, Scanner};

use crate::file_tree::FileTree;
use crate::viewer::MarkdownViewer;

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
    pub fn new(_cc: &eframe::CreationContext, initial_folder: PathBuf) -> Self {
        let mut app = Self {
            folder: None,
            files: Vec::new(),
            filter: String::new(),
            tree: FileTree::default(),
            viewer: MarkdownViewer::default(),
            watcher: None,
            status: "Ready — open a folder to get started".into(),
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
        if let Ok(changed) = w.rx.try_recv() {
            if self.viewer.current_path() == Some(&changed) {
                self.viewer.reload();
                self.status = format!("Reloaded: {}", changed.display());
            }
        }
    }

    fn handle_drops(&mut self, ctx: &egui::Context) {
        let dropped: Vec<egui::DroppedFile> = ctx.input(|i| i.raw.dropped_files.clone());
        for file in dropped {
            if let Some(path) = file.path {
                if path.is_dir() {
                    self.load_folder(path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    if let Some(parent) = path.parent() {
                        self.load_folder(parent.to_path_buf());
                    }
                }
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            if ui.button("📁 Open Folder").clicked() {
                if let Some(path) = pick_folder(self.folder.clone()) {
                    self.load_folder(path);
                }
            }
            if ui.button("🔄 Refresh").clicked() {
                if let Some(f) = self.folder.clone() {
                    let current = self.viewer.current_path().cloned();
                    self.load_folder(f);
                    if let Some(p) = current {
                        if p.exists() {
                            self.open_file(p);
                        }
                    }
                }
            }
            ui.separator();
            ui.menu_button("💾 Export", |ui| {
                if ui.button("Export as HTML…").clicked() {
                    ui.close_menu();
                    if let Some(src) = self.viewer.current_path().cloned() {
                        if let Some(dst) = save_file("html") {
                            let title = src
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Document");
                            let md = std::fs::read_to_string(&src).unwrap_or_default();
                            let body = render_to_html(&md);
                            match export_html_file(&body, title, &dst) {
                                Ok(_) => {
                                    self.status = format!("HTML saved: {}", dst.display())
                                }
                                Err(e) => self.status = format!("Export error: {e}"),
                            }
                        }
                    }
                }
            });
            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("ffmdx v0.1").weak());
            });
        });

        ctx.input_mut(|i| {
            use egui::Key;
            if i.consume_key(egui::Modifiers::CTRL, Key::O) {
                if let Some(path) = pick_folder(self.folder.clone()) {
                    self.load_folder(path);
                }
            }
            if i.consume_key(egui::Modifiers::NONE, Key::F5) {
                if let Some(f) = self.folder.clone() {
                    self.load_folder(f);
                }
            }
        });
    }
}

impl eframe::App for MdxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_watcher();
        self.handle_drops(ctx);

        if self.watcher.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.toolbar(ui, ctx);
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
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.filter);
                });
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtered: Vec<&FileEntry> = self
                        .files
                        .iter()
                        .filter(|f| {
                            self.filter.is_empty()
                                || f.relative_path
                                    .to_lowercase()
                                    .contains(&self.filter.to_lowercase())
                        })
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
