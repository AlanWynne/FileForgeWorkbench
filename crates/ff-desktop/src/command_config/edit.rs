//! Add/Edit form state and the variant-specific Command_Target editor.
//!
//! `EditForm` holds the raw text fields for one definition being created or
//! edited. `build()` converts the fields into a validated `CommandDefinition`
//! (Requirement 4.1, 4.2); the shell then commits it to the `CommandStore`.
//!
//! Validates: command-configurator Requirement 2.6, 4.1, 4.2.

use eframe::egui;

use ff_command::{CommandTarget, ExternalMode, MacroSource, TargetParams};

use super::CommandDefinition;

/// Which Command_Target variant the form is editing.
///
/// Validates: command-configurator Requirement 2.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetVariant {
    /// Open a menu workspace by name.
    Menu,
    /// Open a built-in Context.
    CustomWorkspace,
    /// Invoke a registered command id.
    Function,
    /// Run a macro by name or path.
    Macro,
    /// Run an external program.
    External,
}

impl TargetVariant {
    /// All variants in display order.
    pub const ALL: [TargetVariant; 5] = [
        TargetVariant::Menu,
        TargetVariant::CustomWorkspace,
        TargetVariant::Function,
        TargetVariant::Macro,
        TargetVariant::External,
    ];

    /// A short, stable label for the selector.
    pub fn label(self) -> &'static str {
        match self {
            TargetVariant::Menu => "menu",
            TargetVariant::CustomWorkspace => "custom_workspace",
            TargetVariant::Function => "function",
            TargetVariant::Macro => "macro",
            TargetVariant::External => "external",
        }
    }

    /// The variant of an existing target.
    pub fn of(target: &CommandTarget) -> TargetVariant {
        match target {
            CommandTarget::Menu { .. } => TargetVariant::Menu,
            CommandTarget::CustomWorkspace { .. } => TargetVariant::CustomWorkspace,
            CommandTarget::Function { .. } => TargetVariant::Function,
            CommandTarget::Macro { .. } => TargetVariant::Macro,
            CommandTarget::External { .. } => TargetVariant::External,
        }
    }
}

/// Editable fields for one `CommandDefinition`.
///
/// Validates: command-configurator Requirement 2.6.
#[derive(Debug, Clone, PartialEq)]
pub struct EditForm {
    /// True when editing an existing definition (id is read-only then).
    pub is_edit: bool,
    /// Definition id.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Optional one-line description.
    pub description: String,
    /// Selected target variant.
    pub variant: TargetVariant,
    /// Menu name (Menu variant).
    pub menu_name: String,
    /// Workspace kind (CustomWorkspace variant).
    pub workspace_kind: String,
    /// Command id (Function variant).
    pub command_id: String,
    /// Macro name-or-path text (Macro variant).
    pub macro_source: String,
    /// True when the macro source is a path rather than a name.
    pub macro_is_path: bool,
    /// External program (External variant).
    pub program: String,
    /// External args, whitespace-separated (External variant).
    pub args: String,
    /// External working directory (External variant).
    pub working_dir: String,
    /// External execution mode is Captured when true, Detached when false.
    pub external_captured: bool,
}

impl Default for EditForm {
    fn default() -> Self {
        Self {
            is_edit: false,
            id: String::new(),
            label: String::new(),
            description: String::new(),
            variant: TargetVariant::Function,
            menu_name: String::new(),
            workspace_kind: String::new(),
            command_id: String::new(),
            macro_source: String::new(),
            macro_is_path: false,
            program: String::new(),
            args: String::new(),
            working_dir: String::new(),
            external_captured: true,
        }
    }
}

impl EditForm {
    /// A blank form for adding a new definition.
    pub fn new_add() -> Self {
        Self::default()
    }

    /// Pre-fill the form from an existing definition for editing.
    pub fn from_definition(def: &CommandDefinition) -> Self {
        let mut form = Self {
            is_edit: true,
            id: def.id.clone(),
            label: def.label.clone(),
            description: def.description.clone().unwrap_or_default(),
            variant: TargetVariant::of(&def.target),
            ..Self::default()
        };
        match &def.target {
            CommandTarget::Menu { name } => form.menu_name = name.clone(),
            CommandTarget::CustomWorkspace { workspace_kind, .. } => {
                form.workspace_kind = workspace_kind.clone()
            }
            CommandTarget::Function { command_id, .. } => form.command_id = command_id.clone(),
            CommandTarget::Macro { source } => match source {
                MacroSource::Name(n) => {
                    form.macro_source = n.clone();
                    form.macro_is_path = false;
                }
                MacroSource::Path(p) => {
                    form.macro_source = p.clone();
                    form.macro_is_path = true;
                }
            },
            CommandTarget::External {
                program,
                args,
                working_dir,
                mode,
            } => {
                form.program = program.clone();
                form.args = args.join(" ");
                form.working_dir = working_dir.clone().unwrap_or_default();
                form.external_captured = matches!(mode, ExternalMode::Captured);
            }
        }
        form
    }

    /// Render the form fields, showing only the fields for the chosen variant.
    ///
    /// Validates: command-configurator Requirement 2.6.
    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("command_edit_fields")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Id");
                ui.add_enabled(!self.is_edit, egui::TextEdit::singleline(&mut self.id));
                ui.end_row();

                ui.label("Label");
                ui.text_edit_singleline(&mut self.label);
                ui.end_row();

                ui.label("Description");
                ui.text_edit_singleline(&mut self.description);
                ui.end_row();

                ui.label("Variant");
                egui::ComboBox::from_id_salt("command_variant")
                    .selected_text(self.variant.label())
                    .show_ui(ui, |ui| {
                        for v in TargetVariant::ALL {
                            ui.selectable_value(&mut self.variant, v, v.label());
                        }
                    });
                ui.end_row();
            });

        ui.separator();

        // Variant-specific fields (Requirement 2.6).
        egui::Grid::new("command_target_fields")
            .num_columns(2)
            .show(ui, |ui| match self.variant {
                TargetVariant::Menu => {
                    ui.label("Menu name");
                    ui.text_edit_singleline(&mut self.menu_name);
                    ui.end_row();
                }
                TargetVariant::CustomWorkspace => {
                    ui.label("Workspace kind");
                    ui.text_edit_singleline(&mut self.workspace_kind);
                    ui.end_row();
                }
                TargetVariant::Function => {
                    ui.label("Command id");
                    ui.text_edit_singleline(&mut self.command_id);
                    ui.end_row();
                }
                TargetVariant::Macro => {
                    ui.label("Macro");
                    ui.text_edit_singleline(&mut self.macro_source);
                    ui.end_row();
                    ui.label("Is path");
                    ui.checkbox(&mut self.macro_is_path, "treat as file path");
                    ui.end_row();
                }
                TargetVariant::External => {
                    ui.label("Program");
                    ui.text_edit_singleline(&mut self.program);
                    ui.end_row();
                    ui.label("Args");
                    ui.text_edit_singleline(&mut self.args);
                    ui.end_row();
                    ui.label("Working dir");
                    ui.text_edit_singleline(&mut self.working_dir);
                    ui.end_row();
                    ui.label("Captured");
                    ui.checkbox(&mut self.external_captured, "capture output");
                    ui.end_row();
                }
            });
    }

    /// Build the target from the current fields.
    fn build_target(&self) -> CommandTarget {
        match self.variant {
            TargetVariant::Menu => CommandTarget::Menu {
                name: self.menu_name.trim().to_string(),
            },
            TargetVariant::CustomWorkspace => CommandTarget::CustomWorkspace {
                workspace_kind: self.workspace_kind.trim().to_string(),
                params: TargetParams::new(),
            },
            TargetVariant::Function => CommandTarget::Function {
                command_id: self.command_id.trim().to_string(),
                params: TargetParams::new(),
            },
            TargetVariant::Macro => {
                let src = self.macro_source.trim().to_string();
                let source = if self.macro_is_path {
                    MacroSource::Path(src)
                } else {
                    MacroSource::Name(src)
                };
                CommandTarget::Macro { source }
            }
            TargetVariant::External => CommandTarget::External {
                program: self.program.trim().to_string(),
                args: self
                    .args
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect(),
                working_dir: {
                    let wd = self.working_dir.trim();
                    if wd.is_empty() {
                        None
                    } else {
                        Some(wd.to_string())
                    }
                },
                mode: if self.external_captured {
                    ExternalMode::Captured
                } else {
                    ExternalMode::Detached
                },
            },
        }
    }

    /// Convert the form to a `CommandDefinition`.
    ///
    /// Field-level structural checks (empty menu name, empty command id, etc.)
    /// are enforced here; id / label / reserved-id / external-program rules are
    /// enforced by `store::validate_definition` when the shell commits.
    ///
    /// Validates: command-configurator Requirement 4.1, 4.2.
    pub fn build(&self) -> Result<CommandDefinition, String> {
        let target = self.build_target();
        // Variant-specific required-field checks (Requirement 4.1).
        match &target {
            CommandTarget::Menu { name } if name.is_empty() => {
                return Err("menu name must not be empty".to_string());
            }
            CommandTarget::CustomWorkspace { workspace_kind, .. } if workspace_kind.is_empty() => {
                return Err("workspace kind must not be empty".to_string());
            }
            CommandTarget::Function { command_id, .. } if command_id.is_empty() => {
                return Err("command id must not be empty".to_string());
            }
            CommandTarget::Macro { source } => {
                let empty = match source {
                    MacroSource::Name(n) => n.is_empty(),
                    MacroSource::Path(p) => p.is_empty(),
                };
                if empty {
                    return Err("macro name/path must not be empty".to_string());
                }
            }
            _ => {}
        }
        let description = {
            let d = self.description.trim();
            if d.is_empty() {
                None
            } else {
                Some(d.to_string())
            }
        };
        Ok(CommandDefinition {
            id: self.id.trim().to_string(),
            label: self.label.trim().to_string(),
            description,
            category: "user".to_string(),
            target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.2 -- an External form builds an External target.
    #[test]
    fn build_external_target_from_form() {
        let mut form = EditForm::new_add();
        form.id = "build.release".to_string();
        form.label = "Build".to_string();
        form.variant = TargetVariant::External;
        form.program = "pwsh".to_string();
        form.args = "-File build.ps1".to_string();
        form.external_captured = true;
        let def = form.build().expect("builds");
        match def.target {
            CommandTarget::External {
                program,
                args,
                mode,
                ..
            } => {
                assert_eq!(program, "pwsh");
                assert_eq!(args, vec!["-File".to_string(), "build.ps1".to_string()]);
                assert_eq!(mode, ExternalMode::Captured);
            }
            other => panic!("expected External, got {other:?}"),
        }
    }

    // Validates: Requirement 4.1 -- a Function form with an empty command id is rejected.
    #[test]
    fn build_function_rejects_empty_command_id() {
        let mut form = EditForm::new_add();
        form.id = "x.y".to_string();
        form.label = "X".to_string();
        form.variant = TargetVariant::Function;
        form.command_id = "   ".to_string();
        assert!(form.build().is_err());
    }

    // Validates: Requirement 2.6 -- editing an existing definition pre-fills fields.
    #[test]
    fn from_definition_prefills_external_fields() {
        let def = CommandDefinition {
            id: "run.tool".to_string(),
            label: "Run".to_string(),
            description: Some("desc".to_string()),
            category: "user".to_string(),
            target: CommandTarget::External {
                program: "tool".to_string(),
                args: vec!["-a".to_string()],
                working_dir: Some("/tmp".to_string()),
                mode: ExternalMode::Detached,
            },
        };
        let form = EditForm::from_definition(&def);
        assert!(form.is_edit);
        assert_eq!(form.variant, TargetVariant::External);
        assert_eq!(form.program, "tool");
        assert_eq!(form.args, "-a");
        assert_eq!(form.working_dir, "/tmp");
        assert!(!form.external_captured);
    }

    // Validates: Requirement 2.6 -- round-trip build then edit preserves the id.
    #[test]
    fn edit_form_keeps_id_read_only_flag() {
        let def = CommandDefinition {
            id: "keep.id".to_string(),
            label: "L".to_string(),
            description: None,
            category: "user".to_string(),
            target: CommandTarget::Function {
                command_id: "file.save".to_string(),
                params: TargetParams::new(),
            },
        };
        let form = EditForm::from_definition(&def);
        assert!(form.is_edit);
        assert_eq!(form.id, "keep.id");
    }
}
