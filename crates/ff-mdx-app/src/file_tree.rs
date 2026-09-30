use std::collections::BTreeMap;
use std::path::PathBuf;

use ff_md_viewer::FileEntry;

#[derive(Default)]
pub struct FileTree {
    selected: Option<PathBuf>,
}

impl FileTree {
    pub fn set_files(&mut self, _files: &[FileEntry]) {
        self.selected = None;
    }

    /// Render the tree. Returns the path of a clicked file.
    pub fn show(&mut self, ui: &mut egui::Ui, files: &[&FileEntry]) -> Option<PathBuf> {
        let mut clicked = None;

        // Partition into root-level files and folder buckets
        let mut root_files: Vec<&FileEntry> = Vec::new();
        let mut folders: BTreeMap<String, Vec<&FileEntry>> = BTreeMap::new();

        for entry in files {
            let mut parts = entry.relative_path.splitn(2, '/');
            let first = parts.next().unwrap_or("");
            if parts.next().is_none() {
                root_files.push(entry);
            } else {
                folders.entry(first.to_string()).or_default().push(entry);
            }
        }

        for entry in root_files {
            let selected = self.selected.as_deref() == Some(&entry.full_path);
            if ui.add(egui::SelectableLabel::new(selected, format!("📄 {}", entry.relative_path))).clicked() {
                self.selected = Some(entry.full_path.clone());
                clicked = Some(entry.full_path.clone());
            }
        }

        for (folder, entries) in &folders {
            egui::CollapsingHeader::new(format!("📁 {folder}"))
                .default_open(true)
                .show(ui, |ui| {
                    for entry in entries {
                        let file_name = entry.relative_path.rsplit('/').next()
                            .unwrap_or(&entry.relative_path);
                        let selected = self.selected.as_deref() == Some(&entry.full_path);
                        if ui.add(egui::SelectableLabel::new(selected, format!("📄 {file_name}"))).clicked() {
                            self.selected = Some(entry.full_path.clone());
                            clicked = Some(entry.full_path.clone());
                        }
                    }
                });
        }

        clicked
    }
}
