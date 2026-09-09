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
    /// Commit a Command Configurator action to the store.
    ///
    /// Validates: command-configurator Requirement 2.3, 2.4, 2.5, 4.1, 4.2, 4.6.
    pub(super) fn apply_configurator_action(&mut self, action: ConfiguratorAction) {
        match action {
            ConfiguratorAction::None => {}
            ConfiguratorAction::Save => self.commit_configurator_form(),
            ConfiguratorAction::Delete(id) => {
                // Req 2.5: remove and write the store back.
                if self.command_store.remove(&id) {
                    if let Err(e) = self.command_store.save() {
                        self.command_configurator_panel.error = Some(e);
                    } else {
                        self.command_configurator_panel.error = None;
                    }
                }
            }
        }
    }

    /// Validate and commit the open edit form (add or edit).
    ///
    /// On validation failure the store is left unchanged and the error is shown
    /// (Requirement 2.4).
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

        if form.is_edit {
            // Replace the existing definition in place (preserve order).
            if let Some(slot) = self
                .command_store
                .definitions
                .iter_mut()
                .find(|d| d.id == def.id)
            {
                *slot = def;
            } else {
                self.command_store.definitions.push(def);
            }
        } else {
            // Add: reject a duplicate id (Requirement 2.3, 4.1).
            if self.command_store.find(&def.id).is_some() {
                self.command_configurator_panel.error =
                    Some(format!("command id '{}' already exists", def.id));
                return;
            }
            self.command_store.definitions.push(def);
        }

        // Persist the full store (Requirement 2.4).
        if let Err(e) = self.command_store.save() {
            self.command_configurator_panel.error = Some(e);
            return;
        }
        // Success: close the form, clear any error.
        self.command_configurator_panel.form = None;
        self.command_configurator_panel.error = None;
    }
}
