//! Stable Automation ID constants for all automatable controls in ff-desktop.
//!
//! Every constant follows the dot-separated `<panel>.<group>.<control>` convention.
//! Constants are API surface for FFTest scripts and are not all called from
//! production code yet.
//!
//! Validates: Requirement 2.2, 2.4 (automated-dialog-testing)

#![allow(dead_code)]

// === Command field ==========================================================

/// The primary ISPF-style command input field ("Command ===>").
pub const COMMAND_FIELD: &str = "shell.command_field";

/// The SCROLL ===> amount input field.
pub const SCROLL_FIELD: &str = "shell.scroll_field";

// === Status bar =============================================================

/// The status bar message / error area.
pub const STATUSBAR_MESSAGE: &str = "statusbar.message";

/// The line/column indicator in the status bar.
pub const STATUSBAR_LINE_COL: &str = "statusbar.line_col";

/// The encoding indicator in the status bar.
pub const STATUSBAR_ENCODING: &str = "statusbar.encoding";

/// The modified indicator dot in the status bar.
pub const STATUSBAR_MODIFIED: &str = "statusbar.modified";

// === Tab bar ================================================================

/// A tab header button -- append the 0-based index, e.g. "tab.header.0".
pub const TAB_HEADER_PREFIX: &str = "tab.header.";

/// The POM tab header.
pub const TAB_POM: &str = "tab.pom";

/// The Files Panel tab header.
pub const TAB_FILES_PANEL: &str = "tab.files_panel";

/// The Settings Panel tab header.
pub const TAB_SETTINGS: &str = "tab.settings";

/// The File Explorer Panel tab header.
pub const TAB_FILE_EXPLORER: &str = "tab.file_explorer";

// === Primary Option Menu ====================================================

/// A POM option row button -- append the 0-based index, e.g. "pom.option.0".
pub const POM_OPTION_PREFIX: &str = "pom.option.";

/// The POM exit line button.
pub const POM_EXIT: &str = "pom.exit";

/// The POM calendar previous-month button.
pub const POM_CALENDAR_PREV: &str = "pom.calendar.prev";

/// The POM calendar next-month button.
pub const POM_CALENDAR_NEXT: &str = "pom.calendar.next";

// === Menu bar ===============================================================

/// A top-level menu bar heading -- append the label, e.g. "menu.bar.File".
pub const MENU_BAR_PREFIX: &str = "menu.bar.";

/// The File > Open menu item.
pub const MENU_FILE_OPEN: &str = "menu.file.open";

/// The File > Save menu item.
pub const MENU_FILE_SAVE: &str = "menu.file.save";

/// The File > Close menu item.
pub const MENU_FILE_CLOSE: &str = "menu.file.close";

/// The Help > About menu item.
pub const MENU_HELP_ABOUT: &str = "menu.help.about";

// === Catalog Manager Dialog =================================================

/// The catalog name text field in the New/Edit Catalog dialog.
pub const DIALOG_CATALOG_NAME: &str = "dialog.catalog_manager.name_field";

/// The catalog type selector in the New Catalog dialog.
pub const DIALOG_CATALOG_TYPE: &str = "dialog.catalog_manager.type_selector";

/// The repository path field in the New/Edit Catalog dialog.
pub const DIALOG_CATALOG_REPO_PATH: &str = "dialog.catalog_manager.repo_path";

/// The Confirm/Save button in the Catalog Manager dialog.
pub const DIALOG_CATALOG_CONFIRM: &str = "dialog.catalog_manager.confirm_button";

/// The Cancel button in the Catalog Manager dialog.
pub const DIALOG_CATALOG_CANCEL: &str = "dialog.catalog_manager.cancel_button";

// === Dataset Allocation Dialog ==============================================

/// The dataset name field in the Allocate Dataset dialog.
pub const DIALOG_ALLOC_DSN: &str = "dialog.dataset_alloc.dsn_field";

/// The RECFM field in the Allocate Dataset dialog.
pub const DIALOG_ALLOC_RECFM: &str = "dialog.dataset_alloc.recfm_field";

/// The LRECL field in the Allocate Dataset dialog.
pub const DIALOG_ALLOC_LRECL: &str = "dialog.dataset_alloc.lrecl_field";

/// The Allocate button in the Allocate Dataset dialog.
pub const DIALOG_ALLOC_CONFIRM: &str = "dialog.dataset_alloc.confirm_button";

/// The Cancel button in the Allocate Dataset dialog.
pub const DIALOG_ALLOC_CANCEL: &str = "dialog.dataset_alloc.cancel_button";

// === Settings Panel =========================================================

/// The filter input field in the Settings Panel.
pub const SETTINGS_FILTER: &str = "settings.filter_field";

// === Key Config Dialog ======================================================

/// The Save button in the Key Configuration dialog.
pub const DIALOG_KEYS_SAVE: &str = "dialog.key_config.save_button";

/// The Cancel button in the Key Configuration dialog.
pub const DIALOG_KEYS_CANCEL: &str = "dialog.key_config.cancel_button";

// === About Dialog ===========================================================

/// The Close button in the Help > About dialog.
pub const DIALOG_ABOUT_CLOSE: &str = "dialog.about.close_button";

// === File Explorer Panel ====================================================

/// The sidebar catalog list in the File Explorer Panel.
pub const EXPLORER_SIDEBAR: &str = "file_explorer.sidebar";

/// A catalog mount node in the File Explorer sidebar -- append catalog name.
pub const EXPLORER_CATALOG_PREFIX: &str = "file_explorer.catalog.";

// === Editor Panel ===========================================================

/// The active editor content area.
pub const EDITOR_CONTENT: &str = "editor.content";
