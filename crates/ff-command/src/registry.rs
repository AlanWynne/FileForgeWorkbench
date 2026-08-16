//! `CommandRegistry` — thread-safe global registry of all registered commands.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::context::ExecutionContext;
use crate::error::CommandError;
use crate::handler::{AsyncCommandHandler, CommandHandler, CommandHandlerKind};
use crate::id::CommandId;
use crate::metadata::CommandMetadata;
use crate::params::CommandParams;
use crate::result::CommandResult;

/// A registered command entry: combines ID, metadata, and handler.
pub(crate) struct CommandEntry {
    #[allow(dead_code)]
    pub id: CommandId,
    pub metadata: CommandMetadata,
    pub handler: CommandHandlerKind,
}

/// The global, thread-safe command registry.
///
/// Stores commands indexed by `CommandId`. Supports registration, lookup,
/// deregistration, and discovery. All operations are thread-safe.
pub struct CommandRegistry {
    entries: RwLock<HashMap<CommandId, CommandEntry>>,
}

impl CommandRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a synchronous command with its metadata and handler.
    ///
    /// Returns `Err(DuplicateId)` if a command with the same ID already exists.
    pub fn register(
        &self,
        id: CommandId,
        metadata: CommandMetadata,
        handler: Box<dyn CommandHandler>,
    ) -> Result<(), CommandError> {
        let mut map = self.entries.write().expect("registry lock poisoned");
        if map.contains_key(&id) {
            return Err(CommandError::DuplicateId { id: id.to_string() });
        }
        map.insert(
            id.clone(),
            CommandEntry {
                id,
                metadata,
                handler: CommandHandlerKind::Sync(handler),
            },
        );
        Ok(())
    }

    /// Registers an asynchronous command.
    ///
    /// Returns `Err(DuplicateId)` if a command with the same ID already exists.
    pub fn register_async(
        &self,
        id: CommandId,
        metadata: CommandMetadata,
        handler: Box<dyn AsyncCommandHandler>,
    ) -> Result<(), CommandError> {
        let mut map = self.entries.write().expect("registry lock poisoned");
        if map.contains_key(&id) {
            return Err(CommandError::DuplicateId { id: id.to_string() });
        }
        map.insert(
            id.clone(),
            CommandEntry {
                id,
                metadata,
                handler: CommandHandlerKind::Async(handler),
            },
        );
        Ok(())
    }

    /// Deregisters a command by ID. Returns true if removed, false if not found.
    pub fn deregister(&self, id: &CommandId) -> bool {
        let mut map = self.entries.write().expect("registry lock poisoned");
        map.remove(id).is_some()
    }

    /// Looks up a command by ID string. Returns `None` if not found.
    pub fn contains(&self, id: &CommandId) -> bool {
        let map = self.entries.read().expect("registry lock poisoned");
        map.contains_key(id)
    }

    /// Returns the metadata for a command, if registered.
    pub fn metadata(&self, id: &CommandId) -> Option<CommandMetadata> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.get(id).map(|entry| entry.metadata.clone())
    }

    /// Lists all registered command IDs.
    pub fn list_all(&self) -> Vec<CommandId> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.keys().cloned().collect()
    }

    /// Lists commands whose ID starts with the given category prefix.
    ///
    /// Matches IDs where `id.has_prefix(prefix)` is true.
    pub fn list_by_category(&self, prefix: &str) -> Vec<CommandId> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.keys()
            .filter(|id| id.has_prefix(prefix))
            .cloned()
            .collect()
    }

    /// Returns the total number of registered commands.
    pub fn count(&self) -> usize {
        let map = self.entries.read().expect("registry lock poisoned");
        map.len()
    }

    /// Checks if a command is enabled in the given context.
    pub fn is_enabled(&self, id: &CommandId, ctx: &ExecutionContext) -> Option<bool> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.get(id).map(|entry| match &entry.handler {
            CommandHandlerKind::Sync(h) => h.is_enabled(ctx),
            CommandHandlerKind::Async(h) => h.is_enabled(ctx),
        })
    }

    /// Checks if a command is visible in the given context.
    pub fn is_visible(&self, id: &CommandId, ctx: &ExecutionContext) -> Option<bool> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.get(id).map(|entry| match &entry.handler {
            CommandHandlerKind::Sync(h) => h.is_visible(ctx),
            CommandHandlerKind::Async(h) => h.is_visible(ctx),
        })
    }

    /// Checks if a command is undoable.
    pub fn is_undoable(&self, id: &CommandId) -> Option<bool> {
        let map = self.entries.read().expect("registry lock poisoned");
        map.get(id).map(|entry| match &entry.handler {
            CommandHandlerKind::Sync(h) => h.is_undoable(),
            CommandHandlerKind::Async(h) => h.is_undoable(),
        })
    }

    /// Executes a command synchronously. Returns `Err(NotFound)` if unregistered.
    pub(crate) fn execute_sync(
        &self,
        id: &CommandId,
        ctx: &ExecutionContext,
        params: &CommandParams,
    ) -> CommandResult {
        let map = self.entries.read().expect("registry lock poisoned");
        match map.get(id) {
            Some(entry) => match &entry.handler {
                CommandHandlerKind::Sync(handler) => handler.execute(ctx, params),
                CommandHandlerKind::Async(_) => CommandResult::Err(CommandError::ExecutionFailed {
                    id: id.to_string(),
                    description: "async command cannot be executed synchronously".to_string(),
                }),
            },
            None => CommandResult::Err(CommandError::NotFound { id: id.to_string() }),
        }
    }

    /// Executes a command asynchronously.
    #[allow(clippy::await_holding_lock)]
    pub(crate) async fn execute_async(
        &self,
        id: &CommandId,
        ctx: &ExecutionContext,
        params: &CommandParams,
    ) -> CommandResult {
        // We need to extract the handler reference under the lock, then release.
        // Since AsyncCommandHandler requires &self, we hold the read lock during execution.
        let map = self.entries.read().expect("registry lock poisoned");
        match map.get(id) {
            Some(entry) => match &entry.handler {
                CommandHandlerKind::Sync(handler) => handler.execute(ctx, params),
                CommandHandlerKind::Async(handler) => handler.execute(ctx, params).await,
            },
            None => CommandResult::Err(CommandError::NotFound { id: id.to_string() }),
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopHandler;

    impl CommandHandler for NoopHandler {
        fn is_undoable(&self) -> bool {
            false
        }

        fn execute(&self, _ctx: &ExecutionContext, _params: &CommandParams) -> CommandResult {
            CommandResult::Ok
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

    fn meta(name: &str, cat: &str) -> CommandMetadata {
        CommandMetadata::builder(name, "test command")
            .category(cat)
            .build()
    }

    // Validates: Requirement 1.1, 1.3
    #[test]
    fn register_and_lookup_command() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("file.save").unwrap();
        registry
            .register(id.clone(), meta("Save", "file"), Box::new(NoopHandler))
            .unwrap();

        assert!(registry.contains(&id));
        assert_eq!(registry.count(), 1);
    }

    // Validates: Requirement 1.2
    #[test]
    fn duplicate_registration_returns_error() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("file.save").unwrap();
        registry
            .register(id.clone(), meta("Save", "file"), Box::new(NoopHandler))
            .unwrap();

        let result = registry.register(id, meta("Save2", "file"), Box::new(NoopHandler));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CommandError::DuplicateId { .. }
        ));
    }

    // Validates: Requirement 1.5
    #[test]
    fn lookup_missing_id_returns_none() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("nonexistent.cmd").unwrap();
        assert!(!registry.contains(&id));
        assert!(registry.metadata(&id).is_none());
    }

    // Validates: Requirement 1.7
    #[test]
    fn deregister_removes_command() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("plugin.test").unwrap();
        registry
            .register(id.clone(), meta("Test", "plugin"), Box::new(NoopHandler))
            .unwrap();

        assert!(registry.deregister(&id));
        assert!(!registry.contains(&id));
        assert_eq!(registry.count(), 0);
    }

    // Validates: Requirement 1.7
    #[test]
    fn deregister_nonexistent_returns_false() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("nonexistent.cmd").unwrap();
        assert!(!registry.deregister(&id));
    }

    // Validates: Requirement 1.6
    #[test]
    fn list_all_returns_all_registered_ids() {
        let registry = CommandRegistry::new();
        let id1 = CommandId::new("file.save").unwrap();
        let id2 = CommandId::new("edit.copy").unwrap();
        registry
            .register(id1.clone(), meta("Save", "file"), Box::new(NoopHandler))
            .unwrap();
        registry
            .register(id2.clone(), meta("Copy", "edit"), Box::new(NoopHandler))
            .unwrap();

        let all = registry.list_all();
        assert_eq!(all.len(), 2);
        assert!(all.contains(&id1));
        assert!(all.contains(&id2));
    }

    // Validates: Requirement 1.6
    #[test]
    fn list_by_category_filters_by_prefix() {
        let registry = CommandRegistry::new();
        let id1 = CommandId::new("file.save").unwrap();
        let id2 = CommandId::new("file.open").unwrap();
        let id3 = CommandId::new("edit.copy").unwrap();
        registry
            .register(id1.clone(), meta("Save", "file"), Box::new(NoopHandler))
            .unwrap();
        registry
            .register(id2.clone(), meta("Open", "file"), Box::new(NoopHandler))
            .unwrap();
        registry
            .register(id3.clone(), meta("Copy", "edit"), Box::new(NoopHandler))
            .unwrap();

        let file_cmds = registry.list_by_category("file");
        assert_eq!(file_cmds.len(), 2);
        assert!(file_cmds.contains(&id1));
        assert!(file_cmds.contains(&id2));
        assert!(!file_cmds.contains(&id3));
    }

    // Validates: Requirement 3.6
    #[test]
    fn metadata_query_returns_fields() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("file.save").unwrap();
        registry
            .register(id.clone(), meta("Save File", "file"), Box::new(NoopHandler))
            .unwrap();

        let m = registry.metadata(&id).unwrap();
        assert_eq!(m.display_name, "Save File");
        assert_eq!(m.category, "file");
    }

    // Validates: Requirement 3.4
    #[test]
    fn is_enabled_returns_handler_predicate_result() {
        let registry = CommandRegistry::new();
        let id = CommandId::new("edit.paste").unwrap();
        registry
            .register(id.clone(), meta("Paste", "edit"), Box::new(DisabledHandler))
            .unwrap();

        let ctx = ExecutionContext::empty();
        assert_eq!(registry.is_enabled(&id, &ctx), Some(false));
    }
}
