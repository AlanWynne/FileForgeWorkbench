use std::collections::BTreeMap;
use std::path::PathBuf;

use ff_md_viewer::FileEntry;

/// The file-tree side panel, tracking the currently selected file.
#[derive(Default)]
pub struct FileTree {
    selected: Option<PathBuf>,
}

impl FileTree {
    /// Reconcile the selection against a new file set.
    ///
    /// Clears the current selection unless the selected file is still present
    /// in `files`, so that a rescan of the same folder preserves the user's
    /// selection while loading a different folder resets it.
    pub fn set_files(&mut self, files: &[FileEntry]) {
        let still_present = self
            .selected
            .as_ref()
            .is_some_and(|sel| files.iter().any(|f| &f.full_path == sel));
        if !still_present {
            self.selected = None;
        }
    }

    /// The currently selected file path, if any.
    #[cfg(test)]
    pub fn selected(&self) -> Option<&PathBuf> {
        self.selected.as_ref()
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
            if ui
                .add(egui::Button::selectable(
                    selected,
                    entry.relative_path.clone(),
                ))
                .clicked()
            {
                self.selected = Some(entry.full_path.clone());
                clicked = Some(entry.full_path.clone());
            }
        }

        for (folder, entries) in &folders {
            egui::CollapsingHeader::new(folder.to_string())
                .default_open(true)
                .show(ui, |ui| {
                    for entry in entries {
                        let file_name = entry
                            .relative_path
                            .rsplit('/')
                            .next()
                            .unwrap_or(&entry.relative_path);
                        let selected = self.selected.as_deref() == Some(&entry.full_path);
                        if ui
                            .add(egui::Button::selectable(selected, file_name.to_string()))
                            .clicked()
                        {
                            self.selected = Some(entry.full_path.clone());
                            clicked = Some(entry.full_path.clone());
                        }
                    }
                });
        }

        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::FileTree;
    use crate::helpers::filter_matches;
    use ff_md_viewer::FileEntry;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn entry(rel: &str) -> FileEntry {
        FileEntry {
            relative_path: rel.to_string(),
            full_path: PathBuf::from(format!("/root/{rel}")),
        }
    }

    #[test]
    fn clicking_a_file_entry_selects_it_and_returns_full_path() {
        // Validates: Requirement 16.1
        use egui_kittest::kittest::Queryable;
        use egui_kittest::Harness;
        let files = vec![entry("alpha.md"), entry("beta.md")];
        let clicked: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
        let tree = Rc::new(RefCell::new(FileTree::default()));

        let clicked_c = Rc::clone(&clicked);
        let tree_c = Rc::clone(&tree);
        let files_c = files.clone();
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 400.0))
            .build_ui(move |ui| {
                let refs: Vec<&FileEntry> = files_c.iter().collect();
                if let Some(p) = tree_c.borrow_mut().show(ui, &refs) {
                    *clicked_c.borrow_mut() = Some(p);
                }
            });
        harness.run();

        harness.get_by_label("beta.md").click();
        harness.run();

        assert_eq!(
            *clicked.borrow(),
            Some(PathBuf::from("/root/beta.md")),
            "clicking an entry returns its full_path"
        );
        assert_eq!(
            tree.borrow().selected(),
            Some(&PathBuf::from("/root/beta.md")),
            "clicking an entry marks it selected"
        );
    }

    #[test]
    fn filter_narrows_displayed_entries_case_insensitively() {
        // Validates: Requirement 16.2
        use egui_kittest::kittest::Queryable;
        use egui_kittest::Harness;
        let files = vec![entry("Guide.md"), entry("notes.md")];

        // Filter "guide" (lowercase) must match "Guide.md" and exclude "notes.md".
        let files_c = files.clone();
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 400.0))
            .build_ui(move |ui| {
                let mut tree = FileTree::default();
                let refs: Vec<&FileEntry> = files_c
                    .iter()
                    .filter(|f| filter_matches(&f.relative_path, "guide"))
                    .collect();
                let _ = tree.show(ui, &refs);
            });
        harness.run();

        assert!(
            harness.query_by_label("Guide.md").is_some(),
            "matching entry is shown"
        );
        assert!(
            harness.query_by_label("notes.md").is_none(),
            "non-matching entry is filtered out"
        );
    }

    #[test]
    fn empty_filter_shows_all_entries() {
        // Validates: Requirement 16.2
        use egui_kittest::kittest::Queryable;
        use egui_kittest::Harness;
        let files = vec![entry("Guide.md"), entry("notes.md")];

        let files_c = files.clone();
        let mut harness = Harness::builder()
            .with_size(egui::Vec2::new(400.0, 400.0))
            .build_ui(move |ui| {
                let mut tree = FileTree::default();
                let refs: Vec<&FileEntry> = files_c
                    .iter()
                    .filter(|f| filter_matches(&f.relative_path, ""))
                    .collect();
                let _ = tree.show(ui, &refs);
            });
        harness.run();

        assert!(harness.query_by_label("Guide.md").is_some());
        assert!(harness.query_by_label("notes.md").is_some());
    }
}
