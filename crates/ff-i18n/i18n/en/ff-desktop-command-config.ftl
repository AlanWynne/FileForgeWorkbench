# English Identity_Base keys for the Command Configurator Context
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/command_config/{render.rs,edit.rs}, byte-for-byte,
# ASCII only. Render code resolves each displayed string through the
# ff_i18n::t()/t_args() seam (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): egui id_salt values, the TargetVariant enum
# `label()` identity strings (menu/custom_workspace/function/macro/external),
# `variant_name()` and the external mode column labels (logic/identity, asserted
# by unit tests).

# --- List / toolbar ---------------------------------------------------------
ff-desktop-command-config-add = Add
ff-desktop-command-config-empty = No user-defined commands. Use Add to create one.
ff-desktop-command-config-col-id = Id
ff-desktop-command-config-col-label = Label
ff-desktop-command-config-col-variant = Variant
ff-desktop-command-config-col-mode = Mode
ff-desktop-command-config-col-actions = Actions
ff-desktop-command-config-edit = Edit
ff-desktop-command-config-delete = Delete

# --- Delete confirmation ----------------------------------------------------
ff-desktop-command-config-confirm-delete-title = Confirm delete
ff-desktop-command-config-confirm-delete-body = Delete command definition '{ $id }'?
ff-desktop-command-config-cancel = Cancel

# --- Add/Edit form ----------------------------------------------------------
ff-desktop-command-config-window-edit = Edit command
ff-desktop-command-config-window-add = Add command
ff-desktop-command-config-save = Save

# --- Edit form field labels -------------------------------------------------
ff-desktop-command-config-field-id = Id
ff-desktop-command-config-field-label = Label
ff-desktop-command-config-field-description = Description
ff-desktop-command-config-field-variant = Variant
ff-desktop-command-config-field-menu-name = Menu name
ff-desktop-command-config-field-workspace-kind = Workspace kind
ff-desktop-command-config-field-command-id = Command id
ff-desktop-command-config-field-macro = Macro
ff-desktop-command-config-field-is-path = Is path
ff-desktop-command-config-check-treat-as-path = treat as file path
ff-desktop-command-config-field-program = Program
ff-desktop-command-config-field-args = Args
ff-desktop-command-config-field-working-dir = Working dir
ff-desktop-command-config-field-captured = Captured
ff-desktop-command-config-check-capture-output = capture output
