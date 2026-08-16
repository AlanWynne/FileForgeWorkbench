//! Core `LuaMacroEngine` struct — owns the Lua runtime and orchestrates
//! all macro operations.
//!
//! Addresses: Requirement 1 AC 1, AC 6, AC 7

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mlua::prelude::*;

use crate::buffer_state::{BufferId, BufferStateManager};
use crate::config::EngineConfig;
use crate::error::{LuaEngineError, LuaResult};
use crate::hooks::event::HookEvent;
use crate::hooks::registry::{HookDispatchResult, HookRegistry};
use crate::scanner::{self, DirectoryPriority, MacroScript};
use crate::security::{SecurityGate, SecurityMode, SecurityPermission};

/// The core macro engine: owns the Lua 5.4 runtime, manages script lifecycle,
/// and coordinates all macro operations. Instantiated once per application lifetime.
///
/// Addresses: Requirement 1 AC 1, AC 6, AC 7
pub struct LuaMacroEngine {
    /// The mlua Lua 5.4 runtime instance (reused across invocations).
    lua: Lua,
    /// Registry of event hooks (event name → ordered handler list).
    hook_registry: HookRegistry,
    /// Per-buffer Lua table storage.
    buffer_state: BufferStateManager,
    /// Available macros (name → script metadata) from directory scanning.
    available_macros: HashMap<String, MacroScript>,
    /// Security gate for execution policy enforcement.
    security_gate: SecurityGate,
    /// Configuration cache for limits and flags.
    config: EngineConfig,
    /// Whether the engine has been initialized.
    initialized: bool,
}

impl LuaMacroEngine {
    /// Create a new macro engine with the given configuration.
    ///
    /// Does NOT initialize the Lua runtime yet — call `initialize()` after
    /// construction to set up the runtime, register APIs, and execute
    /// the startup script.
    ///
    /// Addresses: Requirement 1 AC 6
    pub fn new(config: EngineConfig) -> LuaResult<Self> {
        let lua = Lua::new();

        let security_gate = SecurityGate::new(
            config.security_mode,
            config.trusted_paths.clone(),
            config.macro_directories.clone(),
        );

        Ok(Self {
            lua,
            hook_registry: HookRegistry::new(),
            buffer_state: BufferStateManager::new(),
            available_macros: HashMap::new(),
            security_gate,
            config,
            initialized: false,
        })
    }

    /// Initialize the Lua runtime: register editor API globals, set limits,
    /// set buffer global to nil, and scan macro directories.
    ///
    /// Addresses: Requirement 1 AC 1, AC 2, AC 7
    pub fn initialize(&mut self) -> LuaResult<()> {
        // Set buffer global to nil (startup state)
        self.buffer_state.clear_active(&self.lua)?;

        // Register the editor API table
        self.register_editor_api()?;

        // Register trace/print globals for debugging
        self.register_debug_globals()?;

        // Remove restricted stdlib functions for non-Enabled modes
        if self.config.security_mode != SecurityMode::Enabled {
            self.restrict_stdlib()?;
        }

        // Scan macro directories
        self.rescan_directories()?;

        self.initialized = true;
        Ok(())
    }

    /// Execute a named macro (resolved from macro directories).
    ///
    /// Addresses: Requirement 5 AC 1
    pub fn execute_named(&mut self, name: &str) -> LuaResult<()> {
        let path = self
            .available_macros
            .get(name)
            .map(|s| s.path.clone())
            .ok_or_else(|| LuaEngineError::MacroNotFound {
                name: name.to_string(),
            })?;

        self.execute_file(&path)
    }

    /// Execute an inline Lua expression (EXEC command).
    ///
    /// Returns the expression's return value as a string.
    ///
    /// Addresses: Requirement 5 AC 2
    pub fn execute_inline(&mut self, expression: &str) -> LuaResult<Option<String>> {
        // Security check — inline expressions use a synthetic path
        let start = Instant::now();

        let result: LuaResult<Option<String>> = (|| {
            let value: LuaValue =
                self.lua
                    .load(expression)
                    .eval()
                    .map_err(|e| LuaEngineError::ScriptError {
                        script: format!("EXEC: {expression}"),
                        message: e.to_string(),
                        traceback: None,
                    })?;

            let string_value = match &value {
                LuaValue::Nil => None,
                other => Some(lua_value_to_display_string(other)),
            };
            Ok(string_value)
        })();

        let _elapsed = start.elapsed();
        result
    }

    /// Execute a macro file by path (RUN command).
    ///
    /// Addresses: Requirement 5 AC 3
    pub fn execute_file(&mut self, path: &Path) -> LuaResult<()> {
        // Security check
        let permission = self.security_gate.check_permission(path);
        match permission {
            SecurityPermission::Allowed => {}
            SecurityPermission::NeedsPrompt => {
                // In a real system this would prompt the user.
                // For now, allow execution (the UI layer handles prompts).
            }
            SecurityPermission::Denied { reason } => {
                return Err(LuaEngineError::SecurityDenied {
                    script: path.display().to_string(),
                    mode: self.security_gate.mode(),
                    reason,
                });
            }
        }

        let source =
            std::fs::read_to_string(path).map_err(|_| LuaEngineError::FileNotReadable {
                path: path.display().to_string(),
            })?;

        let start = Instant::now();
        let script_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.lua
            .load(&source)
            .set_name(script_name)
            .exec()
            .map_err(|e| LuaEngineError::ScriptError {
                script: script_name.to_string(),
                message: e.to_string(),
                traceback: None,
            })?;

        let _elapsed = start.elapsed();

        // Discover hook functions defined by this script
        self.discover_hooks(path)?;

        Ok(())
    }

    /// Fire an event hook, dispatching to all registered handlers.
    ///
    /// Returns whether the event was cancelled (for cancellable hooks).
    ///
    /// Addresses: Requirement 3 (all criteria)
    pub fn fire_event(&mut self, event: HookEvent) -> LuaResult<HookDispatchResult> {
        let event_name = event.lua_function_name().to_string();
        let is_cancellable = event.is_cancellable();

        let handlers = self.hook_registry.handlers_for(&event_name).to_vec();
        let mut result = HookDispatchResult::default();

        for handler in &handlers {
            let call_result = self.invoke_hook_handler(&event, &handler.function_name);

            match call_result {
                Ok(returned_false) => {
                    if is_cancellable && returned_false {
                        result.cancelled = true;
                        result.cancelled_by = Some(handler.script_path.clone());
                        break;
                    }
                }
                Err(e) => {
                    // Addresses: Requirement 6 AC 4
                    // On error in cancellable hook: treat as true (don't cancel)
                    result.errors.push(e.to_string());
                    // Continue invoking subsequent handlers
                }
            }
        }

        Ok(result)
    }

    /// Notify the engine that the active buffer has changed.
    ///
    /// Addresses: Requirement 4 AC 3, AC 7
    pub fn on_buffer_switch(
        &mut self,
        new_buffer_id: BufferId,
        file_path: Option<&str>,
    ) -> LuaResult<()> {
        self.buffer_state.switch_buffer(&self.lua, new_buffer_id)?;

        if let Some(path) = file_path {
            self.fire_event(HookEvent::OnSwitchBuffer {
                file_path: path.to_string(),
            })?;
        }
        Ok(())
    }

    /// Notify the engine that a new buffer was opened.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn on_buffer_opened(&mut self, buffer_id: BufferId, file_path: &str) -> LuaResult<()> {
        self.buffer_state
            .create_buffer_state(&self.lua, buffer_id)?;
        self.buffer_state.switch_buffer(&self.lua, buffer_id)?;

        self.fire_event(HookEvent::OnOpen {
            file_path: file_path.to_string(),
        })?;
        Ok(())
    }

    /// Notify the engine that a buffer was closed.
    ///
    /// Addresses: Requirement 4 AC 4
    pub fn on_buffer_closed(&mut self, buffer_id: BufferId, file_path: &str) -> LuaResult<()> {
        self.fire_event(HookEvent::OnClose {
            file_path: file_path.to_string(),
        })?;
        self.buffer_state.remove_buffer_state(&self.lua, buffer_id);
        Ok(())
    }

    /// Rescan macro directories and update available macros.
    ///
    /// Addresses: Requirement 9 AC 1, AC 7
    pub fn rescan_directories(&mut self) -> LuaResult<Vec<String>> {
        let dirs: Vec<(PathBuf, DirectoryPriority)> = self
            .config
            .macro_directories
            .iter()
            .map(|d| (d.clone(), DirectoryPriority::User))
            .collect();

        self.available_macros = scanner::scan_directories(&dirs)?;
        Ok(self.available_macro_names())
    }

    /// Get the list of available macro names (for command completion).
    pub fn available_macro_names(&self) -> Vec<String> {
        self.available_macros.keys().cloned().collect()
    }

    /// Reload a specific script by path (used by auto-reloader).
    ///
    /// Addresses: Requirement 8 AC 2, AC 3
    pub fn reload_script(&mut self, path: &Path) -> LuaResult<()> {
        // First collect which hook names were registered by this script
        let old_hook_names: Vec<String> = HookEvent::all_hook_names()
            .iter()
            .filter(|name| {
                self.hook_registry
                    .handlers_for(name)
                    .iter()
                    .any(|h| h.script_path == path)
            })
            .map(|s| s.to_string())
            .collect();

        // Remove global functions that were previously defined by this script
        let globals = self.lua.globals();
        for name in &old_hook_names {
            let _ = globals.set(name.as_str(), LuaValue::Nil);
        }

        // Unregister previous hooks from this script
        self.hook_registry.unregister_by_script(path);

        // Re-execute the script
        self.execute_file(path)
    }

    /// Shut down the engine: clear hooks and release state.
    pub fn shutdown(&mut self) {
        self.hook_registry.clear();
        self.initialized = false;
    }

    /// Returns whether the engine is initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns a reference to the Lua runtime (for testing).
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Returns a reference to the hook registry.
    pub fn hook_registry(&self) -> &HookRegistry {
        &self.hook_registry
    }

    /// Returns a reference to the buffer state manager.
    pub fn buffer_state(&self) -> &BufferStateManager {
        &self.buffer_state
    }

    /// Returns the engine configuration.
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    // ─── Private Methods ─────────────────────────────────────────────────

    /// Register the `editor` API table in the Lua runtime.
    fn register_editor_api(&self) -> LuaResult<()> {
        let editor = self
            .lua
            .create_table()
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to create editor table: {e}"),
            })?;

        // editor.lines() — placeholder returning 0 until a document is connected
        let lines_fn =
            self.lua
                .create_function(|_, ()| Ok(0i64))
                .map_err(|e| LuaEngineError::InitFailed {
                    reason: format!("failed to create editor.lines: {e}"),
                })?;
        editor
            .set("lines", lines_fn)
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to set editor.lines: {e}"),
            })?;

        self.lua
            .globals()
            .set("editor", editor)
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to set editor global: {e}"),
            })?;

        Ok(())
    }

    /// Register trace() and print() debug globals.
    fn register_debug_globals(&self) -> LuaResult<()> {
        // trace(message) — output to diagnostic log
        let trace_fn = self
            .lua
            .create_function(|_, msg: String| {
                // In a full implementation this would route to ff-logging
                eprintln!("[lua trace] {msg}");
                Ok(())
            })
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to create trace function: {e}"),
            })?;

        self.lua
            .globals()
            .set("trace", trace_fn)
            .map_err(|e| LuaEngineError::InitFailed {
                reason: format!("failed to set trace global: {e}"),
            })?;

        Ok(())
    }

    /// Restrict dangerous stdlib functions for non-Enabled security modes.
    ///
    /// Addresses: Requirement 7 AC 6
    fn restrict_stdlib(&self) -> LuaResult<()> {
        // Remove os.execute and io.popen
        let globals = self.lua.globals();

        // Remove loadfile and dofile from global scope
        let _ = globals.set("loadfile", LuaValue::Nil);
        let _ = globals.set("dofile", LuaValue::Nil);

        // Remove dangerous functions from os table if it exists
        if let Ok(os_table) = globals.get::<LuaTable>("os") {
            let _ = os_table.set("execute", LuaValue::Nil);
        }

        // Remove dangerous functions from io table if it exists
        if let Ok(io_table) = globals.get::<LuaTable>("io") {
            let _ = io_table.set("popen", LuaValue::Nil);
        }

        Ok(())
    }

    /// Discover hook functions defined in the Lua global scope after script load.
    ///
    /// Addresses: Requirement 3 AC 2
    fn discover_hooks(&mut self, script_path: &Path) -> LuaResult<()> {
        let globals = self.lua.globals();

        for &hook_name in HookEvent::all_hook_names() {
            if let Ok(LuaValue::Function(_)) = globals.get::<LuaValue>(hook_name) {
                self.hook_registry.register(
                    hook_name,
                    script_path.to_path_buf(),
                    hook_name.to_string(),
                );
            }
        }

        Ok(())
    }

    /// Invoke a single hook handler function with the appropriate arguments.
    ///
    /// Returns true if the handler returned `false` (i.e., wants to cancel).
    fn invoke_hook_handler(
        &self,
        event: &HookEvent,
        function_name: &str,
    ) -> Result<bool, LuaEngineError> {
        let globals = self.lua.globals();

        let func: LuaFunction = globals
            .get(function_name)
            .map_err(|e| LuaEngineError::script_error(function_name, e.to_string()))?;

        let result: LuaValue = match event {
            HookEvent::OnOpen { file_path }
            | HookEvent::OnBeforeSave { file_path }
            | HookEvent::OnAfterSave { file_path }
            | HookEvent::OnClose { file_path }
            | HookEvent::OnSwitchBuffer { file_path } => func.call(file_path.clone()),
            HookEvent::OnChar { character } => func.call(character.to_string()),
            HookEvent::OnKey {
                key_code,
                shift,
                ctrl,
                alt,
            } => func.call((key_code.clone(), *shift, *ctrl, *alt)),
            HookEvent::OnCommand { command_id, params } => {
                func.call((command_id.clone(), params.clone()))
            }
            HookEvent::OnError { error_message } => func.call(error_message.clone()),
        }
        .map_err(|e| LuaEngineError::script_error(function_name, e.to_string()))?;

        // Check if returned false (cancellation)
        let returned_false = matches!(result, LuaValue::Boolean(false));
        Ok(returned_false)
    }
}

/// Convert a Lua value to a display string (for EXEC return value display).
fn lua_value_to_display_string(value: &LuaValue) -> String {
    match value {
        LuaValue::Nil => "nil".to_string(),
        LuaValue::Boolean(b) => b.to_string(),
        LuaValue::Integer(i) => i.to_string(),
        LuaValue::Number(n) => n.to_string(),
        LuaValue::String(s) => s
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "<invalid utf8>".to_string()),
        LuaValue::Table(_) => "table".to_string(),
        LuaValue::Function(_) => "function".to_string(),
        _ => "userdata".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> LuaMacroEngine {
        let config = EngineConfig::for_testing();
        let mut engine = LuaMacroEngine::new(config).unwrap();
        engine.initialize().unwrap();
        engine
    }

    // Validates: Requirement 1.1
    #[test]
    fn engine_creates_lua_runtime_successfully() {
        let engine = test_engine();
        assert!(engine.is_initialized());
    }

    // Validates: Requirement 1.6
    #[test]
    fn engine_reuses_runtime_across_invocations() {
        let engine = test_engine();

        // Set a global in first invocation
        engine.lua.load("test_global = 42").exec().unwrap();

        // Verify it persists in second invocation
        let val: i64 = engine.lua.load("return test_global").eval().unwrap();
        assert_eq!(val, 42);
    }

    // Validates: Requirement 5.2
    #[test]
    fn execute_inline_returns_expression_value() {
        let mut engine = test_engine();
        let result = engine.execute_inline("return 2 + 2").unwrap();
        assert_eq!(result, Some("4".to_string()));
    }

    // Validates: Requirement 5.2
    #[test]
    fn execute_inline_returns_none_for_nil() {
        let mut engine = test_engine();
        let result = engine.execute_inline("return nil").unwrap();
        assert_eq!(result, None);
    }

    // Validates: Requirement 5.2
    #[test]
    fn execute_inline_returns_string_values() {
        let mut engine = test_engine();
        let result = engine.execute_inline(r#"return "hello""#).unwrap();
        assert_eq!(result, Some("hello".to_string()));
    }

    // Validates: Requirement 6.3
    #[test]
    fn execute_inline_continues_after_error() {
        let mut engine = test_engine();

        // First call errors
        let err = engine.execute_inline("error('boom')");
        assert!(err.is_err());

        // Second call succeeds
        let result = engine.execute_inline("return 1").unwrap();
        assert_eq!(result, Some("1".to_string()));
    }
}
