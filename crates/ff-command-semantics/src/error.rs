//! Error types for the command semantics engine.

/// Errors produced by the command semantics engine.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommandSemanticsError {
    /// A syntax error during command-line parsing.
    #[error("[command-semantics] parse: {detail}")]
    ParseError { detail: String },

    /// A structural error (block command mismatch, overlapping blocks).
    #[error("[command-semantics] structure: {detail}")]
    StructureError { detail: String },

    /// A scope resolution failure.
    #[error("[command-semantics] scope: no valid scope for command '{command}'")]
    NoValidScope { command: String },

    /// Command name not recognised after normalization.
    #[error("[command-semantics] dispatch: unknown command '{name}'")]
    UnknownCommand { name: String },

    /// Command and scope are incompatible.
    #[error("[command-semantics] validate: command '{command}' incompatible with {scope_desc}")]
    IncompatibleScope { command: String, scope_desc: String },

    /// Runtime execution failure.
    #[error("[command-semantics] execute '{command}': {detail}")]
    ExecutionFailed { command: String, detail: String },

    /// Invalid line command (when policy is reject).
    #[error("[command-semantics] line-command: unrecognised '{text}'")]
    InvalidLineCommand { text: String },

    /// Line command count out of range.
    #[error("[command-semantics] line-command: count {count} exceeds maximum 99999")]
    LineCommandCountOverflow { count: u64 },

    /// Configuration value invalid (informational — default applied).
    #[error("[command-semantics] config: invalid value for '{key}', using default")]
    ConfigInvalid { key: String },
}

/// Parser-specific error for primary command parsing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// Unclosed quoted string.
    #[error("unclosed quote starting at position {position}")]
    UnclosedQuote { position: usize },

    /// Invalid hex literal format.
    #[error("invalid hex literal at position {position}: {detail}")]
    InvalidHexLiteral { position: usize, detail: String },
}

/// Scope resolution specific error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScopeError {
    /// No scope could be resolved from any priority level.
    #[error("no valid scope found")]
    NoScope,

    /// Block commands not properly paired.
    #[error("block command '{kind}' at line {line} has no matching pair")]
    UnpairedBlock { kind: String, line: u64 },

    /// Overlapping block command ranges.
    #[error("overlapping block commands: {first} and {second}")]
    OverlappingBlocks { first: String, second: String },
}
