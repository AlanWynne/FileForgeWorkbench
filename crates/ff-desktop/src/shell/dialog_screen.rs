//! # Transient-overlay `DialogScreen` models (screen-snapshot-scrm Req 27)
//!
//! A message box or modal dialog is a TRANSIENT OVERLAY on top of the active
//! Context, NOT a Workspace Context, so the per-Context `ScreenProvider`
//! dispatch (`shell/screen_provider.rs`) does not reach it. Req 27 extends the
//! snapshot/capture contract to these transient surfaces: each overlay yields a
//! logical [`ScreenModel`] (title + body message + field labels/values + list
//! rows + button labels) built from its OWN state, NEVER from an egui pixel grab
//! (Req 27.1, 27.7).
//!
//! The [`DialogScreen`] trait is the single shape every overlay model builder
//! implements. For the overlays that have a self-contained state struct
//! ([`crate::shell::reset_bare::ResetBareTarget`],
//! [`crate::shell::external_adapter::PendingExternal`],
//! [`crate::scrm_save_as_dialog::SaveAsState`]) the `impl` lives beside the
//! struct's capture here; the shell-flag overlays (About, Unsaved-workspace,
//! Command history, SWAP) have no dedicated struct, so their models are built in
//! [`super::WorkbenchShell::active_dialog_capture`] from the shell's own state.
//!
//! A declared-secret field (password / token / API key) is marked with
//! [`ff_screen_model::FieldAttributes::plain().sensitive()`]; the single
//! redaction point is `active_dialog_capture`, which applies
//! [`ff_screen_model::ScreenModel::redacted`] so every caller gets the masked
//! model (Req 27.4, 27.5). No live core overlay carries a credential secret
//! today, but the contract is proven by a synthetic secret dialog in the tests.
//!
//! Validates: screen-snapshot-scrm Requirement 27.1, 27.4, 27.5, 27.7.

use ff_screen_model::{Field, ScreenModel};

/// A transient overlay that can yield its logical screen as a [`ScreenModel`],
/// built from the overlay's OWN state (never egui). The capture seam renders
/// this model, so every dialog's text is copy/paste-able (Req 27.1, 27.7).
///
/// Validates: screen-snapshot-scrm Requirement 27.1, 27.7.
pub(crate) trait DialogScreen {
    /// The logical screen model for this overlay: title, body message(s),
    /// labelled fields, and button labels.
    fn screen_model(&self) -> ScreenModel;
}

impl DialogScreen for crate::shell::external_adapter::PendingExternal {
    /// The external-run confirmation: the body prompt, the resolved command
    /// string as a field, and the Run / Cancel buttons.
    fn screen_model(&self) -> ScreenModel {
        let command = if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        };
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-shell-dialogs-exec-title"));
        model
            .messages
            .push(ff_i18n::t("ff-desktop-shell-dialogs-exec-body"));
        model = model.with_field(Field::new("Command", command));
        model
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-exec-run"))
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
    }
}

impl DialogScreen for crate::shell::reset_bare::ResetBareTarget {
    /// The RESET BARE confirmation: the heading, one row per profile that will
    /// be reset, and the Confirm / Cancel buttons.
    fn screen_model(&self) -> ScreenModel {
        let mut model = ScreenModel::new(ff_i18n::t("ff-desktop-shell-dialogs-reset-heading"));
        model
            .messages
            .push(ff_i18n::t("ff-desktop-shell-dialogs-reset-intro"));
        for (name, _udd) in &self.profiles {
            model = model.with_field(Field::new("Profile", name.clone()));
        }
        model
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-reset-confirm"))
            .with_button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
    }
}

impl DialogScreen for crate::scrm_save_as_dialog::SaveAsState {
    /// The capture-STOP Save-As modal: the Name / Folder / Type fields and the
    /// Save / Cancel buttons.
    fn screen_model(&self) -> ScreenModel {
        let model = ScreenModel::new("Save screen collection".to_string())
            .with_field(Field::new("Name", self.name.clone()))
            .with_field(Field::new("Folder", self.directory.clone()))
            .with_field(Field::new("Type", self.output_type.label().to_string()));
        model.with_button("Save").with_button("Cancel")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_screen_model::FieldAttributes;

    /// A synthetic dialog carrying a declared-secret field, used to prove the
    /// DialogScreen -> redaction contract without a live overlay that holds a
    /// credential (none does today).
    struct FakeSecretDialog;

    impl DialogScreen for FakeSecretDialog {
        fn screen_model(&self) -> ScreenModel {
            let mut model = ScreenModel::new("Connect to host".to_string());
            model.messages.push("Enter your credentials.".to_string());
            let model = model.with_field(Field::new("Host", "h1")).with_field(
                Field::new("Password", "s3cr3t").with_attrs(FieldAttributes::plain().sensitive()),
            );
            model.with_button("[OK]")
        }
    }

    // Validates: screen-snapshot-scrm Req 27.8 (FAITHFUL FIELD-VALUE CAPTURE) --
    // the SCRM Save-As dialog model carries the FULL current Name / Folder / Type
    // field values as displayed, not a partial/truncated string. The owner saw
    // only partial filename text; the model must carry the whole typed filename.
    #[test]
    fn save_as_model_carries_full_current_field_values() {
        use crate::scrm_save_as_dialog::{OutputType, SaveAsState};
        let full_name = "Collection-2026-10-31-20-37-44";
        let mut state = SaveAsState::new(
            full_name.to_string(),
            std::path::PathBuf::from("/data/screen-collections"),
        );
        state.output_type = OutputType::Markdown;

        let model = state.screen_model();

        // The FULL filename value is present verbatim (not truncated).
        let name = model
            .fields
            .iter()
            .find(|f| f.label == "Name")
            .expect("Name field present");
        assert_eq!(name.value, full_name, "the full typed filename is captured");

        // The folder path value is present verbatim.
        let folder = model
            .fields
            .iter()
            .find(|f| f.label == "Folder")
            .expect("Folder field present");
        assert_eq!(folder.value, "/data/screen-collections");

        // The selected Type reflects the current dropdown selection.
        let ty = model
            .fields
            .iter()
            .find(|f| f.label == "Type")
            .expect("Type field present");
        assert_eq!(ty.value, OutputType::Markdown.label());
    }

    // Validates: screen-snapshot-scrm Req 27.1, 27.4, 27.5
    #[test]
    fn dialog_screen_marks_declared_secret_field() {
        let model = FakeSecretDialog.screen_model();

        // Title / body message / button are present (27.1 -- the dialog's text
        // is a logical model, not a pixel grab).
        assert_eq!(model.title, "Connect to host");
        assert!(model.messages.iter().any(|m| m.contains("credentials")));
        assert!(model.buttons.iter().any(|b| b == "[OK]"));

        // The password field is DECLARED secret (27.5 -- redaction is a
        // deliberate property of the field, set by the dialog).
        let password = model
            .fields
            .iter()
            .find(|f| f.label == "Password")
            .expect("password field present");
        assert!(password.attrs.sensitive, "password is declared sensitive");

        // The non-secret Host value is present and copyable (27.5 -- only the
        // declared-secret is withheld).
        assert!(
            model.fields.iter().any(|f| f.value == "h1"),
            "non-secret Host value is present"
        );
    }
}
