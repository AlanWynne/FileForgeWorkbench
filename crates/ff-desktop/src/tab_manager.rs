//! `TabManager` — manages the ordered list of open tabs.
//!
//! Owns all `TabState` instances, tracks which tab is active, and provides
//! `open_file` to load a file from the local filesystem into a new tab.

use ff_connector_local_fs::LocalFsProvider;
use ff_document_model::{new_document, BytePosition};
use ff_vfs::VfsProvider;
use tokio::runtime::Runtime;

use crate::tab_state::{TabId, TabKind, TabState};

/// Manages all open tabs and the active tab index.
pub struct TabManager {
    tabs: Vec<TabState>,
    active: usize,
    next_id: u64,
}

impl TabManager {
    /// Create a manager with a single untitled welcome tab.
    pub fn new(runtime: &Runtime, welcome: &str) -> Self {
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), welcome.as_bytes());
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::untitled(TabId(0), document, line_count);
        Self {
            tabs: vec![tab],
            active: 0,
            next_id: 1,
        }
    }

    /// Number of open tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Active tab index.
    pub fn active_index(&self) -> usize {
        self.active
    }

    /// Set the active tab by index. Clamps to valid range.
    pub fn set_active(&mut self, index: usize) {
        self.active = index.min(self.tabs.len().saturating_sub(1));
    }

    /// Immutable slice of all tabs (for rendering the tab bar).
    pub fn tabs(&self) -> &[TabState] {
        &self.tabs
    }

    /// Mutable slice of all tabs.
    pub fn tabs_mut(&mut self) -> &mut Vec<TabState> {
        &mut self.tabs
    }

    /// Mutable reference to the active tab.
    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active]
    }

    /// Immutable reference to the active tab.
    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active]
    }

    /// Close the initial welcome/placeholder tab if it is the only tab and has no path.
    ///
    /// Called before inserting the POM tab on first launch so the POM is the
    /// sole tab at index 0 rather than sitting behind a blank welcome tab.
    /// Validates: Requirement 14.1 — POM is always in first position on launch.
    pub fn close_welcome_tab(&mut self) {
        if self.tabs.len() == 1 && self.tabs[0].path.is_none() {
            // Replace the single placeholder tab with an empty vec; insert_pom_tab
            // will add the real first tab immediately after.
            // We cannot call close_tab (it guards len >= 1), so swap directly.
            self.tabs.clear();
            self.active = 0;
        }
    }

    /// Insert a Primary Option Menu tab at index 0 and make it active.
    ///
    /// Inserts a new POM tab and makes it active.
    /// Validates: Requirement 14.1, 14.13
    pub fn insert_pom_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::pom(id, document);
        self.tabs.insert(0, tab);
        self.active = 0;
        let _ = runtime;
    }

    /// Insert a new untitled editor tab and make it active.
    ///
    /// Validates: Requirement 14.9
    pub fn new_untitled_tab(&mut self, runtime: &Runtime) {
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::untitled(id, document, 1);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        let _ = runtime;
    }

    /// Open the Files Panel (Virtual Catalog Manager) tab.
    ///
    /// If a FilesPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 1.1, 11.2
    pub fn open_files_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self.tabs.iter().position(|t| t.kind == TabKind::FilesPanel) {
            self.active = idx;
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::files_panel(id, document);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        let _ = runtime;
    }

    /// Open the Settings Panel tab.
    ///
    /// If a SettingsPanel tab already exists, activates it instead of inserting a duplicate.
    /// Validates: Requirement 15.1, 15.9
    pub fn open_settings_panel_tab(&mut self, runtime: &Runtime) {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::SettingsPanel)
        {
            self.active = idx;
            return;
        }
        let document = ff_document_model::new_document();
        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::settings_panel(id, document);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        let _ = runtime;
    }

    /// Transform the active tab in-place from `PrimaryOptionMenu` to a new kind.
    ///
    /// No-op if the active tab is not a `PrimaryOptionMenu` tab.
    /// Validates: Requirement 14.6
    pub fn transform_active_pom_tab(&mut self, kind: TabKind, title: &str) {
        let tab = &mut self.tabs[self.active];
        if tab.kind == TabKind::PrimaryOptionMenu {
            tab.kind = kind;
            tab.title = title.to_string();
        }
    }

    ///
    /// If the file is already open (same path), activates the existing tab
    /// instead of opening a duplicate.
    ///
    /// Returns `Err(message)` if the file cannot be read.
    pub fn open_file(&mut self, path: &str, runtime: &Runtime) -> Result<(), String> {
        // Duplicate detection — activate existing tab if already open.
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.path.as_deref() == Some(path))
        {
            self.active = idx;
            return Ok(());
        }

        let bytes = runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .read(path)
                .await
                .map_err(|e| format!("Cannot read '{path}': {e}"))
        })?;

        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), &bytes);
        });
        let (line_count, line_end_mode) = runtime.block_on(async {
            let doc = document.read().await;
            (doc.line_count(), doc.line_end_mode())
        });

        let id = TabId(self.next_id);
        self.next_id += 1;
        let tab = TabState::for_file(id, path.to_string(), document, line_count, line_end_mode);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        Ok(())
    }

    /// Save the active tab's document to its associated file path.
    ///
    /// Returns `Err` if the tab has no path (untitled) or the write fails.
    /// On success, clears `is_modified` and marks the document save point.
    pub fn save_active_tab(&mut self, runtime: &Runtime) -> Result<(), String> {
        let tab = &mut self.tabs[self.active];
        let path = tab
            .path
            .as_deref()
            .ok_or_else(|| "Cannot save: no file path (untitled document)".to_string())?;
        let path = path.to_string();

        let bytes = runtime.block_on(async {
            let mut doc = tab.document.write().await;
            let view = doc.contiguous_view().to_vec();
            view
        });

        runtime.block_on(async {
            let provider =
                LocalFsProvider::with_defaults().map_err(|e| format!("VFS init failed: {e}"))?;
            provider
                .write(&path, &bytes)
                .await
                .map_err(|e| format!("Save failed: {e}"))
        })?;

        // Clear dirty flag and mark save point
        tab.is_modified = false;
        runtime.block_on(async {
            tab.document.write().await.set_save_point();
        });
        Ok(())
    }

    /// Close the tab at `index`. If it is the active tab, activates the
    /// nearest remaining tab. Always keeps at least one tab open.
    #[allow(dead_code)]
    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(index);
        if self.active >= self.tabs.len() {
            self.active = self.tabs.len() - 1;
        } else if index < self.active {
            self.active -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// Validates: Requirement 14.13 — POM tab title is [POM].
    #[test]
    fn pom_tab_title_is_pom() {
        // Validates: Requirement 14.13
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.tabs()[0].title, "[POM]");
    }

    /// Validates: Requirement 14.1 — POM tab has kind PrimaryOptionMenu.
    #[test]
    fn pom_tab_has_kind_primary_option_menu() {
        // Validates: Requirement 14.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.tabs()[0].kind, TabKind::PrimaryOptionMenu);
    }

    /// Validates: Requirement 14.1 — POM tab is inserted at index 0.
    #[test]
    fn pom_tab_inserted_at_index_zero() {
        // Validates: Requirement 14.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.tabs()[0].kind, TabKind::PrimaryOptionMenu);
    }

    /// Validates: Requirement 14.1 — inserting POM twice opens two POM tabs.
    #[test]
    fn insert_pom_tab_twice_opens_two_pom_tabs() {
        // Validates: Requirement 14.1 — START always opens a new POM tab
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        let count_after_first = mgr.len();
        mgr.insert_pom_tab(&runtime);
        assert_eq!(
            mgr.len(),
            count_after_first + 1,
            "second insert_pom_tab must open a new POM tab"
        );
        assert_eq!(mgr.active_tab().kind, TabKind::PrimaryOptionMenu);
    }

    /// Validates: Requirement 14.9 — new_untitled_tab adds an Untitled tab.
    #[test]
    fn new_untitled_tab_adds_untitled_kind() {
        // Validates: Requirement 14.9
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let before = mgr.len();
        mgr.new_untitled_tab(&runtime);
        assert_eq!(mgr.len(), before + 1);
        assert_eq!(mgr.active_tab().kind, TabKind::Untitled);
    }

    /// Validates: Requirement 14.1 — file tab has kind FileEditor.
    #[test]
    fn file_tab_has_kind_file_editor() {
        // Validates: Requirement 14.1
        use std::io::Write;
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let mut tmp = NamedTempFile::new().expect("tempfile");
        writeln!(tmp, "hello").expect("write");
        let path = tmp.path().to_string_lossy().into_owned();
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_file(&path, &runtime).expect("open");
        assert_eq!(mgr.active_tab().kind, TabKind::FileEditor);
    }

    #[test]
    fn save_writes_document_content_to_file() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        // Replace the welcome tab with a file-backed tab at the temp path
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"saved content");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        mgr.tabs[0] = tab;

        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_ok(), "save should succeed: {result:?}");

        let written = std::fs::read(&path).expect("read back");
        assert_eq!(written, b"saved content");
    }

    /// Validates: file-operations Requirement 1.2 — save clears the modified flag.
    #[test]
    fn save_clears_modified_flag() {
        use tempfile::NamedTempFile;
        let runtime = Runtime::new().expect("runtime");
        let tmp = NamedTempFile::new().expect("tempfile");
        let path = tmp.path().to_string_lossy().into_owned();

        let mut mgr = TabManager::new(&runtime, "");
        let document = new_document();
        runtime.block_on(async {
            let mut doc = document.write().await;
            let _ = doc.insert(BytePosition(0), b"hello");
        });
        let line_count = runtime.block_on(async { document.read().await.line_count() });
        let mut tab = TabState::for_file(
            TabId(1),
            path.clone(),
            document,
            line_count,
            ff_document_model::LineEndMode::Default,
        );
        tab.is_modified = true;
        mgr.tabs[0] = tab;

        mgr.save_active_tab(&runtime).expect("save");
        assert!(!mgr.active_tab().is_modified);
    }

    /// Validates: file-operations Requirement 1.4 — save on untitled tab returns error.
    #[test]
    fn save_on_untitled_tab_is_noop() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "some content");
        // The default welcome tab is untitled (no path)
        let result = mgr.save_active_tab(&runtime);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("untitled"));
    }

    /// Validates: task 18.4 — TabManager starts with one untitled tab.
    #[test]
    fn new_manager_has_one_tab() {
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.active_index(), 0);
        assert_eq!(mgr.active_tab().title, "Untitled");
    }

    /// Validates: task 18.4 — opening a missing file returns an error.
    #[test]
    fn open_nonexistent_file_returns_error() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        let result = mgr.open_file("/nonexistent/path/file.txt", &runtime);
        assert!(result.is_err());
        assert_eq!(mgr.len(), 1); // no new tab added
    }

    /// Validates: task 18.5 — closing a tab reduces count; last tab is preserved.
    #[test]
    fn close_tab_preserves_minimum_one() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.close_tab(0);
        assert_eq!(mgr.len(), 1); // cannot go below 1
    }

    /// Validates: task 18.5 — set_active clamps to valid range.
    #[test]
    fn set_active_clamps_to_valid_range() {
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.set_active(999);
        assert_eq!(mgr.active_index(), 0);
    }

    /// Validates: Requirement 18.7 — untitled tab reports correct line_count.
    #[test]
    fn untitled_tab_line_count_matches_content() {
        // Validates: Requirement 7.4 — total line count segment shows real count
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "line1\nline2\nline3\n");
        // 3 newlines → 4 lines (last empty line after final \n)
        assert_eq!(mgr.active_tab().line_count, 4);
    }

    /// Validates: Requirement 18.7 — untitled tab encoding_label defaults to UTF-8.
    #[test]
    fn untitled_tab_encoding_label_is_utf8() {
        // Validates: Requirement 7.3 — encoding segment shows detected encoding
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "hello\n");
        assert_eq!(mgr.active_tab().encoding_label(), "UTF-8");
    }

    // ── Phase AM: Detachable tab windows ────────────────────────────────────

    /// Validates: Requirement 18.4 — is_floating defaults to false on new tabs.
    #[test]
    fn floating_tab_is_floating_flag_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mgr = TabManager::new(&runtime, "");
        assert!(!mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — is_floating can be set to true.
    #[test]
    fn floating_tab_is_floating_flag_can_be_set() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.tabs_mut()[0].is_floating = true;
        assert!(mgr.active_tab().is_floating);
    }

    /// Validates: Requirement 18.4 — POM tab also defaults is_floating to false.
    #[test]
    fn pom_tab_is_floating_defaults_to_false() {
        // Validates: Requirement 18.4
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.insert_pom_tab(&runtime);
        assert!(!mgr.tabs()[0].is_floating);
    }

    /// Validates: Requirement 1.1, 11.2 — option 1 opens a FilesPanel tab with title [FILES].
    #[test]
    fn files_panel_tab_has_kind_files_panel_and_title() {
        // Validates: Requirement 1.1, 11.2
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let tab = mgr.active_tab();
        assert_eq!(tab.kind, TabKind::FilesPanel);
        assert_eq!(tab.title, "[FILES]");
    }

    /// Validates: Requirement 1.1 — opening FilesPanel twice does not duplicate it.
    #[test]
    fn open_files_panel_tab_twice_does_not_duplicate() {
        // Validates: Requirement 1.1
        let runtime = Runtime::new().expect("runtime");
        let mut mgr = TabManager::new(&runtime, "");
        mgr.open_files_panel_tab(&runtime);
        let count = mgr.len();
        mgr.open_files_panel_tab(&runtime);
        assert_eq!(mgr.len(), count, "second open must not add a duplicate");
    }
}
