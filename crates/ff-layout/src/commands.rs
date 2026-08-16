//! Layout command definitions.
//!
//! Defines the layout commands that are registered with `ff-command`'s
//! shortcut registry. The actual registration happens during integration
//! wiring (task 16) when `ff-command` is available as a dependency.

/// Command identifier for toggling dock/float on the focused panel.
pub const CMD_UNDOCK: &str = "layout.undock";

/// Command identifier for redocking a floating panel.
pub const CMD_REDOCK: &str = "layout.redock";

/// Command identifier for toggling a named panel's visibility.
pub const CMD_TOGGLE_PANEL: &str = "layout.toggle_panel";

/// Command identifier for splitting the active tab group horizontally.
pub const CMD_SPLIT_HORIZONTAL: &str = "layout.split_horizontal";

/// Command identifier for splitting the active tab group vertically.
pub const CMD_SPLIT_VERTICAL: &str = "layout.split_vertical";

/// Command identifier for undocking the active tab.
pub const CMD_UNDOCK_TAB: &str = "layout.undock_tab";

/// Command identifier for redocking a floating tab.
pub const CMD_REDOCK_TAB: &str = "layout.redock_tab";

/// Command identifier for activating a persona by name.
pub const CMD_PERSONA_ACTIVATE: &str = "layout.persona.activate";

/// Command identifier for saving the current layout as a persona.
pub const CMD_PERSONA_SAVE: &str = "layout.persona.save";

/// Command identifier for resetting to the default layout.
pub const CMD_RESET: &str = "layout.reset";

/// Command identifier for exporting the current layout to a file.
pub const CMD_EXPORT: &str = "layout.export";

/// Command identifier for importing a layout from a file.
pub const CMD_IMPORT: &str = "layout.import";

/// All layout command identifiers.
pub const ALL_COMMANDS: &[&str] = &[
    CMD_UNDOCK,
    CMD_REDOCK,
    CMD_TOGGLE_PANEL,
    CMD_SPLIT_HORIZONTAL,
    CMD_SPLIT_VERTICAL,
    CMD_UNDOCK_TAB,
    CMD_REDOCK_TAB,
    CMD_PERSONA_ACTIVATE,
    CMD_PERSONA_SAVE,
    CMD_RESET,
    CMD_EXPORT,
    CMD_IMPORT,
];
