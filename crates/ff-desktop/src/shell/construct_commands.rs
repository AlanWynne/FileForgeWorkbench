//! # Built-in Command Registration
//!
//! The `register_builtin_commands` free function extracted VERBATIM from
//! `construct.rs` as part of the behaviour-preserving file-size split (pure
//! code movement -- no logic edits). It registers the shell's built-in
//! registry commands (file.open, file.exit, menu.open, config.open) during
//! construction.

use std::sync::Arc;

use ff_command::{CommandId, CommandMetadata, CommandRegistry};

use super::handlers::{ConfigOpenHandler, FileExitHandler, FileOpenHandler, MenuOpenHandler};
use super::state::PendingOpen;

/// Register the shell's built-in registry commands on `registry`: file.open,
/// file.exit, menu.open, and config.open. The `pending_open` and `should_close`
/// shared cells are the ones the shell checks each frame; the two marker
/// handlers (menu.open, config.open) exist so a bare `MENU`/`CONFIG` resolves as
/// a built-in, with the shell intercepting them in `handle_command`.
pub(super) fn register_builtin_commands(
    registry: &CommandRegistry,
    pending_open: &PendingOpen,
    should_close: &Arc<std::sync::Mutex<bool>>,
) {
    // Register file.open
    let open_id = CommandId::new("file.open").expect("valid id");
    let open_meta = CommandMetadata::builder("Open File", "Open a file from disk")
        .category("file")
        .build();
    registry
        .register(
            open_id,
            open_meta,
            Box::new(FileOpenHandler {
                pending: pending_open.clone(),
            }),
        )
        .expect("file.open registration");

    // Register file.exit
    let exit_id = CommandId::new("file.exit").expect("valid id");
    let exit_meta = CommandMetadata::builder("Exit", "Exit the application")
        .category("file")
        .build();
    registry
        .register(
            exit_id,
            exit_meta,
            Box::new(FileExitHandler {
                should_close: should_close.clone(),
            }),
        )
        .expect("file.exit registration");

    // Register menu.open (menu-workspace Requirement 11.6) -- marker handler;
    // the shell intercepts MENU / menu.open in handle_command.
    let menu_open_id = CommandId::new("menu.open").expect("valid id");
    let menu_open_meta = CommandMetadata::builder("Open Menu", "Open or return to a menu")
        .category("menu")
        .build();
    registry
        .register(menu_open_id, menu_open_meta, Box::new(MenuOpenHandler))
        .expect("menu.open registration");

    // Register config.open (CR-CH-025, configuration-system Requirement 20)
    // -- marker handler; the shell intercepts CONFIG in handle_command. Being
    // registered lets a bare `CONFIG` resolve as a built-in (chain stage 2),
    // shadowing any same-named menu/macro.
    let config_open_id = CommandId::new("config.open").expect("valid id");
    let config_open_meta = CommandMetadata::builder(
        "Configuration",
        "Browse all configuration keys (optionally filtered by namespace)",
    )
    .build();
    registry
        .register(
            config_open_id,
            config_open_meta,
            Box::new(ConfigOpenHandler),
        )
        .expect("config.open registration");
}
