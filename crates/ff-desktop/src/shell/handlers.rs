//! # Built-in Command Handlers
//!
//! The small `CommandHandler` / `ContextProvider` structs the shell registers
//! on its `CommandRegistry` in `new_with_history_store`. Moved out of `mod.rs`
//! verbatim (Phase 2 task 2.2 file-size split); behaviour, names, and the
//! shell-intercept routing comments are unchanged.

use std::sync::{Arc, Mutex};

use ff_command::{CommandHandler, ContextProvider};
use ff_command::{CommandParams, CommandResult, ExecutionContext};

/// Handler for `file.open` -- sets `pending_open` via a shared channel.
/// The shell reads `pending_open` at the top of each frame.
///
/// The pending payload is `(path, owning_env)` (CR-CH-053 Task 19, Req 15.2):
/// the OPTIONAL `owning_env` param carries the Owning_Environment NAME captured
/// from the originating catalog/provider (the file system that owns the resource
/// being opened). When absent -- every host-path open today -- the opened tab
/// defaults to the host FS environment (`DEFAULT_OWNING_ENVIRONMENT`), so
/// existing opens are behaviour-preserving (Req 15.3).
pub(super) struct FileOpenHandler {
    pub(super) pending: super::state::PendingOpen,
}

impl CommandHandler for FileOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
        match params.get_string("path") {
            Some(path) if !path.is_empty() => {
                let owning_env = params
                    .get_string("owning_env")
                    .filter(|e| !e.is_empty())
                    .map(|e| e.to_string());
                *self.pending.lock().expect("pending lock") = Some((path.to_string(), owning_env));
                CommandResult::Ok
            }
            _ => CommandResult::Err(ff_command::CommandError::ExecutionFailed {
                id: "file.open".to_string(),
                description: "missing or empty 'path' parameter".to_string(),
            }),
        }
    }
}

/// Handler for `file.exit` -- sets a shared close flag.
pub(super) struct FileExitHandler {
    pub(super) should_close: Arc<std::sync::Mutex<bool>>,
}

impl CommandHandler for FileExitHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        *self.should_close.lock().expect("close lock") = true;
        CommandResult::Ok
    }
}

/// Handler for `menu.open` -- a marker registration so the id is dispatchable
/// and palette-visible (menu-workspace Requirement 11.6). The actual
/// menu-opening is performed by the shell, which intercepts `MENU` /
/// `menu.open` in `handle_command` before registry dispatch.
pub(super) struct MenuOpenHandler;

/// A [`ContextProvider`](ff_command::ContextProvider) backed by the shell's live
/// Cursor_Context snapshot (CR-CH-028, command-framework Requirement 12.4).
///
/// The shell refreshes the shared `snapshot` cell from live focus/selection at
/// each dispatch; this provider returns a fresh `ExecutionContext` carrying a
/// clone of that snapshot, so a command dispatched through the registry receives
/// the SAME package the string-path commands see (command parity, Requirement
/// 12.4). `current_context` takes `&self`, hence the shared cell.
pub(super) struct ShellContextProvider {
    pub(super) snapshot: Arc<Mutex<ff_command::CursorContext>>,
}

impl ContextProvider for ShellContextProvider {
    fn current_context(&self) -> ExecutionContext {
        let cc = self.snapshot.lock().map(|g| g.clone()).unwrap_or_default();
        ExecutionContext::builder().cursor_context(cc).build()
    }
}

impl CommandHandler for MenuOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        // Routing is handled by the shell intercept; nothing to do here.
        CommandResult::Ok
    }
}

/// Marker handler for `config.open` (CR-CH-025). Registration makes bare
/// `CONFIG` resolve as a built-in command through the resolution chain; the
/// actual opening of the flat config-key view is performed by the shell
/// intercept in `handle_command`.
pub(super) struct ConfigOpenHandler;

impl CommandHandler for ConfigOpenHandler {
    fn is_undoable(&self) -> bool {
        false
    }

    fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
        // Routing is handled by the shell intercept; nothing to do here.
        CommandResult::Ok
    }
}
