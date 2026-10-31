//! # SCRM saved-session picker modal (CR-NR-098, Stream SCRM Phase 2C FEAT-003).
//!
//! Bare `CAPTURE REPLAY` opens this modal overlay. It LISTS the saved sessions
//! (the `*.ffscrm` archives under `scrm_dir()` = the discoverable ffwb-domain
//! captures dir, the SAME directory the Save-As modal defaults to) and lets the
//! user OPEN / SELECT one for replay. On selection the shell dispatches
//! `CAPTURE REPLAY <absolute-path>` through the single front door (command
//! parity, Req 10.12) -- this module introduces NO new load or dispatch path. On
//! Cancel / Escape / dimmed-background click nothing happens and the modal
//! closes.
//!
//! The render fn is a pure `egui::Modal` over a `ReplayPickerState` plus the
//! list of discovered entries (the shell enumerates `scrm_dir()` and passes the
//! entries in). It returns an [`Outcome`] the shell acts on. Modelled on
//! `scrm_save_as_dialog::render` and the SWAP picker in
//! `shell/update_dialogs.rs`.
//!
//! Validates: screen-snapshot-scrm Requirement 10.12, 24.6.

use std::path::PathBuf;

use eframe::egui;

/// One saved session discovered in `scrm_dir()`: the absolute archive path plus
/// the display stem (the file name without the `.ffscrm` extension).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEntry {
    /// The absolute path to the `.ffscrm` archive (dispatched verbatim).
    pub path: PathBuf,
    /// The display label (file stem).
    pub stem: String,
}

impl SessionEntry {
    /// Build an entry from an absolute `.ffscrm` path, deriving the display
    /// stem from its file name.
    pub fn from_path(path: PathBuf) -> Self {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Self { path, stem }
    }
}

/// Transient state for the saved-session picker modal. Held on the shell as
/// `scrm_ui.replay_picker: Option<ReplayPickerState>`; `Some` means the modal
/// is open. It carries no list of its own -- the shell enumerates `scrm_dir()`
/// fresh each frame and passes the entries to [`render`].
#[derive(Debug, Clone, Default)]
pub struct ReplayPickerState {}

impl ReplayPickerState {
    /// Open the picker.
    pub fn new() -> Self {
        Self {}
    }
}

/// The result of rendering the saved-session picker for one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Still open; keep rendering.
    Open,
    /// The user selected a session: dispatch `CAPTURE REPLAY <path>` and close.
    Selected(PathBuf),
    /// The user cancelled (Cancel button, Escape, or dimmed-background click):
    /// do nothing and close.
    Cancel,
}

/// Render the saved-session picker for one frame against `state`, listing
/// `entries` (the `*.ffscrm` files the shell found in `scrm_dir()`). Returns an
/// [`Outcome`] the shell acts on. The caller only calls this when the modal is
/// open (its `scrm_ui.replay_picker` is `Some`).
///
/// Validates: screen-snapshot-scrm Requirement 10.12, 24.6.
pub fn render(
    ctx: &egui::Context,
    _state: &mut ReplayPickerState,
    entries: &[SessionEntry],
) -> Outcome {
    let mut selected: Option<PathBuf> = None;
    let mut cancel_clicked = false;

    let modal = egui::Modal::new(egui::Id::new("scrm_replay_picker_modal")).show(ctx, |ui| {
        ui.set_max_width(520.0);
        ui.heading("Replay a saved screen collection");
        ui.label("Choose a saved session to open for replay.");
        ui.add_space(6.0);

        if entries.is_empty() {
            ui.label("No saved sessions found. Use CAPTURE STOP to save one first.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for entry in entries.iter() {
                        let row = ui
                            .selectable_label(false, egui::RichText::new(&entry.stem).monospace())
                            .on_hover_text(entry.path.to_string_lossy());
                        if row.clicked() {
                            selected = Some(entry.path.clone());
                        }
                    }
                });
        }

        ui.add_space(8.0);
        ui.separator();
        if ui
            .add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 0.0)))
            .clicked()
        {
            cancel_clicked = true;
        }
    });

    // Escape or a click on the dimmed background cancels -- the safe default.
    if modal.should_close() {
        cancel_clicked = true;
    }

    if let Some(path) = selected {
        Outcome::Selected(path)
    } else if cancel_clicked {
        Outcome::Cancel
    } else {
        Outcome::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Validates: Requirement 10.12 -- an entry derives its display stem from the
    // archive file name.
    #[test]
    fn entry_from_path_derives_stem() {
        // Validates: Requirement 10.12
        let entry = SessionEntry::from_path(PathBuf::from("/data/screen-collections/Repro.ffscrm"));
        assert_eq!(entry.stem, "Repro");
        assert!(entry.path.ends_with("Repro.ffscrm"));
    }

    // Validates: Requirement 10.12 -- a path with no file name still yields a
    // non-empty label (falls back to the whole path).
    #[test]
    fn entry_from_path_without_stem_falls_back() {
        // Validates: Requirement 10.12
        let entry = SessionEntry::from_path(PathBuf::from("/"));
        assert!(!entry.stem.is_empty());
    }
}
