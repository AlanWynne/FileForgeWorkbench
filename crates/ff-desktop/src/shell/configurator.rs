//! Apply Command Configurator UI actions against the `CommandStore`.
//!
//! The Command Configurator render returns a [`ConfiguratorAction`]; this
//! module commits it: validating and adding/updating a definition on Save, or
//! removing one on Delete, then persisting the store to `commands.toml`.
//!
//! Validates: command-configurator Requirement 2.3, 2.4, 2.5, 4.1, 4.2, 4.6.

use ff_command::CommandId;

use super::WorkbenchShell;
use crate::command_config::render::ConfiguratorAction;
use crate::command_config::store::validate_definition;

impl WorkbenchShell {
    /// Commit a Command Configurator action to the WORKING COPY (B081,
    /// project-master Req 11). Add / edit / delete now mutate the panel's
    /// whole-store working copy ONLY -- the backing `commands.toml` is untouched
    /// until SAVE / END flush the working copy through the CR-CH-071 seam
    /// (`command_configurator_persist`). Nothing is written on a bare edit
    /// (Req 11.2 -- autosave removed).
    ///
    /// Validates: command-configurator Requirement 2.3, 2.4, 2.5, 4.1, 4.2, 4.6;
    /// project-master Requirement 11.2.
    pub(super) fn apply_configurator_action(&mut self, action: ConfiguratorAction) {
        match action {
            ConfiguratorAction::None => {}
            ConfiguratorAction::Save => self.commit_configurator_form(),
            ConfiguratorAction::Delete(id) => {
                // Req 2.5 + Req 11.2: remove from the WORKING COPY only (no store
                // write; deferred to SAVE/END). Take a working copy from the
                // persisted store on first edit, then drop the row.
                let persisted = self.command_store.definitions.clone();
                let working = self.command_configurator_panel.working_mut(&persisted);
                working.retain(|d| d.id != id);
                self.command_configurator_panel.error = None;
            }
        }
    }

    /// Validate and commit the open edit form (add or edit) INTO THE WORKING
    /// COPY.
    ///
    /// On validation failure the working copy is left unchanged and the error is
    /// shown (Requirement 2.4). On success the add/edit is applied to the working
    /// copy only -- the backing store is written later by SAVE / END (Req 11.2).
    ///
    /// Validates: command-configurator Requirement 2.3, 2.4, 4.1, 4.2, 4.6;
    /// project-master Requirement 11.2.
    fn commit_configurator_form(&mut self) {
        let form = match self.command_configurator_panel.form.as_ref() {
            Some(f) => f.clone(),
            None => return,
        };

        // Build the definition from the form (variant-specific field checks).
        let def = match form.build() {
            Ok(d) => d,
            Err(e) => {
                self.command_configurator_panel.error = Some(e);
                return;
            }
        };

        // Full definition validation: id rules, reserved id, label, external
        // program (Requirement 4.1, 4.2, 4.6). Reserved = a registered built-in.
        let registry = &self.cmd_registry;
        let is_reserved = |id: &str| match CommandId::new(id.to_string()) {
            Some(cid) => registry.contains(&cid),
            None => false,
        };
        if let Err(e) = validate_definition(&def, is_reserved) {
            self.command_configurator_panel.error = Some(e.to_string());
            return;
        }

        // Apply to the WORKING COPY (taken from the persisted store on first
        // edit), NOT the backing store.
        let persisted = self.command_store.definitions.clone();
        if !form.is_edit && persisted.iter().any(|d| d.id == def.id) {
            // Add: reject a duplicate id against the persisted store (Req 2.3,
            // 4.1) -- a definition with this id already exists and is not being
            // edited.
            self.command_configurator_panel.error =
                Some(format!("command id '{}' already exists", def.id));
            return;
        }
        let working = self.command_configurator_panel.working_mut(&persisted);
        if form.is_edit {
            // Replace the existing definition in place (preserve order).
            if let Some(slot) = working.iter_mut().find(|d| d.id == def.id) {
                *slot = def;
            } else {
                working.push(def);
            }
        } else if working.iter().any(|d| d.id == def.id) {
            // A duplicate already staged in the working copy.
            self.command_configurator_panel.error =
                Some(format!("command id '{}' already exists", def.id));
            return;
        } else {
            working.push(def);
        }

        // Success: close the form, clear any error. The working copy is NOT
        // flushed here -- SAVE / END commits it (Req 11.2).
        self.command_configurator_panel.form = None;
        self.command_configurator_panel.error = None;
    }
}
