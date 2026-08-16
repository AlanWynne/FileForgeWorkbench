//! `ScriptingBridge` — interface for Lua macro engine command invocation.
//!
//! Converts between Lua-compatible types and the command framework's native types.

use std::collections::HashMap;
use std::sync::Arc;

use crate::dispatch::CommandDispatch;
use crate::error::ScriptingError;
use crate::params::{CommandParams, ParamValue};
use crate::result::CommandResult;

/// Lua-compatible parameter representation (converted from Lua tables).
#[derive(Debug, Clone)]
pub enum LuaParams {
    /// No parameters.
    None,
    /// A table of key-value pairs.
    Table(HashMap<String, LuaValue>),
}

/// Lua-compatible value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    /// Nil/null value.
    Nil,
    /// Boolean value.
    Boolean(bool),
    /// Integer value.
    Integer(i64),
    /// Floating-point number.
    Number(f64),
    /// String value.
    String(String),
    /// Table (nested map).
    Table(HashMap<String, LuaValue>),
}

/// Command info for scripting discovery.
#[derive(Debug, Clone)]
pub struct ScriptingCommandInfo {
    /// The command ID.
    pub id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Category string.
    pub category: String,
    /// Description of the command.
    pub description: String,
}

/// The interface through which the Lua macro engine invokes commands.
///
/// Converts Lua tables to `CommandParams`, dispatches commands through
/// the standard path, and converts results back to Lua-compatible values.
pub struct ScriptingBridge {
    dispatch: Arc<CommandDispatch>,
}

impl ScriptingBridge {
    /// Creates a new bridge connected to the command dispatch.
    pub fn new(dispatch: Arc<CommandDispatch>) -> Self {
        Self { dispatch }
    }

    /// Executes a command from a Lua script.
    ///
    /// Converts Lua parameters to `CommandParams`, dispatches the command,
    /// and converts the result to a Lua-compatible value.
    pub fn execute(
        &self,
        command_id: &str,
        lua_params: LuaParams,
    ) -> Result<LuaValue, ScriptingError> {
        let params = lua_params_to_command_params(lua_params)?;
        let result = self.dispatch.execute_command(command_id, params);

        match result {
            CommandResult::Ok => Ok(LuaValue::Nil),
            CommandResult::OkValue(value) => Ok(param_value_to_lua(&value)),
            CommandResult::OkUndoable { .. } => Ok(LuaValue::Nil),
            CommandResult::OkValueUndoable { value, .. } => Ok(param_value_to_lua(&value)),
            CommandResult::Err(err) => {
                let id = command_id.to_string();
                match err {
                    crate::error::CommandError::NotFound { .. } => {
                        Err(ScriptingError::CommandNotFound { id })
                    }
                    other => Err(ScriptingError::ExecutionFailed {
                        id,
                        description: other.to_string(),
                    }),
                }
            }
        }
    }

    /// Lists all registered commands with metadata.
    ///
    /// Returns data suitable for conversion to a Lua table.
    pub fn list_commands(&self) -> Vec<ScriptingCommandInfo> {
        // Access the dispatch's registry through executing a discovery query.
        // For now, return empty — the dispatch doesn't expose registry directly.
        // This will be wired when the full system is integrated.
        Vec::new()
    }
}

/// Converts `LuaParams` to `CommandParams`.
fn lua_params_to_command_params(params: LuaParams) -> Result<CommandParams, ScriptingError> {
    match params {
        LuaParams::None => Ok(CommandParams::new()),
        LuaParams::Table(table) => {
            let mut cmd_params = CommandParams::new();
            for (key, value) in table {
                let param_value = lua_value_to_param_value(&value)?;
                cmd_params.insert(key, param_value);
            }
            Ok(cmd_params)
        }
    }
}

/// Converts a `LuaValue` to a `ParamValue`.
fn lua_value_to_param_value(value: &LuaValue) -> Result<ParamValue, ScriptingError> {
    match value {
        LuaValue::Nil => Err(ScriptingError::ParamConversion {
            description: "nil values cannot be converted to command parameters".to_string(),
        }),
        LuaValue::Boolean(b) => Ok(ParamValue::Boolean(*b)),
        LuaValue::Integer(i) => Ok(ParamValue::Integer(*i)),
        LuaValue::Number(f) => Ok(ParamValue::Float(*f)),
        LuaValue::String(s) => Ok(ParamValue::String(s.clone())),
        LuaValue::Table(t) => {
            let mut map = HashMap::new();
            for (k, v) in t {
                map.insert(k.clone(), lua_value_to_param_value(v)?);
            }
            Ok(ParamValue::Map(map))
        }
    }
}

/// Converts a `ParamValue` to a `LuaValue`.
pub fn param_value_to_lua(value: &ParamValue) -> LuaValue {
    match value {
        ParamValue::String(s) => LuaValue::String(s.clone()),
        ParamValue::Integer(i) => LuaValue::Integer(*i),
        ParamValue::Float(f) => LuaValue::Number(*f),
        ParamValue::Boolean(b) => LuaValue::Boolean(*b),
        ParamValue::Map(m) => {
            let table: HashMap<String, LuaValue> = m
                .iter()
                .map(|(k, v)| (k.clone(), param_value_to_lua(v)))
                .collect();
            LuaValue::Table(table)
        }
    }
}

/// Converts `CommandParams` to `LuaParams` (for round-trip testing).
pub fn command_params_to_lua(params: &CommandParams) -> LuaParams {
    if params.is_empty() {
        return LuaParams::None;
    }
    let mut table = HashMap::new();
    for (key, value) in params.iter() {
        table.insert(key.clone(), param_value_to_lua(value));
    }
    LuaParams::Table(table)
}

/// Converts `LuaParams` back to `CommandParams` (public for testing).
pub fn lua_params_to_params(params: LuaParams) -> Result<CommandParams, ScriptingError> {
    lua_params_to_command_params(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6.2
    #[test]
    fn lua_table_converts_to_command_params() {
        let mut table = HashMap::new();
        table.insert(
            "path".to_string(),
            LuaValue::String("/tmp/file.txt".to_string()),
        );
        table.insert("line".to_string(), LuaValue::Integer(42));
        table.insert("force".to_string(), LuaValue::Boolean(true));

        let params = lua_params_to_command_params(LuaParams::Table(table)).unwrap();
        assert_eq!(params.get_string("path"), Some("/tmp/file.txt"));
        assert_eq!(params.get_integer("line"), Some(42));
        assert_eq!(params.get_bool("force"), Some(true));
    }

    // Validates: Requirement 6.2
    #[test]
    fn none_lua_params_converts_to_empty() {
        let params = lua_params_to_command_params(LuaParams::None).unwrap();
        assert!(params.is_empty());
    }

    // Validates: Requirement 6.3
    #[test]
    fn param_value_converts_to_lua_value() {
        assert_eq!(
            param_value_to_lua(&ParamValue::String("hello".to_string())),
            LuaValue::String("hello".to_string())
        );
        assert_eq!(
            param_value_to_lua(&ParamValue::Integer(42)),
            LuaValue::Integer(42)
        );
        assert_eq!(
            param_value_to_lua(&ParamValue::Float(3.14)),
            LuaValue::Number(3.14)
        );
        assert_eq!(
            param_value_to_lua(&ParamValue::Boolean(true)),
            LuaValue::Boolean(true)
        );
    }

    // Validates: Requirement 6.2, 6.3 — round-trip
    #[test]
    fn param_value_round_trips_through_lua() {
        let original = ParamValue::String("test".to_string());
        let lua = param_value_to_lua(&original);
        let back = lua_value_to_param_value(&lua).unwrap();
        assert_eq!(original, back);
    }

    // Validates: Requirement 6.5
    #[test]
    fn nil_value_returns_conversion_error() {
        let result = lua_value_to_param_value(&LuaValue::Nil);
        assert!(result.is_err());
    }

    // Validates: Requirement 6.2
    #[test]
    fn nested_table_converts_correctly() {
        let mut inner = HashMap::new();
        inner.insert("key".to_string(), LuaValue::String("value".to_string()));

        let mut outer = HashMap::new();
        outer.insert("nested".to_string(), LuaValue::Table(inner));

        let params = lua_params_to_command_params(LuaParams::Table(outer)).unwrap();
        let map = params.get_map("nested").unwrap();
        assert_eq!(
            map.get("key"),
            Some(&ParamValue::String("value".to_string()))
        );
    }
}
