//! Per-buffer Lua state management.
//!
//! Maintains isolated Lua tables for each open buffer, swapping them
//! automatically on buffer switch.
//! Addresses: Requirement 4 (all criteria)

use std::collections::HashMap;

use mlua::prelude::*;

use crate::error::LuaEngineError;

/// Opaque buffer identifier (matches the document model's buffer concept).
pub type BufferId = u64;

/// Manages per-buffer Lua table storage with automatic swap on buffer switch.
///
/// Addresses: Requirement 4 (all criteria)
#[derive(Debug)]
pub struct BufferStateManager {
    /// Map from buffer ID to Lua registry key of the buffer's table.
    buffer_tables: HashMap<BufferId, LuaRegistryKey>,
    /// Currently active buffer ID (None during startup).
    active_buffer: Option<BufferId>,
}

impl BufferStateManager {
    /// Creates a new buffer state manager with no active buffer.
    pub fn new() -> Self {
        Self {
            buffer_tables: HashMap::new(),
            active_buffer: None,
        }
    }

    /// Returns the currently active buffer ID, or None during startup.
    pub fn active_buffer(&self) -> Option<BufferId> {
        self.active_buffer
    }

    /// Returns the number of tracked buffers.
    pub fn buffer_count(&self) -> usize {
        self.buffer_tables.len()
    }

    /// Create a new empty buffer state for a newly opened buffer.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn create_buffer_state(
        &mut self,
        lua: &Lua,
        buffer_id: BufferId,
    ) -> Result<(), LuaEngineError> {
        let table = lua.create_table().map_err(|e| LuaEngineError::InitFailed {
            reason: format!("failed to create buffer table: {e}"),
        })?;
        let key = lua
            .create_registry_value(table)
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to store buffer table in registry: {e}"),
            })?;
        self.buffer_tables.insert(buffer_id, key);
        Ok(())
    }

    /// Switch to a different buffer: save current `buffer` global, restore target.
    ///
    /// Addresses: Requirement 4 AC 3, AC 7
    pub fn switch_buffer(
        &mut self,
        lua: &Lua,
        new_buffer_id: BufferId,
    ) -> Result<(), LuaEngineError> {
        // Set the new buffer's table as the global `buffer`
        if let Some(key) = self.buffer_tables.get(&new_buffer_id) {
            let table: LuaTable =
                lua.registry_value(key)
                    .map_err(|e| LuaEngineError::InitFailed {
                        reason: format!("failed to retrieve buffer table: {e}"),
                    })?;
            lua.globals()
                .set("buffer", table)
                .map_err(|e| LuaEngineError::InitFailed {
                    reason: format!("failed to set buffer global: {e}"),
                })?;
        } else {
            // No state for this buffer — set to nil
            lua.globals()
                .set("buffer", LuaValue::Nil)
                .map_err(|e| LuaEngineError::InitFailed {
                    reason: format!("failed to clear buffer global: {e}"),
                })?;
        }

        self.active_buffer = Some(new_buffer_id);
        Ok(())
    }

    /// Discard state for a closed buffer.
    ///
    /// Addresses: Requirement 4 AC 4
    pub fn remove_buffer_state(&mut self, lua: &Lua, buffer_id: BufferId) {
        if let Some(key) = self.buffer_tables.remove(&buffer_id) {
            let _ = lua.remove_registry_value(key);
        }
        if self.active_buffer == Some(buffer_id) {
            self.active_buffer = None;
        }
    }

    /// Set the `buffer` global to nil (used during startup before any buffer is active).
    ///
    /// Addresses: Requirement 4 AC 6
    pub fn clear_active(&self, lua: &Lua) -> Result<(), LuaEngineError> {
        lua.globals()
            .set("buffer", LuaValue::Nil)
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to clear buffer global: {e}"),
            })
    }
}

impl Default for BufferStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_lua() -> Lua {
        Lua::new()
    }

    // Validates: Requirement 4.2
    #[test]
    fn create_buffer_state_creates_empty_table() {
        let lua = create_lua();
        let mut mgr = BufferStateManager::new();
        mgr.create_buffer_state(&lua, 1).unwrap();
        assert_eq!(mgr.buffer_count(), 1);
    }

    // Validates: Requirement 4.3
    #[test]
    fn switch_buffer_sets_correct_global() {
        let lua = create_lua();
        let mut mgr = BufferStateManager::new();

        mgr.create_buffer_state(&lua, 1).unwrap();
        mgr.create_buffer_state(&lua, 2).unwrap();

        // Switch to buffer 1, set a key
        mgr.switch_buffer(&lua, 1).unwrap();
        lua.load(r#"buffer.test_key = "buffer1_value""#)
            .exec()
            .unwrap();

        // Switch to buffer 2, set a different key
        mgr.switch_buffer(&lua, 2).unwrap();
        lua.load(r#"buffer.test_key = "buffer2_value""#)
            .exec()
            .unwrap();

        // Switch back to buffer 1, verify isolation
        mgr.switch_buffer(&lua, 1).unwrap();
        let result: String = lua.load(r#"return buffer.test_key"#).eval().unwrap();
        assert_eq!(result, "buffer1_value");
    }

    // Validates: Requirement 4.1
    #[test]
    fn buffer_state_is_isolated_between_buffers() {
        let lua = create_lua();
        let mut mgr = BufferStateManager::new();

        mgr.create_buffer_state(&lua, 1).unwrap();
        mgr.create_buffer_state(&lua, 2).unwrap();

        // Write to buffer 1
        mgr.switch_buffer(&lua, 1).unwrap();
        lua.load(r#"buffer.secret = 42"#).exec().unwrap();

        // Buffer 2 should not see it
        mgr.switch_buffer(&lua, 2).unwrap();
        let result: LuaValue = lua.load(r#"return buffer.secret"#).eval().unwrap();
        assert_eq!(result, LuaValue::Nil);
    }

    // Validates: Requirement 4.4
    #[test]
    fn remove_buffer_state_discards_table() {
        let lua = create_lua();
        let mut mgr = BufferStateManager::new();

        mgr.create_buffer_state(&lua, 1).unwrap();
        assert_eq!(mgr.buffer_count(), 1);

        mgr.remove_buffer_state(&lua, 1);
        assert_eq!(mgr.buffer_count(), 0);
    }

    // Validates: Requirement 4.6
    #[test]
    fn buffer_global_is_nil_during_startup() {
        let lua = create_lua();
        let mgr = BufferStateManager::new();

        mgr.clear_active(&lua).unwrap();
        let result: LuaValue = lua.load(r#"return buffer"#).eval().unwrap();
        assert_eq!(result, LuaValue::Nil);
    }

    // Validates: Requirement 4.5
    #[test]
    fn scripts_can_freely_write_to_buffer_table() {
        let lua = create_lua();
        let mut mgr = BufferStateManager::new();

        mgr.create_buffer_state(&lua, 1).unwrap();
        mgr.switch_buffer(&lua, 1).unwrap();

        // Write various types
        lua.load(
            r#"
            buffer.counter = 0
            buffer.name = "test"
            buffer.flags = { a = true, b = false }
        "#,
        )
        .exec()
        .unwrap();

        let counter: i64 = lua.load(r#"return buffer.counter"#).eval().unwrap();
        assert_eq!(counter, 0);

        let name: String = lua.load(r#"return buffer.name"#).eval().unwrap();
        assert_eq!(name, "test");
    }
}
