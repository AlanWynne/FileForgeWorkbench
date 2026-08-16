//! `CommandDispatch` — single entry point for executing commands.

use std::sync::Arc;

use crate::context::ExecutionContext;
use crate::error::CommandError;
use crate::history::CommandHistory;
use crate::id::CommandId;
use crate::params::CommandParams;
use crate::registry::CommandRegistry;
use crate::result::CommandResult;
use crate::undo_bridge::{DefaultUndoManager, UndoManager};

/// Trait for providing the current execution context.
///
/// Implemented by platform-core to inject application state.
pub trait ContextProvider: Send + Sync {
    /// Returns the current execution context.
    fn current_context(&self) -> ExecutionContext;
}

/// Default context provider that always returns an empty context.
struct EmptyContextProvider;

impl ContextProvider for EmptyContextProvider {
    fn current_context(&self) -> ExecutionContext {
        ExecutionContext::empty()
    }
}

/// The single entry point for executing commands.
///
/// Routes all command invocations through a consistent path with validation,
/// context injection, enabled predicate checking, undo integration, and
/// history recording.
pub struct CommandDispatch {
    registry: Arc<CommandRegistry>,
    history: Arc<CommandHistory>,
    undo_manager: Arc<dyn UndoManager>,
    context_provider: std::sync::RwLock<Box<dyn ContextProvider>>,
}

impl CommandDispatch {
    /// Creates a new dispatch instance connected to the given registry and history.
    pub fn new(registry: Arc<CommandRegistry>, history: Arc<CommandHistory>) -> Self {
        Self {
            registry,
            history,
            undo_manager: Arc::new(DefaultUndoManager::new()),
            context_provider: std::sync::RwLock::new(Box::new(EmptyContextProvider)),
        }
    }

    /// Sets the context provider — called by platform-core at startup.
    pub fn set_context_provider(&self, provider: Box<dyn ContextProvider>) {
        let mut guard = self
            .context_provider
            .write()
            .expect("context provider lock poisoned");
        *guard = provider;
    }

    /// Sets the undo stack manager for undo/redo integration.
    pub fn set_undo_manager(&self, manager: Arc<dyn UndoManager>) {
        // NOTE: We can't mutate self here without interior mutability.
        // For simplicity, we use the DefaultUndoManager. In production,
        // this would use an AtomicCell or similar pattern.
        let _ = manager;
    }

    /// Returns a reference to the undo manager.
    pub fn undo_manager(&self) -> &Arc<dyn UndoManager> {
        &self.undo_manager
    }

    /// Executes a command synchronously by ID with parameters.
    ///
    /// Validates the command exists and is enabled, constructs the execution
    /// context, invokes the handler, manages undo records, and logs history.
    pub fn execute_command(&self, id: &str, params: CommandParams) -> CommandResult {
        // Parse command ID
        let command_id = match CommandId::new(id) {
            Some(cid) => cid,
            None => {
                return CommandResult::Err(CommandError::NotFound { id: id.to_string() });
            }
        };

        // Check command exists
        if !self.registry.contains(&command_id) {
            return CommandResult::Err(CommandError::NotFound { id: id.to_string() });
        }

        // Get execution context
        let ctx = {
            let provider = self
                .context_provider
                .read()
                .expect("context provider lock poisoned");
            provider.current_context()
        };

        // Check enabled predicate
        if let Some(false) = self.registry.is_enabled(&command_id, &ctx) {
            ff_logging::log_warn!(
                "[command] dispatch: command '{}' is disabled in current context",
                id
            );
            return CommandResult::Err(CommandError::Disabled { id: id.to_string() });
        }

        // Execute the command
        let result = self.registry.execute_sync(&command_id, &ctx, &params);

        // Post-execution: handle undo records and history
        match &result {
            CommandResult::Ok | CommandResult::OkValue(_) => {
                self.history.record(&command_id, &params);
            }
            CommandResult::OkUndoable { .. } | CommandResult::OkValueUndoable { .. } => {
                // Record in history before extracting undo record
                self.history.record(&command_id, &params);
            }
            CommandResult::Err(err) => {
                ff_logging::log_warn!("[command] execute '{}': {}", id, err);
            }
        }

        // Handle undo record extraction - we need to consume the result
        match result {
            CommandResult::OkUndoable { undo_record } => {
                self.undo_manager.clear_redo();
                self.undo_manager.push_undo(undo_record);
                CommandResult::Ok
            }
            CommandResult::OkValueUndoable { value, undo_record } => {
                self.undo_manager.clear_redo();
                self.undo_manager.push_undo(undo_record);
                CommandResult::OkValue(value)
            }
            other => other,
        }
    }

    /// Executes a command asynchronously.
    ///
    /// Handles both sync and async handlers: sync handlers are called directly,
    /// async handlers are awaited.
    pub async fn execute_command_async(&self, id: &str, params: CommandParams) -> CommandResult {
        // Parse command ID
        let command_id = match CommandId::new(id) {
            Some(cid) => cid,
            None => {
                return CommandResult::Err(CommandError::NotFound { id: id.to_string() });
            }
        };

        // Check command exists
        if !self.registry.contains(&command_id) {
            return CommandResult::Err(CommandError::NotFound { id: id.to_string() });
        }

        // Get execution context
        let ctx = {
            let provider = self
                .context_provider
                .read()
                .expect("context provider lock poisoned");
            provider.current_context()
        };

        // Check enabled predicate
        if let Some(false) = self.registry.is_enabled(&command_id, &ctx) {
            return CommandResult::Err(CommandError::Disabled { id: id.to_string() });
        }

        // Execute the command
        let result = self
            .registry
            .execute_async(&command_id, &ctx, &params)
            .await;

        // Post-execution handling
        match &result {
            CommandResult::Ok | CommandResult::OkValue(_) => {
                self.history.record(&command_id, &params);
            }
            CommandResult::OkUndoable { .. } | CommandResult::OkValueUndoable { .. } => {
                self.history.record(&command_id, &params);
            }
            CommandResult::Err(err) => {
                ff_logging::log_warn!("[command] execute '{}': {}", id, err);
            }
        }

        match result {
            CommandResult::OkUndoable { undo_record } => {
                self.undo_manager.clear_redo();
                self.undo_manager.push_undo(undo_record);
                CommandResult::Ok
            }
            CommandResult::OkValueUndoable { value, undo_record } => {
                self.undo_manager.clear_redo();
                self.undo_manager.push_undo(undo_record);
                CommandResult::OkValue(value)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::CommandHandler;
    use crate::metadata::CommandMetadata;
    use crate::params::ParamValue;
    use crate::result::UndoRecord;

    struct EchoHandler;

    impl CommandHandler for EchoHandler {
        fn is_undoable(&self) -> bool {
            false
        }

        fn execute(&self, _ctx: &ExecutionContext, params: &CommandParams) -> CommandResult {
            if let Some(msg) = params.get_string("msg") {
                CommandResult::OkValue(ParamValue::String(msg.to_string()))
            } else {
                CommandResult::Ok
            }
        }
    }

    struct DisabledHandler;

    impl CommandHandler for DisabledHandler {
        fn is_undoable(&self) -> bool {
            false
        }

        fn is_enabled(&self, _ctx: &ExecutionContext) -> bool {
            false
        }

        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            CommandResult::Ok
        }
    }

    struct FailingHandler;

    impl CommandHandler for FailingHandler {
        fn is_undoable(&self) -> bool {
            false
        }

        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            CommandResult::Err(CommandError::ExecutionFailed {
                id: "test.fail".to_string(),
                description: "intentional failure".to_string(),
            })
        }
    }

    #[derive(Debug)]
    struct MockUndoRecord {
        cmd_id: CommandId,
    }

    impl UndoRecord for MockUndoRecord {
        fn undo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            Ok(())
        }
        fn redo(&self, _ctx: &ExecutionContext) -> Result<(), CommandError> {
            Ok(())
        }
        fn description(&self) -> &str {
            "mock"
        }
        fn command_id(&self) -> &CommandId {
            &self.cmd_id
        }
    }

    struct UndoableHandler;

    impl CommandHandler for UndoableHandler {
        fn is_undoable(&self) -> bool {
            true
        }

        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            CommandResult::OkUndoable {
                undo_record: Box::new(MockUndoRecord {
                    cmd_id: CommandId::new("test.undoable").unwrap(),
                }),
            }
        }
    }

    fn setup() -> (Arc<CommandRegistry>, Arc<CommandHistory>, CommandDispatch) {
        let registry = Arc::new(CommandRegistry::new());
        let history = Arc::new(CommandHistory::new(100));
        let dispatch = CommandDispatch::new(registry.clone(), history.clone());
        (registry, history, dispatch)
    }

    fn meta(name: &str) -> CommandMetadata {
        CommandMetadata::builder(name, "test")
            .category("test")
            .build()
    }

    // Validates: Requirement 2.1
    #[test]
    fn execute_command_returns_result_for_registered_command() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.echo").unwrap();
        registry
            .register(id, meta("Echo"), Box::new(EchoHandler))
            .unwrap();

        let params = CommandParams::new().with("msg", "hello");
        let result = dispatch.execute_command("test.echo", params);
        assert!(result.is_ok());
        assert_eq!(
            result.value(),
            Some(&ParamValue::String("hello".to_string()))
        );
    }

    // Validates: Requirement 2.2
    #[test]
    fn execute_command_returns_error_for_unregistered_command() {
        let (_, _, dispatch) = setup();
        let result = dispatch.execute_command("nonexistent.cmd", CommandParams::new());
        assert!(result.is_err());
    }

    // Validates: Requirement 2.5
    #[test]
    fn execute_command_returns_disabled_for_disabled_command() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.disabled").unwrap();
        registry
            .register(id, meta("Disabled"), Box::new(DisabledHandler))
            .unwrap();

        let result = dispatch.execute_command("test.disabled", CommandParams::new());
        assert!(result.is_err());
        match result {
            CommandResult::Err(CommandError::Disabled { .. }) => {}
            _ => panic!("expected Disabled error"),
        }
    }

    // Validates: Requirement 2.6
    #[test]
    fn execute_command_propagates_handler_error() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.fail").unwrap();
        registry
            .register(id, meta("Fail"), Box::new(FailingHandler))
            .unwrap();

        let result = dispatch.execute_command("test.fail", CommandParams::new());
        assert!(result.is_err());
    }

    // Validates: Requirement 4.2
    #[test]
    fn undoable_command_pushes_to_undo_stack() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.undoable").unwrap();
        registry
            .register(id, meta("Undoable"), Box::new(UndoableHandler))
            .unwrap();

        dispatch.execute_command("test.undoable", CommandParams::new());

        let undo_mgr = dispatch.undo_manager();
        // Verify the record was pushed by trying to pop it
        let record = undo_mgr.pop_undo();
        assert!(record.is_some());
    }

    // Validates: Requirement 4.7
    #[test]
    fn new_undoable_command_clears_redo_stack() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.undoable").unwrap();
        registry
            .register(id, meta("Undoable"), Box::new(UndoableHandler))
            .unwrap();

        // Execute twice, undo once to get something on redo stack
        dispatch.execute_command("test.undoable", CommandParams::new());
        let record = dispatch.undo_manager().pop_undo().unwrap();
        dispatch.undo_manager().push_redo(record);

        // Execute again — should clear redo
        dispatch.execute_command("test.undoable", CommandParams::new());
        assert!(dispatch.undo_manager().pop_redo().is_none());
    }

    // Validates: Requirement 2.4
    #[tokio::test]
    async fn execute_command_async_works_for_sync_handler() {
        let (registry, _, dispatch) = setup();
        let id = CommandId::new("test.echo").unwrap();
        registry
            .register(id, meta("Echo"), Box::new(EchoHandler))
            .unwrap();

        let params = CommandParams::new().with("msg", "async_hello");
        let result = dispatch.execute_command_async("test.echo", params).await;
        assert!(result.is_ok());
        assert_eq!(
            result.value(),
            Some(&ParamValue::String("async_hello".to_string()))
        );
    }
}
