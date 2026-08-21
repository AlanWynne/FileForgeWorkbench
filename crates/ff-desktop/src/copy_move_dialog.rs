//! # Copy To… / Move To… Dialog
//!
//! Modal dialog for copying or moving a file/dataset to a new location.
//! Handles naming-rule transformation between catalog types and dispatches
//! the background I/O operation via `ff-bgio` (stubbed in this release).
//!
//! Validates: Requirement 16.12

use eframe::egui;

// === Operation kind =========================================================

/// Whether this dialog is performing a copy or a move.
///
/// Validates: Requirement 16.12 AC 10
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMoveKind {
    Copy,
    Move,
}

impl CopyMoveKind {
    pub fn title(self) -> &'static str {
        match self {
            CopyMoveKind::Copy => "Copy To\u{2026}",
            CopyMoveKind::Move => "Move To\u{2026}",
        }
    }
}

// === Dialog state ===========================================================

/// State for the Copy To… / Move To… modal dialog.
///
/// Validates: Requirement 16.12 AC 1–10
#[derive(Debug, Clone)]
pub struct CopyMoveDialog {
    /// Whether the dialog is currently open.
    pub open: bool,
    /// Copy or Move.
    pub kind: CopyMoveKind,
    /// Source full path or DSN.
    pub source_path: String,
    /// Target directory path (editable by user).
    pub target_dir: String,
    /// Proposed name after naming-rule transformation (editable by user).
    pub proposed_name: String,
    /// Inline validation error message (empty = no error).
    pub error: String,
    /// True once the user confirms — caller dispatches the bgio task.
    pub confirmed: bool,
}

impl Default for CopyMoveDialog {
    fn default() -> Self {
        Self {
            open: false,
            kind: CopyMoveKind::Copy,
            source_path: String::new(),
            target_dir: String::new(),
            proposed_name: String::new(),
            error: String::new(),
            confirmed: false,
        }
    }
}

impl CopyMoveDialog {
    /// Open the dialog for a given source path.
    ///
    /// `proposed` is the pre-transformed name to show in the name field.
    ///
    /// Validates: Requirement 16.12 AC 1
    pub fn open(&mut self, kind: CopyMoveKind, source_path: &str, proposed: &str) {
        self.kind = kind;
        self.source_path = source_path.to_string();
        self.proposed_name = proposed.to_string();
        self.target_dir = String::new();
        self.error = String::new();
        self.confirmed = false;
        self.open = true;
    }

    /// Render the dialog.  Returns `true` if the dialog was just confirmed.
    ///
    /// Validates: Requirement 16.12 AC 1–9
    pub fn render(&mut self, ctx: &egui::Context) -> bool {
        if !self.open {
            return false;
        }

        let mut just_confirmed = false;
        let mut close = false;

        egui::Window::new(self.kind.title())
            .collapsible(false)
            .resizable(false)
            .min_width(400.0)
            .show(ctx, |ui| {
                ui.label(format!("Source: {}", self.source_path));
                ui.separator();

                ui.label("Target directory:");
                ui.text_edit_singleline(&mut self.target_dir);

                ui.label("Name:");
                ui.text_edit_singleline(&mut self.proposed_name);

                if !self.error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error);
                }

                ui.separator();
                ui.horizontal(|ui| {
                    let confirm_enabled = !self.target_dir.is_empty()
                        && !self.proposed_name.is_empty()
                        && self.error.is_empty();
                    if ui
                        .add_enabled(confirm_enabled, egui::Button::new("Confirm"))
                        .clicked()
                    {
                        if let Some(err) = validate_proposed_name(&self.proposed_name) {
                            self.error = err;
                        } else {
                            self.confirmed = true;
                            just_confirmed = true;
                            close = true;
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        if close {
            self.open = false;
        }

        just_confirmed
    }
}

// === Naming-rule transformation =============================================

/// Transform a native filename stem to a valid Mainframe PDS member name:
/// uppercase, strip non-alphanumeric/national chars, truncate to 8.
///
/// Validates: Requirement 16.12 AC 2
pub fn native_to_mainframe_name(native_name: &str) -> String {
    let stem = std::path::Path::new(native_name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| native_name.to_string());
    stem.to_uppercase()
        .chars()
        .filter(|c| matches!(c, 'A'..='Z' | '0'..='9' | '@' | '#' | '$'))
        .take(8)
        .collect()
}

/// Transform a Mainframe member name to a native filename (lowercase, no extension).
///
/// Validates: Requirement 16.12 AC 4
#[allow(dead_code)]
pub fn mainframe_to_native_name(member: &str) -> String {
    member.to_lowercase()
}

/// Validate the proposed name in the dialog.
/// Returns `Some(error_message)` if invalid, `None` if valid.
///
/// Validates: Requirement 16.12 AC 6
fn validate_proposed_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Name cannot be empty.".to_string());
    }
    // Basic OS filename check: no path separators
    if name.contains('/') || name.contains('\\') {
        return Some("Name must not contain path separators.".to_string());
    }
    None
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: Requirement 16.12 AC 2 — Native→Mainframe uppercase + truncate
    #[test]
    fn native_to_mainframe_uppercases_and_truncates() {
        assert_eq!(native_to_mainframe_name("my_long_filename.rs"), "MYLONGFI");
        assert_eq!(native_to_mainframe_name("hello.jcl"), "HELLO");
        assert_eq!(native_to_mainframe_name("abcdefgh.txt"), "ABCDEFGH");
        assert_eq!(native_to_mainframe_name("abcdefghi.txt"), "ABCDEFGH"); // truncated
    }

    /// Validates: Requirement 16.12 AC 4 — Mainframe→Native lowercase
    #[test]
    fn mainframe_to_native_lowercases() {
        assert_eq!(mainframe_to_native_name("MYJOB"), "myjob");
        assert_eq!(mainframe_to_native_name("PAYROLL1"), "payroll1");
    }

    /// Validates: Requirement 16.12 AC 2 — strips invalid chars
    #[test]
    fn native_to_mainframe_strips_invalid_chars() {
        // underscores and hyphens are not valid mainframe chars
        assert_eq!(native_to_mainframe_name("my-file.txt"), "MYFILE");
        assert_eq!(native_to_mainframe_name("test_data.dat"), "TESTDATA");
    }

    /// Validates: Requirement 16.12 AC 1 — dialog opens with correct state
    #[test]
    fn dialog_open_sets_fields() {
        let mut dlg = CopyMoveDialog::default();
        dlg.open(CopyMoveKind::Copy, "/home/user/hello.rs", "HELLO");
        assert!(dlg.open);
        assert_eq!(dlg.kind, CopyMoveKind::Copy);
        assert_eq!(dlg.source_path, "/home/user/hello.rs");
        assert_eq!(dlg.proposed_name, "HELLO");
        assert!(!dlg.confirmed);
    }

    /// Validates: Requirement 16.12 — Move kind title is correct
    #[test]
    fn move_kind_title() {
        assert_eq!(CopyMoveKind::Move.title(), "Move To\u{2026}");
        assert_eq!(CopyMoveKind::Copy.title(), "Copy To\u{2026}");
    }
}
