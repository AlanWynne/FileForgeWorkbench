//! # SCRM capture-STOP Save-As modal (CR-NR-098, Stream SCRM Phase 2C FEAT-002).
//!
//! On `CAPTURE STOP` with a non-empty active Collection the shell opens this
//! modal overlay. It NAMES the session (pre-filled from the Collection name),
//! DEFAULTS the save location to `scrm_dir()` = `<User_Data_Dir>/screen-collections/`
//! (the discoverable ffwb-domain captures dir, editable so the user can navigate
//! away), and lets the user choose the output TYPE. On SAVE the shell dispatches
//! the EXISTING `CAPTURE SAVE` / `CAPTURE EXPORT` verbs through the single front
//! door (command parity, Req 24.4) -- this module introduces NO new persistence
//! path. On CANCEL nothing is written (Req 24.5).
//!
//! The render fn is a pure `egui::Modal` over a `SaveAsState`; it returns an
//! [`Outcome`] the shell acts on. Modelled on `about_dialog::render` and the
//! RESET BARE modal in `shell/update_dialogs.rs`.
//!
//! Validates: screen-snapshot-scrm Requirement 24.1, 24.2, 24.4, 24.5, 24.6.

use std::path::PathBuf;

use eframe::egui;

/// The output type the user chooses in the Save-As modal. Each maps to an
/// EXISTING front-door verb: `Ffscrm` -> `CAPTURE SAVE`; every other variant ->
/// `CAPTURE EXPORT <FMT>`. ODT/ODP are deferred (Req 24.2) and not offered.
///
/// Validates: screen-snapshot-scrm Requirement 24.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    /// Native zip-compatible Collection archive (`.ffscrm`) -- `CAPTURE SAVE`.
    Ffscrm,
    /// Plain text export -- `CAPTURE EXPORT TEXT`.
    Text,
    /// Markdown export -- `CAPTURE EXPORT MD`.
    Markdown,
    /// HTML export -- `CAPTURE EXPORT HTML`.
    Html,
    /// PDF export -- `CAPTURE EXPORT PDF`.
    Pdf,
}

impl OutputType {
    /// The ordered set of types offered in the selector (Req 24.2: the
    /// IMPLEMENTED subset; ODT/ODP deferred and omitted).
    pub const ALL: [OutputType; 5] = [
        OutputType::Ffscrm,
        OutputType::Text,
        OutputType::Markdown,
        OutputType::Html,
        OutputType::Pdf,
    ];

    /// Human label for the selector.
    pub fn label(self) -> &'static str {
        match self {
            OutputType::Ffscrm => "Native archive (.ffscrm)",
            OutputType::Text => "Text (.txt)",
            OutputType::Markdown => "Markdown (.md)",
            OutputType::Html => "HTML (.html)",
            OutputType::Pdf => "PDF (.pdf)",
        }
    }

    /// The file extension for this type.
    pub fn ext(self) -> &'static str {
        match self {
            OutputType::Ffscrm => "ffscrm",
            OutputType::Text => "txt",
            OutputType::Markdown => "md",
            OutputType::Html => "html",
            OutputType::Pdf => "pdf",
        }
    }
}

/// Transient state for the capture-STOP Save-As modal. Held on the shell as
/// `scrm_ui.save_as: Option<SaveAsState>`; `Some` means the modal is open.
///
/// Validates: screen-snapshot-scrm Requirement 24.1, 24.2.
#[derive(Debug, Clone)]
pub struct SaveAsState {
    /// Session name buffer, pre-filled from the Collection name.
    pub name: String,
    /// Directory buffer, pre-filled from `scrm_dir()` and editable so the user
    /// can navigate away from the ffwb-domain default (owner directive).
    pub directory: String,
    /// The selected output type.
    pub output_type: OutputType,
}

impl SaveAsState {
    /// Build the modal state for a session, defaulting the directory to
    /// `scrm_dir()` (the discoverable ffwb-domain captures dir) and the type to
    /// the native archive.
    ///
    /// Validates: screen-snapshot-scrm Requirement 24.1.
    pub fn new(default_name: String, default_dir: PathBuf) -> Self {
        Self {
            name: default_name,
            directory: default_dir.to_string_lossy().into_owned(),
            output_type: OutputType::Ffscrm,
        }
    }

    /// The full output path assembled from the directory and name buffers plus
    /// the selected type's extension. A name already ending in the correct
    /// extension is not doubled.
    pub fn output_path(&self) -> PathBuf {
        let ext = self.output_type.ext();
        let name = self.name.trim();
        let stem = if name.is_empty() { "collection" } else { name };
        let file = if stem
            .rsplit('.')
            .next()
            .map(|e| e.eq_ignore_ascii_case(ext))
            .unwrap_or(false)
        {
            stem.to_string()
        } else {
            format!("{stem}.{ext}")
        };
        PathBuf::from(self.directory.trim()).join(file)
    }

    /// The EXISTING front-door command string that persists this session
    /// (command parity, Req 24.4). `Ffscrm` -> `CAPTURE SAVE <path>`; every other
    /// type -> `CAPTURE EXPORT <FMT> <path>`. No new persistence path.
    ///
    /// Validates: screen-snapshot-scrm Requirement 24.4.
    pub fn save_command(&self) -> String {
        let path = self.output_path();
        let path = path.display();
        match self.output_type {
            OutputType::Ffscrm => format!("CAPTURE SAVE {path}"),
            OutputType::Text => format!("CAPTURE EXPORT TEXT {path}"),
            OutputType::Markdown => format!("CAPTURE EXPORT MD {path}"),
            OutputType::Html => format!("CAPTURE EXPORT HTML {path}"),
            OutputType::Pdf => format!("CAPTURE EXPORT PDF {path}"),
        }
    }
}

/// The result of rendering the Save-As modal for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Still open; keep rendering.
    Open,
    /// The user clicked Save: dispatch the front-door command and close.
    Save,
    /// The user cancelled (Cancel button, Escape, or dimmed-background click):
    /// write nothing and close (Req 24.5).
    Cancel,
}

/// Render the Save-As modal for one frame against `state`. Returns an
/// [`Outcome`] the shell acts on. The caller only calls this when the modal is
/// open (its `scrm_ui.save_as` is `Some`).
///
/// Validates: screen-snapshot-scrm Requirement 24.1, 24.2, 24.5, 24.6.
pub fn render(ctx: &egui::Context, state: &mut SaveAsState) -> Outcome {
    let mut save_clicked = false;
    let mut cancel_clicked = false;

    let modal = egui::Modal::new(egui::Id::new("scrm_save_as_modal")).show(ctx, |ui| {
        ui.set_max_width(520.0);
        ui.heading("Save screen collection");
        ui.label("Name the session and choose where to save it.");
        ui.add_space(6.0);

        egui::Grid::new("scrm_save_as_grid")
            .num_columns(2)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.name)
                        .id(name_field_id())
                        .desired_width(360.0),
                );
                ui.end_row();

                ui.label("Folder:");
                ui.add(
                    egui::TextEdit::singleline(&mut state.directory)
                        .id(dir_field_id())
                        .desired_width(360.0),
                );
                ui.end_row();

                ui.label("Type:");
                let selected = state.output_type;
                egui::ComboBox::from_id_salt("scrm_save_as_type")
                    .selected_text(selected.label())
                    .show_ui(ui, |ui| {
                        for ty in OutputType::ALL {
                            ui.selectable_value(&mut state.output_type, ty, ty.label());
                        }
                    });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Save").min_size(egui::vec2(96.0, 0.0)))
                .clicked()
            {
                save_clicked = true;
            }
            if ui
                .add(egui::Button::new("Cancel").min_size(egui::vec2(96.0, 0.0)))
                .clicked()
            {
                cancel_clicked = true;
            }
        });
    });

    // Escape or a click on the dimmed background cancels (Req 24.5): the safe
    // default that writes nothing.
    if modal.should_close() {
        cancel_clicked = true;
    }

    if save_clicked {
        Outcome::Save
    } else if cancel_clicked {
        Outcome::Cancel
    } else {
        Outcome::Open
    }
}

/// Stable id of the name field (first interior control of the modal).
pub fn name_field_id() -> egui::Id {
    egui::Id::new("scrm_save_as_name")
}

/// Stable id of the directory field.
pub fn dir_field_id() -> egui::Id {
    egui::Id::new("scrm_save_as_dir")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Validates: Requirement 24.1 -- the directory defaults to scrm_dir() and
    // the name is pre-filled.
    #[test]
    fn new_defaults_directory_and_name() {
        // Validates: Requirement 24.1
        let dir = PathBuf::from("/data/screen-collections");
        let s = SaveAsState::new("Repro".to_string(), dir.clone());
        assert_eq!(s.name, "Repro");
        assert_eq!(s.directory, dir.to_string_lossy());
        assert_eq!(s.output_type, OutputType::Ffscrm);
    }

    // Validates: Requirement 24.2 -- the selector offers the implemented subset
    // (native + TEXT/MD/HTML/PDF), ODT/ODP omitted.
    #[test]
    fn offers_implemented_output_types_only() {
        // Validates: Requirement 24.2
        assert_eq!(OutputType::ALL.len(), 5);
        assert!(OutputType::ALL.contains(&OutputType::Ffscrm));
        assert!(OutputType::ALL.contains(&OutputType::Pdf));
    }

    // Validates: Requirement 24.4 -- a native-archive save dispatches the
    // EXISTING `CAPTURE SAVE <path>` front-door verb.
    #[test]
    fn ffscrm_save_command_uses_capture_save() {
        // Validates: Requirement 24.4
        let s = SaveAsState::new("Repro".to_string(), PathBuf::from("/data"));
        let cmd = s.save_command();
        assert!(cmd.starts_with("CAPTURE SAVE "), "got: {cmd}");
        assert!(cmd.contains("Repro.ffscrm"), "got: {cmd}");
    }

    // Validates: Requirement 24.4 -- a PDF save dispatches the EXISTING
    // `CAPTURE EXPORT PDF <path>` front-door verb.
    #[test]
    fn pdf_save_command_uses_capture_export_pdf() {
        // Validates: Requirement 24.4
        let mut s = SaveAsState::new("Repro".to_string(), PathBuf::from("/data"));
        s.output_type = OutputType::Pdf;
        let cmd = s.save_command();
        assert!(cmd.starts_with("CAPTURE EXPORT PDF "), "got: {cmd}");
        assert!(cmd.contains("Repro.pdf"), "got: {cmd}");
    }

    // Validates: Requirement 24.4 -- the extension is not doubled when the name
    // already carries it.
    #[test]
    fn output_path_does_not_double_extension() {
        // Validates: Requirement 24.4
        let s = SaveAsState::new("repro.ffscrm".to_string(), PathBuf::from("/data"));
        let path = s.output_path();
        assert!(path.ends_with("repro.ffscrm"), "got: {}", path.display());
    }

    // Validates: Requirement 24.4 -- each output type maps to the matching
    // front-door verb and extension.
    #[test]
    fn each_type_maps_to_front_door_verb() {
        // Validates: Requirement 24.4
        let cases = [
            (OutputType::Text, "CAPTURE EXPORT TEXT ", "txt"),
            (OutputType::Markdown, "CAPTURE EXPORT MD ", "md"),
            (OutputType::Html, "CAPTURE EXPORT HTML ", "html"),
        ];
        for (ty, prefix, ext) in cases {
            let mut s = SaveAsState::new("S".to_string(), PathBuf::from("/d"));
            s.output_type = ty;
            assert_eq!(ty.ext(), ext);
            assert!(s.save_command().starts_with(prefix), "type {ty:?}");
        }
    }
}
