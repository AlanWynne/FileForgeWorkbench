//! # ff-idcams — IDCAMS Emulator for FileForgeWorkbench
//!
//! This crate is a thin command interpreter and orchestration layer for IBM IDCAMS
//! (Access Method Services). It owns **only** command parsing and execution
//! orchestration — all actual catalog, VSAM, allocation, and filesystem operations
//! are delegated to downstream services through trait interfaces.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Error types for the IDCAMS emulator.
pub mod error;

/// Downstream service trait definitions and dependency injection container.
pub mod services;

/// IDC message catalogue and formatting.
pub mod messages;

/// IDCAMS control statement parser (lexer, AST, recursive-descent parser).
pub mod parser;

/// Command executor and orchestration logic.
pub mod executor;

/// Pretty printer for formatting AST back to IDCAMS control statements.
pub mod pretty_printer;

/// SYSIN input processing and reading modes.
pub mod sysin;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use error::IdcamsError;
pub use executor::{CommandExecutor, ExecutionState, IdcamsResult};
pub use messages::{ConditionCode, IdcamsMessage, MessageCode, Severity};
pub use parser::ast::Command;
pub use parser::IdcamsParser;
pub use pretty_printer::{pretty_print, PrintMode};
pub use services::{AllocatorService, CatalogService, IdcamsServices, VsamService};
pub use sysin::InputSource;

/// Execute IDCAMS control statements from a string input.
pub fn execute_idcams(input: &str, services: &IdcamsServices) -> IdcamsResult {
    let commands = IdcamsParser::parse(input);
    let mut executor = CommandExecutor::new(services);
    executor.execute_commands(commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mocks::TestServicesBuilder;

    #[test]
    fn smoke_test_parser_produces_commands() {
        let commands = IdcamsParser::parse("SET LASTCC(0)");
        assert_eq!(commands.len(), 1);
    }

    #[test]
    fn smoke_test_execute_idcams_empty_input() {
        let services = TestServicesBuilder::new().build();
        let result = execute_idcams("", &services);
        assert_eq!(result.maxcc, ConditionCode::Success);
    }

    #[test]
    fn execute_set_lastcc_updates_register() {
        let services = TestServicesBuilder::new().build();
        let result = execute_idcams("SET LASTCC(4)", &services);
        // After SET LASTCC(4), LASTCC=4, MAXCC=4
        // Then final summary is emitted
        assert_eq!(result.maxcc, ConditionCode::Warning);
    }
}
