//! `CommandHandler` trait — defines command execution behaviour.
//!
//! Implementors define what happens when a command is invoked.

use crate::context::ExecutionContext;
use crate::params::CommandParams;
use crate::result::CommandResult;

/// The execution trait for a command. Implementors define the command's behaviour.
///
/// Commands implement this trait to provide synchronous execution logic.
/// The dispatch layer invokes `execute` after validating the command and
/// constructing the `ExecutionContext`.
pub trait CommandHandler: Send + Sync {
    /// Whether this command is undoable (produces an `UndoRecord`).
    ///
    /// Declared at registration time. If true, the handler is expected to
    /// return `CommandResult::OkUndoable` on success.
    fn is_undoable(&self) -> bool;

    /// Evaluates whether the command can currently execute given the context.
    ///
    /// Must complete within 1ms and produce no side effects.
    /// Returns true by default if not overridden.
    fn is_enabled(&self, _ctx: &ExecutionContext) -> bool {
        true
    }

    /// Evaluates whether the command should appear in menus and palettes.
    ///
    /// Must complete within 1ms and produce no side effects.
    /// Returns true by default if not overridden.
    fn is_visible(&self, _ctx: &ExecutionContext) -> bool {
        true
    }

    /// Execute the command synchronously.
    fn execute(&self, ctx: &ExecutionContext, params: &CommandParams) -> CommandResult;
}

/// Async variant for commands that perform I/O or long-running operations.
#[async_trait::async_trait]
pub trait AsyncCommandHandler: Send + Sync {
    /// Whether this command is undoable.
    fn is_undoable(&self) -> bool;

    /// Evaluates whether the command can currently execute.
    fn is_enabled(&self, _ctx: &ExecutionContext) -> bool {
        true
    }

    /// Evaluates whether the command should appear in menus and palettes.
    fn is_visible(&self, _ctx: &ExecutionContext) -> bool {
        true
    }

    /// Execute the command asynchronously.
    async fn execute(&self, ctx: &ExecutionContext, params: &CommandParams) -> CommandResult;
}

/// Internal enum to store either a sync or async handler.
pub(crate) enum CommandHandlerKind {
    /// A synchronous command handler.
    Sync(Box<dyn CommandHandler>),
    /// An asynchronous command handler.
    Async(Box<dyn AsyncCommandHandler>),
}

impl std::fmt::Debug for CommandHandlerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(_) => f.write_str("CommandHandlerKind::Sync(...)"),
            Self::Async(_) => f.write_str("CommandHandlerKind::Async(...)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamValue;

    struct TestHandler {
        undoable: bool,
    }

    impl CommandHandler for TestHandler {
        fn is_undoable(&self) -> bool {
            self.undoable
        }

        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            CommandResult::OkValue(ParamValue::String("executed".to_string()))
        }
    }

    // Validates: Requirement 1.3
    #[test]
    fn handler_trait_can_be_implemented() {
        let handler = TestHandler { undoable: false };
        let ctx = ExecutionContext::empty();
        let params = CommandParams::new();
        let result = handler.execute(&ctx, &params);
        assert!(result.is_ok());
    }

    // Validates: Requirement 3.4
    #[test]
    fn default_enabled_predicate_returns_true() {
        let handler = TestHandler { undoable: false };
        let ctx = ExecutionContext::empty();
        assert!(handler.is_enabled(&ctx));
    }

    // Validates: Requirement 3.5
    #[test]
    fn default_visible_predicate_returns_true() {
        let handler = TestHandler { undoable: false };
        let ctx = ExecutionContext::empty();
        assert!(handler.is_visible(&ctx));
    }

    // Validates: Requirement 4.1
    #[test]
    fn undoable_flag_is_accessible() {
        let handler = TestHandler { undoable: true };
        assert!(handler.is_undoable());

        let handler2 = TestHandler { undoable: false };
        assert!(!handler2.is_undoable());
    }
}
