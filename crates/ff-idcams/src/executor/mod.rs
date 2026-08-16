//! Command executor and orchestration logic.
//!
//! Processes parsed IDCAMS commands sequentially, delegating actual operations
//! to downstream services through trait interfaces. Maintains per-invocation
//! execution state (LASTCC, MAXCC, messages).

mod context;
mod handlers;

pub use context::ExecutionState;

use crate::messages::{ConditionCode, IdcamsMessage, MessageCode};
use crate::parser::ast::*;
use crate::services::IdcamsServices;

/// Structured result of an IDCAMS invocation.
#[derive(Debug, Clone, PartialEq)]
pub struct IdcamsResult {
    /// The condition code of the last executed command.
    pub lastcc: ConditionCode,
    /// The maximum condition code across all commands.
    pub maxcc: ConditionCode,
    /// All messages generated during execution.
    pub messages: Vec<IdcamsMessage>,
}

/// The command executor. Processes parsed commands using injected services.
pub struct CommandExecutor<'a> {
    services: &'a IdcamsServices,
    state: ExecutionState,
}

impl<'a> CommandExecutor<'a> {
    /// Creates a new executor with the given services.
    pub fn new(services: &'a IdcamsServices) -> Self {
        Self {
            services,
            state: ExecutionState::new(),
        }
    }

    /// Executes a sequence of parsed commands and returns the result.
    pub fn execute_commands(&mut self, commands: Vec<Command>) -> IdcamsResult {
        if commands.is_empty() {
            self.state
                .emit_message(MessageCode::IDC0640I, "NO COMMANDS TO PROCESS");
            return self.state.to_result();
        }

        for command in commands {
            self.execute_single(command);

            // CC=16 terminates processing immediately
            if self.state.lastcc == ConditionCode::Catastrophic {
                break;
            }
        }

        // Final summary message
        self.state.emit_message(
            MessageCode::IDC0002I,
            &format!(
                "IDCAMS PROCESSING COMPLETE. MAXIMUM CONDITION CODE WAS {}",
                self.state.maxcc.value()
            ),
        );

        self.state.to_result()
    }

    fn execute_single(&mut self, command: Command) {
        match command {
            Command::DefineCluster(cmd) => {
                handlers::execute_define_cluster(cmd, self.services, &mut self.state)
            }
            Command::DefineAix(cmd) => {
                handlers::execute_define_aix(cmd, self.services, &mut self.state)
            }
            Command::DefinePath(cmd) => {
                handlers::execute_define_path(cmd, self.services, &mut self.state)
            }
            Command::DefineGdg(cmd) => {
                handlers::execute_define_gdg(cmd, self.services, &mut self.state)
            }
            Command::Delete(cmd) => handlers::execute_delete(cmd, self.services, &mut self.state),
            Command::Alter(cmd) => handlers::execute_alter(cmd, self.services, &mut self.state),
            Command::Listcat(cmd) => handlers::execute_listcat(cmd, self.services, &mut self.state),
            Command::Print(cmd) => handlers::execute_print(cmd, self.services, &mut self.state),
            Command::Repro(cmd) => handlers::execute_repro(cmd, self.services, &mut self.state),
            Command::Verify(cmd) => handlers::execute_verify(cmd, self.services, &mut self.state),
            Command::Export(cmd) => handlers::execute_export(cmd, self.services, &mut self.state),
            Command::Import(cmd) => handlers::execute_import(cmd, self.services, &mut self.state),
            Command::Bldindex(cmd) => {
                handlers::execute_bldindex(cmd, self.services, &mut self.state)
            }
            Command::Set(cmd) => handlers::execute_set(cmd, &mut self.state),
            Command::If(cmd) => self.execute_if(cmd),
            Command::Error(err) => {
                self.state.emit_message(
                    MessageCode::IDC0001E,
                    &format!("{}: {}", err.code, err.message),
                );
                self.state.set_lastcc(ConditionCode::Severe);
            }
        }
    }

    fn execute_if(&mut self, cmd: IfCommand) {
        let condition_met = self.evaluate_condition(&cmd.condition);

        if condition_met {
            for c in cmd.then_commands {
                self.execute_single(c);
                if self.state.lastcc == ConditionCode::Catastrophic {
                    break;
                }
            }
        } else if let Some(else_cmds) = cmd.else_commands {
            for c in else_cmds {
                self.execute_single(c);
                if self.state.lastcc == ConditionCode::Catastrophic {
                    break;
                }
            }
        }
    }

    fn evaluate_condition(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Compare {
                register,
                op,
                value,
            } => {
                let reg_value = match register {
                    ConditionRegister::LastCC => self.state.lastcc.value(),
                    ConditionRegister::MaxCC => self.state.maxcc.value(),
                };
                op.evaluate(reg_value, *value)
            }
            Condition::And(left, right) => {
                self.evaluate_condition(left) && self.evaluate_condition(right)
            }
            Condition::Or(left, right) => {
                self.evaluate_condition(left) || self.evaluate_condition(right)
            }
        }
    }
}
